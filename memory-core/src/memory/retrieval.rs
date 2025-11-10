//! Episode and pattern retrieval

use crate::episode::Episode;
use crate::extraction::{deduplicate_patterns, rank_patterns};
use crate::pattern::Pattern;
use crate::types::TaskContext;
use tracing::{debug, info, instrument};

use super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Retrieve relevant past episodes for a new task.
    ///
    /// Searches the memory for episodes similar to the given task, enabling
    /// the system to learn from past experience. Similarity is determined by:
    /// - **Domain match**: Same problem domain
    /// - **Language/framework**: Same technology stack
    /// - **Tags**: Overlapping tags
    /// - **Description**: Common keywords in task descriptions
    ///
    /// Results are ranked by a relevance score combining context match (40%),
    /// reward quality (30%), and description similarity (30%).
    ///
    /// # Search Strategy
    ///
    /// 1. Filters to completed episodes only
    /// 2. Matches on context fields (domain, language, framework, tags)
    /// 3. Performs basic text matching on descriptions
    /// 4. Scores and ranks by relevance
    /// 5. Returns top N results
    ///
    /// # Arguments
    ///
    /// * `task_description` - Description of the new task you're about to perform
    /// * `context` - Context for the new task (same structure as when starting episodes)
    /// * `limit` - Maximum number of episodes to return
    ///
    /// # Returns
    ///
    /// Vector of episodes sorted by relevance (highest first), limited to `limit` items.
    /// Returns empty vector if no relevant episodes found.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, TaskContext, TaskType, ComplexityLevel};
    ///
    /// # async fn example() {
    /// let memory = SelfLearningMemory::new();
    ///
    /// // Query for relevant past episodes
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     framework: Some("axum".to_string()),
    ///     complexity: ComplexityLevel::Moderate,
    ///     domain: "web-api".to_string(),
    ///     tags: vec!["rest".to_string(), "authentication".to_string()],
    /// };
    ///
    /// let relevant_episodes = memory.retrieve_relevant_context(
    ///     "Implement OAuth2 authentication".to_string(),
    ///     context,
    ///     5,  // Get top 5 most relevant
    /// ).await;
    ///
    /// // Use retrieved episodes to inform approach
    /// for episode in relevant_episodes {
    ///     println!("Similar task: {}", episode.task_description);
    ///     println!("Reward: {:?}", episode.reward);
    ///
    ///     if let Some(reflection) = episode.reflection {
    ///         println!("Key insights:");
    ///         for insight in reflection.insights {
    ///             println!("  - {}", insight);
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`retrieve_relevant_patterns()`](SelfLearningMemory::retrieve_relevant_patterns) - Get patterns instead of full episodes
    #[instrument(skip(self))]
    pub async fn retrieve_relevant_context(
        &self,
        task_description: String,
        context: TaskContext,
        limit: usize,
    ) -> Vec<Episode> {
        let episodes = self.episodes_fallback.read().await;

        debug!(
            total_episodes = episodes.len(),
            limit = limit,
            "Retrieving relevant context"
        );

        // Find relevant episodes
        let mut relevant: Vec<Episode> = episodes
            .values()
            .filter(|e| e.is_complete())
            .filter(|e| self.is_relevant_episode(e, &context, &task_description))
            .cloned()
            .collect();

        // Sort by relevance (using reward as proxy for quality)
        relevant.sort_by(|a, b| {
            let a_score = self.calculate_relevance_score(a, &context, &task_description);
            let b_score = self.calculate_relevance_score(b, &context, &task_description);

            b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        relevant.truncate(limit);

        info!(
            retrieved_count = relevant.len(),
            "Retrieved relevant episodes"
        );

        relevant
    }

    /// Retrieve relevant patterns for a task context
    ///
    /// Finds patterns that match the given context and ranks them
    /// by relevance and success rate.
    ///
    /// # Arguments
    ///
    /// * `context` - Task context to match against
    /// * `limit` - Maximum number of patterns to return
    ///
    /// # Returns
    ///
    /// Vector of relevant patterns, sorted by relevance and quality
    #[instrument(skip(self))]
    pub async fn retrieve_relevant_patterns(
        &self,
        context: &TaskContext,
        limit: usize,
    ) -> Vec<Pattern> {
        let patterns = self.patterns_fallback.read().await;

        debug!(
            total_patterns = patterns.len(),
            limit = limit,
            "Retrieving relevant patterns"
        );

        let all_patterns: Vec<Pattern> = patterns.values().cloned().collect();

        // Rank patterns by relevance and quality
        let mut ranked = rank_patterns(all_patterns, context);

        // Deduplicate
        ranked = deduplicate_patterns(ranked);

        // Limit results
        ranked.truncate(limit);

        info!(
            retrieved_count = ranked.len(),
            "Retrieved relevant patterns"
        );

        ranked
    }

    /// Check if episode is relevant to the query
    fn is_relevant_episode(
        &self,
        episode: &Episode,
        context: &TaskContext,
        task_description: &str,
    ) -> bool {
        // Match on domain
        if episode.context.domain == context.domain {
            return true;
        }

        // Match on language
        if episode.context.language == context.language && episode.context.language.is_some() {
            return true;
        }

        // Match on framework
        if episode.context.framework == context.framework && episode.context.framework.is_some() {
            return true;
        }

        // Match on tags
        let common_tags: Vec<_> = episode
            .context
            .tags
            .iter()
            .filter(|t| context.tags.contains(t))
            .collect();

        if !common_tags.is_empty() {
            return true;
        }

        // Simple text matching on description (very basic)
        let desc_lower = task_description.to_lowercase();
        let episode_desc_lower = episode.task_description.to_lowercase();

        let common_words: Vec<_> = desc_lower
            .split_whitespace()
            .filter(|w| w.len() > 3) // Ignore short words
            .filter(|w| episode_desc_lower.contains(w))
            .collect();

        !common_words.is_empty()
    }

    /// Calculate relevance score for an episode
    fn calculate_relevance_score(
        &self,
        episode: &Episode,
        context: &TaskContext,
        task_description: &str,
    ) -> f32 {
        let mut score = 0.0;

        // Reward quality (30% weight)
        if let Some(reward) = &episode.reward {
            score += reward.total * 0.3;
        }

        // Context match (40% weight)
        let mut context_score = 0.0;

        if episode.context.domain == context.domain {
            context_score += 0.4;
        }

        if episode.context.language == context.language && episode.context.language.is_some() {
            context_score += 0.3;
        }

        if episode.context.framework == context.framework && episode.context.framework.is_some() {
            context_score += 0.2;
        }

        let common_tags: Vec<_> = episode
            .context
            .tags
            .iter()
            .filter(|t| context.tags.contains(t))
            .collect();

        if !common_tags.is_empty() {
            context_score += 0.1 * common_tags.len() as f32;
        }

        score += context_score.min(0.4);

        // Description similarity (30% weight)
        let desc_lower = task_description.to_lowercase();
        let episode_desc_lower = episode.task_description.to_lowercase();

        let desc_words: Vec<_> = desc_lower.split_whitespace().collect();
        let common_words: Vec<_> = desc_words
            .iter()
            .filter(|w| w.len() > 3)
            .filter(|w| episode_desc_lower.contains(**w))
            .collect();

        if !desc_words.is_empty() {
            let similarity = common_words.len() as f32 / desc_words.len() as f32;
            score += similarity * 0.3;
        }

        score
    }
}
