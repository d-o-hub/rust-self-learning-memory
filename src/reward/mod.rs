pub mod types;
pub mod decay;
pub mod normalization;

use std::time::Duration;
use crate::domain::DomainStatistics;
use types::DualRewardScore;
use decay::{DecayConfig, decay_factor};
use normalization::normalize_reward;

/// Trait for reward calculation.
pub trait RewardCalculator {
    /// Compute effective reward from raw reward, domain statistics, and episode age.
    fn calculate(&self, raw_reward: f64, domain_stats: &DomainStatistics, age: Duration) -> DualRewardScore;
}

/// The adaptive reward calculator implementation.
#[derive(Debug, Clone)]
pub struct AdaptiveRewardCalculator {
    pub decay_config: DecayConfig,
}

impl Default for AdaptiveRewardCalculator {
    fn default() -> Self {
        Self {
            decay_config: DecayConfig::default(),
        }
    }
}

impl RewardCalculator for AdaptiveRewardCalculator {
    fn calculate(&self, raw_reward: f64, domain_stats: &DomainStatistics, age: Duration) -> DualRewardScore {
        let normalized = normalize_reward(raw_reward, domain_stats);
        let factor = decay_factor(age, self.decay_config.half_life_secs);
        let decayed = normalized * factor;
        // Effective reward can combine both; here we use decayed as effective.
        DualRewardScore {
            raw_reward,
            normalized_reward: normalized,
            decayed_reward: decayed,
            effective_reward: decayed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainStatistics;
    use std::time::Duration;

    #[test]
    fn test_calculate() {
        let calculator = AdaptiveRewardCalculator::default();
        let stats = DomainStatistics {
            task_type: "test".to_string(),
            agent_type: "default".to_string(),
            complexity_band: "low".to_string(),
            mean: 100.0,
            std_dev: 20.0,
        };
        let age = Duration::from_secs(86400); // 1 day
        let score = calculator.calculate(120.0, &stats, age);
        assert!((score.raw_reward - 120.0).abs() < 1e-6);
        assert!((score.normalized_reward - 1.0).abs() < 1e-6);
        assert!((score.decayed_reward - 0.5).abs() < 1e-6);
        assert!((score.effective_reward - 0.5).abs() < 1e-6);
    }
}