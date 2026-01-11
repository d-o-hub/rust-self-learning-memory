//! # Bayesian Online Change Point Detection (BOCPD)
//!
//! Implementation of BOCPD algorithm for detecting changepoints in time-series data.

use anyhow::Result;
use std::collections::VecDeque;

use super::types::{BOCPDConfig, BOCPDResult, BOCPDState};

/// Simple BOCPD detector
#[derive(Debug)]
pub struct SimpleBOCPD {
    pub(crate) config: BOCPDConfig,
    pub(crate) state: BOCPDState,
}

/// Simple BOCPD Implementation
impl SimpleBOCPD {
    /// Create a new simple BOCPD detector
    pub fn new(config: BOCPDConfig) -> Self {
        let state = BOCPDState {
            log_posterior: vec![0.0; config.max_run_length_hypotheses],
            hazard_rate: config.hazard_rate,
            data_buffer: VecDeque::with_capacity(config.buffer_size),
            max_buffer_size: config.buffer_size,
            processed_points: 0,
        };

        Self { config, state }
    }

    /// Detect changepoints using BOCD
    pub fn detect_changepoints(&mut self, data: &[f64]) -> Result<Vec<BOCPDResult>> {
        let mut results = Vec::new();

        for (i, &value) in data.iter().enumerate() {
            self.update_state(value)?;

            // Check for changepoint
            if let Some(prob) = self.extract_changepoint()? {
                if prob > self.config.alert_threshold {
                    results.push(BOCPDResult {
                        changepoint_index: Some(i),
                        changepoint_probability: prob,
                        map_run_length: self.compute_map_run_length(),
                        run_length_distribution: self.normalize_distribution(),
                        confidence: prob.min(1.0),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Update BOCPD state with new observation
    pub(crate) fn update_state(&mut self, observation: f64) -> Result<()> {
        // Add to buffer
        self.state.data_buffer.push_back(observation);
        if self.state.data_buffer.len() > self.state.max_buffer_size {
            self.state.data_buffer.pop_front();
        }

        self.state.processed_points += 1;

        // Update posterior if we have enough data
        if self.state.data_buffer.len() >= 2 {
            self.update_posterior(observation)?;
        }

        Ok(())
    }

    /// Update posterior distribution
    #[allow(clippy::needless_range_loop)]
    fn update_posterior(&mut self, observation: f64) -> Result<()> {
        let max_r = self.state.log_posterior.len() - 1;
        let mut new_posterior = vec![f64::NEG_INFINITY; self.state.log_posterior.len()];

        let hazard_prob = (self.state.hazard_rate / (1.0 + self.state.hazard_rate)).ln();
        let survival_prob = (1.0 / (1.0 + self.state.hazard_rate)).ln();

        // Pre-compute likelihood once for this observation
        let log_likelihood = self.compute_likelihood(observation)?;

        // For r=0 (changepoint case): use cached max instead of iterating all prev_r
        let max_prev_log_post = self
            .state
            .log_posterior
            .iter()
            .filter(|&&x| x.is_finite())
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_prev_log_post.is_finite() {
            let finite_count = self
                .state
                .log_posterior
                .iter()
                .filter(|&&x| x.is_finite())
                .count();
            new_posterior[0] =
                (finite_count as f64).ln() + max_prev_log_post + hazard_prob + log_likelihood;
        }

        // For r>0 (continuity case): shift previous posterior
        for r in 1..=max_r {
            if self.state.log_posterior[r - 1].is_finite() {
                new_posterior[r] = self.state.log_posterior[r - 1] + survival_prob + log_likelihood;
            }
        }

        // Normalize
        let log_normalizer = log_sum_exp(&new_posterior);
        if log_normalizer.is_finite() {
            for val in &mut new_posterior {
                if val.is_finite() {
                    *val -= log_normalizer;
                }
            }
        }

        self.state.log_posterior = new_posterior;
        Ok(())
    }

    /// Compute likelihood of observation
    fn compute_likelihood(&self, observation: f64) -> Result<f64> {
        if self.state.data_buffer.len() < 2 {
            return Ok(0.0);
        }

        let recent_data: Vec<f64> = self
            .state
            .data_buffer
            .iter()
            .rev()
            .skip(1)
            .take(5)
            .cloned()
            .collect();

        if recent_data.is_empty() {
            return Ok(0.0);
        }

        let mean = recent_data.iter().sum::<f64>() / recent_data.len() as f64;
        let variance =
            recent_data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / recent_data.len() as f64;
        let std_dev = variance.sqrt().max(0.01);

        let z_score = (observation - mean) / std_dev;
        let log_likelihood = -0.5 * (z_score.powi(2) + (2.0 * std_dev * std_dev * std_dev).ln());

        Ok(log_likelihood)
    }

    /// Extract changepoint if detected
    fn extract_changepoint(&self) -> Result<Option<f64>> {
        if self.state.processed_points < 10 {
            return Ok(None);
        }

        // Low-variance guard
        let recent: Vec<f64> = self
            .state
            .data_buffer
            .iter()
            .rev()
            .take(20)
            .copied()
            .collect();
        if recent.len() >= 5 {
            let mean = recent.iter().sum::<f64>() / recent.len() as f64;
            let var = recent
                .iter()
                .map(|&x| {
                    let d = x - mean;
                    d * d
                })
                .sum::<f64>()
                / recent.len() as f64;
            if var < 1e-6 {
                return Ok(Some(0.0));
            }
        }

        let changepoint_prob = self.state.log_posterior[0].exp();
        Ok(Some(changepoint_prob))
    }

    /// Compute MAP run length
    fn compute_map_run_length(&self) -> usize {
        let mut max_log_prob = f64::NEG_INFINITY;
        let mut map_index = 0;

        for (i, &log_prob) in self.state.log_posterior.iter().enumerate() {
            if log_prob > max_log_prob {
                max_log_prob = log_prob;
                map_index = i;
            }
        }

        map_index
    }

    /// Normalize distribution
    pub(crate) fn normalize_distribution(&self) -> Vec<f64> {
        let log_normalizer = log_sum_exp(&self.state.log_posterior);
        self.state
            .log_posterior
            .iter()
            .map(|&x| (x - log_normalizer).exp())
            .collect()
    }
}

/// Compute log-sum-exp of a vector in log space
pub fn log_sum_exp(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NEG_INFINITY;
    }

    let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    if max_val.is_infinite() && max_val < 0.0 {
        return max_val;
    }

    let sum: f64 = values
        .iter()
        .filter(|&&x| x.is_finite())
        .map(|&x| (x - max_val).exp())
        .sum();

    if sum == 0.0 {
        max_val
    } else {
        max_val + sum.ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_sum_exp() {
        let values = vec![1.0, 2.0, 3.0];
        let result = log_sum_exp(&values);
        assert!(result.is_finite());
        assert!(result > 3.0); // Should be > max value
    }

    #[test]
    fn test_log_sum_exp_empty() {
        let values: Vec<f64> = vec![];
        let result = log_sum_exp(&values);
        assert_eq!(result, f64::NEG_INFINITY);
    }
}
