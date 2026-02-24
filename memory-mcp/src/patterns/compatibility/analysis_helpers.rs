use super::*;

impl CompatibilityAssessor {
    pub(super) fn identify_risk_factors(
        &self,
        tool_caps: &ToolCapabilities,
        context: &PatternContext,
    ) -> Vec<RiskFactor> {
        let mut risks = Vec::new();

        if context.data_quality < tool_caps.min_data_quality {
            risks.push(RiskFactor {
                factor_type: RiskFactorType::DataQuality,
                severity: (tool_caps.min_data_quality - context.data_quality).clamp(0.0, 1.0),
                description: format!(
                    "Data quality ({:.2}) below tool requirement ({:.2})",
                    context.data_quality, tool_caps.min_data_quality
                ),
                mitigation: Some("Improve data quality or use alternative tool".to_string()),
            });
        }

        if !tool_caps.supported_domains.contains(&context.domain) {
            risks.push(RiskFactor {
                factor_type: RiskFactorType::DomainMismatch,
                severity: 0.7,
                description: format!(
                    "Domain '{}' not in tool's supported domains",
                    context.domain
                ),
                mitigation: Some(
                    "Verify tool compatibility or map domain to supported type".to_string(),
                ),
            });
        }

        if context.temporal_stability < 0.7 {
            risks.push(RiskFactor {
                factor_type: RiskFactorType::TemporalDrift,
                severity: 1.0 - context.temporal_stability,
                description: format!(
                    "Pattern shows signs of temporal drift (stability: {:.2})",
                    context.temporal_stability
                ),
                mitigation: Some("Use adaptive models or refresh pattern regularly".to_string()),
            });
        }

        if context.available_memory_mb < tool_caps.max_memory_mb {
            risks.push(RiskFactor {
                factor_type: RiskFactorType::ResourceConstraint,
                severity: (tool_caps.max_memory_mb - context.available_memory_mb) as f64
                    / tool_caps.max_memory_mb as f64,
                description: format!(
                    "Insufficient memory: have {} MB, need {} MB",
                    context.available_memory_mb, tool_caps.max_memory_mb
                ),
                mitigation: Some("Increase available resources or use lighter tool".to_string()),
            });
        }

        if context.occurrences < self.config.min_occurrences {
            risks.push(RiskFactor {
                factor_type: RiskFactorType::ModelPerformance,
                severity: (self.config.min_occurrences - context.occurrences) as f64
                    / self.config.min_occurrences as f64,
                description: format!(
                    "Pattern occurs only {} times (minimum: {})",
                    context.occurrences, self.config.min_occurrences
                ),
                mitigation: Some("Gather more data or use simpler model".to_string()),
            });
        }

        risks
    }

    pub(super) fn determine_risk_level(
        &self,
        score: f64,
        risk_factors: &[RiskFactor],
    ) -> RiskLevel {
        for risk in risk_factors {
            if risk.severity > 0.9 {
                return RiskLevel::Critical;
            }
        }

        if score >= self.config.low_risk_threshold && risk_factors.is_empty() {
            RiskLevel::Low
        } else if score >= self.config.medium_risk_threshold {
            RiskLevel::Medium
        } else if score > 0.4 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }

    pub(super) fn generate_recommendations(
        &self,
        risk_factors: &[RiskFactor],
        tool_name: &str,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if risk_factors.is_empty() {
            recommendations.push(format!(
                "{} is well-suited for this pattern. Proceed with confidence.",
                tool_name
            ));
            return recommendations;
        }

        for risk in risk_factors {
            if let Some(mitigation) = &risk.mitigation {
                recommendations.push(mitigation.clone());
            }
        }

        if risk_factors.len() > 2 {
            recommendations.push(
                "Multiple risk factors detected. Consider using an alternative tool.".to_string(),
            );
        }

        recommendations
    }

    pub(super) fn compute_confidence_interval(
        &self,
        score: f64,
        confidence: f64,
        occurrences: usize,
    ) -> (f64, f64) {
        let z = 1.96;
        let n = occurrences as f64;

        if n < 2.0 {
            return (0.0_f64.max(score - 0.5), 1.0_f64.min(score + 0.5));
        }

        let width = (1.0 - confidence) * z / (2.0 * n.sqrt());
        let lower = (score - width).max(0.0);
        let upper = (score + width).min(1.0);
        (lower, upper)
    }
}
