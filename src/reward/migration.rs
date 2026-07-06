use std::time::SystemTime;
use super::types::*;
use super::calculator::AdaptiveRewardCalculator;

/// Recompute reward components for existing episodes.
/// All episodes should implement `HasTimestamp` and provide a mutable `reward_components` field.
pub fn recompute_rewards_for_episodes<T>(
    episodes: &mut [T],
    calculator: &AdaptiveRewardCalculator,
    get_timestamp: fn(&T) -> SystemTime,
    get_raw_reward: fn(&T) -> f64,
    set_components: fn(&mut T, RewardComponents),
) where T: Sized {
    for episode in episodes.iter_mut() {
        let raw = get_raw_reward(episode);
        let ts = get_timestamp(episode);
        let components = calculator.calculate(raw, ts);
        set_components(episode, components);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    struct TestEpisode {
        timestamp: SystemTime,
        raw_reward: f64,
        components: Option<RewardComponents>,
    }

    impl TestEpisode {
        fn new(age_secs: u64, raw: f64) -> Self {
            let now = SystemTime::now();
            Self {
                timestamp: now - Duration::from_secs(age_secs),
                raw_reward: raw,
                components: None,
            }
        }
    }

    #[test]
    fn test_recompute() {
        let config = RewardConfig::default();
        let calc = AdaptiveRewardCalculator::new(config);
        let mut episodes = vec![
            TestEpisode::new(0, 100.0),
            TestEpisode::new(86400, 200.0),
        ];

        recompute_rewards_for_episodes(
            &mut episodes,
            &calc,
            |e| e.timestamp,
            |e| e.raw_reward,
            |e, c| e.components = Some(c),
        );

        for ep in &episodes {
            assert!(ep.components.is_some());
            let comp = ep.components.as_ref().unwrap();
            assert_eq!(comp.raw_reward, ep.raw_reward);
        }
    }
}
