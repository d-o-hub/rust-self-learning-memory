//! JSON snapshot and Prometheus text rendering for retrieval telemetry
//! (issue #962). Zero-valued series are omitted from both renderings.

use serde_json::{Value, json};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

use super::labels::{
    CacheLayer, EmbeddingOutcome, EmbeddingProviderLabel, FALLBACK_REASONS, FeedbackSignal,
    RetrievalOperation, RetrievalOutcome, RetrievalStage, RetrievalTier,
};
use super::registry::RetrievalMetrics;

const ALL_OPERATIONS: [RetrievalOperation; 2] =
    [RetrievalOperation::Query, RetrievalOperation::Cascade];

const ALL_TIERS: [RetrievalTier; 11] = [
    RetrievalTier::Cache,
    RetrievalTier::Hybrid,
    RetrievalTier::Semantic,
    RetrievalTier::Hierarchical,
    RetrievalTier::Keyword,
    RetrievalTier::Bm25,
    RetrievalTier::Hdc,
    RetrievalTier::ConceptGraph,
    RetrievalTier::Api,
    RetrievalTier::Blended,
    RetrievalTier::None,
];

const ALL_OUTCOMES: [RetrievalOutcome; 2] = [RetrievalOutcome::Hit, RetrievalOutcome::Miss];

const ALL_PROVIDERS: [EmbeddingProviderLabel; 5] = [
    EmbeddingProviderLabel::Local,
    EmbeddingProviderLabel::OpenAI,
    EmbeddingProviderLabel::Mistral,
    EmbeddingProviderLabel::AzureOpenAI,
    EmbeddingProviderLabel::Custom,
];

const ALL_SIGNALS: [FeedbackSignal; 4] = [
    FeedbackSignal::Success,
    FeedbackSignal::Partial,
    FeedbackSignal::Failure,
    FeedbackSignal::Abstained,
];

impl RetrievalMetrics {
    /// JSON snapshot for MCP `get_metrics` (zero series omitted).
    #[must_use]
    pub fn snapshot(&self) -> Value {
        let mut requests = Vec::new();
        for op in ALL_OPERATIONS {
            for tier in ALL_TIERS {
                self.push_request_rows(&mut requests, op, tier);
            }
        }
        json!({
            "requests": requests,
            "fallbacks": self.str_map(&self.fallbacks, &FALLBACK_REASONS),
            "feedback": self.signal_map(),
            "cache": self.cache_map(),
            "embeddings": self.embeddings_map(),
            "candidates": self.candidates_map(),
        })
    }

    /// Append nonzero request rows for one (operation, tier) series.
    fn push_request_rows(
        &self,
        requests: &mut Vec<Value>,
        op: RetrievalOperation,
        tier: RetrievalTier,
    ) {
        for outcome in ALL_OUTCOMES {
            let count =
                self.requests[op.index()][tier.index()][outcome.index()].load(Ordering::Relaxed);
            if count == 0 {
                continue;
            }
            let latency = self.durations_ms[op.index()][tier.index()].lock();
            let (p50, p95, p99) = latency.percentiles_ms();
            requests.push(json!({
                "operation": op.as_str(),
                "tier": tier.as_str(),
                "outcome": outcome.as_str(),
                "count": count,
                "latency_ms": {"p50": p50, "p95": p95, "p99": p99, "avg": latency.avg_ms()},
            }));
        }
    }

    /// Write nonzero request-count samples for one (operation, tier).
    fn write_request_lines(&self, out: &mut String, op: RetrievalOperation, tier: RetrievalTier) {
        for outcome in ALL_OUTCOMES {
            let count =
                self.requests[op.index()][tier.index()][outcome.index()].load(Ordering::Relaxed);
            if count == 0 {
                continue;
            }
            let _ = writeln!(
                out,
                "memory_retrieval_requests_total{{operation=\"{}\",tier=\"{}\",outcome=\"{}\"}} {}",
                op.as_str(),
                tier.as_str(),
                outcome.as_str(),
                count
            );
        }
    }

    /// Write the quantile samples for one (operation, tier) latency series.
    fn write_duration_lines(&self, out: &mut String, op: RetrievalOperation, tier: RetrievalTier) {
        let latency = self.durations_ms[op.index()][tier.index()].lock();
        if latency.count() == 0 {
            return;
        }
        let (p50, p95, p99) = latency.percentiles_ms();
        for (quantile, value_ms) in [("0.5", p50), ("0.95", p95), ("0.99", p99)] {
            let _ = writeln!(
                out,
                "memory_retrieval_duration_seconds{{operation=\"{}\",tier=\"{}\",quantile=\"{}\"}} {:.3}",
                op.as_str(),
                tier.as_str(),
                quantile,
                value_ms as f64 / 1000.0
            );
        }
    }

