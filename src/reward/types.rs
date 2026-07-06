use std::time::{Duration, SystemTime};
use crate::reward::DomainStatistics;

/// Configuration for reward computation.
pub struct RewardConfig {
    /// Half-life for exponential decay per domain (keyed by domain id).
    pub half_life_by_domain: std::collections::HashMap<String, Duration>,
    /// Default half-life.
    pub default_half_life: Duration,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            half_life_by_domain: std::collections::HashMap::new(),
            default_half_life: Duration::from_secs(86400 * 30), // 30 days
        }
    }
}

/// Complete reward output for a single episode.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EffectiveReward {
    /// Raw reward as originally computed (e.g., from DualRewardScore).
    pub raw_reward: f64,
    /// Reward normalized by domain statistics.
    pub normalized_reward: f64,
    /// Decayed version of normalized reward.
    pub decayed_reward: f64,
    /// Final effective reward used for ranking.
    pub effective_reward: f64,
}

impl EffectiveReward {
    pub fn new(raw: f64) -> Self {
        Self {
            raw_reward: raw,
            normalized_reward: raw,
            decayed_reward: raw,
            effective_reward: raw,
        }
    }
}

/// Represents the decay state for an episode.
#[derive(Debug, Clone)]
pub struct DecayState {
    pub event_time: SystemTime,
    pub half_life: Duration,
}

impl DecayState {
    pub fn new(half_life: Duration) -> Self {
        Self {
            event_time: SystemTime::now(),
            half_life,
        }
    }

    pub fn decay_factor(&self, now: SystemTime) -> f64 {
        let age = now.duration_since(self.event_time).unwrap_or_default();
        let half_life_secs = self.half_life.as_secs_f64();
        if half_life_secs <= 0.0 {
            return 1.0;
        }
        (-age.as_secs_f64() / half_life_secs * std::f64::consts::LN_2).exp()
    }
}
