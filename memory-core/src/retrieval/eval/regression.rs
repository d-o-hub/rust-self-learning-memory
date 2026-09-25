//! Regression checking and report formatting for retrieval evaluation benchmarks.

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

use super::types::{BenchmarkMetrics, BenchmarkReport, EVIDENCE_DIMENSIONS, RegressionThresholds};

/// Results of a regression check comparing current run against baseline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionCheckResult {
    /// True if no regression thresholds were exceeded.
    pub passed: bool,
    /// List of human-readable threshold violation details.
    pub violations: Vec<String>,
    /// Summary message.
    pub summary: String,
}

/// Evaluates benchmark metrics against a baseline report to detect regressions.
pub struct RegressionChecker {
    thresholds: RegressionThresholds,
}

impl RegressionChecker {
    /// Create a checker with specific threshold limits.
    #[must_use]
    pub fn new(thresholds: RegressionThresholds) -> Self {
        Self { thresholds }
    }

    /// Compare current benchmark report against baseline report.
    pub fn check(
        &self,
        current: &BenchmarkReport,
        baseline: &BenchmarkReport,
    ) -> RegressionCheckResult {
        self.check_ignoring(current, baseline, &[])
    }

    /// Compare current report against baseline, skipping comparison-only strategies.
    ///
    /// `comparison_only` names strategies that are reported on demand and are
    /// deliberately absent from baseline artifacts — currently the semantic
    /// rerank arm ([`crate::retrieval::eval::RERANK_COMPARISON_STRATEGY`]) and
    /// the evidence-classification arm
    /// ([`crate::retrieval::eval::EVIDENCE_COMPARISON_STRATEGY`]). Such a
    /// strategy is skipped when the baseline has no counterpart instead of being
    /// flagged, so `--rerank` and `--evidence` runs stay comparable against the
    /// shipped baseline. Every other strategy keeps the strict "present in
    /// current run but missing in baseline" check, and a comparison-only strategy
    /// that *is* present in the baseline is still regression-checked.
    ///
    /// Evidence false drops are *not* skipped: they are checked for every
    /// strategy in the current run, including one the baseline does not track.
    pub fn check_ignoring(
        &self,
        current: &BenchmarkReport,
        baseline: &BenchmarkReport,
        comparison_only: &[&str],
    ) -> RegressionCheckResult {
        let mut violations = Vec::new();

        for (strategy_name, curr_m) in &current.strategies {
            if let Some(base_m) = baseline.strategies.get(strategy_name) {
                self.check_strategy(strategy_name, curr_m, base_m, &mut violations);
            } else if !comparison_only.contains(&strategy_name.as_str()) {
                violations.push(format!(
                    "Strategy '{strategy_name}' present in current run but missing in baseline"
                ));
            }

            // Evidence false drops (issue #1032): dropping a labelled candidate
            // the fixture expects to keep is a classification defect, not a
            // trade-off, so any non-zero count fails the check — including for a
            // comparison-only arm the baseline does not track, which would
            // otherwise never be gated.
            let false_drops = curr_m
                .evidence_metrics
                .as_ref()
                .filter(|metrics| metrics.false_drop_count > 0);
            if let Some(evidence) = false_drops {
                violations.push(format!(
                    "[{strategy_name}] Evidence classification false-dropped {} labelled keep candidate(s) (rate {:.4}); false drops are release-blocking",
                    evidence.false_drop_count, evidence.false_drop_rate
                ));
            }
        }

        let passed = violations.is_empty();
        let summary = if passed {
            format!(
                "PASS: Evaluation passed all regression thresholds against baseline (version: {})",
                baseline.corpus_version
            )
        } else {
            format!(
                "FAIL: Found {} regression violation(s) against baseline (version: {})",
                violations.len(),
                baseline.corpus_version
            )
        };

        RegressionCheckResult {
            passed,
            violations,
            summary,
        }
    }

