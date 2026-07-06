use std::time::{Duration, SystemTime};
use memory_core::reward::{
    AdaptiveRewardCalculator,
    DomainStatistics,
    RewardConfig,
    decay::decay_factor,
    normalize::normalize_reward_auto,
};
use memory_core::reward::types::EffectiveReward;

#[test]
fn test_decay_factor_age() {
    let half_life = Duration::from_secs(86400); // 1 day
    // At age = 0, factor = 1
    assert!((decay_factor(Duration::from_secs(0), half_life) - 1.0).abs() < 1e-6);
    // At age = half_life, factor = 0.5
    let factor = decay_factor(Duration::from_secs(86400), half_life);
    assert!((factor - 0.5).abs() < 1e-6);
    // At age = 2*half_life, factor = 0.25
    let factor = decay_factor(Duration::from_secs(2 * 86400), half_life);
    assert!((factor - 0.25).abs() < 1e-6);
}

#[test]
fn test_normalization_zscore() {
    let stats = DomainStatistics {
        mean: 10.0,
        std: 2.0,
        min: 0.0,
        max: 20.0,
    };
    let raw = 12.0;
    let normalized = normalize_reward_auto(raw, &stats);
    assert!((normalized - 1.0).abs() < 1e-6);
}

#[test]
fn test_normalization_minmax() {
    let stats = DomainStatistics {
        mean: 10.0,
        std: 0.0,
        min: 0.0,
        max: 20.0,
    };
    let raw = 5.0;
    let normalized = normalize_reward_auto(raw, &stats);
    assert!((normalized - 0.25).abs() < 1e-6);
}

#[test]
fn test_effective_reward_computation() {
    let mut config = RewardConfig::default();
    config.half_life_by_domain.insert("test_domain".to_string(), Duration::from_secs(86400));
    let mut calculator = AdaptiveRewardCalculator::new(config);
    let mut stats = std::collections::HashMap::new();
    stats.insert("test_domain".to_string(), DomainStatistics {
        mean: 0.0,
        std: 1.0,
        min: -10.0,
        max: 10.0,
    });
    calculator.update_domain_stats(stats);

    let raw = 5.0;
    let event_time = SystemTime::now() - Duration::from_secs(86400); // 1 day ago
    let now = SystemTime::now();
    let effective = calculator.compute_effective_reward(raw, event_time, "test_domain", now);

    // raw=5, normalized = (5-0)/1 =5, decay factor for age=1 day half_life=1 day => 0.5, decayed=2.5, effective=2.5
    assert!((effective.normalized_reward - 5.0).abs() < 1e-6);
    assert!((effective.decayed_reward - 2.5).abs() < 1e-6);
    assert!((effective.effective_reward - 2.5).abs() < 1e-6);
}
