//! Telemetry registry: fixed-size counters, latency trackers, and the
//! process-global instance for retrieval observability (issue #962).

use parking_lot::Mutex;
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use super::super::storage_metrics::OperationLatency;
use super::labels::{
    CacheLayer, EmbeddingOutcome, EmbeddingProviderLabel, FallbackReason, FeedbackSignal,
    N_EMB_OUTCOMES, N_FALLBACK_REASONS, N_LAYERS, N_OPERATIONS, N_OUTCOMES, N_PROVIDERS, N_SIGNALS,
    N_STAGES, N_TIERS, RetrievalOperation, RetrievalOutcome, RetrievalStage, RetrievalTier,
    fallback_index,
};
use crate::retrieval::cascade::CascadeResult;

/// Retrieval-plane telemetry: fixed-size counters and O(1) latency trackers.
///
/// Clone shares nothing; each instance is independent (tests). Production
/// call sites use the process-global instance from
/// [`global_retrieval_metrics`].
pub struct RetrievalMetrics {
    pub(super) requests: [[[AtomicU64; N_OUTCOMES]; N_TIERS]; N_OPERATIONS],
    pub(super) durations_ms: [[Mutex<OperationLatency>; N_TIERS]; N_OPERATIONS],
    pub(super) candidates_sum: [AtomicU64; N_STAGES],
    pub(super) candidates_count: [AtomicU64; N_STAGES],
    pub(super) cache: [[AtomicU64; N_OUTCOMES]; N_LAYERS],
    pub(super) embeddings: [[AtomicU64; N_EMB_OUTCOMES]; N_PROVIDERS],
    pub(super) embedding_durations_ms: [Mutex<OperationLatency>; N_PROVIDERS],
    pub(super) fallbacks: [AtomicU64; N_FALLBACK_REASONS],
    pub(super) feedback: [AtomicU64; N_SIGNALS],
}

impl RetrievalMetrics {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            requests: Default::default(),
            durations_ms: Default::default(),
            candidates_sum: Default::default(),
            candidates_count: Default::default(),
            cache: Default::default(),
            embeddings: Default::default(),
            embedding_durations_ms: Default::default(),
            fallbacks: Default::default(),
            feedback: Default::default(),
        }
    }

    /// Record one retrieval request with its serving duration.
    pub fn record_request(
        &self,
        operation: RetrievalOperation,
        tier: RetrievalTier,
        outcome: RetrievalOutcome,
        duration_ms: u64,
    ) {
        self.requests[operation.index()][tier.index()][outcome.index()]
            .fetch_add(1, Ordering::Relaxed);
        self.durations_ms[operation.index()][tier.index()]
            .lock()
            .record(duration_ms);
    }

    /// Record a candidate-set size at one pipeline stage.
    pub fn record_candidates(&self, stage: RetrievalStage, count: usize) {
        self.candidates_sum[stage.index()].fetch_add(count as u64, Ordering::Relaxed);
        self.candidates_count[stage.index()].fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache lookup result.
    pub fn record_cache(&self, layer: CacheLayer, outcome: RetrievalOutcome) {
        self.cache[layer.index()][outcome.index()].fetch_add(1, Ordering::Relaxed);
    }

    /// Record one embedding provider call with its serving duration.
    pub fn record_embedding(
        &self,
        provider: EmbeddingProviderLabel,
        outcome: EmbeddingOutcome,
        duration_ms: u64,
    ) {
        self.embeddings[provider.index()][outcome.index()].fetch_add(1, Ordering::Relaxed);
        self.embedding_durations_ms[provider.index()]
            .lock()
            .record(duration_ms);
    }

    /// Record one Tier-4 fallback decision.
    pub fn record_fallback(&self, reason: FallbackReason) {
        self.fallbacks[fallback_index(reason)].fetch_add(1, Ordering::Relaxed);
    }

    /// Record one accepted recommendation-feedback signal.
    pub fn record_feedback(&self, signal: FeedbackSignal) {
        self.feedback[signal.index()].fetch_add(1, Ordering::Relaxed);
    }

    /// Record one finished cascade retrieval: request, duration,
    /// candidate-set size, and Tier-4 fallback decision.
    pub fn record_cascade(&self, duration_ms: u64, result: &CascadeResult) {
        let tier = cascade_tier(&result.contributing_tiers);
        let outcome = RetrievalOutcome::from_count(result.episode_ids.len());
        self.record_request(RetrievalOperation::Cascade, tier, outcome, duration_ms);
        self.record_candidates(RetrievalStage::Cascade, result.episode_ids.len());
        self.record_fallback(provisional_fallback_reason(result));
    }

    /// Reset all series (tests and explicit operator reset).
    pub fn reset(&self) {
        for op in &self.requests {
            for tier in op {
                for cell in tier {
                    cell.store(0, Ordering::Relaxed);
                }
            }
        }
        for op in &self.durations_ms {
            for cell in op {
                *cell.lock() = OperationLatency::default();
            }
        }
        for cell in &self.candidates_sum {
            cell.store(0, Ordering::Relaxed);
        }
        for cell in &self.candidates_count {
            cell.store(0, Ordering::Relaxed);
        }
        for layer in &self.cache {
            for cell in layer {
                cell.store(0, Ordering::Relaxed);
            }
        }
        for provider in &self.embeddings {
            for cell in provider {
                cell.store(0, Ordering::Relaxed);
            }
        }
        for cell in &self.embedding_durations_ms {
            *cell.lock() = OperationLatency::default();
        }
        for cell in &self.fallbacks {
            cell.store(0, Ordering::Relaxed);
        }
        for cell in &self.feedback {
            cell.store(0, Ordering::Relaxed);
        }
    }
}

