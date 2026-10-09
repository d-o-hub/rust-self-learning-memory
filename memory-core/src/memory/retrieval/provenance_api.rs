//! F4.1 — retrieval with redacted provenance envelope (ADR-074).

use crate::episode::Episode;
use crate::retrieval::{CacheKey, RANKING_CONFIG_VERSION, RetrievalProvenance};
use crate::types::TaskContext;
use std::sync::Arc;
use std::time::Instant;
use tracing::instrument;

use super::super::SelfLearningMemory;
use super::report::RetrievalExecution;

/// Result of a provenance-aware retrieval (F4.1).
#[derive(Debug, Clone)]
pub struct ProvenancedRetrieval {
    /// Ranked episodes (same ordering as `retrieve_relevant_context`)
    pub episodes: Vec<Arc<Episode>>,
    /// Redacted provenance (no raw query text)
    pub provenance: RetrievalProvenance,
    /// Wall-clock latency for the call in milliseconds
    pub latency_ms: u64,
    /// Execution-backed metadata for the retrieval that served this result.
    ///
    /// Issue #1079: the cache hit/miss, serving tier, fallback, candidate
    /// count, and pipeline latency come from the single execution that
    /// produced `episodes`, not from a second cache probe.
    pub execution: RetrievalExecution,
}

impl SelfLearningMemory {
    /// Build the full ADR-074 cache identity for a retrieval request.
    #[must_use]
    pub fn build_retrieval_cache_key(
        &self,
        task_description: &str,
        context: &TaskContext,
        limit: usize,
    ) -> CacheKey {
        CacheKey::new(task_description.to_string())
            .with_task_context(context)
            .with_limit(limit)
            .with_retrieval_mode(self.config.retrieval_mode.to_string())
            // Provenance follows the runtime-activated provider (issue #1072).
            .with_provider_identity(self.effective_provider_identity())
            .with_ranking_config_version(RANKING_CONFIG_VERSION)
            .with_index_generation(self.query_cache.index_generation())
    }

