use crate::reward::{AdaptiveRewardCalculator, RewardCalculator, DualRewardScore};
use crate::domain::DomainStatistics;
use crate::episode::Episode; // Assuming Episode exists with reward, domain_stats, timestamp
use std::time::{SystemTime, Duration};

/// Recompute effective reward for all existing episodes.
/// This is a lazy recomputation that can be run on startup or as a migration.
pub fn recompute_rewards_for_episodes(episodes: &mut [Episode], calculator: &AdaptiveRewardCalculator) {
    let now = SystemTime::now();
    for episode in episodes.iter_mut() {
        if let Some(domain_stats) = &episode.domain_stats {
            let age = now.duration_since(episode.timestamp)
                .unwrap_or(Duration::from_secs(0));
            let score = calculator.calculate(episode.raw_reward, domain_stats, age);
            episode.effective_reward = score.effective_reward;
            // Store other components if needed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainStatistics;
    use std::time::{SystemTime, Duration};

    struct FakeEpisode {
        raw_reward: f64,
        domain_stats: Option<DomainStatistics>,
        timestamp: SystemTime,
        effective_reward: f64,
    }

    impl Episode for FakeEpisode {}

    #[test]
    fn test_recompute() {
        let mut episodes = vec![
            FakeEpisode {
                raw_reward: 100.0,
                domain_stats: Some(DomainStatistics {
                    task_type: "test".to_string(),
                    agent_type: "default".to_string(),
                    complexity_band: "low".to_string(),
                    mean: 80.0,
                    std_dev: 10.0,
                }),
                timestamp: SystemTime::now() - Duration::from_secs(86400),
                effective_reward: 0.0,
            }
        ];
        let calculator = AdaptiveRewardCalculator::default();
        recompute_rewards_for_episodes(&mut episodes, &calculator);
        // At half-life (1 day), normalized reward = (100-80)/10 = 2.0, decay = 0.5, effective = 1.0
        assert!((episodes[0].effective_reward - 1.0).abs() < 1e-6);
    }
}