//! Evidence-classification telemetry rendering (issue #1032): the bounded
//! Prometheus families and the JSON `evidence` snapshot block. Zero-valued
//! series are omitted from both renderings.

use serde_json::{Value, json};
use std::fmt::Write as _;
use std::sync::atomic::Ordering;

use super::super::labels::{DISPOSITIONS, EvidenceStatus};
use super::super::registry::RetrievalMetrics;

const ALL_EVIDENCE_STATUSES: [EvidenceStatus; 6] = [
    EvidenceStatus::Disabled,
    EvidenceStatus::NotConfigured,
    EvidenceStatus::Applied,
    EvidenceStatus::LowConfidence,
    EvidenceStatus::ProviderError,
    EvidenceStatus::Invalid,
];

impl RetrievalMetrics {
    /// Evidence families emit their header only alongside samples: without a
    /// judge call there is nothing to report.
    pub(super) fn write_evidence_lines(&self, out: &mut String) {
        let status_counts = ALL_EVIDENCE_STATUSES
            .map(|status| self.evidence[status.index()].load(Ordering::Relaxed));
        if status_counts.iter().any(|&count| count > 0) {
            out.push_str(
                "# HELP memory_evidence_requests_total Evidence-classification invocations by status\n",
            );
            out.push_str("# TYPE memory_evidence_requests_total counter\n");
            for (status, count) in ALL_EVIDENCE_STATUSES.iter().zip(status_counts) {
                if count == 0 {
                    continue;
                }
                let _ = writeln!(
                    out,
                    "memory_evidence_requests_total{{status=\"{}\"}} {}",
                    status.as_str(),
                    count
                );
            }
        }

        let mut has_dispositions = false;
        for cell in &self.evidence_dispositions {
            if cell.load(Ordering::Relaxed) > 0 {
                has_dispositions = true;
                break;
            }
        }
        if has_dispositions {
            out.push_str(
                "# HELP memory_evidence_dispositions_total Classified candidate dispositions\n",
            );
            out.push_str("# TYPE memory_evidence_dispositions_total counter\n");
            for (disposition, cell) in DISPOSITIONS.iter().zip(&self.evidence_dispositions) {
                let count = cell.load(Ordering::Relaxed);
                if count == 0 {
                    continue;
                }
                let _ = writeln!(
                    out,
                    "memory_evidence_dispositions_total{{disposition=\"{disposition}\"}} {count}"
                );
            }
        }

        let evidence_latency = self.evidence_durations_ms.lock();
        if evidence_latency.count() > 0 {
            out.push_str(
                "# HELP memory_evidence_duration_seconds Judge-backed evidence-classification latency\n",
            );
            out.push_str("# TYPE memory_evidence_duration_seconds summary\n");
            let (p50, p95, p99) = evidence_latency.percentiles_ms();
            for (quantile, value_ms) in [("0.5", p50), ("0.95", p95), ("0.99", p99)] {
                let _ = writeln!(
                    out,
                    "memory_evidence_duration_seconds{{quantile=\"{}\"}} {:.3}",
                    quantile,
                    value_ms as f64 / 1000.0
                );
            }
        }
        drop(evidence_latency);

        let candidate_count = self.evidence_candidates_count.load(Ordering::Relaxed);
        if candidate_count > 0 {
            let candidate_sum = self.evidence_candidates_sum.load(Ordering::Relaxed);
            out.push_str("# HELP memory_evidence_candidates Candidates classified for evidence\n");
            out.push_str("# TYPE memory_evidence_candidates summary\n");
            let _ = writeln!(out, "memory_evidence_candidates_sum {candidate_sum}");
            let _ = writeln!(out, "memory_evidence_candidates_count {candidate_count}");
        }
    }

