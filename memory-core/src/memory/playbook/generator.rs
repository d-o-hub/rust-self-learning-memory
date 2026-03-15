//! Playbook Generator - Template-driven synthesis of actionable playbooks.
//!
//! This module provides the `PlaybookGenerator` which synthesizes playbooks from
//! patterns, reflections, and summaries WITHOUT using LLM on the hot path.

use tracing::{info, instrument};
use uuid::Uuid;

use super::types::{
    PlaybookPitfall, PlaybookRequest, PlaybookStep, PlaybookSynthesisSource, RecommendedPlaybook,
};
use crate::error::Result;
use crate::pattern::Pattern;
use crate::semantic::EpisodeSummary;
use crate::types::TaskContext;

/// Template-driven playbook generator.
///
/// Synthesizes actionable playbooks from patterns, reflections, and summaries
/// using templates - NO LLM on the hot path.
///
/// # Example
///
/// ```no_run
/// use memory_core::memory::playbook::PlaybookGenerator;
///
/// let generator = PlaybookGenerator::new();
/// // Use generate() to create playbooks
/// ```
#[derive(Clone)]
pub struct PlaybookGenerator {
    /// Minimum confidence threshold for including patterns
    min_pattern_confidence: f32,
    /// Maximum number of patterns to synthesize
    max_patterns: usize,
}

