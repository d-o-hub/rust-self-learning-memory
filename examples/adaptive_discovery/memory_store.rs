//! Episodic-memory adapter for discovery attempts.
//!
//! Every attempt becomes one episode: the plan is carried by step parameters and
//! by tags, the outcome class by a tag, and the candidate endpoint only in its
//! redacted form. Retrieval reads the tags back, so the example exercises the
//! same record-then-retrieve path an external application would.
//!
//! An attempt also carries a stable identity, and that identity decides which
//! episode it resolves to. Seeding is therefore a *replay*: recording the same
//! dataset twice leaves one episode per attempt instead of minting a second
//! observation that the planner would count as independent evidence.

use anyhow::{Context, Result};
use do_memory_core::{
    ExecutionResult, ExecutionStep, MemoryConfig, Pattern, SelfLearningMemory, TaskContext,
    TaskOutcome, TaskType,
};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use crate::adaptive_discovery::dataset::{
    HistoryAttempt, OutcomeClass, QueryFamily, RedactedEndpoint,
};
use crate::adaptive_discovery::planner::Observation;
use crate::adaptive_discovery::tags::{attempt_tag, family_tag, parse_observation, tags_for};

/// How many retrieval candidates to request per observation the caller wants.
///
/// Retrieval ranks by a quality-weighted relevance score, so the episodes a
/// planner needs most — blocked or failed attempts — are the ones that rank
/// lowest. Asking for only as many candidates as you intend to use lets a
/// busier store crowd them out entirely.
const CANDIDATE_WINDOW_MULTIPLIER: usize = 4;

/// Lower bound on the retrieval window, so a small `--limit` still sees enough
/// of the store to find the family's history.
const CANDIDATE_WINDOW_FLOOR: usize = 64;

/// The retrieval window to ask for when `limit` observations are wanted.
#[must_use]
fn candidate_window(limit: usize) -> usize {
    limit
        .saturating_mul(CANDIDATE_WINDOW_MULTIPLIER)
        .max(CANDIDATE_WINDOW_FLOOR)
}

// Episode tags are normalised and validated by the memory system: only
// alphanumeric characters, hyphens, and underscores survive. The tag vocabulary
// lives in the `tags` module so recording and retrieval cannot drift apart.

/// What recording an attempt did to the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordState {
    /// The attempt had no episode yet; a new one was recorded.
    Created,
    /// The attempt's completed episode already existed and was left untouched.
    Replayed,
    /// An interrupted attempt's partial episode was discarded and re-recorded.
    Retried,
}

/// The episode an attempt resolved to, and how it got there.
#[derive(Debug, Clone, Copy)]
pub struct Recorded {
    /// Episode that now records the attempt.
    pub episode_id: Uuid,
    /// Whether that episode was created, replayed, or retried.
    pub state: RecordState,
}

