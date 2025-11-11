//! Constants used in reward calculation

/// Threshold for "efficient" episode duration (in seconds)
pub const EFFICIENT_DURATION_SECS: f32 = 60.0;

/// Threshold for "efficient" step count
pub const EFFICIENT_STEP_COUNT: usize = 10;

/// Maximum efficiency multiplier
pub const MAX_EFFICIENCY_MULTIPLIER: f32 = 1.5;

/// Minimum efficiency multiplier
pub const MIN_EFFICIENCY_MULTIPLIER: f32 = 0.5;
