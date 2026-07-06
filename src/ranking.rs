use crate::reward::DualRewardScore;

/// Represents a retrievable episode with its reward score.
#[derive(Debug, Clone)]
pub struct RankedEpisode {
    pub id: String,
    pub score: DualRewardScore,
}

/// Engine that ranks episodes using effective reward.
pub struct RankingEngine;

impl RankingEngine {
    pub fn new() -> Self {
        Self
    }

    /// Sort episodes by effective reward descending.
    pub fn rank(&self, mut episodes: Vec<RankedEpisode>) -> Vec<RankedEpisode> {
        episodes.sort_by(|a, b| {
            b.score.effective_reward
                .partial_cmp(&a.score.effective_reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        episodes
    }

    /// Select top-k episodes.
    pub fn top_k(&self, episodes: Vec<RankedEpisode>, k: usize) -> Vec<RankedEpisode> {
        let mut ranked = self.rank(episodes);
        ranked.truncate(k);
        ranked
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reward::DualRewardScore;

    #[test]
    fn test_ranking() {
        let engine = RankingEngine::new();
        let mut eps = vec![
            RankedEpisode { id: "a".to_string(), score: DualRewardScore { raw_reward: 10.0, normalized_reward: 10.0, decayed_reward: 9.0, effective_reward: 9.0 } },
            RankedEpisode { id: "b".to_string(), score: DualRewardScore { raw_reward: 5.0, normalized_reward: 5.0, decayed_reward: 4.0, effective_reward: 4.0 } },
            RankedEpisode { id: "c".to_string(), score: DualRewardScore { raw_reward: 20.0, normalized_reward: 20.0, decayed_reward: 15.0, effective_reward: 15.0 } },
        ];
        let ranked = engine.top_k(eps, 2);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].id, "c");
        assert_eq!(ranked[1].id, "a");
    }
}
