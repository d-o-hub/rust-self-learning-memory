//! Pattern recommendation engine.
//!
//! Provides high-level functions for recommending patterns for tasks
//! and discovering analogous patterns across domains.

use crate::embeddings::SemanticService;
use crate::pattern::Pattern;
use crate::types::TaskContext;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

use super::scoring::{calculate_pattern_score, SearchConfig};

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

        // Calculate multi-signal score
        let score_breakdown = calculate_pattern_score(
            &query_embedding,
            &pattern,
            context,
            semantic_service,
            &config,
        )
        .await?;

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

/// Recommend patterns for a specific task
pub async fn recommend_patterns_for_task(
    task_description: &str,
    context: TaskContext,
    patterns: Vec<Pattern>,
    semantic_service: Option<&Arc<SemanticService>>,
    limit: usize,
) -> Result<Vec<PatternSearchResult>> {
    debug!(
        task = %task_description,
        domain = %context.domain,
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

    search_patterns_semantic(
        task_description,
        patterns,
        &context,
        semantic_service,
        config,
        limit,
    )
    .await
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
            effectiveness: crate::pattern::PatternEffectiveness::new(),
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
}
