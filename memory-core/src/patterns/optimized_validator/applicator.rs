//! Enhanced pattern application with risk assessment

use super::planned_step::PlannedStep;
use super::risk::RiskAssessment;
use super::tool::Tool;
use super::validator::OptimizedPatternValidator;
use crate::pattern::Pattern;
use crate::types::TaskContext;
use anyhow::Result;

/// Enhanced pattern application with risk assessment
pub struct EnhancedPatternApplicator {
    validator: OptimizedPatternValidator,
    risk_threshold: f32,
}

impl EnhancedPatternApplicator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            validator: OptimizedPatternValidator::default(),
            risk_threshold: 0.7, // Validated threshold for validation step injection
        }
    }

    /// Assess tool compatibility with task context
    #[must_use]
    pub fn assess_tool_compatibility(&self, tool: &Tool, context: &TaskContext) -> f32 {
        // Historical usage analysis
        let historical_usage = self.calculate_historical_success_rate(tool, context);

        // Success rate calculation for tool-task context pairs
        let context_success_rate = self.calculate_context_success_rate(tool, context);

        // Context compatibility analysis
        let compatibility_score = self.analyze_context_compatibility(tool, context);

        // Capability match analysis
        let capability_match = self.analyze_capability_match(tool, context);

        // Enhanced weighted combination of factors with better scoring
        let mut overall_score = 0.0;

        // Context success rate gets highest weight (50%)
        overall_score += context_success_rate * 0.5;

        // Historical usage analysis (25%)
        overall_score += historical_usage * 0.25;

        // Compatibility score (15%)
        overall_score += compatibility_score * 0.15;

        // Apply capability match as a bonus multiplier (cap at 1.0)
        (overall_score * (1.0 + capability_match * 0.3)).min(1.0)
    }

    /// Calculate historical success rate for tool in similar contexts
    fn calculate_historical_success_rate(&self, tool: &Tool, context: &TaskContext) -> f32 {
        if tool.success_history.is_empty() {
            return 0.5; // Neutral score for tools with no history
        }

        let mut total_rate = 0.0;
        let mut weight_sum = 0.0;

        // Weight by domain similarity
        for (domain, rate) in &tool.success_history {
            let weight = if domain == &context.domain {
                1.0 // Exact domain match
            } else if self.is_related_domain(domain, &context.domain) {
                0.7 // Related domain
            } else {
                0.3 // Different domain
            };

            total_rate += rate * weight;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            total_rate / weight_sum
        } else {
            0.5
        }
    }

    /// Check if two domains are related
    fn is_related_domain(&self, domain1: &str, domain2: &str) -> bool {
        let related_pairs = [
            ("api_development", "web_development"),
            ("data_processing", "data_science"),
            ("testing", "debugging"),
            ("refactoring", "code_generation"),
        ];

        related_pairs
            .iter()
            .any(|(a, b)| (domain1 == *a && domain2 == *b) || (domain1 == *b && domain2 == *a))
    }

    /// Calculate success rate for specific context
    fn calculate_context_success_rate(&self, tool: &Tool, context: &TaskContext) -> f32 {
        // Find similar contexts in tool's typical contexts
        let mut similar_contexts = 0;
        let mut successful_contexts = 0;
        let mut total_similarity = 0.0;

        for typical_context in &tool.typical_contexts {
            let similarity = self
                .validator
                .calculate_context_similarity(typical_context, context);

            if similarity > 0.5 {
                // Lower threshold for considering contexts similar
                similar_contexts += 1;
                total_similarity += similarity;

                // Check if this context was successful
                let domain_key = &typical_context.domain;
                if let Some(&success_rate) = tool.success_history.get(domain_key) {
                    #[allow(clippy::excessive_nesting)]
                    if success_rate > 0.6 {
                        // Lowered threshold to 60%
                        successful_contexts += 1;
                    }
                }
            }
        }

        if similar_contexts > 0 {
            // Weight by similarity scores
            let avg_similarity = total_similarity / similar_contexts as f32;
            let base_success_rate = successful_contexts as f32 / similar_contexts as f32;

            // Combine success rate with similarity weighting
            base_success_rate * avg_similarity + (1.0 - avg_similarity) * 0.7
        } else {
            // If no similar contexts, check domain success history directly
            if let Some(&domain_rate) = tool.success_history.get(&context.domain) {
                domain_rate * 0.8 // Slight penalty for no direct context match
            } else {
                0.5 // Neutral score when no context information available
            }
        }
    }

    /// Analyze context compatibility between tool and task
    fn analyze_context_compatibility(&self, tool: &Tool, context: &TaskContext) -> f32 {
        let mut compatibility: f32 = 0.0;
        let mut factors_considered = 0;

        // Domain compatibility (40% weight) - Check exact matches first
        for typical_context in &tool.typical_contexts {
            if typical_context.domain == context.domain {
                compatibility += 0.4;
                factors_considered += 1;
                break;
            }
        }

        // If no exact domain match, check for related domains
        if factors_considered == 0 {
            for typical_context in &tool.typical_contexts {
                if self.is_related_domain(&typical_context.domain, &context.domain) {
                    compatibility += 0.25; // Partial credit for related domains
                    factors_considered += 1;
                    break;
                }
            }
        }

        // Language compatibility (25% weight)
        if let (Some(context_lang), Some(tool_lang_context)) = (
            &context.language,
            tool.typical_contexts
                .iter()
                .find_map(|tc| tc.language.as_ref()),
        ) {
            if context_lang == tool_lang_context {
                compatibility += 0.25;
                factors_considered += 1;
            }
        }

        // Framework compatibility (20% weight)
        if let (Some(context_framework), Some(tool_framework_context)) = (
            &context.framework,
            tool.typical_contexts
                .iter()
                .find_map(|tc| tc.framework.as_ref()),
        ) {
            if context_framework == tool_framework_context {
                compatibility += 0.20;
                factors_considered += 1;
            }
        }

        // Complexity compatibility (15% weight)
        let has_matching_complexity = tool
            .typical_contexts
            .iter()
            .any(|tc| tc.complexity == context.complexity);

        if has_matching_complexity {
            compatibility += 0.15;
            factors_considered += 1;
        }

        // Bonus for tools that have been used in exactly this domain before
        if tool.success_history.contains_key(&context.domain) {
            compatibility += 0.1; // Small bonus for known domain usage
            factors_considered += 1;
        }

        // If no factors were considered, return a score based on domain popularity
        if factors_considered == 0 {
            match context.domain.as_str() {
                "api_development" | "web_development" => 0.6, // Common domains get base score
                _ => 0.4,
            }
        } else {
            compatibility.min(1.0)
        }
    }

    /// Analyze capability match between tool and task requirements
    fn analyze_capability_match(&self, tool: &Tool, context: &TaskContext) -> f32 {
        if tool.capabilities.is_empty() {
            return 0.5; // Neutral score for tools with no capability restrictions
        }

        // Get expected capabilities for the context
        let expected_capabilities = self.get_expected_capabilities_for_context(context);

        if expected_capabilities.is_empty() {
            return 1.0; // No specific capabilities required
        }

        let mut matched_capabilities = 0;

        // Check for exact matches
        for capability in &tool.capabilities {
            if expected_capabilities.contains(capability) {
                matched_capabilities += 1;
            }
        }

        // Check for partial matches (e.g., "rust_compiler" matches "compiler" requirement)
        for tool_cap in &tool.capabilities {
            for expected_cap in &expected_capabilities {
                if tool_cap.contains(expected_cap) || expected_cap.contains(tool_cap) {
                    matched_capabilities += 1;
                    break; // Don't double count
                }
            }
        }

        // Calculate match ratio
        let match_ratio = matched_capabilities as f32 / expected_capabilities.len() as f32;

        // Bonus for exact domain-specific tools
        let domain_bonus = match context.domain.as_str() {
            "api_development" => {
                if tool
                    .capabilities
                    .iter()
                    .any(|c| c.contains("rust") || c.contains("compiler"))
                {
                    0.2
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        (match_ratio + domain_bonus).min(1.0)
    }

    /// Get expected capabilities for a given context
    fn get_expected_capabilities_for_context(&self, context: &TaskContext) -> Vec<String> {
        let mut capabilities = Vec::new();

        // Domain-based capabilities
        match context.domain.as_str() {
            "api_development" => {
                capabilities.extend(vec!["http_client".to_string(), "json_parser".to_string()]);
            }
            "data_processing" => {
                capabilities.extend(vec!["file_reader".to_string(), "data_analyzer".to_string()]);
            }
            "debugging" => {
                capabilities.extend(vec!["error_analyzer".to_string(), "log_viewer".to_string()]);
            }
            "testing" => {
                capabilities.extend(vec![
                    "test_runner".to_string(),
                    "assertion_checker".to_string(),
                ]);
            }
            _ => {
                capabilities.push("general_processor".to_string());
            }
        }

        // Language-based capabilities
        if let Some(lang) = &context.language {
            capabilities.push(format!("{lang}_compiler"));
            capabilities.push(format!("{lang}_linter"));
        }

        // Framework-based capabilities
        if let Some(framework) = &context.framework {
            capabilities.push(format!("{framework}_integration"));
        }

        capabilities
    }

    /// Apply pattern with risk assessment and validation injection
    pub fn apply_pattern_with_validation(
        &self,
        pattern: &Pattern,
        context: &TaskContext,
        planned_steps: &mut Vec<PlannedStep>,
    ) -> Result<bool> {
        // First check if pattern should be applied with enhanced validation
        if !self.validator.should_apply_pattern(pattern, context) {
            return Ok(false);
        }

        // Assess risk for validation step injection
        let risk = self.assess_risk(pattern, context, planned_steps);

        // Inject validation step if risk is high
        if risk.should_inject_validation {
            self.inject_validation_step(planned_steps);
        }

        Ok(true)
    }

    /// Assess risk for validation step requirement
    fn assess_risk(
        &self,
        pattern: &Pattern,
        context: &TaskContext,
        planned_steps: &[PlannedStep],
    ) -> RiskAssessment {
        let mut risk_score = 0.0;

        // Context complexity risk (40% weight)
        let complexity_risk = match context.complexity {
            crate::types::ComplexityLevel::Simple => 0.2,
            crate::types::ComplexityLevel::Moderate => 0.5,
            crate::types::ComplexityLevel::Complex => 0.8,
        };
        risk_score += complexity_risk * 0.4;

        // Pattern confidence inverse risk (30% weight)
        let confidence_risk = (1.0 - pattern.confidence()) * 0.9;
        risk_score += confidence_risk * 0.3;

        // Step count risk (20% weight)
        let step_count_risk = (planned_steps.len() as f32 / 10.0).min(1.0);
        risk_score += step_count_risk * 0.2;

        // Tool compatibility risk (10% weight) - NEW IMPLEMENTATION
        let tool_compatibility_risk =
            self.calculate_tool_compatibility_risk(pattern, context, planned_steps);
        risk_score += tool_compatibility_risk * 0.1;

        // Domain-specific risk (10% weight)
        let domain_risk = match context.domain.as_str() {
            "debugging" => 0.8,       // High risk domain
            "api_development" => 0.4, // Moderate risk
            "configuration" => 0.2,   // Low risk
            _ => 0.5,
        };
        risk_score += domain_risk * 0.1;

        RiskAssessment {
            overall_risk_score: risk_score,
            step_level_risks: vec![risk_score; planned_steps.len()],
            context_complexity_risk: complexity_risk,
            tool_compatibility_risk,
            should_inject_validation: risk_score > self.risk_threshold,
        }
    }

    /// Calculate tool compatibility risk based on planned steps
    fn calculate_tool_compatibility_risk(
        &self,
        _pattern: &Pattern,
        context: &TaskContext,
        planned_steps: &[PlannedStep],
    ) -> f32 {
        if planned_steps.is_empty() {
            return 0.5; // Neutral risk when no steps planned
        }

        let mut total_compatibility = 0.0;
        let mut tools_assessed = 0;

        // Extract tools from planned steps and assess compatibility
        for step in planned_steps {
            let tool = Tool::new(step.tool.clone());
            let compatibility = self.assess_tool_compatibility(&tool, context);
            total_compatibility += compatibility;
            tools_assessed += 1;
        }

        if tools_assessed > 0 {
            let avg_compatibility = total_compatibility / tools_assessed as f32;
            // Convert compatibility score to risk score (1.0 - compatibility)
            (1.0 - avg_compatibility).clamp(0.0, 1.0)
        } else {
            0.5 // Neutral risk
        }
    }

    /// Inject validation step into planned steps
    fn inject_validation_step(&self, planned_steps: &mut Vec<PlannedStep>) {
        let validation_step = PlannedStep {
            tool: "validator".to_string(),
            action: "validate_before_execution".to_string(),
            expected_duration_ms: 30000, // 30 seconds
            parameters: serde_json::json!({
                "validation_type": "risk_mitigation",
                "automatic_injection": true
            }),
        };

        // Insert before the last step (usually the most critical)
        if planned_steps.is_empty() {
            planned_steps.push(validation_step);
        } else {
            let insert_position = planned_steps.len() - 1;
            planned_steps.insert(insert_position, validation_step);
        }
    }
}

impl Default for EnhancedPatternApplicator {
    fn default() -> Self {
        Self::new()
    }
}
