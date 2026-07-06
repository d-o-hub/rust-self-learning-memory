use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::domain_statistics::DomainStatistics;

/// Represents the dual nature of reward: performance and agreement (aligns with existing concept).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualRewardScore {
    pub raw_reward: f64,
    pub normalized_reward: f64,
    pub decayed_reward: f64,
    pub effective_reward: f64,
}

impl DualRewardScore {
    pub fn new(raw: f64) -> Self {
        Self {
            raw_reward: raw,
            normalized_reward: raw,
            decayed_reward: raw,
            effective_reward: raw,
        }
    }

    /// Apply normalization using domain statistics.
    pub fn normalize(&mut self, stats: &DomainStatistics, task_type: &str, agent_type: &str, complexity_band: &str) {
        self.normalized_reward = stats.normalize(self.raw_reward, task_type, agent_type, complexity_band);
    }

    /// Apply temporal decay.
    pub fn decay(&mut self, created_at: DateTime<Utc>, half_life: Duration) {
        let age = Utc::now().signed_duration_since(created_at).to_std().unwrap_or(Duration::ZERO);
        let decay_factor = (- (age.as_secs_f64() * std::f64::consts::LN_2) / half_life.as_secs_f64()).exp();
        self.decayed_reward = self.normalized_reward * decay_factor;
    }

    /// Compute effective reward as combination (here simply decayed).
    pub fn compute_effective(&mut self) {
        self.effective_reward = self.decayed_reward;
    }
}

/// Configuration for reward calculation per domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardConfig {
    pub half_life: Duration,
    pub domain_statistics: DomainStatistics,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            half_life: Duration::from_secs(86400 * 30), // 30 days
            domain_statistics: DomainStatistics::new(),
        }
    }
}

/// Calculator that orchestrates reward computation.
pub struct AdaptiveRewardCalculator {
    pub config: RewardConfig,
}

impl AdaptiveRewardCalculator {
    pub fn new(config: RewardConfig) -> Self {
        Self { config }
    }

    /// Compute effective reward for an episode.
    pub fn compute(
        &self,
        raw: f64,
        created_at: DateTime<Utc>,
        task_type: &str,
        agent_type: &str,
        complexity_band: &str,
    ) -> DualRewardScore {
        let mut score = DualRewardScore::new(raw);
        score.normalize(&self.config.domain_statistics, task_type, agent_type, complexity_band);
        score.decay(created_at, self.config.half_life);
        score.compute_effective();
        score
    }

    /// Update domain statistics with this episode's raw reward.
    pub fn update_statistics(
        &mut self,
        raw: f64,
        task_type: &str,
        agent_type: &str,
        complexity_band: &str,
    ) {
        self.config.domain_statistics.update(raw, task_type, agent_type, complexity_band);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    #[test]
    fn test_reward_computation() {
        let mut calc = AdaptiveRewardCalculator::new(RewardConfig::default());
        let created = Utc::now() - TimeDelta::days(10);
        calc.update_statistics(100.0, "task1", "agentA", "low");
        calc.update_statistics(200.0, "task1", "agentA", "low");
        let score = calc.compute(150.0, created, "task1", "agentA", "low");
        assert!(score.raw_reward == 150.0);
        assert!(score.normalized_reward > 0.0);
        assert!(score.decayed_reward < score.normalized_reward);
        assert!((score.effective_reward - score.decayed_reward).abs() < 1e-10);
    }

    #[test]
    fn test_decay_factor() {
        let calc = AdaptiveRewardCalculator::new(RewardConfig::default());
        let zero_age = Utc::now();
        let score = calc.compute(1.0, zero_age, "t", "a", "c");
        assert!((score.decayed_reward - score.normalized_reward).abs() < 1e-6);
    }
}
