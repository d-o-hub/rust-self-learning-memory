//! Rerank telemetry rendering (issue #1031): the six bounded Prometheus
//! families and the JSON `rerank` snapshot block. Zero-valued series are
//! omitted from both renderings.

use serde_json::{Value, json};
use std::fmt::Write as _;
use std::sync::atomic::Ordering;

use super::super::labels::RerankStatus;
use super::super::registry::RetrievalMetrics;

const ALL_RERANK_STATUSES: [RerankStatus; 6] = [
    RerankStatus::Disabled,
    RerankStatus::NotConfigured,
    RerankStatus::Applied,
    RerankStatus::LowConfidence,
    RerankStatus::ProviderError,
    RerankStatus::Invalid,
];

impl RetrievalMetrics {
    /// Rerank families emit their header only alongside samples: without a
    /// judge call there is nothing to report.
    pub(super) fn write_rerank_lines(&self, out: &mut String) {
        let rerank_counts =
            ALL_RERANK_STATUSES.map(|status| self.rerank[status.index()].load(Ordering::Relaxed));
        if rerank_counts.iter().any(|&count| count > 0) {
            out.push_str(
                "# HELP memory_rerank_requests_total Semantic rerank invocations by status\n",
            );
            out.push_str("# TYPE memory_rerank_requests_total counter\n");
            for (status, count) in ALL_RERANK_STATUSES.iter().zip(rerank_counts) {
                if count == 0 {
                    continue;
                }
                let _ = writeln!(
                    out,
                    "memory_rerank_requests_total{{status=\"{}\"}} {}",
                    status.as_str(),
                    count
                );
            }
        }

        let rerank_latency = self.rerank_durations_ms.lock();
        if rerank_latency.count() > 0 {
            out.push_str("# HELP memory_rerank_duration_seconds Judge-backed rerank latency\n");
            out.push_str("# TYPE memory_rerank_duration_seconds summary\n");
            let (p50, p95, p99) = rerank_latency.percentiles_ms();
            for (quantile, value_ms) in [("0.5", p50), ("0.95", p95), ("0.99", p99)] {
                let _ = writeln!(
                    out,
                    "memory_rerank_duration_seconds{{quantile=\"{}\"}} {:.3}",
                    quantile,
                    value_ms as f64 / 1000.0
                );
            }
        }
        drop(rerank_latency);

        let shortlist_count = self.rerank_shortlist_count.load(Ordering::Relaxed);
        if shortlist_count > 0 {
            let shortlist_sum = self.rerank_shortlist_sum.load(Ordering::Relaxed);
            out.push_str(
                "# HELP memory_rerank_shortlist Candidate-set sizes entering semantic rerank\n",
            );
            out.push_str("# TYPE memory_rerank_shortlist summary\n");
            let _ = writeln!(out, "memory_rerank_shortlist_sum {shortlist_sum}");
            let _ = writeln!(out, "memory_rerank_shortlist_count {shortlist_count}");
        }

        let output_count = self.rerank_output_count.load(Ordering::Relaxed);
        if output_count > 0 {
            let output_sum = self.rerank_output_sum.load(Ordering::Relaxed);
            out.push_str("# HELP memory_rerank_output Reranked result sizes returned\n");
            out.push_str("# TYPE memory_rerank_output summary\n");
            let _ = writeln!(out, "memory_rerank_output_sum {output_sum}");
            let _ = writeln!(out, "memory_rerank_output_count {output_count}");
        }

        let top1_changed = self.rerank_top1_changed.load(Ordering::Relaxed);
        if top1_changed > 0 {
            out.push_str(
                "# HELP memory_rerank_top1_changed_total Rerank invocations that changed the top-ranked candidate\n",
            );
            out.push_str("# TYPE memory_rerank_top1_changed_total counter\n");
            let _ = writeln!(out, "memory_rerank_top1_changed_total {top1_changed}");
        }

        let confidence_count = self.rerank_confidence_count.load(Ordering::Relaxed);
        if confidence_count > 0 {
            // Stored as micro-units; exported back to 0..1.
            let confidence_sum = self.rerank_confidence_sum_micro.load(Ordering::Relaxed);
            out.push_str(
                "# HELP memory_rerank_confidence Judge-reported relevance confidence (0..1)\n",
            );
            out.push_str("# TYPE memory_rerank_confidence summary\n");
            let _ = writeln!(
                out,
                "memory_rerank_confidence_sum {:.6}",
                micro_confidence(confidence_sum)
            );
            let _ = writeln!(out, "memory_rerank_confidence_count {confidence_count}");
        }
    }

