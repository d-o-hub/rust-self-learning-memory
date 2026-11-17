//! Bounded plan selection from episodic evidence.
//!
//! The planner is intentionally conservative. Memory may only do two things:
//!
//! 1. keep the static baseline when the evidence supports it (or when there is
//!    no evidence at all), and
//! 2. move to **one** alternative plan that the evidence already contains, with
//!    a minimum number of observations and a minimum improvement.
//!
//! It never invents a provider, never takes a second hop, and never overrides a
//! plan that is working. That bound is what makes the recommendation safe to
//! surface to an external consumer as advisory input.

use crate::adaptive_discovery::dataset::{OutcomeClass, Plan, QueryFamily};
use serde::Serialize;

/// Alternative plans need at least this many observations before they are used.
pub const MIN_ALTERNATIVE_OBSERVATIONS: usize = 2;
/// A baseline plan at or above this useful rate is left alone.
pub const MIN_BASELINE_USEFUL_RATE: f32 = 0.5;
/// An alternative must beat the baseline useful rate by at least this much.
pub const MIN_USEFUL_RATE_GAIN: f32 = 0.2;

/// One observation recovered from episodic memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// Plan the past attempt used.
    pub plan: Plan,
    /// How that attempt ended.
    pub outcome: OutcomeClass,
}

/// Aggregated evidence for one plan.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Evidence {
    /// Plan the evidence is about.
    pub plan: Plan,
    /// Number of observations.
    pub observations: usize,
    /// Observations with a useful or verified outcome.
    pub useful: usize,
    /// Observations with a blocked or transport-error outcome.
    pub blocking: usize,
    /// Independently verified observations.
    pub verified: usize,
    /// `useful / observations`, rounded for display.
    pub useful_rate: f32,
}

/// Why the planner chose the plan it chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReason {
    /// No prior episode covered this family; the baseline stands.
    NoEvidence,
    /// The baseline plan's evidence is good enough; no change.
    BaselineSupported,
    /// The baseline plan is failing and a bounded alternative is better.
    BlockedAlternative,
    /// The baseline plan is failing but no alternative clears the bound.
    NoBoundedAlternative,
}

impl DecisionReason {
    /// Stable lowercase name used in reports and expectations.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoEvidence => "no_evidence",
            Self::BaselineSupported => "baseline_supported",
            Self::BlockedAlternative => "blocked_alternative",
            Self::NoBoundedAlternative => "no_bounded_alternative",
        }
    }
}

/// The planner's decision for one family.
#[derive(Debug, Clone, Serialize)]
pub struct Decision {
    /// Family the decision applies to.
    pub family: String,
    /// Plan the consuming application should try next.
    pub plan: Plan,
    /// Evidence the decision rests on.
    pub evidence: Vec<Evidence>,
    /// Why this plan was chosen.
    pub reason: DecisionReason,
    /// Retrieved episodes that carried no usable plan/outcome tags.
    pub ignored_episodes: usize,
}

/// Group observations by plan, in a deterministic order.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "one family accumulates a handful of observations"
)]
pub fn aggregate(observations: &[Observation]) -> Vec<Evidence> {
    let mut grouped: std::collections::BTreeMap<Plan, Evidence> = std::collections::BTreeMap::new();
    for observation in observations {
        let entry = grouped
            .entry(observation.plan.clone())
            .or_insert_with(|| Evidence {
                plan: observation.plan.clone(),
                observations: 0,
                useful: 0,
                blocking: 0,
                verified: 0,
                useful_rate: 0.0,
            });
        entry.observations += 1;
        if observation.outcome.is_useful_signal() {
            entry.useful += 1;
        }
        if observation.outcome.is_blocking() {
            entry.blocking += 1;
        }
        if observation.outcome == OutcomeClass::Verified {
            entry.verified += 1;
        }
    }
    grouped
        .into_values()
        .map(|mut entry| {
            entry.useful_rate = entry.useful as f32 / entry.observations as f32;
            entry
        })
        .collect()
}

