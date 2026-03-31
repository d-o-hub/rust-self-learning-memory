//! Playbook Generator - Template-driven synthesis of actionable playbooks
//! from patterns, reflections, and summaries WITHOUT using LLM on the hot path.

use super::builder::ReflectionData;
use super::templates;
use super::types::{PlaybookRequest, PlaybookSynthesisSource, RecommendedPlaybook};
use crate::error::Result;
use crate::pattern::Pattern;
use crate::semantic::EpisodeSummary;
use tracing::{info, instrument};
use uuid::Uuid;

/// Template-driven playbook generator.
///
/// Synthesizes actionable playbooks from patterns, reflections, and summaries
/// using templates - NO LLM on the hot path.
///
/// # Example
///
/// ```no_run
/// use do_memory_core::memory::playbook::PlaybookGenerator;
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
        let task_match_score = templates::calculate_task_match(request, patterns);

        // Step 2: Generate ordered steps from patterns
        let ordered_steps = templates::synthesize_steps(
            patterns,
            &mut source,
            request.max_steps,
            self.max_patterns,
        );

        // Step 3: Extract applicability rules
        let (when_to_apply, when_not_to_apply) =
            templates::synthesize_applicability(patterns, &request.context, self.max_patterns);

        // Step 4: Synthesize pitfalls from reflections
        let pitfalls = templates::synthesize_pitfalls(reflections, &mut source);

        // Step 5: Generate expected outcome
        let expected_outcome =
            templates::synthesize_expected_outcome(patterns, summaries, &mut source);

        // Step 6: Calculate confidence
        let confidence = templates::calculate_confidence(patterns, summaries, &source);

        // Step 7: Generate why_relevant explanation
        let why_relevant = templates::generate_why_relevant(patterns, summaries, &source);

        // Step 8: Collect supporting IDs
        let supporting_pattern_ids = patterns
            .iter()
            .take(self.max_patterns)
            .map(|p| p.id())
            .collect();
        let supporting_episode_ids = source.episode_ids.clone();

        info!(playbook_id = %playbook_id, task_match_score, confidence, step_count = ordered_steps.len(), source_count = source.total_sources(), "Generated playbook");

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
}
