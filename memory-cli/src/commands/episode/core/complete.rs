//! Complete episode command implementation
//!
//! ADR-075: after a successful `complete_episode` call the CLI re-fetches the
//! episode and asserts `is_complete()` before printing success. Operator
//! `episode fail` reuses this path with [`TaskOutcome::Failure`].

use super::types::TaskOutcome;
use crate::config::Config;
#[cfg(feature = "turso")]
use crate::output::Output;
use crate::output::OutputFormat;
use do_memory_core::SelfLearningMemory;
use do_memory_core::TaskOutcome as CoreTaskOutcome;

/// Map CLI [`TaskOutcome`] to the core outcome variant used by memory-core.
#[must_use]
pub(crate) fn map_cli_outcome(outcome: TaskOutcome) -> CoreTaskOutcome {
    match outcome {
        TaskOutcome::Success => CoreTaskOutcome::Success {
            verdict: "Task completed successfully via CLI".to_string(),
            artifacts: vec![],
        },
        TaskOutcome::PartialSuccess => CoreTaskOutcome::PartialSuccess {
            verdict: "Task completed with partial success via CLI".to_string(),
            completed: vec!["Marked as partial success by user".to_string()],
            failed: vec![],
        },
        TaskOutcome::Failure => CoreTaskOutcome::Failure {
            reason: "Task failed via CLI".to_string(),
            error_details: Some("Marked as failed by user".to_string()),
        },
    }
}

/// Return `true` when the stored core outcome kind matches the CLI request.
#[must_use]
pub(crate) fn outcome_kind_matches(expected: TaskOutcome, actual: &CoreTaskOutcome) -> bool {
    matches!(
        (expected, actual),
        (TaskOutcome::Success, CoreTaskOutcome::Success { .. })
            | (
                TaskOutcome::PartialSuccess,
                CoreTaskOutcome::PartialSuccess { .. }
            )
            | (TaskOutcome::Failure, CoreTaskOutcome::Failure { .. })
    )
}

/// Complete an episode and verify durability before printing success (ADR-075).
pub async fn complete_episode(
    episode_id: String,
    outcome: TaskOutcome,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    use uuid::Uuid;

    if dry_run {
        println!(
            "Would complete episode {} with outcome: {:?}",
            episode_id, outcome
        );
        return Ok(());
    }

    // Parse episode ID
    let episode_uuid = Uuid::parse_str(&episode_id)
        .map_err(|_| anyhow::anyhow!("Invalid episode ID format: {}", episode_id))?;

    let core_outcome = map_cli_outcome(outcome);

    // Ensure the episode exists before attempting completion
    memory
        .get_episode(episode_uuid)
        .await
        .map_err(|e| anyhow::anyhow!("Episode not found {}: {}", episode_id, e))?;

    memory
        .complete_episode(episode_uuid, core_outcome)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to complete episode {}: {}", episode_id, e))?;

    // Verify-after-write (ADR-075): re-fetch and assert durable completion.
    // Do not print success unless the re-read confirms is_complete().
    let verified = memory.get_episode(episode_uuid).await.map_err(|e| {
        anyhow::anyhow!(
            "Episode {} completed in-memory but re-fetch failed \
             (possible backend write failure, wrong --db-path, or corrupted entry): {}",
            episode_id,
            e
        )
    })?;

    if !verified.is_complete() {
        anyhow::bail!(
            "Episode {} was not durable after complete (is_complete=false). \
             Likely cause: backend write failure, wrong --db-path, or corrupted entry.",
            episode_id
        );
    }

    match &verified.outcome {
        Some(stored) if outcome_kind_matches(outcome, stored) => {}
        Some(stored) => {
            anyhow::bail!(
                "Episode {} completed but stored outcome kind {:?} does not match requested {:?}.",
                episode_id,
                stored,
                outcome
            );
        }
        None => {
            // is_complete() already requires outcome, but keep an explicit path.
            anyhow::bail!(
                "Episode {} was not durable after complete (missing outcome). \
                 Likely cause: backend write failure, wrong --db-path, or corrupted entry.",
                episode_id
            );
        }
    }

    print_complete_success(&episode_id, outcome, format)
}

