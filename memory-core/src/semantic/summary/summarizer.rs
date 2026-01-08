//! Semantic summarizer implementation.

#![allow(clippy::uninlined_format_args)]

use super::{add_salient_features_summary, extract_key_concepts, extract_key_steps};

use crate::embeddings::EmbeddingProvider;
use crate::episode::Episode;
use crate::semantic::summary::EpisodeSummary;
use crate::types::TaskOutcome;
use anyhow::Result;
use chrono::Utc;

/// Semantic summarizer for episodes.
///
/// Compresses episodes into concise, searchable summaries suitable for
/// efficient storage and retrieval. Extracts key concepts, critical steps,
/// and generates coherent summary text.
///
/// # Configuration
///
/// * `min_summary_length` - Minimum words in summary (default: 100)
/// * `max_summary_length` - Maximum words in summary (default: 200)
/// * `max_key_steps` - Maximum key steps to extract (default: 5)
///
/// # Examples
///
/// ```no_run
/// use memory_core::semantic::SemanticSummarizer;
/// use memory_core::{Episode, TaskContext, TaskType, ExecutionStep, ExecutionResult, TaskOutcome};
///
/// # async fn example() -> anyhow::Result<()> {
/// let summarizer = SemanticSummarizer::new();
///
/// let mut episode = Episode::new(
///     "Implement user authentication".to_string(),
///     TaskContext::default(),
///     TaskType::CodeGeneration,
/// );
///
/// // Add steps
/// let mut step = ExecutionStep::new(1, "planner".to_string(), "Analyze requirements".to_string());
/// step.result = Some(ExecutionResult::Success { output: "Requirements clear".to_string() });
/// episode.add_step(step);
///
/// episode.complete(TaskOutcome::Success {
///     verdict: "Authentication implemented successfully".to_string(),
///     artifacts: vec!["auth.rs".to_string()],
/// });
///
/// let summary = summarizer.summarize_episode(&episode).await?;
/// assert!(summary.summary_text.split_whitespace().count() >= 50);
/// assert!(!summary.key_concepts.is_empty());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SemanticSummarizer {
    /// Minimum words in summary (informational, not enforced to avoid artificial padding)
    #[allow(dead_code)]
    min_summary_length: usize,
    /// Maximum words in summary
    max_summary_length: usize,
    /// Maximum key steps to extract
    max_key_steps: usize,
}

