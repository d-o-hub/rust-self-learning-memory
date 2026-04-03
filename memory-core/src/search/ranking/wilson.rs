//! Wilson score confidence interval computation
//!
//! Provides Bayesian ranking using the Wilson score interval,
//! useful for conservative estimates of true success rates.

/// Z-scores for common confidence levels
pub mod z_scores {
    /// 90% confidence level
    pub const CONFIDENCE_90: f64 = 1.645;
    /// 95% confidence level
    pub const CONFIDENCE_95: f64 = 1.96;
    /// 99% confidence level
    pub const CONFIDENCE_99: f64 = 2.576;
}

/// Calculate Wilson score confidence interval lower bound
///
/// Uses the Wilson score interval to provide a conservative estimate of
/// the true success rate, especially useful for items with few samples.
///
/// # Arguments
///
/// * `successes` - Number of successful outcomes
/// * `trials` - Total number of trials
/// * `z` - Z-score for desired confidence level (e.g., 1.96 for 95%)
///
/// # Returns
///
/// Lower bound of the confidence interval (0.0 to 1.0)
///
/// # Examples
///
/// ```
/// use do_memory_core::search::ranking::{wilson_lower_bound, z_scores};
///
/// // 10 out of 10 successes - high confidence
/// let score = wilson_lower_bound(10, 10, z_scores::CONFIDENCE_95);
/// assert!(score > 0.7);
///
/// // 1 out of 1 success - lower confidence due to small sample
/// let score = wilson_lower_bound(1, 1, z_scores::CONFIDENCE_95);
/// assert!(score < 0.5);
///
/// // 0 trials returns 0.0
/// let score = wilson_lower_bound(0, 0, z_scores::CONFIDENCE_95);
/// assert_eq!(score, 0.0);
/// ```
#[must_use]
pub fn wilson_lower_bound(successes: u64, trials: u64, z: f64) -> f64 {
    if trials == 0 {
        return 0.0;
    }

    let n = trials as f64;
    let p = successes as f64 / n;
    let z_squared = z * z;

    // Wilson score lower bound formula:
    // (p + z²/(2n) - z*sqrt(p(1-p)/n + z²/(4n²))) / (1 + z²/n)
    let numerator =
        p + z_squared / (2.0 * n) - z * (p * (1.0 - p) / n + z_squared / (4.0 * n * n)).sqrt();
    let denominator = 1.0 + z_squared / n;

    (numerator / denominator).clamp(0.0, 1.0)
}

/// Calculate Wilson score confidence interval upper bound
///
/// # Arguments
///
/// * `successes` - Number of successful outcomes
/// * `trials` - Total number of trials
/// * `z` - Z-score for desired confidence level
///
/// # Returns
///
/// Upper bound of the confidence interval (0.0 to 1.0)
#[must_use]
pub fn wilson_upper_bound(successes: u64, trials: u64, z: f64) -> f64 {
    if trials == 0 {
        return 1.0;
    }

    let n = trials as f64;
    let p = successes as f64 / n;
    let z_squared = z * z;

    let numerator =
        p + z_squared / (2.0 * n) + z * (p * (1.0 - p) / n + z_squared / (4.0 * n * n)).sqrt();
    let denominator = 1.0 + z_squared / n;

    (numerator / denominator).clamp(0.0, 1.0)
}

/// Item with success/failure counts for Bayesian ranking
#[derive(Debug, Clone, Copy)]
pub struct RankingItem {
    /// Number of successful outcomes
    pub successes: u64,
    /// Total number of trials
    pub trials: u64,
}

impl RankingItem {
    /// Create a new ranking item
    #[must_use]
    pub fn new(successes: u64, trials: u64) -> Self {
        Self { successes, trials }
    }

    /// Calculate Wilson score lower bound for ranking
    #[must_use]
    pub fn wilson_score(&self, z: f64) -> f64 {
        wilson_lower_bound(self.successes, self.trials, z)
    }
}

/// Rank items by Wilson score confidence interval
///
/// Sorts items by the lower bound of the Wilson score interval,
/// which provides a conservative estimate of true success rate.
///
/// # Arguments
///
/// * `items` - Items with success counts to rank
/// * `z` - Z-score for confidence level
///
/// # Returns
///
/// Sorted indices (highest score first)
#[must_use]
pub fn rank_by_wilson_score(items: &[RankingItem], z: f64) -> Vec<usize> {
    let mut scored: Vec<(usize, f64)> = items
        .iter()
        .enumerate()
        .map(|(i, item)| (i, item.wilson_score(z)))
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|(i, _)| i).collect()
}