    fn check_strategy(
        &self,
        strategy_name: &str,
        curr: &BenchmarkMetrics,
        base: &BenchmarkMetrics,
        violations: &mut Vec<String>,
    ) {
        // Recall@5 regression
        let recall_drop = base.recall_at_5 - curr.recall_at_5;
        if recall_drop > self.thresholds.max_recall_drop {
            violations.push(format!(
                "[{strategy_name}] Recall@5 dropped by {:.4} (baseline: {:.4}, current: {:.4}), exceeding max drop of {:.4}",
                recall_drop, base.recall_at_5, curr.recall_at_5, self.thresholds.max_recall_drop
            ));
        }

        // MRR regression
        let mrr_drop = base.mrr - curr.mrr;
        if mrr_drop > self.thresholds.max_mrr_drop {
            violations.push(format!(
                "[{strategy_name}] MRR dropped by {:.4} (baseline: {:.4}, current: {:.4}), exceeding max drop of {:.4}",
                mrr_drop, base.mrr, curr.mrr, self.thresholds.max_mrr_drop
            ));
        }

        // NDCG@5 regression
        let ndcg_drop = base.ndcg_at_5 - curr.ndcg_at_5;
        if ndcg_drop > self.thresholds.max_ndcg_drop {
            violations.push(format!(
                "[{strategy_name}] NDCG@5 dropped by {:.4} (baseline: {:.4}, current: {:.4}), exceeding max drop of {:.4}",
                ndcg_drop, base.ndcg_at_5, curr.ndcg_at_5, self.thresholds.max_ndcg_drop
            ));
        }

        // Latency regression (P95)
        if base.end_to_end_latency.p95_us > 0 {
            let lat_increase_ratio = (curr.end_to_end_latency.p95_us as f64
                - base.end_to_end_latency.p95_us as f64)
                / base.end_to_end_latency.p95_us as f64;
            if lat_increase_ratio > self.thresholds.max_latency_increase_ratio {
                violations.push(format!(
                    "[{strategy_name}] P95 latency increased by {:.1}% (baseline: {}μs, current: {}μs), exceeding max allowed increase of {:.1}%",
                    lat_increase_ratio * 100.0,
                    base.end_to_end_latency.p95_us,
                    curr.end_to_end_latency.p95_us,
                    self.thresholds.max_latency_increase_ratio * 100.0
                ));
            }
        }

        // Cost regression
        if base.estimated_cost_per_query > 0.0 {
            let cost_increase_ratio = (curr.estimated_cost_per_query
                - base.estimated_cost_per_query)
                / base.estimated_cost_per_query;
            if cost_increase_ratio > self.thresholds.max_cost_increase_ratio {
                violations.push(format!(
                    "[{strategy_name}] Estimated cost per query increased by {:.1}% (baseline: ${:.6}, current: ${:.6}), exceeding max allowed increase of {:.1}%",
                    cost_increase_ratio * 100.0,
                    base.estimated_cost_per_query,
                    curr.estimated_cost_per_query,
                    self.thresholds.max_cost_increase_ratio * 100.0
                ));
            }
        }
    }
}

