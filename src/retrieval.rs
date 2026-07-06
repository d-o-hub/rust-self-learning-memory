use crate::episode::Episode;
use crate::reward::DualRewardScore;

/// Rank episodes by effective reward in descending order.
/// This updates the retrieval ranking to use effective_reward instead of raw score.
pub fn rank_episodes_by_effective_reward(episodes: &mut [Episode]) {
    episodes.sort_by(|a, b| {
        b.effective_reward.partial_cmp(&a.effective_reward).unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeEpisode {
        effective_reward: f64,
    }

    impl Episode for FakeEpisode {}

    #[test]
    fn test_ranking() {
        let mut episodes = vec![
            FakeEpisode { effective_reward: 5.0 },
            FakeEpisode { effective_reward: 10.0 },
            FakeEpisode { effective_reward: 1.0 },
        ];
        rank_episodes_by_effective_reward(&mut episodes);
        assert_eq!(episodes[0].effective_reward, 10.0);
        assert_eq!(episodes[1].effective_reward, 5.0);
        assert_eq!(episodes[2].effective_reward, 1.0);
    }
}