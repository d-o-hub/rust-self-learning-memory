use std::time::{Duration, SystemTime};
use super::types::*;

/// Adaptive reward calculator with decay and normalization.
pub struct AdaptiveRewardCalculator {
    config: RewardConfig,
    domain_stats: Option<DomainStatistics>,
}

impl AdaptiveRewardCalculator {
    pub fn new(config: RewardConfig) -> Self {
        Self {
            config,
            domain_stats: None,
        }
    }

    pub fn with_domain_stats(mut self, stats: DomainStatistics) -> Self {
        self.domain_stats = Some(stats);
        self
    }

    /// Calculate reward components for a given raw reward and timestamp.
    pub fn calculate(&self, raw_reward: f64, timestamp: SystemTime) -> RewardComponents {
        let now = SystemTime::now();
        let age = now.duration_since(timestamp)
            .unwrap_or(Duration::ZERO)
            .as_secs_f64();

        // Normalization
        let normalized_reward = self.normalize(raw_reward);

        // Decay: exponential decay with half-life
        let decay_factor = (2.0_f64).powf(-age / self.config.half_life_seconds);
        let decayed_reward = raw_reward * decay_factor;

        // Effective reward: weighted combination
        let effective_reward = self.config.normalization_weight * normalized_reward
            + self.config.decay_weight * decayed_reward;

        RewardComponents {
            raw_reward,
            normalized_reward,
            decayed_reward,
            effective_reward,
        }
    }

    fn normalize(&self, raw_reward: f64) -> f64 {
        match &self.domain_stats {
            Some(stats) if stats.sample_count > 0 && stats.std_reward > 0.0 => {
                (raw_reward - stats.mean_reward) / stats.std_reward
            }
            _ => raw_reward,
        }
    }

    /// Update domain statistics (e.g., after recomputation).
    pub fn update_domain_stats(&mut self, stats: DomainStatistics) {
        self.domain_stats = Some(stats);
    }

    pub fn config(&self) -> &RewardConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_reward_decay() {
        let config = RewardConfig {
            half_life_seconds: 3600.0, // 1 hour
            ..Default::default()
        };
        let calc = AdaptiveRewardCalculator::new(config);
        let now = SystemTime::now();
        // Episode from 1 hour ago
        let timestamp = now - Duration::from_secs(3600);
        let comp = calc.calculate(100.0, timestamp);
        // Decay factor = 2^(-1) = 0.5
        assert!((comp.decayed_reward - 50.0).abs() < 1e-6);
        // Effective should be average of normalized (same as raw since no stats) and decayed
        let expected_effective = 0.5 * 100.0 + 0.5 * 50.0;
        assert!((comp.effective_reward - expected_effective).abs() < 1e-6);
    }

    #[test]
    fn test_normalization() {
        let stats = DomainStatistics {
            task_type: "test".to_string(),
            agent_type: "test".to_string(),
            complexity_band: "low".to_string(),
            mean_reward: 10.0,
            std_reward: 2.0,
            sample_count: 100,
        };
        let config = RewardConfig::default();
        let calc = AdaptiveRewardCalculator::new(config)
            .with_domain_stats(stats);
        let comp = calc.calculate(12.0, SystemTime::now());
        // normalized = (12-10)/2 = 1.0
        assert!((comp.normalized_reward - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_std() {
        let stats = DomainStatistics {
            mean_reward: 5.0,
            std_reward: 0.0,
            sample_count: 1,
            ..Default::default()
        };
        let calc = AdaptiveRewardCalculator::new(RewardConfig::default())
            .with_domain_stats(stats);
        let comp = calc.calculate(5.0, SystemTime::now());
        // No normalization, so normalized equals raw
        assert!((comp.normalized_reward - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_no_stats() {
        let calc = AdaptiveRewardCalculator::new(RewardConfig::default());
        let comp = calc.calculate(42.0, SystemTime::now());
        assert!((comp.normalized_reward - 42.0).abs() < 1e-6);
    }
}
