//! Compatibility assessment types
//!
//! Type definitions for tool compatibility assessment.

use serde::{Deserialize, Serialize};

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
