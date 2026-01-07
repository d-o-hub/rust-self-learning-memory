//! Optimized Pattern Validator with Enhanced Confidence Thresholds
//!
//! Implements validated Quick Win optimizations for improved success rates.

mod applicator;
mod planned_step;
mod risk;
mod tool;
mod validator;

// Re-exports
pub use applicator::EnhancedPatternApplicator;
pub use planned_step::PlannedStep;
pub use risk::RiskAssessment;
pub use tool::{CompatibilityResult, Tool};
pub use validator::OptimizedPatternValidator;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityLevel, TaskContext};
    use std::collections::HashMap;

    #[test]
    fn test_optimized_validator_confidence_threshold() {
        let validator = OptimizedPatternValidator::default();
        assert_eq!(validator.minimum_confidence, 0.85);
        assert_eq!(validator.minimum_sample_size, 5);
    }

    #[test]
    fn test_tool_compatibility_assessment() {
        let applicator = EnhancedPatternApplicator::new();

        let mut success_history = HashMap::new();
        success_history.insert("api_development".to_string(), 0.9);

        let tool = Tool::new("rust_compiler".to_string())
            .with_capabilities(vec!["compile".to_string(), "lint".to_string()])
            .with_success_history(success_history);

        let context = TaskContext {
            domain: "api_development".to_string(),
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: vec![],
        };

        let compatibility = applicator.assess_tool_compatibility(&tool, &context);
        assert!(compatibility > 0.5, "Expected high compatibility score");
    }

    #[test]
    fn test_tool_compatibility_with_empty_history() {
        let applicator = EnhancedPatternApplicator::new();

        // Tool with no success history
        let tool = Tool::new("new_tool".to_string());

        let context = TaskContext {
            domain: "api_development".to_string(),
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Complex,
            tags: vec![],
        };

        let compatibility = applicator.assess_tool_compatibility(&tool, &context);

        // Should return a neutral score for tools with no history
        assert!(compatibility >= 0.0 && compatibility <= 1.0);
    }

    #[test]
    fn test_planned_step_creation() {
        let planned_step = PlannedStep {
            tool: "compiler".to_string(),
            action: "compile".to_string(),
            expected_duration_ms: 5000,
            parameters: serde_json::json!({}),
        };

        assert_eq!(planned_step.tool, "compiler");
        assert_eq!(planned_step.action, "compile");
        assert_eq!(planned_step.expected_duration_ms, 5000);
    }

    #[test]
    fn test_tool_builder_pattern() {
        let tool = Tool::new("test_tool".to_string())
            .with_capabilities(vec!["cap1".to_string(), "cap2".to_string()]);

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.capabilities.len(), 2);
    }

    #[test]
    fn test_validator_custom_thresholds() {
        // Test validator with custom thresholds
        let strict_validator = OptimizedPatternValidator::new(0.95, 10, 0.9);
        let lenient_validator = OptimizedPatternValidator::new(0.7, 3, 0.6);

        assert_eq!(strict_validator.minimum_confidence, 0.95);
        assert_eq!(strict_validator.minimum_sample_size, 10);
        assert_eq!(strict_validator.context_similarity_threshold, 0.9);

        assert_eq!(lenient_validator.minimum_confidence, 0.7);
        assert_eq!(lenient_validator.minimum_sample_size, 3);
        assert_eq!(lenient_validator.context_similarity_threshold, 0.6);
    }

    #[test]
    fn test_validator_thresholds() {
        let validator = OptimizedPatternValidator::new(0.8, 3, 0.7);

        assert_eq!(validator.minimum_confidence, 0.8);
        assert_eq!(validator.minimum_sample_size, 3);
        assert_eq!(validator.context_similarity_threshold, 0.7);
    }

    #[test]
    fn test_enhanced_applicator_creation() {
        let applicator = EnhancedPatternApplicator::new();
        let default_applicator = EnhancedPatternApplicator::default();

        // Both should create valid applicators
        // Test they can assess tool compatibility
        let tool = Tool::new("test".to_string());
        let context = TaskContext {
            domain: "test".to_string(),
            language: None,
            framework: None,
            complexity: ComplexityLevel::Simple,
            tags: vec![],
        };

        let score1 = applicator.assess_tool_compatibility(&tool, &context);
        let score2 = default_applicator.assess_tool_compatibility(&tool, &context);

        assert!(score1 >= 0.0 && score1 <= 1.0);
        assert_eq!(score1, score2);
    }
}
