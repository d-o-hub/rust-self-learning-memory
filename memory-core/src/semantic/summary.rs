//! Episode semantic summarization.
//!
//! Provides semantic summarization of episodes into concise, searchable summaries
//! with key concepts, critical steps, and optional embeddings for retrieval.

use crate::embeddings::EmbeddingProvider;
use crate::episode::Episode;
use crate::pre_storage::SalientFeatures;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// Condensed semantic summary of an episode.
///
/// Compresses a full episode into a concise, searchable summary with:
/// - Summary text (100-200 words)
/// - Key concepts for indexing
/// - Critical steps highlighting important actions
/// - Optional embedding vector for semantic search
///
/// # Fields
///
/// * `episode_id` - Original episode identifier
/// * `summary_text` - Concise 100-200 word summary
/// * `key_concepts` - Important concepts extracted (10-20 items)
/// * `key_steps` - Critical execution steps (3-5 items)
/// * `summary_embedding` - Optional embedding vector for semantic retrieval
/// * `created_at` - When this summary was created
///
/// # Examples
///
/// ```
/// use memory_core::semantic::EpisodeSummary;
/// use uuid::Uuid;
/// use chrono::Utc;
///
/// let summary = EpisodeSummary {
///     episode_id: Uuid::new_v4(),
///     summary_text: "Task: Implement user authentication. Steps: Analyzed requirements, \
///                    designed auth flow, implemented login/logout, added tests. \
///                    Outcome: Successfully implemented with 95% test coverage.".to_string(),
///     key_concepts: vec![
///         "authentication".to_string(),
///         "security".to_string(),
///         "rust".to_string(),
///     ],
///     key_steps: vec![
///         "Step 1: planner - Analyzed requirements".to_string(),
///         "Step 3: code_generator - Implemented login/logout".to_string(),
///         "Step 5: tester - Added comprehensive tests".to_string(),
///     ],
///     summary_embedding: None,
///     created_at: Utc::now(),
/// };
///
/// assert!(summary.summary_text.len() >= 100);
/// assert!(summary.summary_text.len() <= 500); // With some tolerance
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeSummary {
    /// Original episode identifier
    pub episode_id: Uuid,
    /// Concise summary text (100-200 words)
    pub summary_text: String,
    /// Important concepts extracted
    pub key_concepts: Vec<String>,
    /// Critical steps (3-5)
    pub key_steps: Vec<String>,
    /// Optional embedding vector for semantic search
    pub summary_embedding: Option<Vec<f32>>,
    /// When summary was created
    pub created_at: DateTime<Utc>,
}

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
/// # tokio_test::block_on(async {
/// let summary = summarizer.summarize_episode(&episode).await.unwrap();
/// assert!(summary.summary_text.split_whitespace().count() >= 50);
/// assert!(!summary.key_concepts.is_empty());
/// # });
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
    /// let summarizer = SemanticSummarizer::new();
    /// let episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// # tokio_test::block_on(async {
    /// let summary = summarizer.summarize_episode(&episode).await.unwrap();
    /// # });
    /// ```
    pub async fn summarize_episode(&self, episode: &Episode) -> Result<EpisodeSummary> {
        let key_concepts = self.extract_key_concepts(episode);
        let key_steps = self.extract_key_steps(episode);
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

    /// Extract key concepts from an episode.
    ///
    /// Identifies important concepts from:
    /// - Task description and context
    /// - Tools used in execution
    /// - Salient features (if available)
    ///
    /// Returns 10-20 normalized, deduplicated concepts.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to extract concepts from
    ///
    /// # Returns
    ///
    /// Vector of key concepts (normalized, deduplicated)
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::semantic::SemanticSummarizer;
    /// use memory_core::{Episode, TaskContext, TaskType, ComplexityLevel};
    ///
    /// let summarizer = SemanticSummarizer::new();
    ///
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     framework: Some("tokio".to_string()),
    ///     complexity: ComplexityLevel::Moderate,
    ///     domain: "authentication".to_string(),
    ///     tags: vec!["security".to_string()],
    /// };
    ///
    /// let episode = Episode::new(
    ///     "Implement JWT authentication".to_string(),
    ///     context,
    ///     TaskType::CodeGeneration,
    /// );
    ///
    /// let concepts = summarizer.extract_key_concepts(&episode);
    /// assert!(concepts.contains(&"rust".to_string()));
    /// assert!(concepts.contains(&"authentication".to_string()));
    /// ```
    pub fn extract_key_concepts(&self, episode: &Episode) -> Vec<String> {
        let mut concepts = HashSet::new();

        // Extract from task description
        for word in episode.task_description.split_whitespace() {
            let normalized = word
                .to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            if normalized.len() > 3 && !Self::is_stopword(&normalized) {
                concepts.insert(normalized);
            }
        }

        // Extract from context
        if let Some(ref lang) = episode.context.language {
            concepts.insert(lang.to_lowercase());
        }
        if let Some(ref framework) = episode.context.framework {
            concepts.insert(framework.to_lowercase());
        }
        concepts.insert(episode.context.domain.to_lowercase());
        for tag in &episode.context.tags {
            concepts.insert(tag.to_lowercase());
        }

        // Extract from task type
        concepts.insert(format!("{}", episode.task_type).to_lowercase());

        // Extract unique tools used
        for step in &episode.steps {
            concepts.insert(step.tool.to_lowercase());
        }

        // Extract from salient features if available
        if let Some(ref features) = episode.salient_features {
            for decision in &features.critical_decisions {
                for word in decision.split_whitespace() {
                    let normalized = word
                        .to_lowercase()
                        .trim_matches(|c: char| !c.is_alphanumeric())
                        .to_string();
                    if normalized.len() > 3 && !Self::is_stopword(&normalized) {
                        concepts.insert(normalized);
                    }
                }
            }

            for insight in &features.key_insights {
                for word in insight.split_whitespace() {
                    let normalized = word
                        .to_lowercase()
                        .trim_matches(|c: char| !c.is_alphanumeric())
                        .to_string();
                    if normalized.len() > 3 && !Self::is_stopword(&normalized) {
                        concepts.insert(normalized);
                    }
                }
            }
        }

        // Convert to sorted vector and limit
        let mut concept_vec: Vec<String> = concepts.into_iter().collect();
        concept_vec.sort();
        concept_vec.truncate(20);
        concept_vec
    }

    /// Extract critical steps from an episode.
    ///
    /// Selects 3-5 most important steps based on:
    /// - Steps with errors (error recovery learning)
    /// - Steps using unique/critical tools
    /// - Steps mentioned in salient features
    /// - First and last steps (context)
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to extract steps from
    ///
    /// # Returns
    ///
    /// Vector of formatted key steps (max 5)
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::semantic::SemanticSummarizer;
    /// use memory_core::{Episode, TaskContext, TaskType, ExecutionStep, ExecutionResult};
    ///
    /// let summarizer = SemanticSummarizer::new();
    ///
    /// let mut episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// let mut step1 = ExecutionStep::new(1, "planner".to_string(), "Plan approach".to_string());
    /// step1.result = Some(ExecutionResult::Success { output: "Plan ready".to_string() });
    /// episode.add_step(step1);
    ///
    /// let mut step2 = ExecutionStep::new(2, "executor".to_string(), "Execute plan".to_string());
    /// step2.result = Some(ExecutionResult::Error { message: "Failed".to_string() });
    /// episode.add_step(step2);
    ///
    /// let key_steps = summarizer.extract_key_steps(&episode);
    /// assert!(!key_steps.is_empty());
    /// ```
    pub fn extract_key_steps(&self, episode: &Episode) -> Vec<String> {
        if episode.steps.is_empty() {
            return Vec::new();
        }

        let mut key_steps = Vec::new();
        let mut step_indices = Vec::new();

        // Always include first step (context)
        if !episode.steps.is_empty() {
            step_indices.push(0);
        }

        // Include steps with errors (error recovery)
        for (idx, step) in episode.steps.iter().enumerate() {
            if !step.is_success() {
                step_indices.push(idx);
            }
        }

        // Include steps mentioned in salient features
        if let Some(ref features) = episode.salient_features {
            for decision in &features.critical_decisions {
                // Try to extract step number from decision text
                if let Some(step_num) = Self::extract_step_number(decision) {
                    if step_num > 0 && step_num <= episode.steps.len() {
                        step_indices.push(step_num - 1);
                    }
                }
            }
        }

        // Always include last step (outcome)
        if episode.steps.len() > 1 {
            step_indices.push(episode.steps.len() - 1);
        }

        // Deduplicate and sort
        step_indices.sort_unstable();
        step_indices.dedup();

        // If we have too many, prioritize errors and first/last
        if step_indices.len() > self.max_key_steps {
            let mut prioritized = Vec::new();

            // Keep first
            if !step_indices.is_empty() {
                prioritized.push(step_indices[0]);
            }

            // Keep errors
            for &idx in &step_indices {
                if !episode.steps[idx].is_success() && prioritized.len() < self.max_key_steps - 1 {
                    prioritized.push(idx);
                }
            }

            // Keep last
            if let Some(&last) = step_indices.last() {
                if !prioritized.contains(&last) && prioritized.len() < self.max_key_steps {
                    prioritized.push(last);
                }
            }

            // Fill remaining with middle steps
            for &idx in &step_indices {
                if !prioritized.contains(&idx) && prioritized.len() < self.max_key_steps {
                    prioritized.push(idx);
                }
            }

            prioritized.sort_unstable();
            step_indices = prioritized;
        }

        // Format selected steps
        for idx in step_indices {
            let step = &episode.steps[idx];
            let status = if step.is_success() { "" } else { " [ERROR]" };
            key_steps.push(format!(
                "Step {}: {} - {}{}",
                step.step_number, step.tool, step.action, status
            ));
        }

        key_steps
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
            let key_steps = self.extract_key_steps(episode);
            if !key_steps.is_empty() {
                let steps_desc = key_steps.join("; ");
                parts.push(format!("Key steps: {steps_desc}."));
            }
        }

        // Salient features summary
        if let Some(ref features) = episode.salient_features {
            self.add_salient_features_summary(features, &mut parts);
        }

        // Outcome
        if let Some(ref outcome) = episode.outcome {
            let outcome_text = match outcome {
                crate::types::TaskOutcome::Success { verdict, artifacts } => {
                    if artifacts.is_empty() {
                        format!("Outcome: Success - {verdict}")
                    } else {
                        format!(
                            "Outcome: Success - {verdict}. Artifacts: {}",
                            artifacts.join(", ")
                        )
                    }
                }
                crate::types::TaskOutcome::PartialSuccess {
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
                crate::types::TaskOutcome::Failure {
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

    /// Add salient features to summary parts.
    fn add_salient_features_summary(&self, features: &SalientFeatures, parts: &mut Vec<String>) {
        if !features.critical_decisions.is_empty() {
            let decision = &features.critical_decisions[0];
            parts.push(format!("Key decision: {decision}."));
        }

        if !features.error_recovery_patterns.is_empty() {
            let recovery = &features.error_recovery_patterns[0];
            parts.push(format!("Recovery pattern: {recovery}."));
        }

        if !features.key_insights.is_empty() {
            let insight = &features.key_insights[0];
            parts.push(format!("Insight: {insight}."));
        }
    }

    /// Extract step number from text like "Step 5: ..." or "step 3".
    fn extract_step_number(text: &str) -> Option<usize> {
        let text_lower = text.to_lowercase();
        if let Some(pos) = text_lower.find("step") {
            let after_step = &text_lower[pos + 4..];
            for word in after_step.split_whitespace() {
                let num_str: String = word.chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(num) = num_str.parse::<usize>() {
                    return Some(num);
                }
            }
        }
        None
    }

    /// Check if a word is a common stopword.
    fn is_stopword(word: &str) -> bool {
        matches!(
            word,
            "the"
                | "and"
                | "for"
                | "that"
                | "this"
                | "with"
                | "from"
                | "have"
                | "has"
                | "had"
                | "was"
                | "were"
                | "been"
                | "will"
                | "are"
                | "not"
                | "but"
                | "can"
                | "all"
                | "would"
                | "there"
                | "their"
        )
    }
}

impl Default for SemanticSummarizer {
    fn default() -> Self {
        Self::new()
    }
}