    /// Nonzero rerank status counts plus shortlist, output, top-1, and
    /// confidence summaries.
    pub(super) fn rerank_map(&self) -> Value {
        let mut map = serde_json::Map::new();
        for status in ALL_RERANK_STATUSES {
            let count = self.rerank[status.index()].load(Ordering::Relaxed);
            if count > 0 {
                map.insert(format!("status={}", status.as_str()), json!(count));
            }
        }
        let shortlist_count = self.rerank_shortlist_count.load(Ordering::Relaxed);
        if shortlist_count > 0 {
            let shortlist_sum = self.rerank_shortlist_sum.load(Ordering::Relaxed);
            map.insert(
                "shortlist".to_string(),
                json!({"observations": shortlist_count, "total": shortlist_sum}),
            );
        }
        let output_count = self.rerank_output_count.load(Ordering::Relaxed);
        if output_count > 0 {
            let output_sum = self.rerank_output_sum.load(Ordering::Relaxed);
            map.insert(
                "output".to_string(),
                json!({"observations": output_count, "total": output_sum}),
            );
        }
        let top1_changed = self.rerank_top1_changed.load(Ordering::Relaxed);
        if top1_changed > 0 {
            map.insert("top1_changed".to_string(), json!(top1_changed));
        }
        let confidence_count = self.rerank_confidence_count.load(Ordering::Relaxed);
        if confidence_count > 0 {
            let confidence_sum = self.rerank_confidence_sum_micro.load(Ordering::Relaxed);
            // Mean micro-unit observation, rounded to six decimals.
            let avg = (confidence_sum as f64 / confidence_count as f64).round() / 1_000_000.0;
            map.insert(
                "confidence".to_string(),
                json!({"observations": confidence_count, "avg": avg}),
            );
        }
        Value::Object(map)
    }
}

/// Micro-unit confidence accumulator back to a 0..1 value.
fn micro_confidence(micro: u64) -> f64 {
    micro as f64 / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rerank_families_omit_zero_series() {
        let metrics = RetrievalMetrics::new();
        // Nothing judge-backed has run: every rerank family stays silent.
        assert!(!metrics.export_prometheus().contains("memory_rerank"));
        assert!(metrics.snapshot()["rerank"].as_object().unwrap().is_empty());

        metrics.record_rerank(RerankStatus::Disabled, 5, 2, 4, true, 0.9);
        metrics.record_rerank(RerankStatus::Applied, 20, 10, 30, true, 0.75);

        let text = metrics.export_prometheus();
        assert!(text.contains("memory_rerank_requests_total{status=\"disabled\"} 1"));
        assert!(text.contains("memory_rerank_requests_total{status=\"applied\"} 1"));
        assert!(!text.contains("status=\"provider_error\""));
        // Timing, sizes, and confidence describe the judge-backed call only.
        assert_eq!(
            text.matches("# TYPE memory_rerank_duration_seconds")
                .count(),
            1
        );
        assert!(text.contains("memory_rerank_shortlist_sum 20"));
        assert!(text.contains("memory_rerank_shortlist_count 1"));
        assert!(text.contains("memory_rerank_output_sum 10"));
        assert!(text.contains("memory_rerank_output_count 1"));
        assert!(text.contains("memory_rerank_top1_changed_total 1"));
        assert!(text.contains("memory_rerank_confidence_sum 0.750000"));
        assert!(text.contains("memory_rerank_confidence_count 1"));

        let rerank = metrics.snapshot()["rerank"].clone();
        assert_eq!(rerank["status=disabled"], 1);
        assert_eq!(rerank["status=applied"], 1);
        assert_eq!(rerank["shortlist"]["observations"], 1);
        assert_eq!(rerank["shortlist"]["total"], 20);
        assert_eq!(rerank["output"]["observations"], 1);
        assert_eq!(rerank["output"]["total"], 10);
        assert_eq!(rerank["top1_changed"], 1);
        assert_eq!(rerank["confidence"]["observations"], 1);
        assert_eq!(rerank["confidence"]["avg"], 0.75);
    }
}
