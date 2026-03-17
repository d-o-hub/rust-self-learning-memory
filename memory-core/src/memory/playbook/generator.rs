//! Playbook Generator - Template-driven synthesis of actionable playbooks.
//!
//! This module provides the `PlaybookGenerator` which synthesizes playbooks from
//! patterns, reflections, and summaries WITHOUT using LLM on the hot path.

use tracing::{info, instrument};
use uuid::Uuid;

use super::builder::{ReflectionData, StepsBuilder};
use super::types::{
    PlaybookPitfall, PlaybookRequest, PlaybookStep, PlaybookSynthesisSource, RecommendedPlaybook,
};
use crate::error::Result;
use crate::pattern::Pattern;
use crate::semantic::EpisodeSummary;
use crate::types::TaskContext;

/// Template-driven playbook generator.
#[derive(Clone, Default)]
pub struct PlaybookGenerator {
    /// Minimum confidence threshold for including patterns
    min_pattern_confidence: f32,
    /// Maximum number of patterns to synthesize
    max_patterns: usize,
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

        let task_match_score = self.calculate_task_match(request, patterns);
        let ordered_steps = self.synthesize_steps(patterns, &mut source, request.max_steps);
        let (when_to_apply, when_not_to_apply) =
            self.synthesize_applicability(patterns, &request.context);

        let pitfalls = self.synthesize_pitfalls(reflections, &mut source);
        let expected_outcome = self.synthesize_expected_outcome(patterns, summaries, &mut source);
        let confidence = self.calculate_confidence(patterns, summaries, &source);
        let why_relevant = self.generate_why_relevant(patterns, summaries, &source);

        let supporting_pattern_ids: Vec<Uuid> = patterns
            .iter()
            .take(self.max_patterns)
            .map(|p| p.id())
            .collect();