/// Operator path: force-fail an abandoned in-progress episode (ADR-075).
///
/// Equivalent to `episode complete <id> failure` with the same verify-after-write
/// durability rules.
pub async fn fail_episode(
    episode_id: String,
    memory: &SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    complete_episode(
        episode_id,
        TaskOutcome::Failure,
        memory,
        config,
        format,
        dry_run,
    )
    .await
}

fn print_complete_success(
    episode_id: &str,
    outcome: TaskOutcome,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let outcome_str = format!("{:?}", outcome);

    #[cfg(feature = "turso")]
    {
        #[derive(Debug, serde::Serialize)]
        struct CompleteResult {
            episode_id: String,
            status: String,
            outcome: String,
        }

        impl Output for CompleteResult {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;
                writeln!(writer, "{}", "Episode Completed".green().bold())?;
                writeln!(writer, "Episode: {}", self.episode_id.dimmed())?;
                writeln!(writer, "Status: {}", self.status.green())?;
                writeln!(writer, "Outcome: {}", self.outcome)?;
                Ok(())
            }
        }

        let result = CompleteResult {
            episode_id: episode_id.to_string(),
            status: "completed".to_string(),
            outcome: outcome_str,
        };

        return format.print_output(&result);
    }

    #[cfg(not(feature = "turso"))]
    {
        match format {
            OutputFormat::Json => {
                let result = serde_json::json!({
                    "episode_id": episode_id,
                    "status": "completed",
                    "outcome": outcome_str,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            OutputFormat::Yaml => {
                let result = serde_json::json!({
                    "episode_id": episode_id,
                    "status": "completed",
                    "outcome": outcome_str,
                });
                println!("{}", serde_yaml::to_string(&result)?);
            }
            OutputFormat::Human => {
                println!("Episode completed: {}", episode_id);
                println!("Status: completed");
                println!("Outcome: {}", outcome_str);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use do_memory_core::TaskOutcome as CoreTaskOutcome;

    #[test]
    fn map_cli_outcome_success() {
        let mapped = map_cli_outcome(TaskOutcome::Success);
        assert!(matches!(mapped, CoreTaskOutcome::Success { .. }));
    }

    #[test]
    fn map_cli_outcome_partial() {
        let mapped = map_cli_outcome(TaskOutcome::PartialSuccess);
        assert!(matches!(mapped, CoreTaskOutcome::PartialSuccess { .. }));
    }

    #[test]
    fn map_cli_outcome_failure() {
        let mapped = map_cli_outcome(TaskOutcome::Failure);
        assert!(matches!(mapped, CoreTaskOutcome::Failure { .. }));
    }

    #[test]
    fn outcome_kind_matches_success() {
        let actual = CoreTaskOutcome::Success {
            verdict: "ok".into(),
            artifacts: vec![],
        };
        assert!(outcome_kind_matches(TaskOutcome::Success, &actual));
        assert!(!outcome_kind_matches(TaskOutcome::Failure, &actual));
    }

    #[test]
    fn outcome_kind_matches_failure() {
        let actual = CoreTaskOutcome::Failure {
            reason: "boom".into(),
            error_details: None,
        };
        assert!(outcome_kind_matches(TaskOutcome::Failure, &actual));
        assert!(!outcome_kind_matches(TaskOutcome::Success, &actual));
        assert!(!outcome_kind_matches(TaskOutcome::PartialSuccess, &actual));
    }

    #[test]
    fn outcome_kind_matches_partial() {
        let actual = CoreTaskOutcome::PartialSuccess {
            verdict: "half".into(),
            completed: vec![],
            failed: vec![],
        };
        assert!(outcome_kind_matches(TaskOutcome::PartialSuccess, &actual));
        assert!(!outcome_kind_matches(TaskOutcome::Success, &actual));
    }

    #[test]
    fn fail_maps_to_failure_outcome() {
        // episode fail reuses complete_episode with TaskOutcome::Failure
        let mapped = map_cli_outcome(TaskOutcome::Failure);
        assert!(matches!(
            mapped,
            CoreTaskOutcome::Failure {
                reason,
                error_details: Some(_)
            } if reason.contains("CLI")
        ));
    }
}