impl Default for PlaybookGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybookGenerator {
    /// Create a new playbook generator with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_pattern_confidence: 0.5,
            max_patterns: 10,
        }
    }

    /// Set the minimum pattern confidence threshold.
    #[must_use]
    pub fn with_min_pattern_confidence(mut self, confidence: f32) -> Self {
        self.min_pattern_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set the maximum number of patterns to use.
    #[must_use]
    pub fn with_max_patterns(mut self, max: usize) -> Self {
        self.max_patterns = max;
        self
    }

    /// Generate a playbook from patterns, summaries, and reflections.
    ///
    /// This is the main entry point for playbook generation. It synthesizes
    /// actionable guidance from multiple data sources using templates.
    ///
    /// # Arguments
    ///
    /// * `request` - The playbook request with task details
    /// * `patterns` - Relevant patterns to synthesize
    /// * `summaries` - Episode summaries to use for context
    /// * `reflections` - Episode reflections (successes, improvements, insights)
    ///
    /// # Returns
    ///
    /// A `RecommendedPlaybook` with ordered steps, applicability rules, and expected outcomes.
    #[instrument(skip(self, patterns, summaries, reflections), fields(
        task = %request.task_description,
        domain = %request.domain,
        pattern_count = patterns.len(),
        summary_count = summaries.len()
    ))]
    pub fn generate(
        &self,
        request: &PlaybookRequest,
        patterns: &[Pattern],
        summaries: &[EpisodeSummary],
        reflections: &[ReflectionData],
    ) -> Result<RecommendedPlaybook> {
        let playbook_id = Uuid::new_v4();
        let mut source = PlaybookSynthesisSource::new();

        // Step 1: Calculate task match score
        let task_match_score = self.calculate_task_match(request, patterns);

        // Step 2: Generate ordered steps from patterns
        let ordered_steps = self.synthesize_steps(patterns, &mut source, request.max_steps);

        // Step 3: Extract applicability rules
        let (when_to_apply, when_not_to_apply) =
            self.synthesize_applicability(patterns, &request.context);

        // Step 4: Synthesize pitfalls from reflections
        let pitfalls = self.synthesize_pitfalls(reflections, &mut source);

        // Step 5: Generate expected outcome
        let expected_outcome = self.synthesize_expected_outcome(patterns, summaries, &mut source);

        // Step 6: Calculate confidence
        let confidence = self.calculate_confidence(patterns, summaries, &source);

        // Step 7: Generate why_relevant explanation
        let why_relevant = self.generate_why_relevant(patterns, summaries, &source);

        // Step 8: Collect supporting IDs
        let supporting_pattern_ids: Vec<Uuid> = patterns
            .iter()
            .take(self.max_patterns)
            .map(|p| p.id())
            .collect();

        let supporting_episode_ids: Vec<Uuid> = source.episode_ids.clone();

        info!(
            playbook_id = %playbook_id,
            task_match_score = task_match_score,
            confidence = confidence,
            step_count = ordered_steps.len(),
            source_count = source.total_sources(),
            "Generated playbook"
        );

        Ok(RecommendedPlaybook {
            playbook_id,
            task_match_score,
            why_relevant,
            when_to_apply,
            when_not_to_apply,
            ordered_steps,
            pitfalls,
            expected_outcome,
            confidence,
            supporting_pattern_ids,
            supporting_episode_ids,
            created_at: chrono::Utc::now(),
        })
    }

    /// Calculate how well patterns match the task.
    fn calculate_task_match(&self, request: &PlaybookRequest, patterns: &[Pattern]) -> f32 {
        if patterns.is_empty() {
            return 0.0;
        }

        // Calculate average success rate of patterns
        let avg_success_rate: f32 =
            patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32;

        // Calculate context match
        let context_matches: usize = patterns
            .iter()
            .filter_map(|p| p.context())
            .filter(|ctx| {
                ctx.domain == request.domain
                    || ctx.tags.iter().any(|t| request.context.tags.contains(t))
            })
            .count();

        let context_match_ratio = if patterns.is_empty() {
            0.0
        } else {
            context_matches as f32 / patterns.len() as f32
        };

        // Weighted combination
        avg_success_rate * 0.6 + context_match_ratio * 0.4
    }

    /// Synthesize ordered steps from patterns.
    fn synthesize_steps(
        &self,
        patterns: &[Pattern],
        source: &mut PlaybookSynthesisSource,
        max_steps: usize,
    ) -> Vec<PlaybookStep> {
        let mut builder = StepsBuilder::new(max_steps);

        for pattern in patterns.iter().take(self.max_patterns) {
            if builder.is_full() {
                break;
            }

            source.add_pattern(pattern.id());
            builder.add_pattern_steps(pattern, source);
        }

        builder.build()
    }

    /// Synthesize when to apply and when not to apply rules.
    fn synthesize_applicability(
        &self,
        patterns: &[Pattern],
        context: &TaskContext,
    ) -> (Vec<String>, Vec<String>) {
        let mut when_to_apply = Vec::new();
        let mut when_not_to_apply = Vec::new();

        for pattern in patterns.iter().take(self.max_patterns) {
            match pattern {
                Pattern::ToolSequence { tools, context, .. } => {
                    when_to_apply.push(format!(
                        "When working with {} in {} domain",
                        tools.join(", "),
                        context.domain
                    ));
                }
                Pattern::DecisionPoint {
                    condition, action, ..
                } => {
                    when_to_apply.push(format!("When condition '{}' is true", condition));
                    when_not_to_apply.push(format!(
                        "When condition '{}' is false - skip {}",
                        condition, action
                    ));
                }
                Pattern::ErrorRecovery { error_type, .. } => {
                    when_to_apply.push(format!("When encountering {} errors", error_type));
                }
                Pattern::ContextPattern {
                    context_features, ..
                } => {
                    let features = context_features.join(", ");
                    when_to_apply.push(format!("When context includes: {}", features));
                    if !context.tags.is_empty() {
                        when_not_to_apply.push("When task has different context tags".to_string());
                    }
                }
            }
        }

        // Deduplicate
        when_to_apply.sort();
        when_to_apply.dedup();
        when_not_to_apply.sort();
        when_not_to_apply.dedup();

        (when_to_apply, when_not_to_apply)
    }

    /// Synthesize pitfalls from reflections.
    fn synthesize_pitfalls(
        &self,
        reflections: &[ReflectionData],
        source: &mut PlaybookSynthesisSource,
    ) -> Vec<PlaybookPitfall> {
        let mut pitfalls = Vec::new();

        for reflection in reflections {
            source.add_episode(reflection.episode_id);

            // Improvements become pitfalls
            for improvement in &reflection.improvements {
                pitfalls.push(
                    PlaybookPitfall::new(
                        format!("Potential issue: {}", improvement),
                        "Identified from past execution",
                    )
                    .with_mitigation("Review and apply this improvement"),
                );
            }

            // Failed steps become warnings
            for failed_step in &reflection.failed_steps {
                pitfalls.push(PlaybookPitfall::new(
                    format!("Step may fail: {}", failed_step),
                    "Based on historical failures",
                ));
            }
        }

        // Limit to top 5 pitfalls
        pitfalls.truncate(5);
        pitfalls
    }

    /// Synthesize expected outcome from patterns and summaries.
    fn synthesize_expected_outcome(
        &self,
        patterns: &[Pattern],
        summaries: &[EpisodeSummary],
        source: &mut PlaybookSynthesisSource,
    ) -> String {
        let mut outcome_parts = Vec::new();

        // From patterns - use success rates
        let avg_success: f32 = if patterns.is_empty() {
            0.0
        } else {
            patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32
        };

        if avg_success > 0.7 {
            outcome_parts.push("High probability of success".to_string());
        } else if avg_success > 0.5 {
            outcome_parts.push("Moderate probability of success".to_string());
        } else {
            outcome_parts.push("Variable outcomes expected".to_string());
        }

        // From summaries - use key concepts
        for summary in summaries.iter().take(3) {
            source.add_summary(summary.episode_id);
            if !summary.key_concepts.is_empty() {
                outcome_parts.push(format!(
                    "Key concepts: {}",
                    summary
                        .key_concepts
                        .iter()
                        .take(3)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        outcome_parts.join(". ")
    }

    /// Calculate overall confidence from multiple sources.
    fn calculate_confidence(
        &self,
        patterns: &[Pattern],
        summaries: &[EpisodeSummary],
        source: &PlaybookSynthesisSource,
    ) -> f32 {
        if patterns.is_empty() && summaries.is_empty() {
            return 0.0;
        }

        let mut confidence = 0.0;

        // Pattern contribution (0-0.4)
        if !patterns.is_empty() {
            let avg_success: f32 =
                patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32;
            confidence += avg_success * 0.4;
        }

        // Summary contribution (0-0.3)
        if !summaries.is_empty() {
            let summary_boost = (summaries.len() as f32).min(3.0) / 3.0 * 0.3;
            confidence += summary_boost;
        }

        // Source diversity contribution (0-0.3)
        let source_diversity = (source.total_sources() as f32).ln().max(0.0) / 3.0 * 0.3;
        confidence += source_diversity;

        confidence.min(1.0)
    }

    /// Generate why_relevant explanation.
    fn generate_why_relevant(
        &self,
        patterns: &[Pattern],
        summaries: &[EpisodeSummary],
        source: &PlaybookSynthesisSource,
    ) -> String {
        let mut reasons = Vec::new();

        if !patterns.is_empty() {
            reasons.push(format!(
                "Based on {} patterns with {:.0}% average success rate",
                patterns.len(),
                patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32
                    * 100.0
            ));
        }

        if !summaries.is_empty() {
            reasons.push(format!(
                "Synthesized from {} similar episode summaries",
                summaries.len()
            ));
        }

        if source.total_sources() > 0 {
            reasons.push(format!(
                "Supported by {} historical data points",
                source.total_sources()
            ));
        }

        if reasons.is_empty() {
            "Generated from available memory data".to_string()
        } else {
            reasons.join(". ")
        }
    }
}

// Helper functions to reduce nesting in synthesize_steps

fn create_tool_step(order: usize, tool: &str) -> PlaybookStep {
    PlaybookStep::new(order, format!("Use {} tool", tool))
        .with_tool_hint(tool.to_string())
        .with_expected_result(format!("Execute {} operation", tool.replace('_', " ")))
}

fn create_decision_step(order: usize, condition: &str, action: &str) -> PlaybookStep {
    PlaybookStep::new(order, format!("Evaluate: {}", condition))
        .with_expected_result(format!("Then: {}", action))
}

fn create_error_step(order: usize, error_type: &str) -> PlaybookStep {
    PlaybookStep::new(order, format!("Handle {} error", error_type)).with_tool_hint("error_handler")
}

fn create_context_step(order: usize, recommended_approach: &str) -> PlaybookStep {
    PlaybookStep::new(order, recommended_approach.to_string())
        .with_expected_result("Context-appropriate action")
}

/// Builder for creating playbook steps without excessive nesting.
struct StepsBuilder {
    steps: Vec<PlaybookStep>,
    max_steps: usize,
    current_order: usize,
}

impl StepsBuilder {
    fn new(max_steps: usize) -> Self {
        Self {
            steps: Vec::new(),
            max_steps,
            current_order: 1,
        }
    }

    fn is_full(&self) -> bool {
        self.current_order > self.max_steps
    }

    fn add_step(&mut self, step: PlaybookStep) -> bool {
        if self.current_order > self.max_steps {
            return false;
        }
        self.steps.push(step);
        self.current_order += 1;
        true
    }

    fn add_pattern_steps(&mut self, pattern: &Pattern, source: &mut PlaybookSynthesisSource) {
        match pattern {
            Pattern::ToolSequence { tools, .. } => {
                for tool in tools.iter().take(self.max_steps / 2) {
                    if !self.add_step(create_tool_step(self.current_order, tool)) {
                        break;
                    }
                }
            }
            Pattern::DecisionPoint {
                condition, action, ..
            } => {
                self.add_step(create_decision_step(self.current_order, condition, action));
            }
            Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                ..
            } => {
                self.add_step(create_error_step(self.current_order, error_type));
                for recovery in recovery_steps.iter().take(2) {
                    if !self.add_step(PlaybookStep::new(self.current_order, recovery.clone())) {
                        break;
                    }
                }
            }
            Pattern::ContextPattern {
                recommended_approach,
                evidence,
                ..
            } => {
                self.add_step(create_context_step(
                    self.current_order,
                    recommended_approach,
                ));
                for ep_id in evidence {
                    source.add_episode(*ep_id);
                }
            }
        }
    }

    fn build(self) -> Vec<PlaybookStep> {
        self.steps
    }
}

/// Reflection data used for playbook synthesis.
///
/// Extracted from episode reflections for synthesis.
#[derive(Debug, Clone)]
pub struct ReflectionData {
    /// Episode ID this reflection came from
    pub episode_id: Uuid,
    /// Successes from the reflection
    pub successes: Vec<String>,
    /// Improvements identified
    pub improvements: Vec<String>,
    /// Insights gained
    pub insights: Vec<String>,
    /// Steps that failed during execution
    pub failed_steps: Vec<String>,
}

impl ReflectionData {
    /// Create reflection data from a reflection struct.
    #[must_use]
    pub fn from_reflection(episode_id: Uuid, reflection: &crate::types::Reflection) -> Self {
        Self {
            episode_id,
            successes: reflection.successes.clone(),
            improvements: reflection.improvements.clone(),
            insights: reflection.insights.clone(),
            failed_steps: Vec::new(), // Would be extracted from steps
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::PatternEffectiveness;
    use crate::types::{ComplexityLevel, TaskContext};

    fn create_test_pattern() -> Pattern {
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read_file".to_string(), "edit_file".to_string()],
            context: TaskContext {
                domain: "web-api".to_string(),
                language: Some("rust".to_string()),
                framework: Some("axum".to_string()),
                complexity: ComplexityLevel::Moderate,
                tags: vec!["api".to_string()],
            },
            success_rate: 0.85,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: 10,
            effectiveness: PatternEffectiveness::new(),
        }
    }

    fn create_test_summary() -> EpisodeSummary {
        EpisodeSummary::new(Uuid::new_v4())
    }

    #[test]
    fn test_generator_creation() {
        let generator = PlaybookGenerator::new()
            .with_min_pattern_confidence(0.7)
            .with_max_patterns(5);

        assert_eq!(generator.min_pattern_confidence, 0.7);
        assert_eq!(generator.max_patterns, 5);
    }

    #[test]
    fn test_generate_playbook() {
        let generator = PlaybookGenerator::new();
        let request = PlaybookRequest::new("Test task", "web-api");

        let patterns = vec![create_test_pattern()];
        let summaries = vec![create_test_summary()];
        let reflections = vec![];

        let playbook = generator
            .generate(&request, &patterns, &summaries, &reflections)
            .unwrap();

        assert!(playbook.task_match_score > 0.0);
        assert!(!playbook.ordered_steps.is_empty());
        assert!(!playbook.why_relevant.is_empty());
    }

    #[test]
    fn test_generate_playbook_empty_inputs() {
        let generator = PlaybookGenerator::new();
        let request = PlaybookRequest::new("Test task", "web-api");

        let playbook = generator.generate(&request, &[], &[], &[]).unwrap();

        assert_eq!(playbook.task_match_score, 0.0);
        assert!(playbook.ordered_steps.is_empty());
        assert_eq!(playbook.confidence, 0.0);
    }

    #[test]
    fn test_calculate_task_match() {
        let generator = PlaybookGenerator::new();
        let request = PlaybookRequest::new("Test task", "web-api");

        let patterns = vec![create_test_pattern()];
        let score = generator.calculate_task_match(&request, &patterns);

        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_synthesize_steps() {
        let generator = PlaybookGenerator::new();
        let patterns = vec![create_test_pattern()];
        let mut source = PlaybookSynthesisSource::new();

        let steps = generator.synthesize_steps(&patterns, &mut source, 5);

        assert!(!steps.is_empty());
        assert!(steps.len() <= 5);
        assert!(!source.pattern_ids.is_empty());
    }
}
