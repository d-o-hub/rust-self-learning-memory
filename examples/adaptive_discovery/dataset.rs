//! Deterministic fixture dataset for the adaptive source-discovery example.
//!
//! The dataset is the ground truth the example evaluates against. It is split
//! into three parts per query family:
//!
//! - `history`: what a previous session observed. This is what gets recorded into
//!   episodic memory by the `seed` phase.
//! - `attempts`: what each candidate plan would discover *now*. This is the
//!   evaluation ground truth; it is never written to memory before scoring.
//! - `truth`: the independently verified sources for the family, used as the
//!   recall denominator.
//!
//! Candidate URLs in the fixture carry query strings and, in one case, URL
//! userinfo. Only the [`RedactedEndpoint`] projection is ever persisted, so the
//! example can demonstrate redaction without storing a credential.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

/// Dataset schema version this example understands.
pub const DATASET_VERSION: &str = "1.0";

/// Root of the fixture dataset.
#[derive(Debug, Clone, Deserialize)]
pub struct Dataset {
    /// Schema version; must equal [`DATASET_VERSION`].
    pub version: String,
    /// Query families, each with history, evaluation attempts, and truth.
    pub families: Vec<QueryFamily>,
    /// Aggregate and per-family expectations the `verify` phase enforces.
    pub expectations: Expectations,
}

/// One query family: a normalized query plus everything known about it.
#[derive(Debug, Clone, Deserialize)]
pub struct QueryFamily {
    /// Stable family identifier, also used as an episode tag.
    pub id: String,
    /// Normalized query text. Never contains credentials.
    pub query: String,
    /// Domain the consuming application files the query under.
    pub domain: String,
    /// The static plan a memory-less application would use.
    pub baseline: Plan,
    /// Attempts a previous session observed; recorded into memory by `seed`.
    #[serde(default)]
    pub history: Vec<HistoryAttempt>,
    /// Ground-truth attempts used for scoring only.
    pub attempts: Vec<EvalAttempt>,
    /// Independently verified sources for the family.
    pub truth: Truth,
}

/// A discovery plan: one strategy executed against one provider.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, serde::Serialize)]
pub struct Plan {
    /// Discovery strategy, e.g. `keyword` or `semantic`.
    pub strategy: String,
    /// Provider/backend identifier, e.g. `alpha`.
    pub provider: String,
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.strategy, self.provider)
    }
}

/// A past attempt, as observed by a previous session.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryAttempt {
    /// Plan that was attempted.
    pub plan: Plan,
    /// How the attempt ended.
    pub outcome: OutcomeClass,
    /// Observed latency in milliseconds.
    pub latency_ms: u64,
    /// Observed cost in USD.
    pub cost_usd: f64,
    /// A candidate endpoint the attempt touched. Persisted only in redacted form.
    pub endpoint: String,
}

/// A ground-truth attempt, used for scoring only.
#[derive(Debug, Clone, Deserialize)]
pub struct EvalAttempt {
    /// Plan this attempt models.
    pub plan: Plan,
    /// Latency in milliseconds.
    pub latency_ms: u64,
    /// Cost in USD.
    pub cost_usd: f64,
    /// Sources the plan returns.
    pub discovered: Vec<DiscoveredSource>,
}

/// One source a plan returns.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveredSource {
    /// Candidate URL. Persisted only in redacted form.
    pub url: String,
    /// Outcome class assigned by the consuming application.
    pub outcome: OutcomeClass,
    /// Relevance score in `[0, 1]` from the consuming application.
    pub relevance: f32,
    /// Authority score in `[0, 1]` from the consuming application.
    pub authority: f32,
}

/// Independently verified sources for a family.
#[derive(Debug, Clone, Deserialize)]
pub struct Truth {
    /// Canonical URLs the application verified on its own.
    pub verified_urls: Vec<String>,
}

