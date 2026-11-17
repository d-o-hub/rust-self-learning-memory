//! Reporting layer for the adaptive source-discovery example.
//!
//! [`scoring`] scores both plans against the fixture ground truth, [`fingerprint`]
//! makes a decision comparable across processes, and this module assembles the
//! report, renders the summary, and enforces the dataset expectations.

use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use crate::adaptive_discovery::dataset::Dataset;
use crate::adaptive_discovery::planner::Decision;

pub mod fingerprint;
pub mod scoring;

pub use fingerprint::fingerprint;
pub use scoring::{Deltas, Evaluation, evaluate};

/// Report schema version.
pub const REPORT_VERSION: &str = "1.0";

/// The disclaimer every report carries.
pub const ADVISORY_NOTICE: &str = "Memory recommendations are advisory. A discovered source is a candidate: the consuming \
     application must verify it independently before promoting it or taking any side effect.";

/// Redaction statement every report carries.
pub const REDACTION_NOTICE: &str = "Only scheme, host, and path of a candidate endpoint are persisted. URL userinfo, query \
     strings, fragments, credentials, and request headers are never recorded.";

/// Outcome of the dataset expectations.
#[derive(Debug, Clone, Serialize)]
pub struct ExpectationCheck {
    /// Number of expectations evaluated.
    pub checked: usize,
    /// Human-readable failures; empty when the run is acceptable.
    pub failures: Vec<String>,
    /// True when no failure was recorded.
    pub passed: bool,
}

/// The complete machine-readable report.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// Report schema version.
    pub schema_version: String,
    /// Dataset schema version the run used.
    pub dataset_version: String,
    /// Producer identity.
    pub generated_by: String,
    /// Advisory disclaimer.
    pub advisory: String,
    /// Redaction statement.
    pub redaction: String,
    /// Static baseline evaluation.
    pub baseline: Evaluation,
    /// Memory-guided evaluation.
    pub memory_guided: Evaluation,
    /// Memory-guided minus baseline.
    pub deltas: Deltas,
    /// One bounded decision per family.
    pub decisions: Vec<Decision>,
    /// Advisory patterns per family (never used for the decision).
    pub advisory_patterns: BTreeMap<String, Vec<String>>,
    /// Stable fingerprint of the decisions, comparable across processes.
    pub fingerprint: String,
    /// Dataset expectation verdict.
    pub expectations: ExpectationCheck,
}

impl Report {
    /// Write the report as pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when the parent directory cannot be created or the file
    /// cannot be written.
    pub fn write(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    anyhow::anyhow!("cannot create {}: {error}", parent.display())
                })?;
            }
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|error| anyhow::anyhow!("cannot serialise the report: {error}"))?;
        std::fs::write(path, json)
            .map_err(|error| anyhow::anyhow!("cannot write {}: {error}", path.display()))
    }

    /// Render a short human-readable summary.
    #[must_use]
    pub fn render_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("| Family | Baseline | Memory-guided | Recall | Precision | Reason |\n");
        out.push_str("|---|---|---|---|---|---|\n");
        for family in &self.baseline.families {
            let guided = self
                .memory_guided
                .families
                .iter()
                .find(|score| score.family == family.family);
            let decision = self
                .decisions
                .iter()
                .find(|decision| decision.family == family.family);
            let _ = writeln!(
                out,
                "| {} | {} | {} | {:.2} → {:.2} | {:.2} → {:.2} | {} |",
                family.family,
                family.plan,
                guided.map_or_else(|| "?".to_string(), |score| score.plan.to_string()),
                family.recall,
                guided.map_or(0.0, |score| score.recall),
                family.precision,
                guided.map_or(0.0, |score| score.precision),
                decision.map_or("?", |decision| decision.reason.as_str()),
            );
        }
        let _ = writeln!(
            out,
            "\nTotals: recall {:.2} → {:.2} ({:+.2}), precision {:.2} → {:.2} ({:+.2}), \
             diversity {:.2} → {:.2}, latency {}ms → {}ms, cost ${:.4} → ${:.4}",
            self.baseline.mean_recall,
            self.memory_guided.mean_recall,
            self.deltas.recall,
            self.baseline.mean_precision,
            self.memory_guided.mean_precision,
            self.deltas.precision,
            self.baseline.mean_diversity,
            self.memory_guided.mean_diversity,
            self.baseline.total_latency_ms,
            self.memory_guided.total_latency_ms,
            self.baseline.total_cost_usd,
            self.memory_guided.total_cost_usd,
        );
        let _ = writeln!(out, "Fingerprint: {}", self.fingerprint);
        if self.expectations.passed {
            out.push_str("Expectations: passed\n");
        } else {
            out.push_str("Expectations: FAILED\n");
            for failure in &self.expectations.failures {
                let _ = writeln!(out, "  - {failure}");
            }
        }
        out
    }
}

