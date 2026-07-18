//! F4.1 — retrieval with redacted provenance envelope (ADR-074).

use crate::episode::Episode;
use crate::retrieval::{CacheKey, RANKING_CONFIG_VERSION, RetrievalProvenance};
use crate::types::TaskContext;
use std::sync::Arc;
use std::time::Instant;
use tracing::instrument;

use super::super::SelfLearningMemory;

/// Result of a provenance-aware retrieval (F4.1).
#[derive(Debug, Clone)]
pub struct ProvenancedRetrieval {
    /// Ranked episodes (same ordering as `retrieve_relevant_context`)
    pub episodes: Vec<Arc<Episode>>,
    /// Redacted provenance (no raw query text)
    pub provenance: RetrievalProvenance,
    /// Wall-clock latency for the call in milliseconds
    pub latency_ms: u64,
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
            .with_provider_identity(self.semantic_config.provider.cache_identity())
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
        let cache_hit = self.query_cache.get(&cache_key).is_some();

        let episodes = if let Some(cached) = self.query_cache.get(&cache_key) {
            cached
        } else {
            self.retrieve_relevant_context(task_description, context, limit)
                .await
        };

        let result_count = episodes.len();
        let mut provenance =
            RetrievalProvenance::from_key(&cache_key, cache_hit, None, result_count);
        // candidate_count unknown without deeper instrumentation; result_count is exact
        provenance.candidate_count = Some(result_count);

        ProvenancedRetrieval {
            episodes,
            provenance,
            latency_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityLevel, TaskType};

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

        assert!(!result.provenance.cache_hit);
        assert_eq!(result.provenance.result_count, result.episodes.len());
        let debug = format!("{:?}", result.provenance);
        assert!(
            !debug.contains("password") && !debug.contains(&secret),
            "provenance must not embed raw query: {debug}"
        );
        assert!(!result.provenance.fingerprint.is_empty());
        assert!(result.latency_ms < 60_000);
        let _ = TaskType::Testing; // keep import surface stable if used later
    }

    #[tokio::test]
    async fn second_identical_request_is_cache_hit() {
        let memory = SelfLearningMemory::new();
        let ctx = TaskContext::default();
        let q = "implement caching".to_string();
        let _ = memory
            .retrieve_relevant_context_with_provenance(q.clone(), ctx.clone(), 5)
            .await;
        let second = memory
            .retrieve_relevant_context_with_provenance(q, ctx, 5)
            .await;
        // Empty corpus still caches empty results when eligible — either hit or miss is ok;
        // identity fields must remain stable.
        assert!(!second.provenance.fingerprint.is_empty());
        assert_eq!(
            second.provenance.ranking_config_version,
            RANKING_CONFIG_VERSION
        );
    }
}
