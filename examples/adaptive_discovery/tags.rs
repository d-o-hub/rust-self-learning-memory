//! Episode-tag vocabulary for discovery attempts.
//!
//! An episode's tags are the durable record retrieval reads back: the family, the
//! plan, the outcome class, and the attempt's identity. Keeping that projection in
//! one place is what lets the seed phase resolve a replayed attempt to the episode
//! that already records it.

use crate::adaptive_discovery::dataset::{OutcomeClass, Plan, QueryFamily};
use crate::adaptive_discovery::planner::Observation;

/// Tag prefix that identifies the family an episode belongs to.
const FAMILY_TAG: &str = "family_";
/// Tag prefix that identifies the strategy an episode used.
const STRATEGY_TAG: &str = "strategy_";
/// Tag prefix that identifies the provider an episode used.
const PROVIDER_TAG: &str = "provider_";
/// Tag prefix that identifies how an episode ended.
const OUTCOME_TAG: &str = "outcome_";
/// Tag prefix that carries the stable identity of the recorded attempt.
const ATTEMPT_TAG: &str = "attempt_";

// Episode tags are normalised and validated by the memory system: only
// alphanumeric characters, hyphens, and underscores survive, so the prefix
// separator is `_`, not `:`. An external consumer that tags with `family:nba`
// gets a validation error rather than a silently unusable tag.
// The identity is lowercased for the same reason: the store lowercases tags, so
// two identities that differ only by case would collapse into one episode.

/// The tag carrying an attempt's stable identity.
#[must_use]
pub fn attempt_tag(attempt_id: &str) -> String {
    format!("{ATTEMPT_TAG}{}", attempt_id.to_lowercase())
}

/// The tags that make one attempt an observation.
#[must_use]
pub fn tags_for(
    family: &QueryFamily,
    plan: &Plan,
    outcome: OutcomeClass,
    attempt_id: &str,
) -> Vec<String> {
    vec![
        format!("{FAMILY_TAG}{}", family.id),
        format!("{STRATEGY_TAG}{}", plan.strategy),
        format!("{PROVIDER_TAG}{}", plan.provider),
        format!("{OUTCOME_TAG}{outcome}"),
        attempt_tag(attempt_id),
    ]
}

/// One episode's tags, parsed back into the observation they record.
pub struct ParsedObservation {
    /// Family the episode belongs to.
    pub family: String,
    /// Stable identity of the source attempt, when the episode carries one.
    pub attempt: Option<String>,
    /// The observation itself.
    pub observation: Observation,
}

/// Parse an episode's tags back into the observation they record.
///
/// Returns `None` when the tags cannot describe a finished attempt: a missing
/// family, plan, or outcome class is a fixture or write bug, never something to
/// guess at. Episodes written before attempts had identities still parse, with
/// [`ParsedObservation::attempt`] left `None`.
#[must_use]
pub fn parse_observation(tags: &[String]) -> Option<ParsedObservation> {
    let mut family = None;
    let mut strategy = None;
    let mut provider = None;
    let mut outcome = None;
    let mut attempt = None;
    for tag in tags {
        if let Some(value) = tag.strip_prefix(FAMILY_TAG) {
            family = Some(value.to_string());
        } else if let Some(value) = tag.strip_prefix(STRATEGY_TAG) {
            strategy = Some(value.to_string());
        } else if let Some(value) = tag.strip_prefix(PROVIDER_TAG) {
            provider = Some(value.to_string());
        } else if let Some(value) = tag.strip_prefix(OUTCOME_TAG) {
            outcome = OutcomeClass::from_tag(value);
        } else if let Some(value) = tag.strip_prefix(ATTEMPT_TAG) {
            attempt = Some(value.to_string());
        }
    }
    Some(ParsedObservation {
        family: family?,
        attempt,
        observation: Observation {
            plan: Plan {
                strategy: strategy?,
                provider: provider?,
            },
            outcome: outcome?,
        },
    })
}

/// The family tag for `family_id`, used to scope a lookup to one family.
#[must_use]
pub fn family_tag(family_id: &str) -> String {
    format!("{FAMILY_TAG}{family_id}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adaptive_discovery::dataset::Truth;

    fn family() -> QueryFamily {
        QueryFamily {
            id: "nba".to_string(),
            query: "latest basketball scores".to_string(),
            domain: "sports".to_string(),
            baseline: Plan {
                strategy: "keyword".to_string(),
                provider: "alpha".to_string(),
            },
            history: Vec::new(),
            attempts: Vec::new(),
            truth: Truth {
                verified_urls: Vec::new(),
            },
        }
    }

    #[test]
    fn observations_parse_from_tags() {
        let tags = vec![
            "family_nba".to_string(),
            "strategy_keyword".to_string(),
            "provider_alpha".to_string(),
            "outcome_blocked".to_string(),
        ];
        let parsed = parse_observation(&tags).expect("tags should parse");
        assert_eq!(parsed.family, "nba");
        assert_eq!(parsed.observation.plan.strategy, "keyword");
        assert_eq!(parsed.observation.plan.provider, "alpha");
        assert_eq!(parsed.observation.outcome, OutcomeClass::Blocked);
        assert_eq!(parsed.attempt, None);
    }

    #[test]
    fn the_attempt_identity_round_trips_through_its_tag() {
        let tags = tags_for(
            &family(),
            &family().baseline,
            OutcomeClass::Verified,
            "nba-h0",
        );
        let parsed = parse_observation(&tags).expect("tags should parse");
        assert_eq!(parsed.attempt.as_deref(), Some("nba-h0"));
        assert_eq!(tags.last().map(String::as_str), Some("attempt_nba-h0"));
    }

    #[test]
    fn incomplete_tags_are_rejected_instead_of_guessed() {
        let tags = vec![
            "family_nba".to_string(),
            "strategy_keyword".to_string(),
            "outcome_blocked".to_string(),
        ];
        assert!(parse_observation(&tags).is_none());
    }

    #[test]
    fn unknown_outcome_tags_are_rejected() {
        let tags = vec![
            "family_nba".to_string(),
            "strategy_keyword".to_string(),
            "provider_alpha".to_string(),
            "outcome_maybe".to_string(),
        ];
        assert!(parse_observation(&tags).is_none());
    }
}
