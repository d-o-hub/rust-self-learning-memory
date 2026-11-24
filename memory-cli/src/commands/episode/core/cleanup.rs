//! Episode retention cleanup CLI command (`do-memory-cli episode cleanup`).
//!
//! Exposes policy-based episode retention and garbage collection (`WG-075`)
//! with preset policies, per-field overrides, candidate counting, and dry-run
//! safety.

use crate::config::Config;
use crate::output::{Output, OutputFormat};
use clap::ValueEnum;
use do_memory_core::SelfLearningMemory;
use do_memory_core::episode::{EpisodeRetentionPolicy, RetentionCriterion};
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Retention policy preset for `episode cleanup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum RetentionPresetArg {
    /// Standard retention (90 days, min reward 0.3, max 50,000 episodes, combined)
    #[default]
    Default,
    /// Limited storage (30 days, min reward 0.5, max 5,000 episodes, combined)
    StorageLimited,
    /// Aggressive cleanup (7 days, min reward 0.6, max 1,000 episodes, drops pattern sources)
    Aggressive,
    /// Archival mode (keeps all episodes)
    Archival,
}

/// Retention criterion override for `episode cleanup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RetentionCriterionArg {
    /// Keep all episodes
    KeepAll,
    /// Delete episodes older than `--max-age-days`
    AgeBased,
    /// Delete episodes below `--min-reward`
    RewardBased,
    /// Delete episodes with no derived patterns or heuristics
    Unreferenced,
    /// Delete failed episodes with no derived patterns or heuristics
    FailedOnly,
    /// Require age, reward, and reference criteria to retain
    Combined,
}

impl From<RetentionCriterionArg> for RetentionCriterion {
    fn from(arg: RetentionCriterionArg) -> Self {
        match arg {
            RetentionCriterionArg::KeepAll => Self::KeepAll,
            RetentionCriterionArg::AgeBased => Self::AgeBased,
            RetentionCriterionArg::RewardBased => Self::RewardBased,
            RetentionCriterionArg::Unreferenced => Self::Unreferenced,
            RetentionCriterionArg::FailedOnly => Self::FailedOnly,
            RetentionCriterionArg::Combined => Self::Combined,
        }
    }
}

/// Parameters for building an [`EpisodeRetentionPolicy`] from CLI flags.
#[derive(Debug, Clone, Default)]
pub struct CleanupPolicyArgs {
    pub preset: RetentionPresetArg,
    pub criterion: Option<RetentionCriterionArg>,
    pub max_age_days: Option<i64>,
    pub min_reward: Option<f64>,
    pub max_episodes: Option<usize>,
    pub batch_size: Option<usize>,
    pub dry_run: bool,
    /// Explicit permission for live deletion; false keeps the policy in dry-run mode.
    pub confirm: bool,
}

impl CleanupPolicyArgs {
    /// Construct and validate an [`EpisodeRetentionPolicy`] from these CLI arguments.
    pub fn build_policy(&self) -> anyhow::Result<EpisodeRetentionPolicy> {
        let mut policy = match self.preset {
            RetentionPresetArg::Default => EpisodeRetentionPolicy::default(),
            RetentionPresetArg::StorageLimited => EpisodeRetentionPolicy::storage_limited(),
            RetentionPresetArg::Aggressive => EpisodeRetentionPolicy::aggressive(),
            RetentionPresetArg::Archival => EpisodeRetentionPolicy::archival(),
        };

        if let Some(criterion) = self.criterion {
            policy = policy.with_criterion(criterion.into());
        }
        if let Some(days) = self.max_age_days {
            policy = policy.with_max_age_days(days);
        }
        if let Some(min_reward) = self.min_reward {
            if !(0.0..=1.0).contains(&min_reward) {
                anyhow::bail!("--min-reward must be between 0.0 and 1.0 (got {min_reward})");
            }
            policy = policy.with_min_reward_threshold(min_reward);
        }
        if let Some(max_episodes) = self.max_episodes {
            policy = policy.with_max_episodes(max_episodes);
        }
        if let Some(batch_size) = self.batch_size {
            if batch_size == 0 {
                anyhow::bail!("--batch-size must be greater than 0");
            }
            policy = policy.with_cleanup_batch_size(batch_size);
        }
        policy = policy.with_dry_run(self.dry_run || !self.confirm);
        policy
            .validate()
            .map_err(|e| anyhow::anyhow!("Invalid retention policy: {e}"))?;
        Ok(policy)
    }
}

/// Structured output for `do-memory-cli episode cleanup`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpisodeCleanupOutput {
    pub dry_run: bool,
    pub count_only: bool,
    pub criterion: String,
    pub evaluated: usize,
    pub deleted: usize,
    pub kept: usize,
    pub deleted_ids: Vec<String>,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

impl Output for EpisodeCleanupOutput {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        if self.count_only {
            writeln!(
                writer,
                "Cleanup candidates ({}): {}",
                self.criterion, self.deleted
            )?;
            return Ok(());
        }

