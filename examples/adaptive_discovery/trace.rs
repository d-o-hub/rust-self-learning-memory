//! Memory-harness trace emission.
//!
//! Traces use the schema `.agents/skills/memory-harness/record-trace.sh` writes,
//! so the recorded discovery attempts can be replayed with `replay-trace.sh`
//! against `do-memory-cli` instead of a bespoke format. Only redacted endpoints
//! reach a trace.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::adaptive_discovery::dataset::{
    HistoryAttempt, OutcomeClass, QueryFamily, RedactedEndpoint,
};

/// Trace schema version consumed by the memory-harness scripts.
pub const TRACE_VERSION: &str = "1.0";

/// One replayed step.
#[derive(Debug, Clone, Serialize)]
pub struct TraceStep {
    /// Tool name (`provider:<name>`).
    pub tool: String,
    /// Action description; carries the redacted endpoint and outcome class.
    pub action: String,
    /// Observed latency in milliseconds.
    pub latency_ms: u64,
    /// Whether the attempt produced a usable source.
    pub success: bool,
    /// Optional short observation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation: Option<String>,
}

/// A memory-harness compatible trace.
#[derive(Debug, Clone, Serialize)]
pub struct Trace {
    /// Schema version.
    pub version: String,
    /// Episode the trace belongs to.
    pub episode_id: String,
    /// Normalized query.
    pub task: String,
    /// Domain the query was filed under.
    pub domain: String,
    /// Project identifier.
    pub project: String,
    /// `success`, `partial`, or `failed`.
    pub outcome: String,
    /// Recorded steps.
    pub steps: Vec<TraceStep>,
    /// Deterministic file name; never serialised.
    #[serde(skip)]
    file_name: String,
}

impl Trace {
    /// Build a trace for one recorded attempt.
    ///
    /// # Errors
    ///
    /// Returns an error when the attempt's endpoint cannot be redacted.
    pub fn from_attempt(
        family: &QueryFamily,
        attempt: &HistoryAttempt,
        episode_id: uuid::Uuid,
    ) -> Result<Self> {
        let endpoint = RedactedEndpoint::parse(&attempt.endpoint)?;
        Ok(Self {
            version: TRACE_VERSION.to_string(),
            episode_id: episode_id.to_string(),
            task: family.query.clone(),
            domain: family.domain.clone(),
            project: "adaptive-source-discovery".to_string(),
            outcome: harness_outcome(attempt.outcome).to_string(),
            file_name: format!(
                "{}-{}-{}.json",
                slug(&family.id),
                slug(&attempt.plan.strategy),
                slug(&attempt.plan.provider)
            ),
            steps: vec![TraceStep {
                tool: format!("provider:{}", attempt.plan.provider),
                action: format!(
                    "strategy={} discover candidate sources endpoint={} outcome={}",
                    attempt.plan.strategy, endpoint, attempt.outcome
                ),
                latency_ms: attempt.latency_ms,
                success: attempt.outcome.is_useful_signal(),
                observation: Some(format!("cost_usd={:.4}", attempt.cost_usd)),
            }],
        })
    }

    /// Write the trace into `dir`, returning its path.
    ///
    /// The file name is derived from the family and plan so a repeated run
    /// overwrites its own trace instead of accumulating timestamped files.
    ///
    /// # Errors
    ///
    /// Returns an error when the directory cannot be created, the trace cannot be
    /// serialised, or the file cannot be written.
    pub fn write(&self, dir: &Path) -> Result<PathBuf> {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("cannot create trace directory {}", dir.display()))?;
        let path = dir.join(&self.file_name);
        let json = serde_json::to_string_pretty(self).context("cannot serialise the trace")?;
        std::fs::write(&path, json).with_context(|| format!("cannot write {}", path.display()))?;
        Ok(path)
    }
}

/// Map an outcome class onto the three outcomes the harness accepts.
#[must_use]
pub fn harness_outcome(outcome: OutcomeClass) -> &'static str {
    match outcome {
        OutcomeClass::Verified | OutcomeClass::Useful => "success",
        OutcomeClass::Empty | OutcomeClass::Irrelevant => "partial",
        OutcomeClass::Blocked | OutcomeClass::TransportError => "failed",
    }
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adaptive_discovery::dataset::{Plan, Truth};

    fn family() -> QueryFamily {
        QueryFamily {
            id: "demo".to_string(),
            query: "demo query".to_string(),
            domain: "demo".to_string(),
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

    fn attempt(outcome: OutcomeClass) -> HistoryAttempt {
        HistoryAttempt {
            id: None,
            plan: family().baseline,
            outcome,
            latency_ms: 42,
            cost_usd: 0.0,
            endpoint: "https://reader:demo@news.example/nba?utm=feed#top".to_string(),
        }
    }

    #[test]
    fn traces_use_the_harness_schema_and_redacted_endpoints() {
        let trace = Trace::from_attempt(
            &family(),
            &attempt(OutcomeClass::Blocked),
            uuid::Uuid::nil(),
        )
        .expect("trace should build");
        assert_eq!(trace.version, "1.0");
        assert_eq!(trace.outcome, "failed");
        assert_eq!(trace.steps.len(), 1);
        assert!(!trace.steps[0].success);

        let json = serde_json::to_string(&trace).expect("trace should serialise");
        assert!(json.contains("https://news.example/nba"));
        assert!(!json.contains("reader:demo"));
        assert!(!json.contains("utm=feed"));
        assert!(!json.contains('#'));
    }

    #[test]
    fn useful_attempts_map_to_success() {
        assert_eq!(harness_outcome(OutcomeClass::Verified), "success");
        assert_eq!(harness_outcome(OutcomeClass::Irrelevant), "partial");
        assert_eq!(harness_outcome(OutcomeClass::TransportError), "failed");
    }

    #[test]
    fn trace_file_names_are_deterministic() {
        let dir = tempfile::tempdir().expect("temp dir");
        let first = Trace::from_attempt(
            &family(),
            &attempt(OutcomeClass::Verified),
            uuid::Uuid::nil(),
        )
        .expect("trace")
        .write(dir.path())
        .expect("write");
        let second = Trace::from_attempt(
            &family(),
            &attempt(OutcomeClass::Verified),
            uuid::Uuid::nil(),
        )
        .expect("trace")
        .write(dir.path())
        .expect("write");
        assert_eq!(first, second);
        assert!(first.to_string_lossy().ends_with("demo-keyword-alpha.json"));
    }
}
