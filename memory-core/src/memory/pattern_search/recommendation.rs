//! Pattern recommendation engine.
//!
//! Provides high-level functions for recommending patterns for tasks
//! and discovering analogous patterns across domains.

use crate::Result;
use crate::embeddings::SemanticService;
use crate::episode::PatternId;
use crate::patterns::Pattern;
use crate::types::TaskContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use super::scoring::{SearchConfig, calculate_pattern_score};

/// Result from semantic pattern search with scoring details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSearchResult {
    /// The matched pattern
    pub pattern: Pattern,
    /// Overall relevance score (0.0 to 1.0)
    pub relevance_score: f32,
    /// Breakdown of scoring components
    pub score_breakdown: super::scoring::ScoreBreakdown,
}

/// Search for patterns semantically similar to a query
pub async fn search_patterns_semantic(
    query: &str,
    patterns: Vec<Pattern>,
    context: &TaskContext,
    semantic_service: Option<&Arc<SemanticService>>,
    config: SearchConfig,
    limit: usize,
) -> Result<Vec<PatternSearchResult>> {
    debug!(
        query = %query,
        pattern_count = patterns.len(),
        limit = limit,
        "Starting semantic pattern search"
    );

    // Generate embedding for query
    let query_embedding = if let Some(service) = semantic_service {
        // Use the service to generate embeddings for the query text
        match service.provider.embed_text(query).await {
            Ok(embedding) => {
                debug!(
                    "Generated query embedding with {} dimensions",
                    embedding.len()
                );
                embedding
            }
            Err(e) => {
                debug!(
                    "Failed to generate query embedding: {}, using keyword-based scoring",
                    e
                );
                vec![]
            }
        }
    } else {
        debug!("No semantic service available, using keyword-based scoring");
        vec![]
    };

    // Prefetch the candidates' stored vectors once (issue #1076): scoring a
    // candidate must not call the provider when its vector is already stored.
    // Only the misses are embedded, in one batched provider call.
    let pattern_embeddings: HashMap<PatternId, Vec<f32>> = match semantic_service {
        Some(service) if !query_embedding.is_empty() => {
            let ids: Vec<PatternId> = patterns.iter().map(Pattern::id).collect();
            let stored = service.get_pattern_embeddings_batch(&ids).await?;

            let mut by_id: HashMap<PatternId, Vec<f32>> = HashMap::with_capacity(patterns.len());
            let mut missing: Vec<Pattern> = Vec::new();
            for (pattern, maybe_embedding) in patterns.iter().zip(stored) {
                match maybe_embedding {
                    Some(embedding) => {
                        by_id.insert(pattern.id(), embedding);
                    }
                    None => missing.push(pattern.clone()),
                }
            }

            if !missing.is_empty() {
                let computed = service.embed_patterns_batch(&missing).await?;
                for (pattern, embedding) in missing.iter().zip(computed) {
                    by_id.insert(pattern.id(), embedding);
                }
            }

            by_id
        }
        // No provider, or no query vector to compare against: the lexical
        // fallback answers inside the per-pattern scoring call.
        _ => HashMap::new(),
    };

    let mut results = Vec::new();

    for pattern in patterns {
        // Apply pre-filters
        if config.filter_by_domain {
            if let Some(pattern_ctx) = pattern.context() {
                if pattern_ctx.domain != context.domain {
                    continue;
                }
            }
        }

        // Note: TaskContext doesn't have task_type field in current implementation
        // Filtering by task type would require changes to TaskContext struct

        // Calculate multi-signal score from the prefetched candidate vector.
        let score_breakdown = calculate_pattern_score(
            query,
            &query_embedding,
            pattern_embeddings.get(&pattern.id()).map(Vec::as_slice),
            &pattern,
            context,
            &config,
        );

        let relevance_score = super::scoring::combine_scores(&score_breakdown, &config);

        // Filter by minimum relevance
        if relevance_score >= config.min_relevance {
            results.push(PatternSearchResult {
                pattern,
                relevance_score,
                score_breakdown,
            });
        }
    }

    // Sort by relevance (highest first)
    results.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply limit
    results.truncate(limit);

    debug!(
        results_count = results.len(),
        "Semantic pattern search completed"
    );

    Ok(results)
}