        info!(
            playbook_id = %playbook_id,
            confidence = confidence,
            step_count = ordered_steps.len(),
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
            supporting_episode_ids: source.episode_ids.clone(),
            created_at: chrono::Utc::now(),
        })
    }

    /// Calculate how well patterns match the task.
    fn calculate_task_match(&self, request: &PlaybookRequest, patterns: &[Pattern]) -> f32 {
        if patterns.is_empty() {
            return 0.0;
        }
        let avg_success: f32 =
            patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32;
        let context_matches: usize = patterns
            .iter()
            .filter_map(|p| p.context())
            .filter(|ctx| {
                ctx.domain == request.domain
                    || ctx.tags.iter().any(|t| request.context.tags.contains(t))
            })
            .count();
        avg_success * 0.6 + (context_matches as f32 / patterns.len() as f32) * 0.4
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
                    when_to_apply.push(format!(
                        "When context includes: {}",
                        context_features.join(", ")
                    ));
                    if !context.tags.is_empty() {
                        when_not_to_apply.push("When task has different context tags".to_string());
                    }
                }
            }
        }
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
            for improvement in &reflection.improvements {
                pitfalls.push(
                    PlaybookPitfall::new(
                        format!("Potential issue: {}", improvement),
                        "Identified from past execution",
                    )
                    .with_mitigation("Review and apply this improvement"),
                );
            }
            for failed_step in &reflection.failed_steps {
                pitfalls.push(PlaybookPitfall::new(
                    format!("Step may fail: {}", failed_step),
                    "Based on historical failures",
                ));
            }
        }
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
        if !patterns.is_empty() {
            let avg_success: f32 =
                patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32;
            confidence += avg_success * 0.4;
        }
        if !summaries.is_empty() {
            confidence += (summaries.len() as f32).min(3.0) / 3.0 * 0.3;
        }
        confidence += (source.total_sources() as f32).ln().max(0.0) / 3.0 * 0.3;
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
            let avg_success =
                patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32;
            reasons.push(format!(
                "Based on {} patterns with {:.0}% average success rate",
                patterns.len(),
                avg_success * 100.0
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
        let playbook = generator.generate(&request, &patterns, &[], &[]).unwrap();
        assert!(playbook.task_match_score > 0.0);
        assert!(!playbook.ordered_steps.is_empty());
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;
    use crate::pattern::PatternEffectiveness;
    use crate::types::OutcomeStats;

    #[test]
    fn test_synthesize_applicability_all_types() {
        let generator = PlaybookGenerator::new();
        let patterns = vec![
            Pattern::ToolSequence {
                id: Uuid::new_v4(),
                tools: vec!["tool1".to_string()],
                context: TaskContext {
                    domain: "web-api".to_string(),
                    ..Default::default()
                },
                success_rate: 0.9,
                avg_latency: chrono::Duration::zero(),
                occurrence_count: 1,
                effectiveness: PatternEffectiveness::new(),
            },
            Pattern::DecisionPoint {
                id: Uuid::new_v4(),
                condition: "cond".to_string(),
                action: "act".to_string(),
                outcome_stats: OutcomeStats {
                    success_count: 1,
                    failure_count: 0,
                    total_count: 1,
                    avg_duration_secs: 1.0,
                },
                context: TaskContext::default(),
                effectiveness: PatternEffectiveness::new(),
            },
        ];
        let (apply, skip) = generator.synthesize_applicability(&patterns, &TaskContext::default());
        assert_eq!(apply.len(), 2);
        assert_eq!(skip.len(), 1);
    }

    #[test]
    fn test_synthesize_pitfalls() {
        let generator = PlaybookGenerator::new();
        let mut source = PlaybookSynthesisSource::new();
        let reflections = vec![ReflectionData {
            episode_id: Uuid::new_v4(),
            successes: vec![],
            improvements: vec!["i1".to_string()],
            insights: vec![],
            failed_steps: vec!["f1".to_string()],
        }];
        let pitfalls = generator.synthesize_pitfalls(&reflections, &mut source);
        assert_eq!(pitfalls.len(), 2);
    }
}

#[cfg(test)]
mod more_playbook_edge_tests {
    use super::*;
    use crate::pattern::PatternEffectiveness;
    use crate::types::{OutcomeStats, TaskType};

    #[test]
    fn test_generate_with_zero_success_patterns() {
        let generator = PlaybookGenerator::new();
        let request = PlaybookRequest::new("Fail task", "web-api");

        let patterns = vec![
            Pattern::ToolSequence {
                id: Uuid::new_v4(),
                tools: vec!["tool1".to_string()],
                context: TaskContext { domain: "web-api".to_string(), ..Default::default() },
                success_rate: 0.0,
                avg_latency: chrono::Duration::zero(),
                occurrence_count: 10,
                effectiveness: PatternEffectiveness::new(),
            }
        ];

        let playbook = generator.generate(&request, &patterns, &[], &[]).unwrap();
        assert!(playbook.confidence < 0.1);
        assert!(playbook.expected_outcome.contains("Variable outcomes"));
    }

    #[test]
    fn test_generate_with_exceeding_max_patterns() {
        let generator = PlaybookGenerator::new().with_max_patterns(2);
        let request = PlaybookRequest::new("Task", "domain");

        let mut patterns = Vec::new();
        for _ in 0..10 {
            patterns.push(Pattern::ToolSequence {
                id: Uuid::new_v4(),
                tools: vec!["t".to_string()],
                context: TaskContext::default(),
                success_rate: 0.8,
                avg_latency: chrono::Duration::zero(),
                occurrence_count: 1,
                effectiveness: PatternEffectiveness::new(),
            });
        }

        let playbook = generator.generate(&request, &patterns, &[], &[]).unwrap();
        // Should only use 2 patterns for steps, but collect all for supporting_pattern_ids?
        // Wait, the code takes self.max_patterns for supporting_pattern_ids too.
        assert_eq!(playbook.supporting_pattern_ids.len(), 2);
    }

    #[test]
    fn test_synthesize_applicability_duplicates() {
        let generator = PlaybookGenerator::new();
        let patterns = vec![
            Pattern::ErrorRecovery {
                id: Uuid::new_v4(),
                error_type: "timeout".to_string(),
                recovery_steps: vec![],
                success_rate: 0.8,
                context: TaskContext::default(),
                effectiveness: PatternEffectiveness::new(),
            },
            Pattern::ErrorRecovery {
                id: Uuid::new_v4(),
                error_type: "timeout".to_string(),
                recovery_steps: vec![],
                success_rate: 0.9,
                context: TaskContext::default(),
                effectiveness: PatternEffectiveness::new(),
            }
        ];

        let (apply, _) = generator.synthesize_applicability(&patterns, &TaskContext::default());
        // Should be deduplicated
        assert_eq!(apply.len(), 1);
    }
}
