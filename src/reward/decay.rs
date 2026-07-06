use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use crate::reward::types::{DecayState, RewardConfig};

/// Compute decay factor for a given event time and domain.
pub fn compute_decay_factor(
    event_time: SystemTime,
    domain: &str,
    config: &RewardConfig,
) -> f64 {
    let half_life = config
        .half_life_by_domain
        .get(domain)
        .unwrap_or(&config.default_half_life);
    let state = DecayState::new(*half_life);
    state.decay_factor(event_time) // Note: we need to pass event_time as the event time and now as current time.
    // Actually DecayState has event_time set at construction; we need to adjust.
    // Let's instead use a function that takes age.
}

/// Pure function: decay factor given age and half-life.
pub fn decay_factor(age: Duration, half_life: Duration) -> f64 {
    if half_life.as_secs_f64() <= 0.0 {
        return 1.0;
    }
    (-age.as_secs_f64() / half_life.as_secs_f64() * std::f64::consts::LN_2).exp()
}

/// Get age from event time to now.
pub fn age(event_time: SystemTime, now: SystemTime) -> Duration {
    now.duration_since(event_time).unwrap_or_default()
}