/// Recommend patterns for a specific task.
///
/// When `learned` is `Some`, the candidate pool is overfetched and re-ranked by
/// base relevance plus a learned Wilson weight derived from attributed feedback
/// (ADR-082) before truncation to `limit`. When `None`, behavior is identical to
/// a plain relevance-ranked search.
pub async fn recommend_patterns_for_task(
    task_description: &str,
    context: TaskContext,
    patterns: Vec<Pattern>,
    semantic_service: Option<&Arc<SemanticService>>,
    limit: usize,
    learned: Option<&crate::memory::attribution::RankingIndex>,
) -> Result<Vec<PatternSearchResult>> {
    debug!(
        task = %task_description,
        domain = %context.domain,
        learned = learned.is_some(),
        "Recommending patterns for task"
    );

    // Use strict config for recommendations (high quality)
    let config = SearchConfig {
        min_relevance: 0.4,
        semantic_weight: 0.35,
        context_weight: 0.25,
        effectiveness_weight: 0.25,
        recency_weight: 0.1,
        success_weight: 0.05,
        filter_by_domain: true,
        filter_by_task_type: false, // Allow cross-task-type recommendations
    };

    // Overfetch when re-ranking so a boosted pattern can enter the top-N:
    // the learned re-rank runs before truncation.
    let fetch_limit = if learned.is_some() {
        limit.saturating_mul(crate::memory::attribution::RECOMMEND_OVERFETCH_FACTOR)
    } else {
        limit
    };

    let mut results = search_patterns_semantic(
        task_description,
        patterns,
        &context,
        semantic_service,
        config,
        fetch_limit,
    )
    .await?;

    if let Some(index) = learned {
        // Compute each candidate's learned key once (one `String` key + one
        // lookup per candidate), then stable-sort the precomputed scores so the
        // comparator is allocation-free — deriving the key inside `sort_by`
        // would allocate a `String` and look it up ~N log N times.
        let mut scored: Vec<(f32, PatternSearchResult)> = results
            .into_iter()
            .map(|r| {
                (
                    r.relevance_score + index.boost(&r.pattern.id().to_string()),
                    r,
                )
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results = scored.into_iter().map(|(_, r)| r).collect();
        results.truncate(limit);
    }

    Ok(results)
}

/// Discover analogous patterns from a different domain
pub async fn discover_analogous_patterns(
    source_domain: &str,
    target_context: TaskContext,
    patterns: Vec<Pattern>,
    semantic_service: Option<&Arc<SemanticService>>,
    limit: usize,
) -> Result<Vec<PatternSearchResult>> {
    debug!(
        source_domain = %source_domain,
        target_domain = %target_context.domain,
        "Discovering analogous patterns"
    );

    // Filter patterns from source domain
    let source_patterns: Vec<Pattern> = patterns
        .into_iter()
        .filter(|p| p.context().is_some_and(|ctx| ctx.domain == source_domain))
        .collect();

    debug!(
        source_pattern_count = source_patterns.len(),
        "Filtered patterns from source domain"
    );

    // Use relaxed config for cross-domain discovery
    let config = SearchConfig {
        min_relevance: 0.3,
        semantic_weight: 0.5, // Emphasize semantic similarity for cross-domain
        context_weight: 0.1,  // De-emphasize context match
        effectiveness_weight: 0.2,
        recency_weight: 0.1,
        success_weight: 0.1,
        filter_by_domain: false,
        filter_by_task_type: false,
    };

    // Search with target domain context
    let query = format!(
        "Apply patterns from {} to {}",
        source_domain, target_context.domain
    );

    search_patterns_semantic(
        &query,
        source_patterns,
        &target_context,
        semantic_service,
        config,
        limit,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComplexityLevel;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use uuid::Uuid;

    fn create_test_pattern(domain: &str, success_rate: f32) -> Pattern {
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: TaskContext {
                domain: domain.to_string(),
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                tags: vec!["rust".to_string()],
            },
            success_rate,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: crate::patterns::PatternEffectiveness::new(),
        }
    }

    #[tokio::test]
    async fn test_search_patterns_semantic_no_service() {
        let patterns = vec![
            create_test_pattern("web-api", 0.9),
            create_test_pattern("cli", 0.8),
        ];

        let context = TaskContext {
            domain: "web-api".to_string(),
            language: None,
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: vec![],
        };

        let results = search_patterns_semantic(
            "build REST API",
            patterns,
            &context,
            None,
            SearchConfig::default(),
            5,
        )
        .await
        .unwrap();

        assert!(!results.is_empty());
    }

    /// Ordering must follow the query, not just context/effectiveness — the regression that
    /// let a constant `0.5` stand in for lexical relevance.
    #[tokio::test]
    async fn search_without_a_service_orders_by_query_terms() {
        let mut relevant = create_test_pattern("web-api", 0.9);
        if let Pattern::ToolSequence { tools, .. } = &mut relevant {
            *tools = vec!["axum".to_string(), "tokio".to_string()];
        }
        let mut off_topic = create_test_pattern("web-api", 0.9);
        if let Pattern::ToolSequence { tools, .. } = &mut off_topic {
            *tools = vec!["ffmpeg".to_string(), "blender".to_string()];
        }

        let context = TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: vec!["axum".to_string(), "async".to_string()],
        };

        let results = search_patterns_semantic(
            "build an async axum server",
            vec![off_topic, relevant],
            &context,
            None,
            SearchConfig::default(),
            5,
        )
        .await
        .unwrap();

        assert!(
            !results.is_empty(),
            "the matching pattern must survive filtering"
        );
        let top = &results[0];
        assert!(
            matches!(
                &top.pattern,
                Pattern::ToolSequence { tools, .. } if tools.contains(&"axum".to_string())
            ),
            "the pattern naming the query's terms must rank first, got {:?}",
            top.pattern
        );
        let top_semantic = top.score_breakdown.semantic_similarity;
        for other in results.iter().skip(1) {
            assert!(
                top_semantic > other.score_breakdown.semantic_similarity,
                "candidates must be scored on their own text, not a shared constant: \
                 top {top_semantic:.3} vs {:.3}",
                other.score_breakdown.semantic_similarity
            );
        }
    }

    /// Embedding generation fails exactly like a live provider outage.
    struct OutageProvider;

    #[async_trait::async_trait]
    impl crate::embeddings::EmbeddingProvider for OutageProvider {
        async fn embed_text(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
            Err(anyhow::anyhow!("provider unavailable"))
        }

        fn embedding_dimension(&self) -> usize {
            4
        }

        fn model_name(&self) -> &'static str {
            "outage-provider"
        }
    }

    /// A provider error is swallowed into an empty query embedding, so the search must score
    /// the candidates lexically rather than fall back to the old constant.
    #[tokio::test]
    async fn provider_errors_still_score_by_query_terms() {
        use crate::embeddings::{EmbeddingConfig, InMemoryEmbeddingStorage};

        let service = Arc::new(SemanticService::new(
            Box::new(OutageProvider),
            Box::new(InMemoryEmbeddingStorage::new()),
            EmbeddingConfig::default(),
        ));

        let mut relevant = create_test_pattern("web-api", 0.9);
        if let Pattern::ToolSequence { tools, .. } = &mut relevant {
            *tools = vec!["axum".to_string(), "tokio".to_string()];
        }
        let mut off_topic = create_test_pattern("web-api", 0.9);
        if let Pattern::ToolSequence { tools, .. } = &mut off_topic {
            *tools = vec!["ffmpeg".to_string(), "blender".to_string()];
        }

        let context = TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: vec!["axum".to_string(), "async".to_string()],
        };
        let query = "build an async axum server";

        let results = search_patterns_semantic(
            query,
            vec![off_topic, relevant],
            &context,
            Some(&service),
            SearchConfig::default(),
            5,
        )
        .await
        .unwrap();

        let top = results
            .first()
            .expect("the query-matching pattern must clear min_relevance");
        assert!(
            matches!(
                &top.pattern,
                Pattern::ToolSequence { tools, .. } if tools.contains(&"axum".to_string())
            ),
            "a provider outage must not flatten the ranking, got {:?}",
            top.pattern
        );
        assert_eq!(
            top.score_breakdown.semantic_similarity,
            super::super::lexical::calculate_keyword_similarity(query, &top.pattern),
            "the provider-error path must score like the no-provider path"
        );
    }

    /// Counters shared with a provider double so a test can assert what the
    /// search actually asked the provider to embed (issue #1076).
    #[derive(Default)]
    struct EmbeddingCalls {
        embed_text: AtomicUsize,
        embed_batch: AtomicUsize,
        batch_sizes: parking_lot::Mutex<Vec<usize>>,
    }

    /// Provider that records its calls and otherwise behaves like the mock model.
    struct CountingProvider {
        inner: crate::embeddings::MockLocalModel,
        calls: Arc<EmbeddingCalls>,
    }

    #[async_trait::async_trait]
    impl crate::embeddings::EmbeddingProvider for CountingProvider {
        async fn embed_text(&self, text: &str) -> anyhow::Result<Vec<f32>> {
            self.calls.embed_text.fetch_add(1, Ordering::SeqCst);
            self.inner.embed_text(text).await
        }

        async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
            self.calls.embed_batch.fetch_add(1, Ordering::SeqCst);
            self.calls.batch_sizes.lock().push(texts.len());
            self.inner.embed_batch(texts).await
        }

        fn embedding_dimension(&self) -> usize {
            self.inner.embedding_dimension()
        }

        fn model_name(&self) -> &str {
            self.inner.model_name()
        }
    }

    fn counting_service() -> (Arc<SemanticService>, Arc<EmbeddingCalls>) {
        let calls = Arc::new(EmbeddingCalls::default());
        let provider = CountingProvider {
            inner: crate::embeddings::MockLocalModel::new("counting-model".to_string(), 8),
            calls: Arc::clone(&calls),
        };
        // Coalescing is disabled so the counters observe the search's own provider
        // calls instead of the batching layer's flush schedule.
        let config = crate::embeddings::EmbeddingConfig {
            batch: crate::embeddings::BatchingConfig {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let service = Arc::new(SemanticService::new(
            Box::new(provider),
            Box::new(crate::embeddings::InMemoryEmbeddingStorage::new()),
            config,
        ));
        (service, calls)
    }

    /// Stored candidate vectors are reused: only the query is embedded individually
    /// and the single miss goes out as one batch call (issue #1076).
    #[tokio::test]
    async fn stored_pattern_vectors_are_reused_and_misses_are_batched() {
        use super::super::lexical::{pattern_with, query_context};

        let (service, calls) = counting_service();
        let patterns = vec![
            pattern_with(&["axum"], &["async"]),
            pattern_with(&["tokio"], &["async"]),
            pattern_with(&["hyper"], &["async"]),
        ];

        // Seed two vectors as an earlier search would have left them.
        service.embed_pattern(&patterns[0]).await.unwrap();
        service.embed_pattern(&patterns[1]).await.unwrap();
        calls.embed_text.store(0, Ordering::SeqCst);

        let context = query_context(&["async"]);
        let results = search_patterns_semantic(
            "axum tokio",
            patterns,
            &context,
            Some(&service),
            SearchConfig::default(),
            5,
        )
        .await
        .unwrap();

        assert!(!results.is_empty(), "the search must still return matches");
        assert_eq!(
            calls.embed_text.load(Ordering::SeqCst),
            1,
            "only the query may be embedded individually"
        );
        assert_eq!(
            calls.embed_batch.load(Ordering::SeqCst),
            1,
            "the misses must be embedded in one batch call"
        );
        assert_eq!(
            calls.batch_sizes.lock().clone(),
            vec![1],
            "only the candidate without a stored vector may be embedded"
        );
    }

    /// A warm corpus needs no pattern embedding call at all: the search reads every
    /// candidate vector from storage (issue #1076).
    #[tokio::test]
    async fn warm_corpus_needs_no_pattern_embedding_call() {
        use super::super::lexical::{pattern_with, query_context};

        let (service, calls) = counting_service();
        let patterns = vec![
            pattern_with(&["axum"], &["async"]),
            pattern_with(&["tokio"], &["async"]),
        ];
        for pattern in &patterns {
            service.embed_pattern(pattern).await.unwrap();
        }
        calls.embed_text.store(0, Ordering::SeqCst);
        calls.embed_batch.store(0, Ordering::SeqCst);

        let context = query_context(&["async"]);
        let results = search_patterns_semantic(
            "axum tokio",
            patterns,
            &context,
            Some(&service),
            SearchConfig::default(),
            5,
        )
        .await
        .unwrap();

        assert!(!results.is_empty());
        assert_eq!(
            calls.embed_text.load(Ordering::SeqCst),
            1,
            "the query embedding is the only individual call"
        );
        assert_eq!(
            calls.embed_batch.load(Ordering::SeqCst),
            0,
            "a warm corpus must not trigger a pattern embedding batch"
        );
    }
}