    /// Retrieve relevant episodes and attach a redacted provenance envelope (F4.1).
    ///
    /// Does not log or return the raw `task_description`. Cache hit/miss and
    /// index generation are included for incident diagnosis.
    #[instrument(skip(self, context))]
    pub async fn retrieve_relevant_context_with_provenance(
        &self,
        task_description: String,
        context: TaskContext,
        limit: usize,
    ) -> ProvenancedRetrieval {
        let started = Instant::now();
        let cache_key = self.build_retrieval_cache_key(&task_description, &context, limit);

        // Issue #1079: exactly one execution (and thus one cache lookup) backs
        // both the episodes and the provenance. The key built above is only for
        // identity/fingerprint; the pipeline performs the actual cache probe.
        let (episodes, execution) = self
            .retrieve_relevant_context_with_execution(task_description, context, limit)
            .await;

        let mut provenance = RetrievalProvenance::from_key(
            &cache_key,
            execution.cache_hit,
            execution.candidate_count,
            execution.result_count,
        );
        provenance.executed = execution.executed;
        provenance.tier = execution.tier.clone();
        provenance.fallback = execution.fallback;

        ProvenancedRetrieval {
            episodes,
            provenance,
            latency_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            execution,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::Episode;
    use crate::types::{ComplexityLevel, TaskOutcome, TaskType};

    /// Build a completed episode whose context/description make it a candidate.
    fn complete_episode(description: &str, ctx: &TaskContext) -> Arc<Episode> {
        let mut ep = Episode::new(
            description.to_string(),
            ctx.clone(),
            TaskType::CodeGeneration,
        );
        ep.complete(TaskOutcome::Success {
            verdict: "ok".into(),
            artifacts: vec![],
        });
        Arc::new(ep)
    }

    #[tokio::test]
    async fn provenance_has_no_raw_query_and_reports_miss() {
        let memory = SelfLearningMemory::new();
        let ctx = TaskContext {
            language: Some("rust".into()),
            framework: Some("axum".into()),
            complexity: ComplexityLevel::Simple,
            domain: "api".into(),
            tags: vec!["auth".into()],
        };
        let secret = "user password secret query about tokens".to_string();
        let result = memory
            .retrieve_relevant_context_with_provenance(secret.clone(), ctx, 3)
            .await;

        // execution-backed flags: a miss really executed the pipeline
        assert!(!result.provenance.cache_hit);
        assert!(result.provenance.executed);
        assert_eq!(result.provenance.tier, "none"); // empty corpus, no tier served
        assert!(!result.provenance.fallback);
        assert!(!result.execution.cache_hit);
        assert!(result.execution.executed);
        assert_eq!(result.provenance.result_count, result.episodes.len());
        let debug = format!("{:?}", result.provenance);
        assert!(
            !debug.contains("password") && !debug.contains(&secret),
            "provenance must not embed raw query: {debug}"
        );
        assert_ne!(result.provenance.fingerprint.len(), 0);
        assert!(result.latency_ms < 60_000);
    }

    /// Issue #1079 acceptance: a cache hit reports the cache tier, is marked
    /// executed=false, has no candidate count, and probes the cache exactly once.
    #[tokio::test]
    async fn retrieval_telemetry_provenance_cache_hit_counts_once() {
        let memory = SelfLearningMemory::new();
        let ctx = TaskContext::default();
        let q = "implement caching".to_string();

        // Seed the cache with the exact identity the provenance path builds.
        let key = memory.build_retrieval_cache_key(&q, &ctx, 5);
        let cached = complete_episode("Implement caching", &ctx);
        memory.query_cache.put(key, vec![cached]);
        memory.clear_cache_metrics();

        let result = memory
            .retrieve_relevant_context_with_provenance(q, ctx, 5)
            .await;

        // Exactly one query-cache lookup, so exactly one hit and no miss.
        let metrics = memory.get_cache_metrics();
        assert_eq!(metrics.hits, 1, "cache lookup must be counted once");
        assert_eq!(metrics.misses, 0, "no second/spurious cache probe");

        assert!(result.provenance.cache_hit);
        assert!(!result.provenance.executed);
        assert_eq!(result.provenance.tier, "cache");
        assert_eq!(result.provenance.candidate_count, None);
        assert_eq!(result.provenance.result_count, 1);
        assert!(result.execution.cache_hit);
        assert!(!result.execution.executed);
        assert_eq!(result.execution.tier, "cache");
        assert_eq!(result.episodes.len(), 1);
    }

    /// Issue #1079 acceptance: a miss reports the keyword path it actually ran,
    /// and the candidate count is the measured pre-truncation count — never
    /// fabricated from the (truncated) result count.
    #[tokio::test]
    async fn provenance_miss_reports_execution_candidates() {
        use crate::types::MemoryConfig;

        // Disable the spatiotemporal/diversity path so the measured branch is
        // the legacy keyword scorer, whose pre-truncation candidate set we can
        // reason about directly.
        let config = MemoryConfig {
            enable_spatiotemporal_indexing: false,
            enable_diversity_maximization: false,
            ..Default::default()
        };
        let memory = SelfLearningMemory::with_config(config);
        let ctx = TaskContext {
            language: Some("rust".into()),
            framework: Some("axum".into()),
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".into(),
            tags: vec!["rest".into()],
        };
        let q = "implement rust web api with axum".to_string();

        for _ in 0..3 {
            let ep = complete_episode("Implement rust web api with axum", &ctx);
            memory
                .episodes_fallback
                .write()
                .await
                .insert(ep.episode_id, ep);
        }
        memory.clear_cache_metrics();

        let result = memory
            .retrieve_relevant_context_with_provenance(q, ctx, 1)
            .await;

        // One miss, no hits: the pipeline ran once with one cache probe.
        let metrics = memory.get_cache_metrics();
        assert_eq!(metrics.misses, 1, "miss must be counted exactly once");
        assert_eq!(metrics.hits, 0);

        assert!(!result.provenance.cache_hit);
        assert!(result.provenance.executed);
        assert_eq!(result.provenance.tier, "keyword");
        assert!(!result.provenance.fallback);
        assert_eq!(result.episodes.len(), 1);
        assert_eq!(result.provenance.result_count, 1);

        let candidates = result
            .provenance
            .candidate_count
            .expect("keyword path measures candidates before truncation");
        assert!(
            candidates >= 2,
            "candidate_count {candidates} must be the pre-truncation count, not result_count"
        );
        assert_eq!(result.execution.candidate_count, Some(candidates));
    }

    /// Issue #1079: an empty corpus still yields honest (not fabricated) counts.
    #[tokio::test]
    async fn provenance_empty_corpus_has_no_fabricated_candidates() {
        let memory = SelfLearningMemory::new();
        let result = memory
            .retrieve_relevant_context_with_provenance(
                "nothing here".to_string(),
                TaskContext::default(),
                5,
            )
            .await;
        assert!(!result.provenance.cache_hit);
        assert!(result.provenance.executed);
        assert_eq!(result.provenance.candidate_count, Some(0));
        assert_eq!(result.provenance.result_count, 0);
    }

    /// Issue #1072: the provenance envelope must describe the activated provider
    /// and the generation must advance so entries from the previous provider can
    /// never be served again.
    #[tokio::test]
    async fn provenance_identity_follows_provider_activation() {
        use crate::embeddings::{
            EmbeddingConfig, InMemoryEmbeddingStorage, MockLocalModel, SemanticService,
        };
        use std::sync::Arc;

        let memory = SelfLearningMemory::new();
        let ctx = TaskContext::default();

        let before = memory
            .retrieve_relevant_context_with_provenance("q".to_string(), ctx.clone(), 3)
            .await
            .provenance;

        let service = Arc::new(SemanticService::new(
            Box::new(MockLocalModel::new("model-a".to_string(), 4)),
            Box::new(InMemoryEmbeddingStorage::new()),
            EmbeddingConfig::default(),
        ));
        memory
            .activate_semantic_service(service, "local:model-a:4".to_string())
            .await;

        let after = memory
            .retrieve_relevant_context_with_provenance("q".to_string(), ctx, 3)
            .await
            .provenance;

        assert_eq!(after.provider_identity, "local:model-a:4");
        assert_ne!(after.provider_identity, before.provider_identity);
        assert_ne!(after.index_generation, before.index_generation);
    }
}
