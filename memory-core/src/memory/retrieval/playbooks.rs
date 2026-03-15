//! Playbook retrieval implementation (ADR-044 Feature 1)
//!
//! Provides `retrieve_playbooks()` method for synthesizing actionable playbooks
//! from patterns, reflections, and summaries.

use crate::memory::playbook::{
    PlaybookGenerator, PlaybookRequest, RecommendedPlaybook, ReflectionData,
};
use crate::pattern::Pattern;
use crate::semantic::EpisodeSummary;
use crate::types::TaskContext;
use crate::types::TaskType;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use super::super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Retrieve actionable playbooks for a task.
    ///
    /// Synthesizes playbooks from patterns, reflections, and summaries using
    /// template-driven generation (NO LLM on the hot path).
    ///
    /// Playbooks provide:
    /// - Ordered, actionable steps with tool hints
    /// - When to apply / when not to apply guidance
    /// - Pitfalls and warnings to avoid
    /// - Expected outcomes with confidence scores
    ///
    /// # Arguments
    ///
    /// * `task_description` - Description of the task to perform
    /// * `domain` - Domain of the task (e.g., "web-api", "testing")
    /// * `task_type` - Type of task (CodeGeneration, Debugging, etc.)
    /// * `context` - Additional task context
    /// * `max_playbooks` - Maximum number of playbooks to return
    /// * `max_steps_per_playbook` - Maximum steps per playbook
    ///
    /// # Returns
    ///
    /// Vector of recommended playbooks sorted by quality score.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use memory_core::SelfLearningMemory;
    /// use memory_core::{TaskContext, TaskType};
    ///
    /// # async fn example() {
    /// let memory = SelfLearningMemory::new();
    ///
    /// let playbooks = memory.retrieve_playbooks(
    ///     "Implement user authentication",
    ///     "security",
    ///     TaskType::CodeGeneration,
    ///     TaskContext::default(),
    ///     3,
    ///     5,
    /// ).await;
    ///
    /// for playbook in &playbooks {
    ///     println!("Playbook: {} (confidence: {:.0}%)",
    ///         playbook.playbook_id,
    ///         playbook.confidence * 100.0
    ///     );
    ///     for step in &playbook.ordered_steps {
    ///         println!("  Step {}: {}", step.order, step.action);
    ///     }
    /// }
    /// # }
    /// ```
    #[instrument(skip(self, context))]
    pub async fn retrieve_playbooks(
        &self,
        task_description: &str,
        domain: &str,
        task_type: TaskType,
        context: TaskContext,
        max_playbooks: usize,
        max_steps_per_playbook: usize,
    ) -> Vec<RecommendedPlaybook> {
        debug!(
            task_description = %task_description,
            domain = %domain,
            task_type = ?task_type,
            max_playbooks = max_playbooks,
            "Retrieving playbooks"
        );

        // Step 1: Retrieve relevant patterns
        let patterns = self.retrieve_relevant_patterns(&context, 10).await;

        // Step 2: Retrieve relevant episodes for summaries and reflections
        let episodes = self
            .retrieve_relevant_context(task_description.to_string(), context.clone(), 5)
            .await;

        // Step 3: Extract summaries from episodes (if semantic summarizer available)
        let summaries: Vec<EpisodeSummary> = episodes
            .iter()
            .map(|ep| {
                // Create a basic summary from episode data
                EpisodeSummary {
                    episode_id: ep.episode_id,
                    summary_text: ep.task_description.clone(),
                    key_concepts: ep.context.tags.clone(),
                    key_steps: ep
                        .steps
                        .iter()
                        .take(3)
                        .map(|s| format!("{}: {}", s.tool, s.action))
                        .collect(),
                    summary_embedding: None,
                    created_at: ep.start_time,
                }
            })
            .collect();

        // Step 4: Extract reflection data from episodes
        let reflections: Vec<ReflectionData> = episodes
            .iter()
            .filter_map(|ep| {
                ep.reflection.as_ref().map(|r| {
                    let mut data = ReflectionData::from_reflection(ep.episode_id, r);
                    // Add failed steps from episode
                    data.failed_steps = ep
                        .steps
                        .iter()
                        .filter(|s| !s.is_success())
                        .map(|s| format!("{}: {}", s.tool, s.action))
                        .collect();
                    data
                })
            })
            .collect();

        // Step 5: Create playbook generator
        let generator = PlaybookGenerator::new()
            .with_min_pattern_confidence(0.5)
            .with_max_patterns(10);

        // Step 6: Generate playbook request
        let request = PlaybookRequest {
            task_description: task_description.to_string(),
            domain: domain.to_string(),
            task_type,
            context: context.clone(),
            max_steps: max_steps_per_playbook,
        };

        // Step 7: Generate the playbook
        match generator.generate(&request, &patterns, &summaries, &reflections) {
            Ok(playbook) => {
                // Record this recommendation session for attribution tracking
                let session = crate::memory::attribution::RecommendationSession {
                    session_id: Uuid::new_v4(),
                    episode_id: Uuid::nil(), // No episode yet
                    timestamp: chrono::Utc::now(),
                    recommended_pattern_ids: playbook
                        .supporting_pattern_ids
                        .iter()
                        .map(|id| id.to_string())
                        .collect(),
                    recommended_playbook_ids: vec![playbook.playbook_id],
                };

                self.recommendation_tracker.record_session(session).await;

                info!(
                    playbook_id = %playbook.playbook_id,
                    confidence = playbook.confidence,
                    step_count = playbook.ordered_steps.len(),
                    source_count = playbook.supporting_pattern_ids.len() + playbook.supporting_episode_ids.len(),
                    "Generated playbook"
                );

                vec![playbook]
            }
            Err(e) => {
                debug!("Failed to generate playbook: {}", e);
                vec![]
            }
        }
    }

    /// Explain a pattern in human-readable form.
    ///
    /// Provides a detailed explanation of a pattern including:
    /// - What the pattern does
    /// - When to use it
    /// - Expected success rate
    /// - Supporting evidence
    ///
    /// # Arguments
    ///
    /// * `pattern_id` - ID of the pattern to explain
    ///
    /// # Returns
    ///
    /// Human-readable explanation string, or None if pattern not found.
    #[instrument(skip(self))]
    pub async fn explain_pattern(&self, pattern_id: Uuid) -> Option<String> {
        let pattern = self.get_pattern(pattern_id).await.ok()??;

        let explanation = match &pattern {
            Pattern::ToolSequence {
                tools,
                context,
                success_rate,
                occurrence_count,
                ..
            } => {
                format!(
                    "Tool Sequence Pattern:\n\
                     - Tools: {}\n\
                     - Context: {} domain{}\n\
                     - Success Rate: {:.0}%\n\
                     - Occurrences: {}\n\
                     \n\
                     This pattern suggests using the tool sequence {} for tasks in the {} domain. \
                     It has been successful {:.0}% of the time across {} occurrences.",
                    tools.join(" -> "),
                    context.domain,
                    context
                        .language
                        .as_ref()
                        .map(|l| format!(" (language: {})", l))
                        .unwrap_or_default(),
                    success_rate * 100.0,
                    occurrence_count,
                    tools.join(" then "),
                    context.domain,
                    success_rate * 100.0,
                    occurrence_count
                )
            }
            Pattern::DecisionPoint {
                condition,
                action,
                outcome_stats,
                context,
                ..
            } => {
                format!(
                    "Decision Point Pattern:\n\
                     - Condition: {}\n\
                     - Action: {}\n\
                     - Success Rate: {:.0}% ({} of {} attempts)\n\
                     - Context: {} domain\n\
                     \n\
                     When the condition \"{}\" is true, consider taking the action \"{}\". \
                     This decision has led to successful outcomes {:.0}% of the time.",
                    condition,
                    action,
                    outcome_stats.success_rate() * 100.0,
                    outcome_stats.success_count,
                    outcome_stats.total_count,
                    context.domain,
                    condition,
                    action,
                    outcome_stats.success_rate() * 100.0
                )
            }
            Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                success_rate,
                ..
            } => {
                format!(
                    "Error Recovery Pattern:\n\
                     - Error Type: {}\n\
                     - Recovery Steps: {}\n\
                     - Success Rate: {:.0}%\n\
                     \n\
                     When encountering a \"{}\" error, follow these recovery steps:\n{}\n\
                     This recovery approach has been successful {:.0}% of the time.",
                    error_type,
                    recovery_steps.join(" -> "),
                    success_rate * 100.0,
                    error_type,
                    recovery_steps
                        .iter()
                        .enumerate()
                        .map(|(i, s)| format!("  {}. {}", i + 1, s))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    success_rate * 100.0
                )
            }
            Pattern::ContextPattern {
                context_features,
                recommended_approach,
                evidence,
                success_rate,
                ..
            } => {
                format!(
                    "Context Pattern:\n\
                     - Features: {}\n\
                     - Recommended Approach: {}\n\
                     - Success Rate: {:.0}%\n\
                     - Supporting Episodes: {}\n\
                     \n\
                     When working in a context with features {}, \
                     the recommended approach is: \"{}\". \
                     This approach has {:.0}% success rate based on {} episodes.",
                    context_features.join(", "),
                    recommended_approach,
                    success_rate * 100.0,
                    evidence.len(),
                    context_features.join(" and "),
                    recommended_approach,
                    success_rate * 100.0,
                    evidence.len()
                )
            }
        };

        Some(explanation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_playbooks_empty() {
        let memory = SelfLearningMemory::new();

        let playbooks = memory
            .retrieve_playbooks(
                "Test task",
                "test",
                TaskType::CodeGeneration,
                TaskContext::default(),
                3,
                5,
            )
            .await;

        // With no patterns/episodes, should still return an empty or minimal playbook
        // depending on implementation
        assert!(playbooks.len() <= 1);
    }
}
