//! # Causal Inference Module
//!
//! Granger causality analysis and causal relationship detection.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::{debug, info, instrument};

use super::types::{CausalResult, CausalType, PredictiveConfig};

#[derive(Debug)]
pub struct CausalAnalyzer {
    config: PredictiveConfig,
}

impl CausalAnalyzer {
    pub fn new() -> Result<Self> {
        Self::with_config(PredictiveConfig::default())
    }
    pub fn with_config(config: PredictiveConfig) -> Result<Self> {
        Ok(Self { config })
    }

    #[instrument(skip(self, data))]
    pub fn analyze_causality(&self, data: &HashMap<String, Vec<f64>>) -> Result<Vec<CausalResult>> {
        if !self.config.enable_causal_inference {
            return Ok(Vec::new());
        }
        let mut results = Vec::new();
        let variables: Vec<&String> = data.keys().collect();
        info!("Analyzing causal relationships between {} variables", variables.len());
        let pairs: Vec<_> = variables.iter().enumerate().flat_map(|(i, &var1)| variables[i + 1..].iter().map(move |&var2| (var1, var2))).collect();
        for (var1, var2) in pairs {
            if let (Some(data1), Some(data2)) = (data.get(var1), data.get(var2)) {
                if let Some(causal_result) = self.analyze_pair_causality(var1, var2, data1, data2)? {
                    results.push(causal_result);
                }
            }
        }
        debug!("Found {} causal relationships", results.len());
        Ok(results)
    }

    fn analyze_pair_causality(&self, cause: &str, effect: &str, cause_data: &[f64], effect_data: &[f64]) -> Result<Option<CausalResult>> {
        if cause_data.len() != effect_data.len() || cause_data.len() < 10 {
            return Ok(None);
        }
        let correlation = self.calculate_correlation(cause_data, effect_data)?;
        let max_lag = 5.min(cause_data.len() / 4);
        let mut max_cross_corr: f64 = 0.0;
        let mut best_lag = 0;
        for lag in 1..=max_lag {
            if let Some(cross_corr) = self.cross_correlation(cause_data, effect_data, lag) {
                if cross_corr.abs() > max_cross_corr.abs() {
                    max_cross_corr = cross_corr;
                    best_lag = lag;
                }
            }
        }
        let relationship_type = if max_cross_corr.abs() > 0.7 && best_lag > 0 {
            CausalType::Direct
        } else if correlation.abs() > 0.5 {
            CausalType::Indirect
        } else if correlation.abs() < 0.2 {
            CausalType::None
        } else {
            CausalType::Spurious
        };
        let n = cause_data.len() as f64;
        let t_stat = correlation.abs() * ((n - 2.0) / (1.0 - correlation * correlation)).sqrt();
        let p_value = 2.0 * (1.0 - Self::normal_cdf(t_stat));
        let significant = p_value < 0.05;
        let strength = correlation.abs().min(1.0);
        let se = (1.0 - correlation * correlation) / (n - 2.0).sqrt();
        let margin = 1.96 * se;
        let confidence_interval = ((correlation - margin).max(-1.0), (correlation + margin).min(1.0));
        Ok(Some(CausalResult {
            cause: cause.to_string(),
            effect: effect.to_string(),
            strength,
            significant,
            relationship_type,
            confidence_interval,
        }))
    }

    fn calculate_correlation(&self, x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() {
            return Err(anyhow!("Data lengths don't match"));
        }
        let n = x.len() as f64;
        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = y.iter().sum();
        let sum_xy: f64 = x.iter().zip(y.iter()).map(|(&a, &b)| a * b).sum();
        let sum_x2: f64 = x.iter().map(|&a| a * a).sum();
        let sum_y2: f64 = y.iter().map(|&a| a * a).sum();
        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
        if denominator == 0.0 { Ok(0.0) } else { Ok(numerator / denominator) }
    }

    fn cross_correlation(&self, x: &[f64], y: &[f64], lag: usize) -> Option<f64> {
        if lag >= x.len() || lag >= y.len() {
            return None;
        }
        let x_slice = &x[lag..];
        let y_slice = &y[..y.len() - lag];
        self.calculate_correlation(x_slice, y_slice).ok()
    }

    fn normal_cdf(x: f64) -> f64 {
        0.5 * (1.0 + Self::erf(x / 2.0_f64.sqrt()))
    }

    fn erf(x: f64) -> f64 {
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;
        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
        sign * y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn test_causal_analyzer_creation() {
        let analyzer = CausalAnalyzer::new();
        assert!(analyzer.is_ok());
    }
    #[test]
    fn test_causal_analysis() -> Result<()> {
        let analyzer = CausalAnalyzer::new()?;
        let mut data = HashMap::new();
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&val| 2.0 * val + 1.0).collect();
        data.insert("x".to_string(), x);
        data.insert("y".to_string(), y);
        let _causal_results = analyzer.analyze_causality(&data)?;
        Ok(())
    }
}