        let mode = if self.dry_run { "[DRY RUN] " } else { "" };
        writeln!(
            writer,
            "{}Episode Cleanup ({}) — evaluated: {}, deleted: {}, kept: {}, duration: {}ms",
            mode, self.criterion, self.evaluated, self.deleted, self.kept, self.duration_ms
        )?;
        for id in &self.deleted_ids {
            writeln!(writer, "  - {id}")?;
        }
        for err in &self.errors {
            writeln!(writer, "  ! {err}")?;
        }
        Ok(())
    }
}

/// Execute the `episode cleanup` CLI subcommand.
pub async fn cleanup_episodes_command(
    args: CleanupPolicyArgs,
    count_only: bool,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let policy = args.build_policy()?;
    let criterion_label = format!("{:?}", policy.criterion);

    let output = if count_only {
        let count = memory.count_cleanup_candidates(&policy).await?;
        EpisodeCleanupOutput {
            dry_run: true,
            count_only: true,
            criterion: criterion_label,
            evaluated: 0,
            deleted: count,
            kept: 0,
            deleted_ids: Vec::new(),
            duration_ms: 0,
            errors: Vec::new(),
        }
    } else {
        let res = memory.cleanup_episodes(&policy).await?;
        EpisodeCleanupOutput {
            dry_run: policy.dry_run,
            count_only: false,
            criterion: criterion_label,
            evaluated: res.evaluated,
            deleted: res.deleted,
            kept: res.kept,
            deleted_ids: res.deleted_ids.iter().map(ToString::to_string).collect(),
            duration_ms: res.duration_ms,
            errors: res.errors,
        }
    };

    format.print_output(&output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use do_memory_core::{MemoryConfig, TaskContext, TaskOutcome, TaskType};

    #[test]
    fn build_policy_applies_presets_and_overrides() {
        let args = CleanupPolicyArgs {
            preset: RetentionPresetArg::StorageLimited,
            criterion: Some(RetentionCriterionArg::FailedOnly),
            max_age_days: Some(14),
            min_reward: Some(0.45),
            max_episodes: Some(250),
            batch_size: Some(25),
            dry_run: true,
            confirm: false,
        };
        let policy = args.build_policy().unwrap();
        assert_eq!(policy.criterion, RetentionCriterion::FailedOnly);
        assert_eq!(policy.max_age_days, 14);
        assert!((policy.min_reward_threshold - 0.45).abs() < f64::EPSILON);
        assert_eq!(policy.max_episodes, 250);
        assert_eq!(policy.cleanup_batch_size, 25);
        assert!(policy.dry_run);
    }

    #[test]
    fn build_policy_rejects_invalid_values() {
        let bad_age = CleanupPolicyArgs {
            max_age_days: Some(-1),
            ..Default::default()
        };
        assert!(bad_age.build_policy().is_err());

        let bad_reward = CleanupPolicyArgs {
            min_reward: Some(1.5),
            ..Default::default()
        };
        assert!(bad_reward.build_policy().is_err());

        let bad_batch = CleanupPolicyArgs {
            batch_size: Some(0),
            ..Default::default()
        };
        assert!(bad_batch.build_policy().is_err());
    }

    #[tokio::test]
    async fn cleanup_command_counts_and_sweeps_failed_episodes() {
        let memory = SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            ..Default::default()
        });
        let ok_id = memory
            .start_episode(
                "Keep successful episode".to_string(),
                TaskContext::default(),
                TaskType::CodeGeneration,
            )
            .await;
        memory
            .complete_episode(
                ok_id,
                TaskOutcome::Success {
                    verdict: "ok".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

        let fail_id = memory
            .start_episode(
                "Clean failed episode".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory
            .complete_episode(
                fail_id,
                TaskOutcome::Failure {
                    reason: "failed".to_string(),
                    error_details: None,
                },
            )
            .await
            .unwrap();

        let args = CleanupPolicyArgs {
            criterion: Some(RetentionCriterionArg::FailedOnly),
            dry_run: true,
            ..Default::default()
        };
        let policy = args.build_policy().unwrap();
        assert_eq!(memory.count_cleanup_candidates(&policy).await.unwrap(), 1);

        let live_policy = CleanupPolicyArgs {
            criterion: Some(RetentionCriterionArg::FailedOnly),
            dry_run: false,
            confirm: true,
            ..Default::default()
        }
        .build_policy()
        .unwrap();
        let res = memory.cleanup_episodes(&live_policy).await.unwrap();
        assert_eq!(res.deleted_ids, vec![fail_id]);
        assert!(memory.get_episode(fail_id).await.is_err());
        assert!(memory.get_episode(ok_id).await.is_ok());

        let out = EpisodeCleanupOutput {
            dry_run: false,
            count_only: false,
            criterion: "FailedOnly".to_string(),
            evaluated: res.evaluated,
            deleted: res.deleted,
            kept: res.kept,
            deleted_ids: vec![fail_id.to_string()],
            duration_ms: res.duration_ms,
            errors: vec![],
        };
        let mut buf = Vec::new();
        out.write_human(&mut buf).unwrap();
        let rendered = String::from_utf8(buf).unwrap();
        assert!(rendered.contains("deleted: 1"));
        assert!(rendered.contains(&fail_id.to_string()));
    }
}