/// Compare the run against the dataset expectations.
#[must_use]
pub fn check_expectations(
    dataset: &Dataset,
    decisions: &[Decision],
    baseline: &Evaluation,
    guided: &Evaluation,
) -> ExpectationCheck {
    let mut failures = Vec::new();
    let mut checked = 0;

    checked += 1;
    let gain = guided.mean_recall - baseline.mean_recall;
    if gain + f32::EPSILON < dataset.expectations.min_recall_gain {
        failures.push(format!(
            "recall gain {gain:.3} is below the required {:.3}",
            dataset.expectations.min_recall_gain
        ));
    }

    for expected in &dataset.expectations.decisions {
        checked += 1;
        let Some(decision) = decisions.iter().find(|d| d.family == expected.family) else {
            failures.push(format!("no decision for family `{}`", expected.family));
            continue;
        };
        if decision.plan != expected.plan {
            failures.push(format!(
                "family `{}` chose {} but the dataset expects {}",
                expected.family, decision.plan, expected.plan
            ));
        }
        if decision.reason.as_str() != expected.reason {
            failures.push(format!(
                "family `{}` reported reason `{}` but the dataset expects `{}`",
                expected.family,
                decision.reason.as_str(),
                expected.reason
            ));
        }
    }

    ExpectationCheck {
        checked,
        passed: failures.is_empty(),
        failures,
    }
}

