//! # Concept Drift Analysis
//!
//! Analyzes sequences of episode versions to detect statistical shifts (drift)
//! in performance metrics such as reward and latency.

use crate::episode::Episode;
use crate::patterns::changepoint::{ChangepointDetector, ChangepointConfig, Changepoint};
use anyhow::Result;
use tracing::{debug, instrument};

/// Analyzer for detecting concept drift in versioned episodes
#[derive(Debug, Clone)]
pub struct DriftAnalyzer {
    detector: ChangepointDetector,
}

impl DriftAnalyzer {
    /// Create a new drift analyzer with default configuration
    pub fn new() -> Self {
        let config = ChangepointConfig {
            min_observations: 3, // Allow analysis with fewer versions for drift
            ..Default::default()
        };
        Self {
            detector: ChangepointDetector::new(config),
        }
    }

    /// Analyze a sequence of episode versions for drift
    ///
    /// Returns a list of detected changepoints if any significant drift is found.
    #[instrument(skip(self, episodes))]
    pub fn analyze_drift(&mut self, episodes: &[Episode]) -> Result<Vec<Changepoint>> {
        if episodes.len() < self.detector.config().min_observations {
            debug!(
                have = episodes.len(),
                need = self.detector.config().min_observations,
                "Insufficient episode versions for drift analysis"
            );
            return Ok(Vec::new());
        }

        // Extract reward series
        let rewards: Vec<f64> = episodes
            .iter()
            .map(|e| e.reward.as_ref().map(|r| r.total as f64).unwrap_or(0.0))
            .collect();

        // Extract latency series
        let latencies: Vec<f64> = episodes
            .iter()
            .map(|e| {
                e.steps
                    .iter()
                    .map(|s| s.latency_ms as f64)
                    .sum::<f64>()
            })
            .collect();

        // Detect drift in rewards
        let mut all_changepoints = Vec::new();
        if let Ok(cp) = self.detector.detect_metric_changepoints("reward", &rewards) {
            all_changepoints.extend(cp);
        }

        // Detect drift in latencies
        if let Ok(cp) = self.detector.detect_metric_changepoints("latency", &latencies) {
            all_changepoints.extend(cp);
        }

        Ok(all_changepoints)
    }
}

impl Default for DriftAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
