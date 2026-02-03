//! # Tool Compatibility Assessment Module
//!
//! This module assesses the risk of pattern recommendations and validates tool compatibility scoring.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Tool compatibility assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityAssessment {
    /// Assessment ID
    pub id: String,
    /// Pattern being assessed
    pub pattern_id: String,
    /// Tool being assessed
    pub tool_name: String,
    /// Compatibility score (0-1)
    pub compatibility_score: f64,
    /// Confidence in assessment (0-1)
    pub confidence: f64,
    /// Risk factors identified
    pub risk_factors: Vec<RiskFactor>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Confidence interval (lower, upper)
    pub confidence_interval: (f64, f64),
}

/// Risk factor identified during assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Factor type
    pub factor_type: RiskFactorType,
    /// Severity (0-1)
    pub severity: f64,
    /// Description
    pub description: String,
    /// Mitigation suggestions
    pub mitigation: Option<String>,
}

/// Risk factor types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactorType {
    /// Data quality risk (missing, noisy, inconsistent)
    DataQuality,
    /// Model performance risk (accuracy, precision)
    ModelPerformance,
    /// Domain mismatch risk
    DomainMismatch,
    /// Temporal drift risk (pattern changes over time)
    TemporalDrift,
    /// Resource constraint risk (computation, memory)
    ResourceConstraint,
    /// Compatibility risk (tool version, dependencies)
    Compatibility,
}

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk: safe to proceed
    Low,
    /// Medium risk: proceed with caution
    Medium,
    /// High risk: requires review
    High,
    /// Critical risk: do not proceed
    Critical,
}

/// Compatibility assessment configuration
#[derive(Debug, Clone)]
pub struct AssessmentConfig {
    /// Threshold for low risk (>= this score is low risk)
    pub low_risk_threshold: f64,
    /// Threshold for medium risk (>= this score is medium risk)
    pub medium_risk_threshold: f64,
    /// Confidence level for intervals (default: 0.95)
    pub confidence_level: f64,
    /// Minimum pattern occurrences for reliability
    pub min_occurrences: usize,
}

impl Default for AssessmentConfig {
    fn default() -> Self {
        Self {
            low_risk_threshold: 0.8,
            medium_risk_threshold: 0.6,
            confidence_level: 0.95,
            min_occurrences: 3,
        }
    }
}

/// Tool compatibility assessor
pub struct CompatibilityAssessor {
    config: AssessmentConfig,
    /// Tool capabilities registry
    tool_capabilities: HashMap<String, ToolCapabilities>,
}

/// Tool capabilities definition
#[derive(Debug, Clone)]
struct ToolCapabilities {
    /// Supported data types
    _supported_types: HashSet<String>,
    /// Minimum data quality requirements
    min_data_quality: f64,
    /// Maximum resource usage (MB)
    max_memory_mb: usize,
    /// Supported domains
    supported_domains: HashSet<String>,
    /// Performance metrics
    _avg_latency_ms: f64,
    success_rate: f64,
}

impl CompatibilityAssessor {
    /// Create a new compatibility assessor
    pub fn new(config: AssessmentConfig) -> Self {
        let mut assessor = Self {
            config,
            tool_capabilities: HashMap::new(),
        };

        // Initialize with known tools
        assessor.initialize_tool_registry();
        assessor
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(AssessmentConfig::default())
    }

