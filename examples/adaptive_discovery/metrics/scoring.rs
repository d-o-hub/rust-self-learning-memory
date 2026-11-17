//! Scoring for the adaptive source-discovery example.
//!
//! Both the static baseline and the memory-guided plan are scored against the
//! same fixture ground truth, so the comparison never depends on what memory
//! happened to learn. A discovered source is only counted as a hit when the
//! application itself marked it `verified` *and* the family's verified list
//! contains it.

use serde::Serialize;
use std::collections::BTreeSet;

use crate::adaptive_discovery::dataset::{Dataset, OutcomeClass, Plan, RedactedEndpoint};

/// Score for one family under one plan.
#[derive(Debug, Clone, Serialize)]
pub struct FamilyScore {
    /// Family identifier.
    pub family: String,
    /// Plan that was scored.
    pub plan: Plan,
    /// Sources the plan returns.
    pub candidates: usize,
    /// Sources the application marked verified.
    pub verified: usize,
    /// Verified sources that also appear in the family's truth list.
    pub matched_verified: usize,
    /// `verified / candidates`.
    pub precision: f32,
    /// `matched_verified / truth`.
    pub recall: f32,
    /// Distinct hosts among the verified sources.
    pub diversity: usize,
    /// Mean relevance score across all candidates, as reported by the application.
    pub mean_relevance: f32,
    /// Mean authority score across all candidates, as reported by the application.
    pub mean_authority: f32,
    /// Plan latency in milliseconds.
    pub latency_ms: u64,
    /// Plan cost in USD.
    pub cost_usd: f64,
}

/// Aggregate score across families.
#[derive(Debug, Clone, Serialize)]
pub struct Evaluation {
    /// Per-family scores.
    pub families: Vec<FamilyScore>,
    /// Mean recall across families.
    pub mean_recall: f32,
    /// Mean precision across families.
    pub mean_precision: f32,
    /// Mean verified-source diversity across families.
    pub mean_diversity: f32,
    /// Total latency in milliseconds.
    pub total_latency_ms: u64,
    /// Total cost in USD.
    pub total_cost_usd: f64,
}

/// Difference between the memory-guided run and the static baseline.
#[derive(Debug, Clone, Serialize)]
pub struct Deltas {
    /// Recall difference.
    pub recall: f32,
    /// Precision difference.
    pub precision: f32,
    /// Diversity difference.
    pub diversity: f32,
    /// Latency difference in milliseconds (negative is better).
    pub latency_ms: i64,
    /// Cost difference in USD.
    pub cost_usd: f64,
}

