//! Episodic-memory adapter for discovery attempts.
//!
//! Every attempt becomes one episode: the plan is carried by step parameters and
//! by tags, the outcome class by a tag, and the candidate endpoint only in its
//! redacted form. Retrieval reads the tags back, so the example exercises the
//! same record-then-retrieve path an external application would.

use anyhow::{Context, Result};
use do_memory_core::{
    ExecutionResult, ExecutionStep, MemoryConfig, Pattern, SelfLearningMemory, TaskContext,
    TaskOutcome, TaskType,
};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use crate::adaptive_discovery::dataset::{
    HistoryAttempt, OutcomeClass, Plan, QueryFamily, RedactedEndpoint,
};
use crate::adaptive_discovery::planner::Observation;

/// Tag prefix that identifies the family an episode belongs to.
const FAMILY_TAG: &str = "family_";
/// Tag prefix that identifies the strategy an episode used.
const STRATEGY_TAG: &str = "strategy_";
/// Tag prefix that identifies the provider an episode used.
const PROVIDER_TAG: &str = "provider_";
/// Tag prefix that identifies how an episode ended.
const OUTCOME_TAG: &str = "outcome_";

// Episode tags are normalised and validated by the memory system: only
// alphanumeric characters, hyphens, and underscores survive, so the prefix
// separator is `_`, not `:`. An external consumer that tags with `family:nba`
// gets a validation error rather than a silently unusable tag.

/// A durable memory handle backed by a local Turso file plus a redb cache.
pub struct DiscoveryMemory {
    memory: SelfLearningMemory,
}

impl DiscoveryMemory {
    /// Open (or create) the memory database for the example.
    ///
    /// Both backends are files on disk, so evidence recorded by one process is
    /// visible to the next one. That is what makes the restart check meaningful.
    ///
    /// # Errors
    ///
    /// Returns an error when the parent directory cannot be created, the local
    /// database cannot be opened, or the schema cannot be initialised.
    pub async fn open(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .with_context(|| format!("cannot create {}", parent.display()))?;
            }
        }

        let turso = do_memory_storage_turso::TursoStorage::new_local(db_path)
            .await
            .with_context(|| {
                format!("cannot open local Turso database at {}", db_path.display())
            })?;
        turso
            .initialize_schema()
            .await
            .context("cannot initialise the Turso schema")?;

        let cache_path = db_path.with_extension("cache.redb");
        let cache = do_memory_storage_redb::RedbStorage::new(&cache_path)
            .await
            .with_context(|| format!("cannot open redb cache at {}", cache_path.display()))?;

        let memory =
            SelfLearningMemory::with_storage(discovery_config(), Arc::new(turso), Arc::new(cache));
        Ok(Self { memory })
    }

    /// Record one past attempt as a completed episode.
    ///
    /// # Errors
    ///
    /// Returns an error when the endpoint is not a valid absolute URL, a tag is
    /// rejected, or the episode cannot be completed. A partially recorded
    /// attempt is never silently accepted.
    pub async fn record(&self, family: &QueryFamily, attempt: &HistoryAttempt) -> Result<Uuid> {
        let plan = &attempt.plan;
        let endpoint = RedactedEndpoint::parse(&attempt.endpoint)?;

        let episode_id = self
            .memory
            .start_episode(
                family.query.clone(),
                task_context(family),
                TaskType::Analysis,
            )
            .await;

        self.memory
            .add_episode_tags(episode_id, tags_for(family, plan, attempt.outcome))
            .await
            .context("cannot tag the discovery episode")?;

        let mut step = ExecutionStep::new(
            1,
            format!("provider:{}", plan.provider),
            format!("strategy={} discover candidate sources", plan.strategy),
        );
        step.latency_ms = attempt.latency_ms;
        step.set_parameters(json!({
            "family": family.id,
            "strategy": plan.strategy,
            "provider": plan.provider,
            "endpoint": endpoint.as_str(),
            "cost_usd": attempt.cost_usd,
            "outcome": attempt.outcome.as_str(),
        }));
        step.result = Some(step_result(attempt.outcome));
        self.memory.log_step(episode_id, step).await;

        self.memory
            .complete_episode(episode_id, task_outcome(family, attempt, &endpoint))
            .await
            .with_context(|| format!("cannot complete episode {episode_id}"))?;

        Ok(episode_id)
    }

    /// Retrieve prior observations for a family from episodic memory.
    ///
    /// Returns the observations plus the number of retrieved episodes that carried
    /// no usable plan/outcome tags. Episodes from other families are skipped.
    ///
    /// # Errors
    ///
    /// Returns an error when the durable store cannot be queried.
    pub async fn observations(
        &self,
        family: &QueryFamily,
        limit: usize,
    ) -> Result<(Vec<Observation>, usize)> {
        let episodes = self
            .memory
            .retrieve_relevant_context(family.query.clone(), task_context(family), limit)
            .await;

        let mut observations = Vec::new();
        let mut ignored = 0;
        for episode in episodes {
            match parse_observation(&episode.tags) {
                Some((episode_family, observation)) => {
                    if episode_family == family.id {
                        observations.push(observation);
                    }
                }
                None => ignored += 1,
            }
        }
        Ok((observations, ignored))
    }

    /// Ask the memory system for pattern-based advice, as an advisory signal.
    ///
    /// Patterns are reported for the operator but never drive the plan decision:
    /// automatic extraction is heuristic, while the episode tags above are the
    /// record of what actually happened.
    ///
    /// # Errors
    ///
    /// Returns an error when pattern search fails.
    pub async fn advisory_patterns(
        &self,
        family: &QueryFamily,
        limit: usize,
    ) -> Result<Vec<String>> {
        let recommendations = self
            .memory
            .recommend_patterns_for_task(&family.query, task_context(family), limit)
            .await
            .context("pattern recommendation failed")?;
        Ok(recommendations
            .iter()
            .map(|recommendation| {
                format!(
                    "{} (relevance {:.2})",
                    pattern_kind(&recommendation.pattern),
                    recommendation.relevance_score
                )
            })
            .collect())
    }
}

