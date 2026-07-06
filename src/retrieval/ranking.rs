use crate::reward::types::{DualRewardScore, RewardComponents};

/// Rank episodes using effective reward instead of raw score.
pub fn rank_by_effective_reward(scores: &mut [DualRewardScore]) {
    scores.sort_by(|a, b| {
        let effective_a = a.components.as_ref()
            .map(|c| c.effective_reward)
            .unwrap_or(a.combined); // fallback if no components
        let effective_b = b.components.as_ref()
            .map(|c| c.effective_reward)
            .unwrap_or(b.combined);
        effective_b.partial_cmp(&effective_a).unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reward::types::RewardComponents;

    #[test]
    fn test_ranking() {
        let mut scores = vec![
            DualRewardScore {
                intrinsic: 0.0,
                extrinsic: 0.0,
                combined: 10.0,
                components: Some(RewardComponents {
                    raw_reward: 10.0,
                    normalized_reward: 10.0,
                    decayed_reward: 8.0,
                    effective_reward: 9.0,
                }),
            },
            DualRewardScore {
                intrinsic: 0.0,
                extrinsic: 0.0,
                combined: 20.0,
                components: Some(RewardComponents {
                    raw_reward: 20.0,
                    normalized_reward: 20.0,
                    decayed_reward: 5.0,
                    effective_reward: 12.0,
                }),
            },
            DualRewardScore {
                intrinsic: 0.0,
                extrinsic: 0.0,
                combined: 15.0,
                components: None,
            },
        ];
        rank_by_effective_reward(&mut scores);
        // Expected order: second (effective 12), first (9), third (fallback 15? actually combined 15 > 12, but since effective only used if components present, third has combined=15, so after sorting by effective descending, third with combined 15 should be first? need to check.
        // Our function uses effective if components present, else combined. So third has no components, uses combined=15. That is higher than 12 and 9. So order should be: third (15), second (12), first (9).
        assert_eq!(scores[0].combined, 15.0);
        assert_eq!(scores[1].combined, 20.0);
        assert_eq!(scores[2].combined, 10.0);
    }
}