impl Default for RetrievalMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for RetrievalMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetrievalMetrics")
            .field("snapshot", &self.snapshot())
            .finish()
    }
}

static GLOBAL_REGISTRY: OnceLock<Arc<RetrievalMetrics>> = OnceLock::new();

/// Process-global retrieval telemetry registry.
///
/// Production call sites record here; the MCP `get_metrics` retrieval view
/// and Prometheus exposition read here. Tests prefer isolated instances or
/// serial delta assertions (the registry exposes [`RetrievalMetrics::reset`]).
#[must_use]
pub fn global_retrieval_metrics() -> Arc<RetrievalMetrics> {
    Arc::clone(GLOBAL_REGISTRY.get_or_init(|| Arc::new(RetrievalMetrics::new())))
}

/// Serving tier for a cascade result: the single contributing tier,
/// [`RetrievalTier::Blended`] for multi-tier merges, or
/// [`RetrievalTier::None`] when nothing served. Tier markers
/// (`api_fallback_needed`, `none`) never become labels.
#[must_use]
pub fn cascade_tier(contributing_tiers: &[String]) -> RetrievalTier {
    let mut tiers = contributing_tiers
        .iter()
        .filter_map(|tier| match tier.as_str() {
            "bm25" => Some(RetrievalTier::Bm25),
            "hdc" => Some(RetrievalTier::Hdc),
            "concept_graph" => Some(RetrievalTier::ConceptGraph),
            "api" => Some(RetrievalTier::Api),
            _ => None,
        });
    match (tiers.next(), tiers.next()) {
        (Some(first), None) => first,
        (Some(_), Some(_)) => RetrievalTier::Blended,
        (None, _) => RetrievalTier::None,
    }
}

/// Provisional Tier-4 reason for cascades without confidence gating.
///
/// Pre-#968 code cannot distinguish confident from unconfident local
/// results, so any API escalation maps to `InsufficientConfidence`. Once
/// the confidence policy lands, call sites pass
/// `CascadeResult::fallback_reason` straight into
/// [`RetrievalMetrics::record_fallback`] instead.
#[must_use]
pub fn provisional_fallback_reason(result: &CascadeResult) -> FallbackReason {
    if result.episode_ids.is_empty() {
        FallbackReason::NoLocalResults
    } else if result
        .contributing_tiers
        .iter()
        .any(|tier| tier == "api_fallback_needed")
    {
        FallbackReason::InsufficientConfidence
    } else {
        FallbackReason::LocalTierSufficient
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_and_durations_accumulate_per_series() {
        let metrics = RetrievalMetrics::new();
        metrics.record_request(
            RetrievalOperation::Query,
            RetrievalTier::Cache,
            RetrievalOutcome::Hit,
            5,
        );
        metrics.record_request(
            RetrievalOperation::Query,
            RetrievalTier::Cache,
            RetrievalOutcome::Hit,
            15,
        );

        let snapshot = metrics.snapshot();
        let requests = snapshot["requests"].as_array().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["count"], 2);
        assert_eq!(requests[0]["latency_ms"]["avg"], 10);
        // Zero series are omitted from both renderings.
        let text = metrics.export_prometheus();
        assert!(text.contains(
            "memory_retrieval_requests_total{operation=\"query\",tier=\"cache\",outcome=\"hit\"} 2"
        ));
        assert!(!text.contains("tier=\"semantic\""));
    }

    #[test]
    fn reset_clears_every_series() {
        let metrics = RetrievalMetrics::new();
        metrics.record_request(
            RetrievalOperation::Cascade,
            RetrievalTier::Bm25,
            RetrievalOutcome::Hit,
            3,
        );
        metrics.record_fallback(FallbackReason::NoLocalResults);
        metrics.reset();

        assert!(
            metrics.snapshot()["requests"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(!metrics.export_prometheus().contains("tier=\"bm25\""));
    }
}
