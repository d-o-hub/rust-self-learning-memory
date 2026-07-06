use std::time::{Duration, SystemTime};

/// Represents reward components for an episode.
#[derive(Debug, Clone, PartialEq)]
pub struct RewardComponents {
    /// Raw accumulated reward.
    pub raw_reward: f64,
    /// Normalized reward using domain statistics.
    pub normalized_reward: f64,
    /// Reward after temporal decay.
    pub decayed_reward: f64,
    /// Effective reward used for ranking.
    pub effective_reward: f64,
}

/// Configuration for reward decay and normalization.
#[derive(Debug, Clone)]
pub struct RewardConfig {
    /// Half-life in seconds for exponential decay.
    pub half_life_seconds: f64,
    /// Weight for normalization (0..1).
    pub normalization_weight: f64,
    /// Weight for decay (0..1).
    pub decay_weight: f64,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            half_life_seconds: 86400.0, // 1 day
            normalization_weight: 0.5,
            decay_weight: 0.5,
        }
    }
}

/// Domain statistics for normalization.
#[derive(Debug, Clone)]
pub struct DomainStatistics {
    pub task_type: String,
    pub agent_type: String,
    pub complexity_band: String,
    pub mean_reward: f64,
    pub std_reward: f64,
    pub sample_count: u64,
}

/// Dual reward score (existing type, extended).
#[derive(Debug, Clone)]
pub struct DualRewardScore {
    pub intrinsic: f64,
    pub extrinsic: f64,
    pub combined: f64,
    pub components: Option<RewardComponents>,
}

/// Trait for entities that can provide timestamps.
pub trait HasTimestamp {
    fn timestamp(&self) -> SystemTime;
}
