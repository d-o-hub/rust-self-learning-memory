use crate::domain::DomainStatistics;

/// Normalize a raw reward using domain statistics.
/// Uses z-score normalization based on mean and standard deviation.
/// If std_dev is zero, returns raw reward.
pub fn normalize_reward(raw: f64, stats: &DomainStatistics) -> f64 {
    if stats.std_dev <= 0.0 {
        return raw;
    }
    (raw - stats.mean) / stats.std_dev
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainStatistics;

    #[test]
    fn test_normalization() {
        let stats = DomainStatistics {
            task_type: "test".to_string(),
            agent_type: "default".to_string(),
            complexity_band: "low".to_string(),
            mean: 50.0,
            std_dev: 10.0,
        };
        let normalized = normalize_reward(60.0, &stats);
        assert!((normalized - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_std_dev() {
        let stats = DomainStatistics {
            task_type: "test".to_string(),
            agent_type: "default".to_string(),
            complexity_band: "low".to_string(),
            mean: 50.0,
            std_dev: 0.0,
        };
        let normalized = normalize_reward(60.0, &stats);
        assert!((normalized - 60.0).abs() < 1e-6);
    }
}