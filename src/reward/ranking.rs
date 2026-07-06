use crate::reward::EffectiveReward;

/// Score for ranking: higher effective reward is better.
pub fn rank_score(effective_reward: &EffectiveReward) -> f64 {
    effective_reward.effective_reward
}

/// Retrieve results sorted by effective reward descending.
pub fn sort_by_effective_reward(mut results: Vec<(String, EffectiveReward)>) -> Vec<(String, EffectiveReward)> {
    results.sort_by(|a, b| {
        b.1.effective_reward
            .partial_cmp(&a.1.effective_reward)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}