/// How a discovery attempt or candidate source ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeClass {
    /// Nothing was returned.
    Empty,
    /// The provider refused the request (paywall, robots, auth policy).
    Blocked,
    /// The transport failed before a decision could be made.
    TransportError,
    /// A source came back but was off-topic.
    Irrelevant,
    /// A plausible source that the application has not verified yet.
    Useful,
    /// A source the application independently verified.
    Verified,
}

impl OutcomeClass {
    /// True when the outcome is positive evidence for a plan.
    #[must_use]
    pub fn is_useful_signal(self) -> bool {
        matches!(self, Self::Useful | Self::Verified)
    }

    /// True when the outcome says the plan itself is unusable for this family.
    #[must_use]
    pub fn is_blocking(self) -> bool {
        matches!(self, Self::Blocked | Self::TransportError)
    }

    /// Stable lowercase name used in tags, reports, and traces.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Blocked => "blocked",
            Self::TransportError => "transport_error",
            Self::Irrelevant => "irrelevant",
            Self::Useful => "useful",
            Self::Verified => "verified",
        }
    }

    /// Parse the tag form back into an outcome class.
    #[must_use]
    pub fn from_tag(value: &str) -> Option<Self> {
        match value {
            "empty" => Some(Self::Empty),
            "blocked" => Some(Self::Blocked),
            "transport_error" => Some(Self::TransportError),
            "irrelevant" => Some(Self::Irrelevant),
            "useful" => Some(Self::Useful),
            "verified" => Some(Self::Verified),
            _ => None,
        }
    }
}

impl fmt::Display for OutcomeClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Expectations the `verify` phase enforces. Keeping them in the dataset means
/// the example cannot silently regress into "it ran, so it must have learned".
#[derive(Debug, Clone, Deserialize)]
pub struct Expectations {
    /// Minimum recall gain (memory-guided minus baseline) across families.
    pub min_recall_gain: f32,
    /// Expected decision reason per family id.
    pub decisions: Vec<ExpectedDecision>,
}

/// One expected bounded decision.
#[derive(Debug, Clone, Deserialize)]
pub struct ExpectedDecision {
    /// Family the expectation applies to.
    pub family: String,
    /// Plan the memory-guided planner must choose.
    pub plan: Plan,
    /// Decision reason the planner must report.
    pub reason: String,
}

/// A credential-free view of a candidate endpoint.
///
/// Only the scheme, host (without userinfo), and path survive. Query strings,
/// fragments, and URL userinfo are dropped, which is what keeps API keys and
/// session tokens out of episodes, traces, and reports.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RedactedEndpoint(String);

impl RedactedEndpoint {
    /// Project a raw endpoint onto its credential-free form.
    ///
    /// # Errors
    ///
    /// Returns an error when the endpoint is not an absolute `http(s)` URL or has
    /// no host component. Failing loudly is deliberate: silently dropping a
    /// malformed candidate would hide a fixture bug.
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        let (scheme, rest) = trimmed
            .split_once("://")
            .with_context(|| format!("endpoint is not absolute: {trimmed}"))?;
        let scheme = scheme.to_ascii_lowercase();
        if scheme != "http" && scheme != "https" {
            bail!("unsupported endpoint scheme `{scheme}` in {trimmed}");
        }

        let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let authority = &rest[..authority_end];
        // URL userinfo is everything before the last `@`; drop it entirely.
        let host = authority.rsplit('@').next().unwrap_or(authority);
        if host.is_empty() {
            bail!("endpoint has no host: {trimmed}");
        }

        let mut tail = &rest[authority_end..];
        if let Some(cut) = tail.find(['?', '#']) {
            tail = &tail[..cut];
        }
        let path = if tail.is_empty() { "/" } else { tail };

        Ok(Self(format!("{scheme}://{host}{path}")))
    }

    /// The redacted endpoint as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Host portion, used for source diversity.
    #[must_use]
    pub fn host(&self) -> &str {
        let without_scheme = self
            .0
            .split_once("://")
            .map_or(self.0.as_str(), |(_, rest)| rest);
        without_scheme.split('/').next().unwrap_or(without_scheme)
    }
}