/// Format benchmark report and optional regression check into Markdown.
pub fn format_markdown_report(
    report: &BenchmarkReport,
    check_result: Option<&RegressionCheckResult>,
) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "# Retrieval Evaluation Quality & Cost Report");
    let _ = writeln!(out);
    let _ = writeln!(out, "- **Timestamp**: {}", report.timestamp);
    let _ = writeln!(out, "- **Corpus Version**: {}", report.corpus_version);
    let _ = writeln!(out, "- **Corpus Size**: {} items", report.corpus_size);
    let _ = writeln!(
        out,
        "- **Evaluation Queries**: {} queries",
        report.query_count
    );
    let _ = writeln!(out);

    if let Some(res) = check_result {
        let _ = writeln!(out, "## Regression Check Result");
        let _ = writeln!(out);
        if res.passed {
            let _ = writeln!(out, "**Status**: ✅ **PASSED**");
        } else {
            let _ = writeln!(out, "**Status**: ❌ **FAILED**");
        }
        let _ = writeln!(out, "{}", res.summary);
        if !res.violations.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(out, "### Violations");
            for v in &res.violations {
                let _ = writeln!(out, "- {v}");
            }
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "## Quality & Accuracy Metrics");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| Strategy | Recall@1 | Recall@5 | Recall@10 | MRR | NDCG@5 | Rec Acceptance |"
    );
    let _ = writeln!(out, "|:---|:---:|:---:|:---:|:---:|:---:|:---:|");

    for (name, m) in &report.strategies {
        let _ = writeln!(
            out,
            "| **{}** | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.1}% |",
            name,
            m.recall_at_1,
            m.recall_at_5,
            m.recall_at_10,
            m.mrr,
            m.ndcg_at_5,
            m.recommendation_success_rate * 100.0
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## Tier Distribution & External API Usage");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| Strategy | Tier 1 (BM25) | Tier 2 (HDC) | Tier 3 (Concept) | Tier 4 (API) | API Calls/Query |"
    );
    let _ = writeln!(out, "|:---|:---:|:---:|:---:|:---:|:---:|");

    for (name, m) in &report.strategies {
        let _ = writeln!(
            out,
            "| **{}** | {:.1}% ({}) | {:.1}% ({}) | {:.1}% ({}) | {:.1}% ({}) | {:.2} |",
            name,
            m.tier_distribution.tier1_percentage,
            m.tier_distribution.tier1_bm25_count,
            m.tier_distribution.tier2_percentage,
            m.tier_distribution.tier2_hdc_count,
            m.tier_distribution.tier3_percentage,
            m.tier_distribution.tier3_concept_graph_count,
            m.tier_distribution.tier4_percentage,
            m.tier_distribution.tier4_api_count,
            m.embedding_calls_per_query
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## Performance, Candidate Sets & Costs");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| Strategy | Local P50/P95 (μs) | E2E P50/P95 (μs) | Candidates (Pre/Post) | Cost / Query | Cost / Rec |"
    );
    let _ = writeln!(out, "|:---|:---:|:---:|:---:|:---:|:---:|");

    for (name, m) in &report.strategies {
        let _ = writeln!(
            out,
            "| **{}** | {} / {} | {} / {} | {:.1} → {:.1} | ${:.6} | ${:.6} |",
            name,
            m.local_latency.p50_us,
            m.local_latency.p95_us,
            m.end_to_end_latency.p50_us,
            m.end_to_end_latency.p95_us,
            m.avg_candidate_set_before_ranking,
            m.avg_candidate_set_after_ranking,
            m.estimated_cost_per_query,
            m.estimated_cost_per_successful_rec
        );
    }
    let _ = writeln!(out);

    // Rendered only when the run actually exercised the semantic rerank arm, so
    // reports without reranking stay unchanged (issue #1031).
    let reranked = report.strategies.values().any(|m| {
        m.judge_calls_per_query > 0.0
            || m.rerank_candidates_per_query > 0.0
            || m.top1_changed_rate > 0.0
    });

    if reranked {
        let _ = writeln!(out, "## Semantic Rerank Comparison");
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "| Strategy | Judge Calls / Query | Rerank Candidates / Query | Top-1 Changed |"
        );
        let _ = writeln!(out, "|:---|:---:|:---:|:---:|");

        for (name, m) in &report.strategies {
            let _ = writeln!(
                out,
                "| **{}** | {:.2} | {:.2} | {:.1}% |",
                name,
                m.judge_calls_per_query,
                m.rerank_candidates_per_query,
                m.top1_changed_rate * 100.0
            );
        }
        let _ = writeln!(out);
    }

    // Rendered only when the run actually exercised the evidence arm, so
    // reports without evidence classification stay unchanged (issue #1032).
    let classified = report
        .strategies
        .values()
        .any(|m| m.evidence_metrics.is_some());

    if classified {
        let _ = writeln!(out, "## Evidence Classification");
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "| Strategy | Dimension | Precision | Recall | TP | FP | FN | TN |"
        );
        let _ = writeln!(out, "|:---|:---|:---:|:---:|:---:|:---:|:---:|:---:|");
        for (name, m) in &report.strategies {
            let Some(evidence) = m.evidence_metrics.as_ref() else {
                continue;
            };
            for (dimension, metric) in EVIDENCE_DIMENSIONS.iter().zip(&evidence.per_dimension) {
                let _ = writeln!(
                    out,
                    "| **{}** | `{}` | {:.3} | {:.3} | {} | {} | {} | {} |",
                    name,
                    dimension,
                    metric.precision,
                    metric.recall,
                    metric.tp,
                    metric.fp,
                    metric.fn_,
                    metric.tn
                );
            }
        }
        let _ = writeln!(out);

        let _ = writeln!(
            out,
            "| Strategy | Keep | Flag | Demote | Drop | Total | False Drops | False Drop Rate |"
        );
        let _ = writeln!(out, "|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|");
        for (name, m) in &report.strategies {
            let Some(evidence) = m.evidence_metrics.as_ref() else {
                continue;
            };
            let counts = &evidence.dispositions;
            let _ = writeln!(
                out,
                "| **{}** | {} | {} | {} | {} | {} | {} | {:.4} |",
                name,
                counts.keep,
                counts.flag,
                counts.demote,
                counts.drop,
                counts.total,
                evidence.false_drop_count,
                evidence.false_drop_rate
            );
        }
        let _ = writeln!(out);

        let _ = writeln!(
            out,
            "| Strategy | Judge Calls / Query | Added P50 (μs) | Added P95 (μs) |"
        );
        let _ = writeln!(out, "|:---|:---:|:---:|:---:|");
        for (name, m) in &report.strategies {
            let Some(evidence) = m.evidence_metrics.as_ref() else {
                continue;
            };
            let _ = writeln!(
                out,
                "| **{}** | {:.2} | {} | {} |",
                name,
                evidence.judge_calls_per_query,
                evidence.added_latency_p50_us,
                evidence.added_latency_p95_us
            );
        }
        let _ = writeln!(out);
    }

    out
}
