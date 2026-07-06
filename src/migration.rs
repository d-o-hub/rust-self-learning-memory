use crate::domain_statistics::DomainStatistics;
use crate::reward::{AdaptiveRewardCalculator, DualRewardScore, RewardConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents an existing episode with legacy raw reward.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyEpisode {
    pub id: String,
    pub raw_reward: f64,
    pub created_at: DateTime<Utc>,
    pub task_type: String,
    pub agent_type: String,
    pub complexity_band: String,
}

/// Result after migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigratedEpisode {
    pub id: String,
    pub score: DualRewardScore,
}

/// Migrate a batch of legacy episodes to include effective reward.
pub fn migrate_episodes(
    episodes: Vec<LegacyEpisode>,
    config: RewardConfig,
) -> Vec<MigratedEpisode> {
    let calculator = AdaptiveRewardCalculator::new(config);
    episodes
        .into_iter()
        .map(|ep| {
            let score = calculator.compute(
                ep.raw_reward,
                ep.created_at,
                &ep.task_type,
                &ep.agent_type,
                &ep.complexity_band,
            );
            MigratedEpisode {
                id: ep.id,
                score,
            }
        })
        .collect()
}

/// In-place update of a mutable reference.
pub fn recompute_episode(
    episode: &mut LegacyEpisode,
    calculator: &mut AdaptiveRewardCalculator,
) -> DualRewardScore {
    let score = calculator.compute(
        episode.raw_reward,
        episode.created_at,
        &episode.task_type,
        &episode.agent_type,
        &episode.complexity_band,
    );
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    #[test]
    fn test_migration() {
        let config = RewardConfig::default();
        let episodes = vec![LegacyEpisode {
            id: "ep1".to_string(),
            raw_reward: 100.0,
            created_at: Utc::now() - TimeDelta::days(5),
            task_type: "t1".to_string(),
            agent_type: "a1".to_string(),
            complexity_band: "low".to_string(),
        }];
        let migrated = migrate_episodes(episodes, config);
        assert_eq!(migrated.len(), 1);
        assert!(migrated[0].score.effective_reward > 0.0);
    }
}