impl SemanticSummarizer {
    /// Create a new semantic summarizer with default settings.
    ///
    /// Default configuration:
    /// - Min summary length: 100 words
    /// - Max summary length: 200 words
    /// - Max key steps: 5
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::semantic::SemanticSummarizer;
    ///
    /// let summarizer = SemanticSummarizer::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_summary_length: 100,
            max_summary_length: 200,
            max_key_steps: 5,
        }
    }

    /// Create a semantic summarizer with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `min_len` - Minimum words in summary
    /// * `max_len` - Maximum words in summary
    /// * `max_steps` - Maximum key steps to extract
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::semantic::SemanticSummarizer;
    ///
    /// let summarizer = SemanticSummarizer::with_config(50, 150, 3);
    /// ```
    #[must_use]
    pub fn with_config(min_len: usize, max_len: usize, max_steps: usize) -> Self {
        Self {
            min_summary_length: min_len,
            max_summary_length: max_len,
            max_key_steps: max_steps,
        }
    }

    /// Summarize an episode into a semantic summary.
    ///
    /// Extracts key information and compresses the episode into a concise,
    /// searchable summary with key concepts, critical steps, and summary text.
    /// Optionally generates embedding if provider is available.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to summarize
    ///
    /// # Returns
    ///
    /// Semantic summary of the episode
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::semantic::SemanticSummarizer;
    /// use memory_core::{Episode, TaskContext, TaskType};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let summarizer = SemanticSummarizer::new();
    /// let episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// let summary = summarizer.summarize_episode(&episode).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::unused_async)]
    pub async fn summarize_episode(&self, episode: &Episode) -> Result<EpisodeSummary> {
        let key_concepts = extract_key_concepts(episode);
        let key_steps = extract_key_steps(episode, self.max_key_steps);
        let summary_text = self.generate_summary_text(episode);

        Ok(EpisodeSummary {
            episode_id: episode.episode_id,
            summary_text,
            key_concepts,
            key_steps,
            summary_embedding: None,
            created_at: Utc::now(),
        })
    }

    /// Summarize an episode with embedding generation.
    ///
    /// Like `summarize_episode` but also generates an embedding vector for
    /// the summary text using the provided embedding provider.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to summarize
    /// * `embedding_provider` - Provider to generate embeddings
    ///
    /// # Returns
    ///
    /// Semantic summary with embedding vector
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::semantic::SemanticSummarizer;
    /// use memory_core::{Episode, TaskContext, TaskType};
    /// use memory_core::embeddings::EmbeddingProvider;
    ///
    /// # async fn example(episode: Episode, provider: impl EmbeddingProvider) {
    /// let summarizer = SemanticSummarizer::new();
    /// let summary = summarizer.summarize_with_embedding(&episode, &provider).await.unwrap();
    /// assert!(summary.summary_embedding.is_some());
    /// # }
    /// ```
    pub async fn summarize_with_embedding(
        &self,
        episode: &Episode,
        embedding_provider: &impl EmbeddingProvider,
    ) -> Result<EpisodeSummary> {
        let mut summary = self.summarize_episode(episode).await?;

        // Generate embedding for summary text
        let embedding = embedding_provider.embed_text(&summary.summary_text).await?;
        summary.summary_embedding = Some(embedding);

        Ok(summary)
    }

    /// Generate summary text from an episode.
    ///
    /// Creates a coherent 100-200 word summary with:
    /// - Task description
    /// - Key execution steps
    /// - Salient features (if available)
    /// - Outcome and artifacts
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to summarize
    ///
    /// # Returns
    ///
    /// Formatted summary text
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::semantic::SemanticSummarizer;
    /// use memory_core::{Episode, TaskContext, TaskType, TaskOutcome};
    ///
    /// let summarizer = SemanticSummarizer::new();
    ///
    /// let mut episode = Episode::new(
    ///     "Implement authentication".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::CodeGeneration,
    /// );
    ///
    /// episode.complete(TaskOutcome::Success {
    ///     verdict: "Successfully implemented".to_string(),
    ///     artifacts: vec!["auth.rs".to_string()],
    /// });
    ///
    /// let summary = summarizer.generate_summary_text(&episode);
    /// assert!(summary.contains("Task:"));
    /// assert!(summary.contains("Outcome:"));
    /// ```
    #[must_use]
    pub fn generate_summary_text(&self, episode: &Episode) -> String {
        let mut parts = Vec::new();

        // Task description
        parts.push(format!(
            "Task: {}.",
            episode.task_description.trim_end_matches('.')
        ));

        // Context information
        let mut context_parts = Vec::new();
        if let Some(ref lang) = episode.context.language {
            context_parts.push(format!("Language: {lang}"));
        }
        if let Some(ref framework) = episode.context.framework {
            context_parts.push(format!("Framework: {framework}"));
        }
        if !episode.context.domain.is_empty() && episode.context.domain != "general" {
            context_parts.push(format!("Domain: {}", episode.context.domain));
        }
        if !context_parts.is_empty() {
            parts.push(format!("Context: {}.", context_parts.join(", ")));
        }

        // Step summary
        if !episode.steps.is_empty() {
            let total_steps = episode.steps.len();
            let successful = episode.successful_steps_count();
            let failed = episode.failed_steps_count();

            let step_summary = if failed > 0 {
                format!(
                    "Execution: {} steps ({} successful, {} failed)",
                    total_steps, successful, failed
                )
            } else {
                format!("Execution: {} steps (all successful)", total_steps)
            };
            parts.push(format!("{step_summary}."));

            // Add key step actions
            let key_steps = extract_key_steps(episode, self.max_key_steps);
            if !key_steps.is_empty() {
                let steps_desc = key_steps.join("; ");
                parts.push(format!("Key steps: {steps_desc}."));
            }
        }

        // Salient features summary
        if let Some(ref features) = episode.salient_features {
            add_salient_features_summary(features, &mut parts);
        }

        // Outcome
        if let Some(ref outcome) = episode.outcome {
            let outcome_text = match outcome {
                TaskOutcome::Success { verdict, artifacts } => {
                    if artifacts.is_empty() {
                        format!("Outcome: Success - {verdict}")
                    } else {
                        format!(
                            "Outcome: Success - {verdict}. Artifacts: {}",
                            artifacts.join(", ")
                        )
                    }
                }
                TaskOutcome::PartialSuccess {
                    verdict,
                    completed,
                    failed,
                } => {
                    format!(
                        "Outcome: Partial success - {verdict}. Completed: {}. Failed: {}",
                        completed.join(", "),
                        failed.join(", ")
                    )
                }
                TaskOutcome::Failure {
                    reason,
                    error_details,
                } => {
                    if let Some(details) = error_details {
                        format!("Outcome: Failure - {reason}. Details: {details}")
                    } else {
                        format!("Outcome: Failure - {reason}")
                    }
                }
            };
            parts.push(format!("{outcome_text}."));
        }

        // Combine parts and ensure word count
        let mut summary = parts.join(" ");

        // Truncate if too long
        let words: Vec<&str> = summary.split_whitespace().collect();
        if words.len() > self.max_summary_length {
            summary = words[..self.max_summary_length].join(" ");
            summary.push_str("...");
        }

        summary
    }
}

impl Default for SemanticSummarizer {
    fn default() -> Self {
        Self::new()
    }
}