/// Choose a plan for a family from retrieved evidence, under the documented bounds.
#[must_use]
pub fn choose(
    family: &QueryFamily,
    observations: &[Observation],
    ignored_episodes: usize,
) -> Decision {
    let evidence = aggregate(observations);
    if evidence.is_empty() {
        return Decision {
            family: family.id.clone(),
            plan: family.baseline.clone(),
            evidence,
            reason: DecisionReason::NoEvidence,
            ignored_episodes,
        };
    }

    let baseline_rate = evidence
        .iter()
        .find(|entry| entry.plan == family.baseline)
        .map_or(0.0, |entry| entry.useful_rate);
    let baseline_supported = baseline_rate >= MIN_BASELINE_USEFUL_RATE;
    if baseline_supported {
        return Decision {
            family: family.id.clone(),
            plan: family.baseline.clone(),
            evidence,
            reason: DecisionReason::BaselineSupported,
            ignored_episodes,
        };
    }

    let mut candidates: Vec<&Evidence> = evidence
        .iter()
        .filter(|entry| {
            entry.plan != family.baseline
                && entry.observations >= MIN_ALTERNATIVE_OBSERVATIONS
                && entry.useful_rate >= MIN_BASELINE_USEFUL_RATE
                && entry.useful_rate >= baseline_rate + MIN_USEFUL_RATE_GAIN
        })
        .collect();
    candidates.sort_by(|a, b| {
        b.useful_rate
            .total_cmp(&a.useful_rate)
            .then(b.observations.cmp(&a.observations))
            .then(a.plan.cmp(&b.plan))
    });

    match candidates.first() {
        Some(best) => Decision {
            family: family.id.clone(),
            plan: best.plan.clone(),
            evidence,
            reason: DecisionReason::BlockedAlternative,
            ignored_episodes,
        },
        None => Decision {
            family: family.id.clone(),
            plan: family.baseline.clone(),
            evidence,
            reason: DecisionReason::NoBoundedAlternative,
            ignored_episodes,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn family() -> QueryFamily {
        QueryFamily {
            id: "demo".to_string(),
            query: "demo query".to_string(),
            domain: "demo".to_string(),
            baseline: plan("keyword", "alpha"),
            history: Vec::new(),
            attempts: Vec::new(),
            truth: crate::adaptive_discovery::dataset::Truth {
                verified_urls: Vec::new(),
            },
        }
    }

    fn plan(strategy: &str, provider: &str) -> Plan {
        Plan {
            strategy: strategy.to_string(),
            provider: provider.to_string(),
        }
    }

    fn observation(strategy: &str, provider: &str, outcome: OutcomeClass) -> Observation {
        Observation {
            plan: plan(strategy, provider),
            outcome,
        }
    }

    #[test]
    fn no_evidence_keeps_the_baseline() {
        let decision = choose(&family(), &[], 0);
        assert_eq!(decision.plan, plan("keyword", "alpha"));
        assert_eq!(decision.reason, DecisionReason::NoEvidence);
    }

    #[test]
    fn working_baseline_is_not_churned() {
        let observations = vec![
            observation("keyword", "alpha", OutcomeClass::Verified),
            observation("keyword", "alpha", OutcomeClass::Useful),
            observation("semantic", "beta", OutcomeClass::Verified),
            observation("semantic", "beta", OutcomeClass::Verified),
        ];
        let decision = choose(&family(), &observations, 0);
        assert_eq!(decision.plan, plan("keyword", "alpha"));
        assert_eq!(decision.reason, DecisionReason::BaselineSupported);
    }

    #[test]
    fn blocked_baseline_switches_to_a_bounded_alternative() {
        let observations = vec![
            observation("keyword", "alpha", OutcomeClass::Blocked),
            observation("keyword", "alpha", OutcomeClass::Blocked),
            observation("semantic", "beta", OutcomeClass::Verified),
            observation("semantic", "beta", OutcomeClass::Verified),
        ];
        let decision = choose(&family(), &observations, 3);
        assert_eq!(decision.plan, plan("semantic", "beta"));
        assert_eq!(decision.reason, DecisionReason::BlockedAlternative);
        assert_eq!(decision.ignored_episodes, 3);
    }

    #[test]
    fn one_observation_is_not_enough_to_switch() {
        let observations = vec![
            observation("keyword", "alpha", OutcomeClass::TransportError),
            observation("semantic", "beta", OutcomeClass::Verified),
        ];
        let decision = choose(&family(), &observations, 0);
        assert_eq!(decision.plan, plan("keyword", "alpha"));
        assert_eq!(decision.reason, DecisionReason::NoBoundedAlternative);
    }

    #[test]
    fn ties_break_deterministically_toward_the_first_plan() {
        let observations = vec![
            observation("keyword", "alpha", OutcomeClass::Blocked),
            observation("keyword", "gamma", OutcomeClass::Verified),
            observation("keyword", "gamma", OutcomeClass::Verified),
            observation("semantic", "beta", OutcomeClass::Verified),
            observation("semantic", "beta", OutcomeClass::Verified),
        ];
        let first = choose(&family(), &observations, 0);
        let second = choose(&family(), &observations, 0);
        assert_eq!(first.plan, second.plan);
        assert_eq!(first.plan, plan("keyword", "gamma"));
    }
}