/// Memory configuration for a discovery consumer.
///
/// The one deliberate deviation from the defaults is the quality threshold. The
/// default (`0.7`) rejects low-quality episodes at completion time, and a blocked
/// or transport-error attempt scores well below it — which is precisely the
/// evidence that tells the planner a plan has stopped working. An application
/// that stores only its successes cannot learn to abandon a failing strategy.
fn discovery_config() -> MemoryConfig {
    MemoryConfig {
        quality_threshold: 0.0,
        ..MemoryConfig::default()
    }
}

fn task_context(family: &QueryFamily) -> TaskContext {
    TaskContext {
        domain: family.domain.clone(),
        tags: vec![family.id.clone()],
        ..TaskContext::default()
    }
}

fn tags_for(family: &QueryFamily, plan: &Plan, outcome: OutcomeClass) -> Vec<String> {
    vec![
        format!("{FAMILY_TAG}{}", family.id),
        format!("{STRATEGY_TAG}{}", plan.strategy),
        format!("{PROVIDER_TAG}{}", plan.provider),
        format!("{OUTCOME_TAG}{outcome}"),
    ]
}

fn parse_observation(tags: &[String]) -> Option<(String, Observation)> {
    let mut family = None;
    let mut strategy = None;
    let mut provider = None;
    let mut outcome = None;
    for tag in tags {
        if let Some(value) = tag.strip_prefix(FAMILY_TAG) {
            family = Some(value.to_string());
        } else if let Some(value) = tag.strip_prefix(STRATEGY_TAG) {
            strategy = Some(value.to_string());
        } else if let Some(value) = tag.strip_prefix(PROVIDER_TAG) {
            provider = Some(value.to_string());
        } else if let Some(value) = tag.strip_prefix(OUTCOME_TAG) {
            outcome = OutcomeClass::from_tag(value);
        }
    }
    Some((
        family?,
        Observation {
            plan: Plan {
                strategy: strategy?,
                provider: provider?,
            },
            outcome: outcome?,
        },
    ))
}

fn step_result(outcome: OutcomeClass) -> ExecutionResult {
    if outcome.is_useful_signal() {
        ExecutionResult::Success {
            output: format!("outcome={outcome}"),
        }
    } else {
        ExecutionResult::Error {
            message: format!("outcome={outcome}"),
        }
    }
}

fn task_outcome(
    family: &QueryFamily,
    attempt: &HistoryAttempt,
    endpoint: &RedactedEndpoint,
) -> TaskOutcome {
    let plan = &attempt.plan;
    match attempt.outcome {
        OutcomeClass::Verified | OutcomeClass::Useful => TaskOutcome::Success {
            verdict: format!(
                "{} on {} returned a {} source for {}",
                plan.strategy, plan.provider, attempt.outcome, family.id
            ),
            artifacts: vec![endpoint.to_string()],
        },
        OutcomeClass::Empty | OutcomeClass::Irrelevant => TaskOutcome::PartialSuccess {
            verdict: format!(
                "{} on {} returned nothing usable for {}",
                plan.strategy, plan.provider, family.id
            ),
            completed: Vec::new(),
            failed: vec![format!("outcome={}", attempt.outcome)],
        },
        OutcomeClass::Blocked | OutcomeClass::TransportError => TaskOutcome::Failure {
            reason: format!(
                "{} on {} could not discover sources for {}",
                plan.strategy, plan.provider, family.id
            ),
            error_details: Some(format!("outcome={}", attempt.outcome)),
        },
    }
}

fn pattern_kind(pattern: &Pattern) -> &'static str {
    match pattern {
        Pattern::ToolSequence { .. } => "tool_sequence",
        Pattern::DecisionPoint { .. } => "decision_point",
        Pattern::ErrorRecovery { .. } => "error_recovery",
        Pattern::ContextPattern { .. } => "context_pattern",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observations_parse_from_tags() {
        let tags = vec![
            "family_nba".to_string(),
            "strategy_keyword".to_string(),
            "provider_alpha".to_string(),
            "outcome_blocked".to_string(),
        ];
        let (family, observation) = parse_observation(&tags).expect("tags should parse");
        assert_eq!(family, "nba");
        assert_eq!(observation.plan.strategy, "keyword");
        assert_eq!(observation.plan.provider, "alpha");
        assert_eq!(observation.outcome, OutcomeClass::Blocked);
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