    /// Initialize tool registry with known capabilities
    fn initialize_tool_registry(&mut self) {
        // query_memory tool
        self.tool_capabilities.insert(
            "query_memory".to_string(),
            ToolCapabilities {
                _supported_types: vec!["episodic", "semantic", "temporal"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                min_data_quality: 0.5,
                max_memory_mb: 100,
                supported_domains: vec!["web-api", "cli", "data-processing"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                _avg_latency_ms: 10.0,
                success_rate: 0.98,
            },
        );

        // analyze_patterns tool
        self.tool_capabilities.insert(
            "analyze_patterns".to_string(),
            ToolCapabilities {
                _supported_types: vec!["statistical", "predictive", "causal"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                min_data_quality: 0.7,
                max_memory_mb: 200,
                supported_domains: vec!["data-processing", "analytics"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                _avg_latency_ms: 50.0,
                success_rate: 0.92,
            },
        );

        // advanced_pattern_analysis tool
        self.tool_capabilities.insert(
            "advanced_pattern_analysis".to_string(),
            ToolCapabilities {
                _supported_types: vec!["time_series", "multivariate", "temporal"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                min_data_quality: 0.8,
                max_memory_mb: 500,
                supported_domains: vec!["analytics", "forecasting", "anomaly_detection"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                _avg_latency_ms: 100.0,
                success_rate: 0.88,
            },
        );
    }

    /// Assess tool compatibility for a pattern
    pub fn assess_compatibility(
        &self,
        pattern_id: &str,
        tool_name: &str,
        pattern_context: &PatternContext,
    ) -> Result<CompatibilityAssessment> {
        // Get tool capabilities
        let tool_caps = self
            .tool_capabilities
            .get(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown tool: {}", tool_name))?;

        // Compute compatibility score
        let compatibility_score = self.compute_compatibility_score(tool_caps, pattern_context);

        // Compute confidence
        let confidence = self.compute_confidence(tool_caps, pattern_context);

        // Identify risk factors
        let risk_factors = self.identify_risk_factors(tool_caps, pattern_context);

        // Determine risk level
        let risk_level = self.determine_risk_level(compatibility_score, &risk_factors);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&risk_factors, tool_name);

        // Compute confidence interval
        let confidence_interval = self.compute_confidence_interval(
            compatibility_score,
            confidence,
            pattern_context.occurrences,
        );

        Ok(CompatibilityAssessment {
            id: format!("{}_{}", pattern_id, tool_name),
            pattern_id: pattern_id.to_string(),
            tool_name: tool_name.to_string(),
            compatibility_score,
            confidence,
            risk_factors,
            recommendations,
            risk_level,
            confidence_interval,
        })
    }

    /// Compute compatibility score
    fn compute_compatibility_score(
        &self,
        tool_caps: &ToolCapabilities,
        context: &PatternContext,
    ) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Data quality compatibility (weight: 0.3)
        let quality_score = if context.data_quality >= tool_caps.min_data_quality {
            1.0
        } else {
            context.data_quality / tool_caps.min_data_quality
        };
        score += 0.3 * quality_score;
        total_weight += 0.3;

        // Domain compatibility (weight: 0.25)
        let domain_score = if tool_caps.supported_domains.contains(&context.domain) {
            1.0
        } else {
            0.5 // Partial credit if domain not directly supported
        };
        score += 0.25 * domain_score;
        total_weight += 0.25;

        // Occurrence reliability (weight: 0.2)
        let occurrence_score = if context.occurrences >= self.config.min_occurrences {
            1.0
        } else {
            context.occurrences as f64 / self.config.min_occurrences as f64
        };
        score += 0.2 * occurrence_score;
        total_weight += 0.2;

        // Temporal stability (weight: 0.15)
        let stability_score = context.temporal_stability;
        score += 0.15 * stability_score;
        total_weight += 0.15;

        // Resource availability (weight: 0.1)
        let resource_score = if context.available_memory_mb >= tool_caps.max_memory_mb {
            1.0
        } else {
            context.available_memory_mb as f64 / tool_caps.max_memory_mb as f64
        };
        score += 0.1 * resource_score;
        total_weight += 0.1;

        // Normalize score
        if total_weight > 0.0 {
            score / total_weight
        } else {
            0.5 // Default middle score
        }
    }

    /// Compute confidence in assessment
    fn compute_confidence(&self, tool_caps: &ToolCapabilities, context: &PatternContext) -> f64 {
        let mut confidence = 0.5; // Base confidence

        // Increase confidence based on tool success rate
        confidence += 0.2 * tool_caps.success_rate;

        // Increase confidence based on pattern occurrences
        let occurrence_confidence = if context.occurrences >= 10 {
            1.0
        } else {
            context.occurrences as f64 / 10.0
        };
        confidence += 0.2 * occurrence_confidence;

        // Increase confidence based on data quality
        confidence += 0.1 * context.data_quality;

        confidence.clamp(0.0, 1.0)
    }

    /// Identify risk factors
    fn identify_risk_factors(
        &self,
        tool_caps: &ToolCapabilities,
        context: &PatternContext,
    ) -> Vec<RiskFactor> {
        let mut risks = Vec::new();

        // Check data quality
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

        // Check domain mismatch
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

        // Check temporal drift
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

        // Check resource constraints
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

        // Check occurrence reliability
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

    /// Determine risk level
    fn determine_risk_level(&self, score: f64, risk_factors: &[RiskFactor]) -> RiskLevel {
        // Check for critical risk factors
        for risk in risk_factors {
            if risk.severity > 0.9 {
                return RiskLevel::Critical;
            }
        }

        // Determine based on score
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

    /// Generate recommendations
    fn generate_recommendations(
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

        // General recommendations
        if risk_factors.len() > 2 {
            recommendations.push(
                "Multiple risk factors detected. Consider using an alternative tool.".to_string(),
            );
        }

        recommendations
    }

    /// Compute confidence interval
    fn compute_confidence_interval(
        &self,
        score: f64,
        confidence: f64,
        occurrences: usize,
    ) -> (f64, f64) {
        // Use Wilson score interval for binomial proportion
        // Adjusted for our use case

        let z = 1.96; // 95% confidence
        let n = occurrences as f64;

        if n < 2.0 {
            // Too few samples, wide interval
            return (0.0_f64.max(score - 0.5), 1.0_f64.min(score + 0.5));
        }

        // Interval width inversely proportional to confidence and occurrences
        let width = (1.0 - confidence) * z / (2.0 * n.sqrt());

        let lower = (score - width).max(0.0);
        let upper = (score + width).min(1.0);

        (lower, upper)
    }

    /// Batch assess multiple tools
    pub fn batch_assess(
        &self,
        pattern_id: &str,
        tool_names: &[String],
        context: &PatternContext,
    ) -> Result<Vec<CompatibilityAssessment>> {
        let mut assessments = Vec::new();

        for tool_name in tool_names {
            let assessment = self.assess_compatibility(pattern_id, tool_name, context)?;
            assessments.push(assessment);
        }

        Ok(assessments)
    }

    /// Get best tool for a pattern
    pub fn get_best_tool(
        &self,
        pattern_id: &str,
        tool_names: &[String],
        context: &PatternContext,
    ) -> Result<Option<(String, CompatibilityAssessment)>> {
        let assessments = self.batch_assess(pattern_id, tool_names, context)?;

        let best = assessments
            .into_iter()
            .filter(|a| matches!(a.risk_level, RiskLevel::Low | RiskLevel::Medium))
            .max_by(|a, b| {
                a.compatibility_score
                    .partial_cmp(&b.compatibility_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        Ok(best.map(|assessment| (assessment.tool_name.clone(), assessment)))
    }
}

/// Pattern context for compatibility assessment
#[derive(Debug, Clone)]
pub struct PatternContext {
    /// Domain of the pattern
    pub domain: String,
    /// Data quality score (0-1)
    pub data_quality: f64,
    /// Number of times pattern occurs
    pub occurrences: usize,
    /// Temporal stability (0-1, higher = more stable)
    pub temporal_stability: f64,
    /// Available memory in MB
    pub available_memory_mb: usize,
    /// Pattern complexity (0-1)
    pub complexity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compatibility_assessment() {
        let assessor = CompatibilityAssessor::default_config();

        let context = PatternContext {
            domain: "web-api".to_string(),
            data_quality: 0.85,
            occurrences: 10,
            temporal_stability: 0.9,
            available_memory_mb: 200,
            complexity: 0.5,
        };

        let assessment = assessor
            .assess_compatibility("test_pattern", "query_memory", &context)
            .unwrap();

        assert!(assessment.compatibility_score >= 0.0 && assessment.compatibility_score <= 1.0);
        assert!(assessment.confidence >= 0.0 && assessment.confidence <= 1.0);
    }

    #[test]
    fn test_risk_level_determination() {
        let assessor = CompatibilityAssessor::default_config();

        let context = PatternContext {
            domain: "web-api".to_string(),
            data_quality: 0.9,
            occurrences: 20,
            temporal_stability: 0.95,
            available_memory_mb: 500,
            complexity: 0.3,
        };

        let assessment = assessor
            .assess_compatibility("test_pattern", "query_memory", &context)
            .unwrap();

        // High quality should result in low or medium risk
        assert!(matches!(
            assessment.risk_level,
            RiskLevel::Low | RiskLevel::Medium
        ));
    }

    #[test]
    fn test_risk_factor_identification() {
        let assessor = CompatibilityAssessor::default_config();

        // Low quality context should trigger risk factors
        let context = PatternContext {
            domain: "unsupported_domain".to_string(),
            data_quality: 0.3,
            occurrences: 1,
            temporal_stability: 0.5,
            available_memory_mb: 10,
            complexity: 0.9,
        };

        let assessment = assessor
            .assess_compatibility("test_pattern", "query_memory", &context)
            .unwrap();

        // Should have identified several risk factors
        assert!(!assessment.risk_factors.is_empty());
    }

    #[test]
    fn test_confidence_interval_computation() {
        let assessor = CompatibilityAssessor::default_config();

        let context = PatternContext {
            domain: "web-api".to_string(),
            data_quality: 0.8,
            occurrences: 100,
            temporal_stability: 0.9,
            available_memory_mb: 200,
            complexity: 0.5,
        };

        let assessment = assessor
            .assess_compatibility("test_pattern", "query_memory", &context)
            .unwrap();

        // Confidence interval should be valid
        assert!(assessment.confidence_interval.0 >= 0.0);
        assert!(assessment.confidence_interval.1 <= 1.0);
        assert!(assessment.confidence_interval.0 <= assessment.compatibility_score);
        assert!(assessment.confidence_interval.1 >= assessment.compatibility_score);

        // With many occurrences, interval should be relatively narrow
        let interval_width = assessment.confidence_interval.1 - assessment.confidence_interval.0;
        assert!(interval_width < 0.3); // Should be reasonably narrow
    }

    #[test]
    fn test_batch_assessment() {
        let assessor = CompatibilityAssessor::default_config();

        let context = PatternContext {
            domain: "web-api".to_string(),
            data_quality: 0.8,
            occurrences: 10,
            temporal_stability: 0.9,
            available_memory_mb: 300,
            complexity: 0.5,
        };

        let tools = vec![
            "query_memory".to_string(),
            "analyze_patterns".to_string(),
            "advanced_pattern_analysis".to_string(),
        ];

        let assessments = assessor
            .batch_assess("test_pattern", &tools, &context)
            .unwrap();

        assert_eq!(assessments.len(), 3);
    }

    #[test]
    fn test_best_tool_selection() {
        let assessor = CompatibilityAssessor::default_config();

        let context = PatternContext {
            domain: "data-processing".to_string(),
            data_quality: 0.85,
            occurrences: 15,
            temporal_stability: 0.95,
            available_memory_mb: 300,
            complexity: 0.6,
        };

        let tools = vec![
            "query_memory".to_string(),
            "analyze_patterns".to_string(),
            "advanced_pattern_analysis".to_string(),
        ];

        let best = assessor
            .get_best_tool("test_pattern", &tools, &context)
            .unwrap();

        assert!(best.is_some());
        let (tool_name, assessment) = best.unwrap();
        assert!(!tool_name.is_empty());
        assert!(matches!(
            assessment.risk_level,
            RiskLevel::Low | RiskLevel::Medium
        ));
    }
}
