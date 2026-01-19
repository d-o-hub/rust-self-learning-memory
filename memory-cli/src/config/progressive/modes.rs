//! Configuration modes module
//!
//! This module defines configuration modes and usage patterns.

/// Available configuration modes with clear progression.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigurationMode {
    /// Ultra-simple: 30-second setup, minimal decisions
    UltraSimple,
    /// Simple: Preset-based with guided customization
    Simple,
    /// Advanced: Full interactive wizard
    Advanced,
}

impl ConfigurationMode {
    /// Get description of this configuration mode.
    pub fn description(&self) -> &'static str {
        match self {
            ConfigurationMode::UltraSimple => {
                "30-second setup for basic usage\n\
                 • One function call\n\
                 • Auto-detected settings\n\
                 • Minimal configuration\n\
                 • Best for: Quick testing, prototypes"
            }
            ConfigurationMode::Simple => {
                "Guided preset selection with customization\n\
                 • 3-5 function calls\n\
                 • Clear preset options\n\
                 • Easy customization\n\
                 • Best for: Development, specific use cases"
            }
            ConfigurationMode::Advanced => {
                "Full interactive configuration wizard\n\
                 • Comprehensive setup\n\
                 • All options available\n\
                 • Expert-level control\n\
                 • Best for: Production, complex requirements"
            }
        }
    }

    /// Get estimated setup time.
    pub fn setup_time(&self) -> &'static str {
        match self {
            ConfigurationMode::UltraSimple => "30 seconds",
            ConfigurationMode::Simple => "2-5 minutes",
            ConfigurationMode::Advanced => "5-15 minutes",
        }
    }
}

/// Usage patterns to help determine appropriate configuration mode.
#[derive(Debug, Clone)]
pub enum UsagePattern {
    /// Quick prototyping, experimentation, or one-time usage
    QuickTest,
    /// Ongoing development work with frequent iterations
    Development,
    /// Production deployment with reliability requirements
    Production,
    /// Automated testing, CI/CD, or batch processing
    Testing,
}

impl UsagePattern {
    /// Detect usage pattern from environment and usage hints.
    pub fn detect() -> Self {
        if std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok() {
            return UsagePattern::Testing;
        }

        if std::env::var("DEVELOPMENT").is_ok() || std::env::var("DEV").is_ok() {
            return UsagePattern::Development;
        }

        if std::env::var("PRODUCTION").is_ok() {
            return UsagePattern::Production;
        }

        UsagePattern::Development
    }
}

/// Progressive disclosure helper: Check if current mode is appropriate.
pub struct ModeRecommendation {
    /// Recommended configuration mode.
    pub recommended_mode: ConfigurationMode,
    /// Reasoning for the recommendation.
    pub reasoning: String,
    /// Path to upgrade to a different mode.
    pub upgrade_path: String,
}

/// Analyze current usage to recommend the best configuration mode.
pub fn recommend_mode(usage_pattern: &UsagePattern) -> ModeRecommendation {
    match usage_pattern {
        UsagePattern::QuickTest => ModeRecommendation {
            recommended_mode: ConfigurationMode::UltraSimple,
            reasoning: "Quick testing detected - Ultra-Simple mode provides fastest setup".to_string(),
            upgrade_path: "Upgrade to Simple mode when you need custom settings".to_string(),
        },
        UsagePattern::Development => ModeRecommendation {
            recommended_mode: ConfigurationMode::Simple,
            reasoning: "Development usage detected - Simple mode offers best balance of convenience and control".to_string(),
            upgrade_path: "Upgrade to Advanced mode for production deployment".to_string(),
        },
        UsagePattern::Production => ModeRecommendation {
            recommended_mode: ConfigurationMode::Advanced,
            reasoning: "Production usage detected - Advanced mode provides comprehensive configuration".to_string(),
            upgrade_path: "Fine-tune settings using Simple mode for specific optimizations".to_string(),
        },
        UsagePattern::Testing => ModeRecommendation {
            recommended_mode: ConfigurationMode::UltraSimple,
            reasoning: "Testing workflow detected - Ultra-Simple mode minimizes setup overhead".to_string(),
            upgrade_path: "Use Simple mode for test configurations with specific requirements".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_recommendation() {
        let recommendation = recommend_mode(&UsagePattern::Development);
        assert_eq!(recommendation.recommended_mode, ConfigurationMode::Simple);
        assert!(!recommendation.reasoning.is_empty());
    }

    #[test]
    fn test_usage_pattern_detection() {
        let pattern = UsagePattern::detect();
        match pattern {
            UsagePattern::QuickTest
            | UsagePattern::Development
            | UsagePattern::Production
            | UsagePattern::Testing => {}
        }
    }
}