impl fmt::Display for RedactedEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for RedactedEndpoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

impl Dataset {
    /// Load and validate a dataset from disk.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read, the JSON is invalid, or the
    /// dataset fails [`Dataset::validate`].
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read dataset {}", path.display()))?;
        let dataset: Self = serde_json::from_str(&raw)
            .with_context(|| format!("invalid dataset JSON in {}", path.display()))?;
        dataset.validate()?;
        Ok(dataset)
    }

    /// Fail loudly on a dataset that cannot produce a meaningful comparison.
    ///
    /// # Errors
    ///
    /// Returns an error for an unsupported version, duplicate family ids, a
    /// family without a baseline attempt, or a non-parseable endpoint.
    pub fn validate(&self) -> Result<()> {
        if self.version != DATASET_VERSION {
            bail!(
                "dataset version `{}` is not supported (expected `{DATASET_VERSION}`)",
                self.version
            );
        }
        if self.families.is_empty() {
            bail!("dataset has no query families");
        }

        let mut seen = BTreeSet::new();
        for family in &self.families {
            if !seen.insert(family.id.as_str()) {
                bail!("duplicate query family id `{}`", family.id);
            }
            if !family
                .attempts
                .iter()
                .any(|attempt| attempt.plan == family.baseline)
            {
                bail!(
                    "family `{}` has no evaluation attempt for its baseline plan {}",
                    family.id,
                    family.baseline
                );
            }
            for url in &family.truth.verified_urls {
                RedactedEndpoint::parse(url)?;
            }
            for attempt in &family.attempts {
                for source in &attempt.discovered {
                    RedactedEndpoint::parse(&source.url)?;
                }
            }
            for attempt in &family.history {
                RedactedEndpoint::parse(&attempt.endpoint)?;
            }
        }

        for expected in &self.expectations.decisions {
            if !seen.contains(expected.family.as_str()) {
                bail!("expectation names unknown family `{}`", expected.family);
            }
        }
        Ok(())
    }
}

impl QueryFamily {
    /// The ground-truth attempt for a plan, if the fixture defines one.
    #[must_use]
    pub fn attempt_for(&self, plan: &Plan) -> Option<&EvalAttempt> {
        self.attempts.iter().find(|attempt| &attempt.plan == plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redaction_drops_userinfo_query_and_fragment() {
        let endpoint =
            RedactedEndpoint::parse("https://reader:secret@news.example/nba/scores?utm=x#top")
                .expect("endpoint should parse");
        assert_eq!(endpoint.as_str(), "https://news.example/nba/scores");
        assert_eq!(endpoint.host(), "news.example");
    }

    #[test]
    fn redaction_strips_query_only_urls() {
        let endpoint = RedactedEndpoint::parse("http://alpha.example/search?api_key=abc&q=rag")
            .expect("endpoint should parse");
        assert_eq!(endpoint.as_str(), "http://alpha.example/search");
        assert!(!endpoint.as_str().contains("api_key"));
    }

    #[test]
    fn redaction_rejects_relative_and_foreign_schemes() {
        assert!(RedactedEndpoint::parse("news.example/nba").is_err());
        assert!(RedactedEndpoint::parse("ftp://news.example/nba").is_err());
        assert!(RedactedEndpoint::parse("https:///nba").is_err());
    }

    #[test]
    fn outcome_classes_round_trip_through_their_tag_form() {
        for outcome in [
            OutcomeClass::Empty,
            OutcomeClass::Blocked,
            OutcomeClass::TransportError,
            OutcomeClass::Irrelevant,
            OutcomeClass::Useful,
            OutcomeClass::Verified,
        ] {
            assert_eq!(OutcomeClass::from_tag(outcome.as_str()), Some(outcome));
        }
        assert!(OutcomeClass::Verified.is_useful_signal());
        assert!(!OutcomeClass::Irrelevant.is_useful_signal());
        assert!(OutcomeClass::Blocked.is_blocking());
    }
}