/// Build the full report from a completed comparison.
#[must_use]
pub fn build_report(
    dataset: &Dataset,
    baseline: Evaluation,
    guided: Evaluation,
    decisions: Vec<Decision>,
    advisory_patterns: BTreeMap<String, Vec<String>>,
) -> Report {
    let guided_latency = i64::try_from(guided.total_latency_ms).unwrap_or(i64::MAX);
    let baseline_latency = i64::try_from(baseline.total_latency_ms).unwrap_or(i64::MAX);
    let deltas = Deltas {
        recall: guided.mean_recall - baseline.mean_recall,
        precision: guided.mean_precision - baseline.mean_precision,
        diversity: guided.mean_diversity - baseline.mean_diversity,
        latency_ms: guided_latency.saturating_sub(baseline_latency),
        cost_usd: guided.total_cost_usd - baseline.total_cost_usd,
    };
    let expectations = check_expectations(dataset, &decisions, &baseline, &guided);
    let fingerprint = fingerprint(&decisions);
    Report {
        schema_version: REPORT_VERSION.to_string(),
        dataset_version: dataset.version.clone(),
        generated_by: "examples/adaptive_source_discovery.rs".to_string(),
        advisory: ADVISORY_NOTICE.to_string(),
        redaction: REDACTION_NOTICE.to_string(),
        baseline,
        memory_guided: guided,
        deltas,
        decisions,
        advisory_patterns,
        fingerprint,
        expectations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adaptive_discovery::dataset::{
        DiscoveredSource, EvalAttempt, Expectations, ExpectedDecision, OutcomeClass, Plan,
        QueryFamily, Truth,
    };
    use crate::adaptive_discovery::planner::DecisionReason;

    fn plan(strategy: &str, provider: &str) -> Plan {
        Plan {
            strategy: strategy.to_string(),
            provider: provider.to_string(),
        }
    }

    fn source(url: &str, outcome: OutcomeClass) -> DiscoveredSource {
        DiscoveredSource {
            url: url.to_string(),
            outcome,
            relevance: 1.0,
            authority: 1.0,
        }
    }

    fn dataset() -> Dataset {
        Dataset {
            version: "1.0".to_string(),
            expectations: Expectations {
                min_recall_gain: 0.5,
                decisions: Vec::new(),
            },
            families: vec![QueryFamily {
                id: "demo".to_string(),
                query: "demo".to_string(),
                domain: "demo".to_string(),
                baseline: plan("keyword", "alpha"),
                history: Vec::new(),
                truth: Truth {
                    verified_urls: vec!["https://good.example/a".to_string()],
                },
                attempts: vec![
                    EvalAttempt {
                        plan: plan("keyword", "alpha"),
                        latency_ms: 100,
                        cost_usd: 0.0,
                        discovered: vec![source("https://bad.example/x", OutcomeClass::Blocked)],
                    },
                    EvalAttempt {
                        plan: plan("semantic", "beta"),
                        latency_ms: 200,
                        cost_usd: 0.0,
                        discovered: vec![source("https://good.example/a", OutcomeClass::Verified)],
                    },
                ],
            }],
        }
    }

    fn decision(family: &str, chosen: Plan, reason: DecisionReason) -> Decision {
        Decision {
            family: family.to_string(),
            plan: chosen,
            evidence: Vec::new(),
            reason,
            ignored_episodes: 0,
        }
    }

    #[test]
    fn expectations_flag_a_low_recall_gain() {
        let dataset = dataset();
        let mut plans = BTreeMap::new();
        plans.insert("demo".to_string(), plan("keyword", "alpha"));
        let baseline = evaluate(&dataset, &plans).expect("baseline should score");
        let check = check_expectations(
            &dataset,
            &[decision(
                "demo",
                plan("keyword", "alpha"),
                DecisionReason::BaselineSupported,
            )],
            &baseline,
            &baseline,
        );
        assert!(!check.passed);
        assert!(check.failures[0].contains("recall gain"));
    }

    #[test]
    fn expectations_pass_when_the_plan_and_reason_match() {
        let mut dataset = dataset();
        dataset.expectations.min_recall_gain = 0.5;
        dataset.expectations.decisions = vec![ExpectedDecision {
            family: "demo".to_string(),
            plan: plan("semantic", "beta"),
            reason: "blocked_alternative".to_string(),
        }];

        let mut baseline_plans = BTreeMap::new();
        baseline_plans.insert("demo".to_string(), plan("keyword", "alpha"));
        let mut guided_plans = BTreeMap::new();
        guided_plans.insert("demo".to_string(), plan("semantic", "beta"));
        let baseline = evaluate(&dataset, &baseline_plans).expect("baseline should score");
        let guided = evaluate(&dataset, &guided_plans).expect("guided should score");

        let check = check_expectations(
            &dataset,
            &[decision(
                "demo",
                plan("semantic", "beta"),
                DecisionReason::BlockedAlternative,
            )],
            &baseline,
            &guided,
        );
        assert!(check.passed, "unexpected failures: {:?}", check.failures);
        assert_eq!(check.checked, 2);
    }

    #[test]
    fn rendered_summary_carries_the_notices_and_fingerprint() {
        let dataset = dataset();
        let mut plans = BTreeMap::new();
        plans.insert("demo".to_string(), plan("keyword", "alpha"));
        let baseline = evaluate(&dataset, &plans).expect("baseline should score");
        let guided = baseline.clone();
        let report = build_report(&dataset, baseline, guided, Vec::new(), BTreeMap::new());
        let rendered = report.render_markdown();
        assert!(rendered.contains("Fingerprint:"));
        assert!(rendered.contains("| demo |"));
        assert_eq!(report.advisory, ADVISORY_NOTICE);
        assert_eq!(report.redaction, REDACTION_NOTICE);
    }
}
