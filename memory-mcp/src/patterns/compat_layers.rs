//! Tool capabilities layer for compatibility assessment
//!
//! Defines tool capabilities and initialization logic.

use std::collections::{HashMap, HashSet};

use super::compat_types::AssessmentConfig;

/// Tool capabilities definition
pub struct ToolCapabilities {
    /// Supported data types
    pub _supported_types: HashSet<String>,
    /// Minimum data quality requirements
    pub min_data_quality: f64,
    /// Maximum resource usage (MB)
    pub max_memory_mb: usize,
    /// Supported domains
    pub supported_domains: HashSet<String>,
    /// Performance metrics
    pub _avg_latency_ms: f64,
    pub success_rate: f64,
}

/// Initialize tool capabilities registry
pub fn initialize_tool_registry() -> HashMap<String, ToolCapabilities> {
    let mut tool_capabilities = HashMap::new();

    // query_memory tool
    tool_capabilities.insert(
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
    tool_capabilities.insert(
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
    tool_capabilities.insert(
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

    tool_capabilities
}

/// Create default assessment configuration
pub fn default_assessment_config() -> AssessmentConfig {
    AssessmentConfig::default()
}
