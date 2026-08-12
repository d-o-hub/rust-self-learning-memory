use crate::Result;
use crate::memory::attribution::{AttributedPatternResult, RecommendationSession};
use crate::types::TaskContext;

use super::pattern_search;

use super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Search for patterns semantically similar to a query
    ///
    /// Uses multi-signal ranking combining:
    /// - Semantic similarity via embeddings
    /// - Context matching (domain, task type, tags)
    /// - Pattern effectiveness from past usage
    /// - Recency (recently used patterns score higher)
    /// - Success rate
    ///
    /// # Arguments
    /// * `query` - Natural language description of what you're looking for
    /// * `context` - Task context for filtering and scoring
    /// * `limit` - Maximum number of results to return
    ///
    /// # Example
    /// ```no_run
    /// use do_memory_core::{SelfLearningMemory, TaskContext};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     domain: "web-api".to_string(),
    ///     tags: vec!["rest".to_string()],
    ///     ..Default::default()
    /// };
    ///
    /// let results = memory.search_patterns_semantic(
    ///     "How to handle API rate limiting with retries",
    ///     context,
    ///     5,
    /// ).await?;
    ///
    /// for result in results {
    ///     println!("Pattern: {:?}", result.pattern);
    ///     println!("Relevance: {:.2}", result.relevance_score);
    ///     println!("Breakdown: {:?}", result.score_breakdown);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_patterns_semantic(
        &self,
        query: &str,
        context: TaskContext,
        limit: usize,
    ) -> Result<Vec<pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        pattern_search::search_patterns_semantic(
            query,
            patterns,
            &context,
            self.semantic_service.as_ref(),
            pattern_search::SearchConfig::default(),
            limit,
        )
        .await
    }

    /// Search patterns with custom configuration
    pub async fn search_patterns_with_config(
        &self,
        query: &str,
        context: TaskContext,
        config: pattern_search::SearchConfig,
        limit: usize,
    ) -> Result<Vec<pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        pattern_search::search_patterns_semantic(
            query,
            patterns,
            &context,
            self.semantic_service.as_ref(),
            config,
            limit,
        )
        .await
    }

    /// Recommend patterns for a specific task
    ///
    /// Similar to search but uses stricter filtering and emphasizes
    /// effectiveness and context match for high-quality recommendations.
    ///
    /// # Arguments
    /// * `task_description` - Description of task you're working on
    /// * `context` - Task context
    /// * `limit` - Maximum number of recommendations
    ///
    /// # Example
    /// ```no_run
    /// use do_memory_core::{SelfLearningMemory, TaskContext};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     domain: "web-api".to_string(),
    ///     tags: vec!["async".to_string()],
    ///     ..Default::default()
    /// };
    ///
    /// let recommendations = memory.recommend_patterns_for_task(
    ///     "Build an async HTTP client with connection pooling",
    ///     context,
    ///     3,
    /// ).await?;
    ///
    /// for rec in recommendations {
    ///     println!("Recommended: {:?}", rec.pattern);
    ///     println!("Relevance: {:.2}", rec.relevance_score);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recommend_patterns_for_task(
        &self,
        task_description: &str,
        context: TaskContext,
        limit: usize,
    ) -> Result<Vec<pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        pattern_search::recommend_patterns_for_task(
            task_description,
            context,
            patterns,
            self.semantic_service.as_ref(),
            limit,
        )
        .await
    }

    /// Recommend patterns for a task and create an attributed recommendation session (ADR-080 §1–3).
    ///
    /// Requires a valid, non-nil `episode_id`. Returns recommendations together with
    /// a session and a `PersistenceReceipt` describing durability state.
    pub async fn recommend_patterns_attributed(
        &self,
        episode_id: uuid::Uuid,
        task_description: &str,
        context: TaskContext,
        limit: usize,
    ) -> Result<AttributedPatternResult<pattern_search::PatternSearchResult>> {
        if episode_id.is_nil() {
            return Err(crate::error::Error::InvalidInput(
                "Attributed pattern recommendations require a non-nil episode ID".to_string(),
            ));
        }

        // ADR-080 §1: a nonexistent episode must never create a session, so the
        // episode is validated before any recommendation generation.
        self.validate_attributed_episode(episode_id, "recommend_patterns_attributed")
            .await?;

        let recommendations = self
            .recommend_patterns_for_task(task_description, context, limit)
            .await?;

        let recommended_pattern_ids = recommendations
            .iter()
            .map(|r| r.pattern.id().to_string())
            .collect();

        let session = RecommendationSession {
            session_id: uuid::Uuid::new_v4(),
            episode_id,
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids,
            recommended_playbook_ids: vec![],
        };

        let receipt = self
            .record_recommendation_session_checked(session.clone())
            .await;

        Ok(AttributedPatternResult {
            recommendations,
            session,
            receipt,
        })
    }

    /// Discover patterns from one domain that might apply to another
    ///
    /// Finds analogous patterns across domains, useful for:
    /// - Learning from similar problems in different contexts
    /// - Cross-pollinating ideas between domains
    /// - Finding universal patterns
    ///
    /// # Arguments
    /// * `source_domain` - Domain to learn from (e.g., "cli")
    /// * `target_context` - Context to apply patterns to (e.g., "web-api")
    /// * `limit` - Maximum number of patterns to discover
    ///
    /// # Example
    /// ```no_run
    /// use do_memory_core::{SelfLearningMemory, TaskContext};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    /// let target = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     domain: "web-api".to_string(),
    ///     tags: vec![],
    ///     ..Default::default()
    /// };
    ///
    /// // Find patterns from CLI work that might apply to web APIs
    /// let analogous = memory.discover_analogous_patterns(
    ///     "cli",
    ///     target,
    ///     5,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_analogous_patterns(
        &self,
        source_domain: &str,
        target_context: TaskContext,
        limit: usize,
    ) -> Result<Vec<pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        pattern_search::discover_analogous_patterns(
            source_domain,
            target_context,
            patterns,
            self.semantic_service.as_ref(),
            limit,
        )
        .await
    }
}
