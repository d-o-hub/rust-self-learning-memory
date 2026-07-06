use crate::reward::DomainStatistics;

/// Normalize a raw reward using domain statistics.
/// 
/// # Arguments
/// * `raw_reward` - The raw reward value.
/// * `stats` - Domain statistics containing mean and std for the domain.
///
/// Returns a z-score normalized reward.
pub fn normalize_reward(raw_reward: f64, stats: &DomainStatistics) -> f64 {
    if stats.std == 0.0 {
        return 0.0;
    }
    (raw_reward - stats.mean) / stats.std
}

/// Alternative normalization: min-max scaling.
pub fn normalize_reward_minmax(raw_reward: f64, stats: &DomainStatistics) -> f64 {
    if stats.max == stats.min {
        return 0.0;
    }
    (raw_reward - stats.min) / (stats.max - stats.min)
}

/// Choose normalization method based on config.
pub fn normalize_reward_auto(raw_reward: f64, stats: &DomainStatistics) -> f64 {
    // Default to z-score if std > 0, else min-max.
    if stats.std > 1e-10 {
        normalize_reward(raw_reward, stats)
    } else {
        normalize_reward_minmax(raw_reward, stats)
    }
}
