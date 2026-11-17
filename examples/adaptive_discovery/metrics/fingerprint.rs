//! Stable decision fingerprint.
//!
//! The value is written by one process and compared by another, so it must not
//! depend on the toolchain: FNV-1a is hand-rolled instead of `DefaultHasher`,
//! whose output is explicitly not stable across Rust releases.

use std::fmt::Write as _;

use crate::adaptive_discovery::planner::Decision;

/// A stable fingerprint of the decisions, comparable across processes and runs.
#[must_use]
pub fn fingerprint(decisions: &[Decision]) -> String {
    let mut ordered: Vec<&Decision> = decisions.iter().collect();
    ordered.sort_by(|a, b| a.family.cmp(&b.family));

    let mut canonical = String::new();
    for decision in ordered {
        let _ = write!(
            canonical,
            "{}|{}|{}|{}",
            decision.family,
            decision.plan.strategy,
            decision.plan.provider,
            decision.reason.as_str()
        );
        for evidence in &decision.evidence {
            let _ = write!(
                canonical,
                "|{}:{}:{}:{}",
                evidence.plan, evidence.observations, evidence.useful, evidence.verified
            );
        }
        let _ = writeln!(canonical, "|ignored={}", decision.ignored_episodes);
    }

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in canonical.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adaptive_discovery::dataset::Plan;
    use crate::adaptive_discovery::planner::DecisionReason;

    fn plan(strategy: &str, provider: &str) -> Plan {
        Plan {
            strategy: strategy.to_string(),
            provider: provider.to_string(),
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
    fn fingerprint_is_stable_and_sensitive_to_the_plan() {
        let first = fingerprint(&[decision(
            "demo",
            plan("semantic", "beta"),
            DecisionReason::BlockedAlternative,
        )]);
        let second = fingerprint(&[decision(
            "demo",
            plan("semantic", "beta"),
            DecisionReason::BlockedAlternative,
        )]);
        let changed = fingerprint(&[decision(
            "demo",
            plan("keyword", "alpha"),
            DecisionReason::BaselineSupported,
        )]);
        assert_eq!(first, second);
        assert_ne!(first, changed);
        assert_eq!(first.len(), 16);
    }

    #[test]
    fn fingerprint_ignores_the_input_order() {
        let first = decision("a", plan("keyword", "alpha"), DecisionReason::NoEvidence);
        let second = decision("b", plan("keyword", "beta"), DecisionReason::NoEvidence);
        assert_eq!(
            fingerprint(&[first.clone(), second.clone()]),
            fingerprint(&[second, first])
        );
    }
}