    /// Prometheus text exposition (zero series omitted).
    ///
    /// HELP/TYPE lines are emitted exactly once per metric family, before
    /// its samples — duplicate TYPE lines are rejected by strict scrapers.
    #[must_use]
    pub fn export_prometheus(&self) -> String {
        let mut out = String::with_capacity(2048);
        out.push_str("# HELP memory_retrieval_requests_total Retrieval requests by operation, tier, outcome\n");
        out.push_str("# TYPE memory_retrieval_requests_total counter\n");
        for op in ALL_OPERATIONS {
            for tier in ALL_TIERS {
                self.write_request_lines(&mut out, op, tier);
            }
        }

        // Duration quantiles are tracked per (operation, tier); emit each
        // series once even when both hit and miss outcomes are nonzero.
        out.push_str("# HELP memory_retrieval_duration_seconds Retrieval request latency by operation and tier\n");
        out.push_str("# TYPE memory_retrieval_duration_seconds summary\n");
        for op in ALL_OPERATIONS {
            for tier in ALL_TIERS {
                self.write_duration_lines(&mut out, op, tier);
            }
        }

        out.push_str("# HELP memory_retrieval_candidates Candidate-set sizes by pipeline stage\n");
        out.push_str("# TYPE memory_retrieval_candidates summary\n");
        for stage in [RetrievalStage::Cascade, RetrievalStage::Scored] {
            let count = self.candidates_count[stage.index()].load(Ordering::Relaxed);
            if count == 0 {
                continue;
            }
            let sum = self.candidates_sum[stage.index()].load(Ordering::Relaxed);
            let _ = writeln!(
                out,
                "memory_retrieval_candidates_sum{{stage=\"{}\"}} {}",
                stage.as_str(),
                sum
            );
            let _ = writeln!(
                out,
                "memory_retrieval_candidates_count{{stage=\"{}\"}} {}",
                stage.as_str(),
                count
            );
        }

        out.push_str("# HELP memory_cache_requests_total Cache lookups by layer and result\n");
        out.push_str("# TYPE memory_cache_requests_total counter\n");
        for outcome in ALL_OUTCOMES {
            let count =
                self.cache[CacheLayer::Query.index()][outcome.index()].load(Ordering::Relaxed);
            if count == 0 {
                continue;
            }
            let _ = writeln!(
                out,
                "memory_cache_requests_total{{layer=\"query\",result=\"{}\"}} {}",
                outcome.as_str(),
                count
            );
        }

        out.push_str(
            "# HELP memory_embedding_requests_total Embedding calls by provider and result\n",
        );
        out.push_str("# TYPE memory_embedding_requests_total counter\n");
        out.push_str(
            "# HELP memory_embedding_duration_seconds Embedding call latency by provider\n",
        );
        out.push_str("# TYPE memory_embedding_duration_seconds summary\n");
        for provider in ALL_PROVIDERS {
            for outcome in [EmbeddingOutcome::Ok, EmbeddingOutcome::Error] {
                let count =
                    self.embeddings[provider.index()][outcome.index()].load(Ordering::Relaxed);
                if count == 0 {
                    continue;
                }
                let _ = writeln!(
                    out,
                    "memory_embedding_requests_total{{provider=\"{}\",result=\"{}\"}} {}",
                    provider.as_str(),
                    outcome.as_str(),
                    count
                );
            }
            let latency = self.embedding_durations_ms[provider.index()].lock();
            if latency.count() > 0 {
                let (p50, p95, p99) = latency.percentiles_ms();
                for (quantile, value_ms) in [("0.5", p50), ("0.95", p95), ("0.99", p99)] {
                    let _ = writeln!(
                        out,
                        "memory_embedding_duration_seconds{{provider=\"{}\",quantile=\"{}\"}} {:.3}",
                        provider.as_str(),
                        quantile,
                        value_ms as f64 / 1000.0
                    );
                }
            }
        }

        out.push_str(
            "# HELP memory_retrieval_fallback_total Tier-4 fallback decisions by reason\n",
        );
        out.push_str("# TYPE memory_retrieval_fallback_total counter\n");
        for (i, reason) in FALLBACK_REASONS.iter().enumerate() {
            let count = self.fallbacks[i].load(Ordering::Relaxed);
            if count == 0 {
                continue;
            }
            let _ = writeln!(
                out,
                "memory_retrieval_fallback_total{{reason=\"{reason}\"}} {count}"
            );
        }

        out.push_str(
            "# HELP memory_recommendation_feedback_total Accepted feedback by outcome signal\n",
        );
        out.push_str("# TYPE memory_recommendation_feedback_total counter\n");
        for signal in ALL_SIGNALS {
            let count = self.feedback[signal.index()].load(Ordering::Relaxed);
            if count == 0 {
                continue;
            }
            let _ = writeln!(
                out,
                "memory_recommendation_feedback_total{{signal=\"{}\"}} {}",
                signal.as_str(),
                count
            );
        }
        out
    }