    /// Nonzero evidence status and disposition counts plus candidate and
    /// duration summaries.
    pub(super) fn evidence_map(&self) -> Value {
        let mut map = serde_json::Map::new();
        for status in ALL_EVIDENCE_STATUSES {
            let count = self.evidence[status.index()].load(Ordering::Relaxed);
            if count > 0 {
                map.insert(format!("status={}", status.as_str()), json!(count));
            }
        }
        let dispositions: serde_json::Map<String, Value> = DISPOSITIONS
            .iter()
            .zip(&self.evidence_dispositions)
            .filter_map(|(disposition, cell)| {
                let count = cell.load(Ordering::Relaxed);
                (count > 0).then(|| ((*disposition).to_string(), json!(count)))
            })
            .collect();
        if !dispositions.is_empty() {
            map.insert("dispositions".to_string(), Value::Object(dispositions));
        }
        let candidate_count = self.evidence_candidates_count.load(Ordering::Relaxed);
        if candidate_count > 0 {
            let candidate_sum = self.evidence_candidates_sum.load(Ordering::Relaxed);
            map.insert(
                "candidates".to_string(),
                json!({"observations": candidate_count, "total": candidate_sum}),
            );
        }
        let evidence_latency = self.evidence_durations_ms.lock();
        if evidence_latency.count() > 0 {
            let (p50, p95, p99) = evidence_latency.percentiles_ms();
            map.insert(
                "duration_ms".to_string(),
                json!({"p50": p50, "p95": p95, "p99": p99}),
            );
        }
        Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_families_omit_zero_series() {
        let metrics = RetrievalMetrics::new();
        // Nothing judge-backed has run: every evidence family stays silent.
        assert!(!metrics.export_prometheus().contains("memory_evidence"));
        let idle = metrics.snapshot();
        let idle_evidence = idle["evidence"].as_object().unwrap();
        assert!(idle_evidence.is_empty());

        metrics.record_evidence(EvidenceStatus::Disabled, 9, [9, 0, 0, 0], 4);
        metrics.record_evidence(EvidenceStatus::Applied, 20, [12, 3, 4, 1], 30);

        let text = metrics.export_prometheus();
        assert!(text.contains("memory_evidence_requests_total{status=\"disabled\"} 1"));
        assert!(text.contains("memory_evidence_requests_total{status=\"applied\"} 1"));
        assert!(!text.contains("status=\"provider_error\""));
        let requests_type_line = "# TYPE memory_evidence_requests_total";
        assert_eq!(text.matches(requests_type_line).count(), 1);
        // Dispositions and sizes describe the judge-backed call only.
        assert!(text.contains("memory_evidence_dispositions_total{disposition=\"keep\"} 12"));
        assert!(text.contains("memory_evidence_dispositions_total{disposition=\"flag\"} 3"));
        assert!(text.contains("memory_evidence_dispositions_total{disposition=\"demote\"} 4"));
        assert!(text.contains("memory_evidence_dispositions_total{disposition=\"drop\"} 1"));
        assert!(text.contains("memory_evidence_candidates_sum 20"));
        assert!(text.contains("memory_evidence_candidates_count 1"));
        let duration_type_line = "# TYPE memory_evidence_duration_seconds";
        assert_eq!(text.matches(duration_type_line).count(), 1);
        assert!(text.contains("memory_evidence_duration_seconds{quantile=\"0.5\"} 0.030"));

        let evidence = metrics.snapshot()["evidence"].clone();
        assert_eq!(evidence["status=disabled"], 1);
        assert_eq!(evidence["status=applied"], 1);
        assert!(evidence.get("status=provider_error").is_none());
        assert_eq!(evidence["dispositions"]["keep"], 12);
        assert_eq!(evidence["dispositions"]["flag"], 3);
        assert_eq!(evidence["dispositions"]["demote"], 4);
        assert_eq!(evidence["dispositions"]["drop"], 1);
        assert_eq!(evidence["candidates"]["observations"], 1);
        assert_eq!(evidence["candidates"]["total"], 20);
        assert_eq!(evidence["duration_ms"]["p50"], 30);
    }

    /// A zero-count disposition is omitted while the family header still renders
    /// exactly once (the zero-skip path of the dispositions loop).
    #[test]
    fn evidence_dispositions_omit_zero_counts() {
        let metrics = RetrievalMetrics::new();
        metrics.record_evidence(EvidenceStatus::Applied, 2, [1, 0, 0, 0], 5);

        let text = metrics.export_prometheus();
        assert!(text.contains("memory_evidence_dispositions_total{disposition=\"keep\"} 1"));
        assert!(!text.contains("memory_evidence_dispositions_total{disposition=\"flag\""));
        assert!(!text.contains("memory_evidence_dispositions_total{disposition=\"demote\""));
        assert!(!text.contains("memory_evidence_dispositions_total{disposition=\"drop\""));
        assert_eq!(
            text.matches("# TYPE memory_evidence_dispositions_total")
                .count(),
            1,
            "duplicate TYPE line invalidates exposition"
        );
    }
}