/// How an attempt's identity resolved against the existing store.
enum Replay {
    /// Nothing recorded this attempt yet.
    Absent,
    /// A partial episode was removed so the attempt can be recorded again.
    Retried,
    /// A completed episode already records this attempt.
    Completed(Uuid),
}

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
    /// `attempt_id` is the attempt's stable identity (see
    /// [`HistoryAttempt::identity`]). Recording an attempt whose identity already
    /// resolves to a completed episode is a no-op that returns that episode, so a
    /// replay cannot add a second observation. A partial episode — the residue of
    /// an import that failed between tagging and completion — is discarded and
    /// recorded again, which keeps one identity mapped to one completed episode.
    ///
    /// # Errors
    ///
    /// Returns an error when the endpoint is not a valid absolute URL, a tag is
    /// rejected, the store cannot be queried, or the episode cannot be completed.
    /// A partially recorded attempt is never silently accepted.
    pub async fn record(
        &self,
        family: &QueryFamily,
        attempt: &HistoryAttempt,
        attempt_id: &str,
    ) -> Result<Recorded> {
        let plan = &attempt.plan;
        let endpoint = RedactedEndpoint::parse(&attempt.endpoint)?;
        let identity_tag = attempt_tag(attempt_id);

        let state = match self.resolve(&identity_tag).await? {
            Replay::Completed(episode_id) => {
                return Ok(Recorded {
                    episode_id,
                    state: RecordState::Replayed,
                });
            }
            Replay::Retried => RecordState::Retried,
            Replay::Absent => RecordState::Created,
        };

        let episode_id = self
            .memory
            .start_episode(
                family.query.clone(),
                task_context(family),
                TaskType::Analysis,
            )
            .await;

        self.memory
            .add_episode_tags(
                episode_id,
                tags_for(family, plan, attempt.outcome, attempt_id),
            )
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

        Ok(Recorded { episode_id, state })
    }

    /// Resolve an attempt identity against the episodes already stored.
    ///
    /// Completeness is what separates an observation from an interrupted write:
    /// an episode whose outcome is still open carries no finished attempt, so it
    /// is removed rather than counted or trusted.
    ///
    /// The lookup scans the store's episodes. That is fine for a fixture of a few
    /// dozen rows; an importer with a large history should keep its own
    /// attempt-to-episode index rather than reusing this shape.
    async fn resolve(&self, attempt_tag: &str) -> Result<Replay> {
        let existing = self
            .memory
            .list_episodes_by_tags(vec![attempt_tag.to_string()], true, Some(1))
            .await
            .context("cannot look up a previously recorded attempt")?;

        let Some(episode) = existing.into_iter().next() else {
            return Ok(Replay::Absent);
        };
        if episode.outcome.is_some() {
            return Ok(Replay::Completed(episode.episode_id));
        }

        self.memory
            .delete_episode(episode.episode_id)
            .await
            .with_context(|| {
                format!(
                    "cannot discard the interrupted episode {} of {attempt_tag}",
                    episode.episode_id
                )
            })?;
        Ok(Replay::Retried)
    }

    /// Retrieve prior observations for a family from episodic memory.
    ///
    /// `limit` is how many observations the caller wants to *use*. The retrieval
    /// window is wider than that (see [`candidate_window`]) because retrieval
    /// ranks by quality-weighted relevance: a family's blocked attempts rank
    /// last, and those are exactly the evidence that a plan has stopped working.
    /// A wider window keeps a busy store from starving the family, and the
    /// result is truncated back to `limit` afterwards.
    ///
    /// Returns the observations plus the number of retrieved episodes (in the
    /// candidate window) that could not contribute evidence: one without usable
    /// plan/outcome tags, or a second episode for an attempt identity that was
    /// already counted. Episodes from other families are skipped and are not
    /// counted as ignored.
    ///
    /// Retrieval itself only returns completed episodes, so an interrupted import
    /// cannot become evidence even before it is retried.
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
            .retrieve_relevant_context(
                family.query.clone(),
                task_context(family),
                candidate_window(limit),
            )
            .await;

        let mut observations = Vec::new();
        let mut ignored = 0;
        let mut counted: BTreeSet<String> = BTreeSet::new();
        for episode in episodes {
            let Some(parsed) = parse_observation(&episode.tags) else {
                ignored += 1;
                continue;
            };
            if parsed.family != family.id {
                continue;
            }
            if let Some(attempt) = parsed.attempt {
                if !counted.insert(attempt) {
                    ignored += 1;
                    continue;
                }
            }
            observations.push(parsed.observation);
        }
        observations.truncate(limit);
        Ok((observations, ignored))
    }

    /// Count episodes for `family` that carry no attempt identity.
    ///
    /// A database seeded before attempts had identities cannot be replayed: there
    /// is nothing to resolve a fixture row against, so a second seed would mint a
    /// fresh observation for every row. Counting them lets the seed phase refuse
    /// that instead of quietly doubling the evidence.
    ///
    /// # Errors
    ///
    /// Returns an error when the durable store cannot be queried.
    pub async fn legacy_episodes(&self, family: &QueryFamily, limit: usize) -> Result<usize> {
        let episodes = self
            .memory
            .list_episodes_by_tags(
                vec![family_tag(&family.id)],
                true,
                Some(candidate_window(limit)),
            )
            .await
            .context("cannot inspect the family's recorded episodes")?;

        Ok(episodes
            .iter()
            .filter(|episode| {
                parse_observation(&episode.tags)
                    .is_some_and(|parsed| parsed.family == family.id && parsed.attempt.is_none())
            })
            .count())
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
#[path = "memory_store_tests.rs"]
mod tests;
