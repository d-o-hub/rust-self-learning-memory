pub mod types;
pub mod decay;
pub mod normalize;
pub mod ranking;
pub mod migration;

use std::time::SystemTime;
use crate::reward::types::{EffectiveReward, RewardConfig};
use crate::reward::decay::{age, decay_factor};
use crate::reward::normalize::normalize_reward_auto;

/// Domain statistics: mean, std, min, max for a given domain/agent/complexity.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DomainStatistics {
    pub mean: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
}

/// Calculator for adaptive rewards with decay and normalization.
pub struct AdaptiveRewardCalculator {
    config: RewardConfig,
    domain_stats: std::collections::HashMap<String, DomainStatistics>,
}

impl AdaptiveRewardCalculator {
    pub fn new(config: RewardConfig) -> Self {
        Self {
            config,
            domain_stats: std::collections::HashMap::new(),
        }
    }

    /// Update domain statistics (e.g., from a global store).
    pub fn update_domain_stats(&mut self, stats: std::collections::HashMap<String, DomainStatistics>) {
        self.domain_stats = stats;
    }

    /// Compute effective reward from raw reward, event time, domain, and current time.
    pub fn compute_effective_reward(
        &self,
        raw_reward: f64,
        event_time: SystemTime,
        domain: &str,
        now: SystemTime,
    ) -> EffectiveReward {
        let mut eff = EffectiveReward::new(raw_reward);

        // 1. Normalize
        if let Some(stats) = self.domain_stats.get(domain) {
            eff.normalized_reward = normalize_reward_auto(raw_reward, stats);
        } else {
            eff.normalized_reward = raw_reward;
        }

        // 2. Decay
        let age_dur = age(event_time, now);
        let half_life = self.config.half_life_by_domain
            .get(domain)
            .unwrap_or(&self.config.default_half_life);
        let df = decay_factor(age_dur, *half_life);
        eff.decayed_reward = eff.normalized_reward * df;

        // 3. Effective reward (can be same as decayed or combine further)
        eff.effective_reward = eff.decayed_reward;

        eff
    }
}
