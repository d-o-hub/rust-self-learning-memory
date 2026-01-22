//! # Changepoint Algorithms
//!
//! Helper algorithms for changepoint detection including segment statistics
//! and statistical functions.

use super::types::{ChangepointConfig, SegmentStats};

/// Compute statistics for a segment of values
#[must_use]
pub fn compute_segment_stats(values: &[f64]) -> SegmentStats {
    if values.is_empty() {
        return SegmentStats::default();
    }

    let count = values.len();
    let mean: f64 = values.iter().sum::<f64>() / count as f64;

    let variance: f64 = if count > 1 {
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (count - 1) as f64
    } else {
        0.0
    };

    let std_dev = variance.sqrt();

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    SegmentStats {
        count,
        mean,
        std_dev,
        min,
        max,
    }
}

/// Standard normal CDF approximation using error function
/// 
/// Uses the relationship: Φ(x) = 0.5 * (1 + erf(x / sqrt(2)))
/// where erf is the error function.
#[inline]
#[must_use]
pub fn normal_cdf(x: f64) -> f64 {
    // Abramowitz and Stegun approximation for error function
    // erf(z) ≈ sign(z) * (1 - (a1*t + a2*t^2 + a3*t^3 + a4*t^4 + a5*t^5) * exp(-z^2))
    // where t = 1 / (1 + p*|z|)
    let a1 = 0.254_829_592;
    let a2 = -0.284_496_736;
    let a3 = 1.421_413_741;
    let a4 = -1.453_152_027;
    let a5 = 1.061_405_429;
    let p = 0.327_591_1;

    // Scale x for error function: z = x / sqrt(2)
    let z = x / 2_f64.sqrt();
    
    let sign = if z < 0.0 { -1.0 } else { 1.0 };
    let z_abs = z.abs();

    let t = 1.0 / (1.0 + p * z_abs);
    let erf = sign * (1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-z_abs * z_abs).exp());

    // CDF(x) = 0.5 * (1 + erf(x / sqrt(2)))
    0.5 * (1.0 + erf)
}

/// Calculate changepoint probability based on surrounding data
#[must_use]
pub fn calculate_changepoint_probability(
    config: &ChangepointConfig,
    values: &[f64],
    cp_index: usize,
    detection_index: usize,
) -> f64 {
    // Base probability on detection order (earlier detections are more reliable)
    let base_prob = if detection_index == 0 {
        config.min_probability.max(0.7)
    } else {
        config.min_probability
    };

    // Adjust based on local variance
    let window = 5;
    let start = cp_index.saturating_sub(window);
    let end = (cp_index + window).min(values.len().saturating_sub(1));

    if start < end {
        let segment = &values[start..end];
        let mean: f64 = segment.iter().sum::<f64>() / segment.len() as f64;
        let variance: f64 =
            segment.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / segment.len() as f64;
        let std_dev = variance.sqrt();

        // Higher variance reduces confidence
        let variance_factor = (1.0 - (std_dev / mean.abs().max(0.001))).clamp(0.5, 1.0);
        (base_prob * variance_factor).clamp(0.0, 1.0)
    } else {
        base_prob
    }
}