    /// Nonzero fallback counts keyed by reason.
    fn str_map(&self, cells: &[AtomicU64], names: &[&str]) -> Value {
        let map: serde_json::Map<String, Value> = cells
            .iter()
            .zip(names.iter().copied())
            .filter_map(|(cell, name)| {
                let count = cell.load(Ordering::Relaxed);
                (count > 0).then(|| (name.to_string(), json!(count)))
            })
            .collect();
        Value::Object(map)
    }

    /// Nonzero feedback counts keyed by signal.
    fn signal_map(&self) -> Value {
        let map: serde_json::Map<String, Value> = ALL_SIGNALS
            .iter()
            .filter_map(|signal| {
                let count = self.feedback[signal.index()].load(Ordering::Relaxed);
                (count > 0).then(|| (signal.as_str().to_string(), json!(count)))
            })
            .collect();
        Value::Object(map)
    }

    /// Nonzero cache counts keyed by result.
    fn cache_map(&self) -> Value {
        let map: serde_json::Map<String, Value> = ALL_OUTCOMES
            .iter()
            .filter_map(|outcome| {
                let count =
                    self.cache[CacheLayer::Query.index()][outcome.index()].load(Ordering::Relaxed);
                (count > 0).then(|| (outcome.as_str().to_string(), json!(count)))
            })
            .collect();
        Value::Object(map)
    }

    /// Nonzero embedding counts keyed by provider and result.
    fn embeddings_map(&self) -> Value {
        let map: serde_json::Map<String, Value> = ALL_PROVIDERS
            .iter()
            .filter_map(|provider| {
                let ok = self.embeddings[provider.index()][EmbeddingOutcome::Ok.index()]
                    .load(Ordering::Relaxed);
                let err = self.embeddings[provider.index()][EmbeddingOutcome::Error.index()]
                    .load(Ordering::Relaxed);
                (ok + err > 0).then(|| {
                    (
                        provider.as_str().to_string(),
                        json!({"ok": ok, "error": err}),
                    )
                })
            })
            .collect();
        Value::Object(map)
    }

    /// Candidate sum/count keyed by stage.
    fn candidates_map(&self) -> Value {
        let stages = [RetrievalStage::Cascade, RetrievalStage::Scored];
        let map: serde_json::Map<String, Value> = stages
            .iter()
            .filter_map(|stage| {
                let count = self.candidates_count[stage.index()].load(Ordering::Relaxed);
                (count > 0).then(|| {
                    let sum = self.candidates_sum[stage.index()].load(Ordering::Relaxed);
                    (
                        stage.as_str().to_string(),
                        json!({"observations": count, "total": sum}),
                    )
                })
            })
            .collect();
        Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::metrics::FallbackReason;

    #[test]
    fn exposition_carries_all_issue_families() {
        let metrics = RetrievalMetrics::new();
        metrics.record_cache(CacheLayer::Query, RetrievalOutcome::Miss);
        metrics.record_embedding(EmbeddingProviderLabel::Local, EmbeddingOutcome::Ok, 20);
        metrics.record_fallback(FallbackReason::LocalConfident);
        metrics.record_feedback(FeedbackSignal::Success);
        metrics.record_candidates(RetrievalStage::Cascade, 7);

        let text = metrics.export_prometheus();
        for family in [
            "memory_cache_requests_total",
            "memory_embedding_requests_total",
            "memory_embedding_duration_seconds",
            "memory_retrieval_fallback_total",
            "memory_recommendation_feedback_total",
            "memory_retrieval_candidates_sum",
        ] {
            assert!(text.contains(family), "missing family {family}");
        }
        assert!(text.contains("reason=\"local_confident\""));
        assert!(text.contains("signal=\"success\""));
        assert!(text.contains("provider=\"local\""));
    }

    #[test]
    fn type_lines_appear_once_per_family() {
        let metrics = RetrievalMetrics::new();
        metrics.record_request(
            RetrievalOperation::Query,
            RetrievalTier::Cache,
            RetrievalOutcome::Hit,
            4,
        );
        metrics.record_request(
            RetrievalOperation::Query,
            RetrievalTier::Cache,
            RetrievalOutcome::Miss,
            8,
        );

        let text = metrics.export_prometheus();
        assert_eq!(
            text.matches("# TYPE memory_retrieval_duration_seconds")
                .count(),
            1,
            "duplicate TYPE line invalidates exposition"
        );
        // One duration series per (operation, tier), not per outcome.
        assert_eq!(
            text.matches("memory_retrieval_duration_seconds{operation=\"query\",tier=\"cache\"")
                .count(),
            3,
            "expected exactly the 0.5/0.95/0.99 quantile series"
        );
    }
}
