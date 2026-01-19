//! Adaptive reward calculator module
//!
//! Provides adaptive reward calculation using domain-specific statistics
//! to calibrate thresholds based on historical episode data.

mod calculator;

#[cfg(test)]
mod tests;

pub use calculator::AdaptiveRewardCalculator;