/// Score every family under the plan chosen for it.
///
/// # Errors
///
/// Returns an error when a family has no plan or no ground-truth attempt for its
/// plan. Both are fixture errors and must not be papered over.
#[expect(
    clippy::cast_precision_loss,
    reason = "a fixture family scores at most a handful of sources"
)]
pub fn evaluate(
    dataset: &Dataset,
    plans: &std::collections::BTreeMap<String, Plan>,
) -> anyhow::Result<Evaluation> {
    let mut families = Vec::new();
    for family in &dataset.families {
        let Some(plan) = plans.get(&family.id) else {
            anyhow::bail!("no plan chosen for family `{}`", family.id);
        };
        let Some(attempt) = family.attempt_for(plan) else {
            anyhow::bail!(
                "family `{}` has no ground-truth attempt for plan {plan}",
                family.id
            );
        };

        let truth: BTreeSet<String> = family
            .truth
            .verified_urls
            .iter()
            .filter_map(|url| RedactedEndpoint::parse(url).ok())
            .map(|endpoint| endpoint.to_string())
            .collect();

        let mut verified = 0;
        let mut matched = 0;
        let mut hosts = BTreeSet::new();
        let mut relevance_sum = 0.0_f32;
        let mut authority_sum = 0.0_f32;
        for source in &attempt.discovered {
            relevance_sum += source.relevance;
            authority_sum += source.authority;
            if source.outcome != OutcomeClass::Verified {
                continue;
            }
            verified += 1;
            let endpoint = RedactedEndpoint::parse(&source.url)?;
            if truth.contains(endpoint.as_str()) {
                matched += 1;
            }
            hosts.insert(endpoint.host().to_string());
        }

        let candidates = attempt.discovered.len();
        families.push(FamilyScore {
            family: family.id.clone(),
            plan: plan.clone(),
            candidates,
            verified,
            matched_verified: matched,
            precision: ratio(verified, candidates),
            recall: ratio(matched, truth.len()),
            diversity: hosts.len(),
            mean_relevance: relevance_sum / candidates as f32,
            mean_authority: authority_sum / candidates as f32,
            latency_ms: attempt.latency_ms,
            cost_usd: attempt.cost_usd,
        });
    }

    let count = families.len();
    let mean_recall = mean(families.iter().map(|score| score.recall), count);
    let mean_precision = mean(families.iter().map(|score| score.precision), count);
    let mean_diversity = mean(families.iter().map(|score| score.diversity as f32), count);
    Ok(Evaluation {
        total_latency_ms: families.iter().map(|score| score.latency_ms).sum(),
        total_cost_usd: families.iter().map(|score| score.cost_usd).sum(),
        families,
        mean_recall,
        mean_precision,
        mean_diversity,
    })
}

#[expect(
    clippy::cast_precision_loss,
    reason = "a fixture family scores at most a handful of sources"
)]
fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "the dataset has a handful of families"
)]
fn mean(values: impl Iterator<Item = f32>, count: usize) -> f32 {
    if count == 0 {
        0.0
    } else {
        values.sum::<f32>() / count as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adaptive_discovery::dataset::{
        DiscoveredSource, EvalAttempt, Expectations, QueryFamily, Truth,
    };

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
                    verified_urls: vec![
                        "https://good.example/a".to_string(),
                        "https://good.example/b".to_string(),
                    ],
                },
                attempts: vec![
                    EvalAttempt {
                        plan: plan("keyword", "alpha"),
                        latency_ms: 100,
                        cost_usd: 0.01,
                        discovered: vec![source("https://bad.example/x", OutcomeClass::Blocked)],
                    },
                    EvalAttempt {
                        plan: plan("semantic", "beta"),
                        latency_ms: 200,
                        cost_usd: 0.02,
                        discovered: vec![
                            source("https://good.example/a?utm=x", OutcomeClass::Verified),
                            source("https://good.example/b", OutcomeClass::Verified),
                            source("https://bad.example/y", OutcomeClass::Irrelevant),
                        ],
                    },
                ],
            }],
        }
    }

    #[test]
    fn evaluation_scores_recall_precision_and_diversity() {
        let dataset = dataset();
        let mut plans = std::collections::BTreeMap::new();
        plans.insert("demo".to_string(), plan("semantic", "beta"));
        let evaluation = evaluate(&dataset, &plans).expect("evaluation should succeed");
        assert_eq!(evaluation.families[0].candidates, 3);
        assert_eq!(evaluation.families[0].verified, 2);
        assert_eq!(evaluation.families[0].matched_verified, 2);
        assert!((evaluation.families[0].recall - 1.0).abs() < f32::EPSILON);
        assert!((evaluation.families[0].precision - 2.0 / 3.0).abs() < 1e-5);
        assert_eq!(evaluation.families[0].diversity, 1);
        assert_eq!(evaluation.total_latency_ms, 200);
    }

    #[test]
    fn evaluation_fails_when_ground_truth_is_missing() {
        let dataset = dataset();
        let mut plans = std::collections::BTreeMap::new();
        plans.insert("demo".to_string(), plan("semantic", "gamma"));
        let error = evaluate(&dataset, &plans).expect_err("missing attempt must fail");
        assert!(error.to_string().contains("no ground-truth attempt"));
    }
}
