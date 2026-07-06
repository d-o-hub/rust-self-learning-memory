use std::time::Duration;

/// Parameters for exponential decay.
#[derive(Debug, Clone)]
pub struct DecayConfig {
    /// Half-life in seconds. Configurable per domain or globally.
    pub half_life_secs: f64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self { half_life_secs: 86400.0 } // 1 day
    }
}

/// Compute decay factor using exponential decay.
/// Returns a value in (0, 1] where 1 means no decay.
pub fn decay_factor(age: Duration, half_life_secs: f64) -> f64 {
    if half_life_secs <= 0.0 {
        return 1.0;
    }
    let age_secs = age.as_secs_f64();
    (-age_secs * std::f64::consts::LN_2 / half_life_secs).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_no_age_no_decay() {
        let factor = decay_factor(Duration::from_secs(0), 86400.0);
        assert!((factor - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_age_equals_half_life() {
        let factor = decay_factor(Duration::from_secs(86400), 86400.0);
        assert!((factor - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_old_age_small_factor() {
        let factor = decay_factor(Duration::from_secs(86400 * 10), 86400.0);
        assert!(factor < 0.001);
    }
}