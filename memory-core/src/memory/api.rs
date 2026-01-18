//! Public API methods for SelfLearningMemory
//!
//! This module contains all public methods that users interact with,
//! organized by functionality area.

use crate::embeddings::{EmbeddingConfig, SemanticService};
use crate::episode::{Episode, PatternId};
use crate::error::Result;
use crate::learning::queue::QueueConfig;
use crate::monitoring::AgentMetrics;
use crate::pattern::Pattern;
use crate::storage::StorageBackend;
use crate::types::{MemoryConfig, TaskContext};
use std::sync::Arc;
use uuid::Uuid;

use super::SelfLearningMemory;

// ============================================================================
// Monitoring and Statistics
// ============================================================================

impl SelfLearningMemory {
    /// Get statistics about the memory system
    pub async fn get_stats(&self) -> (usize, usize, usize) {
        super::monitoring::get_stats(&self.episodes_fallback, &self.patterns_fallback).await
    }

    /// Record an agent execution for monitoring
    pub async fn record_agent_execution(
        &self,
        agent_name: &str,
        success: bool,
        duration: std::time::Duration,
    ) -> Result<()> {
        super::monitoring::record_agent_execution(
            &self.agent_monitor,
            agent_name,
            success,
            duration,
        )
        .await
    }

    /// Record detailed agent execution information
    pub async fn record_agent_execution_detailed(
        &self,
        agent_name: &str,
        success: bool,
        duration: std::time::Duration,
        task_description: Option<String>,
        error_message: Option<String>,
    ) -> Result<()> {
        super::monitoring::record_agent_execution_detailed(
            &self.agent_monitor,
            agent_name,
            success,
            duration,
            task_description,
            error_message,
        )
        .await
    }

    /// Get performance metrics for a specific agent
    pub async fn get_agent_metrics(&self, agent_name: &str) -> Option<AgentMetrics> {
        super::monitoring::get_agent_metrics(&self.agent_monitor, agent_name).await
    }

    /// Get metrics for all tracked agents
    pub async fn get_all_agent_metrics(&self) -> std::collections::HashMap<String, AgentMetrics> {
        super::monitoring::get_all_agent_metrics(&self.agent_monitor).await
    }

    /// Get monitoring system summary statistics
    pub async fn get_monitoring_summary(&self) -> crate::monitoring::MonitoringSummary {
        super::monitoring::get_monitoring_summary(&self.agent_monitor).await
    }

    /// Get query cache metrics (v0.1.12)
    #[must_use]
    pub fn get_cache_metrics(&self) -> crate::retrieval::CacheMetrics {
        super::monitoring::get_cache_metrics(&self.query_cache)
    }

    /// Clear query cache metrics (v0.1.12)
    pub fn clear_cache_metrics(&self) {
        super::monitoring::clear_cache_metrics(&self.query_cache);
    }

    /// Clear all cached query results (v0.1.12)
    pub fn clear_cache(&self) {
        super::monitoring::clear_cache(&self.query_cache);
    }
}

// ============================================================================
// Storage Backend Accessors
// ============================================================================

impl SelfLearningMemory {
    /// Check if Turso storage is configured
    #[must_use]
    pub fn has_turso_storage(&self) -> bool {
        super::queries::has_turso_storage(&self.turso_storage)
    }

    /// Check if cache storage is configured
    #[must_use]
    pub fn has_cache_storage(&self) -> bool {
        super::queries::has_cache_storage(&self.cache_storage)
    }

    /// Get a reference to the Turso storage backend (if configured)
    #[must_use]
    pub fn turso_storage(&self) -> Option<&Arc<dyn StorageBackend>> {
        super::queries::turso_storage(&self.turso_storage)
    }

    /// Get a reference to the cache storage backend (if configured)
    #[must_use]
    pub fn cache_storage(&self) -> Option<&Arc<dyn StorageBackend>> {
        super::queries::cache_storage(&self.cache_storage)
    }

    /// Get a reference to the semantic service (if configured)
    #[must_use]
    pub fn semantic_service(&self) -> Option<&Arc<SemanticService>> {
        self.semantic_service.as_ref()
    }
}

// ============================================================================
// Episode Queries and Retrieval
// ============================================================================

impl SelfLearningMemory {
    /// Get all episodes with proper lazy loading from storage backends.
    pub async fn get_all_episodes(&self) -> Result<Vec<Episode>> {
        super::queries::get_all_episodes(
            &self.episodes_fallback,
            self.cache_storage.as_ref(),
            self.turso_storage.as_ref(),
        )
        .await
    }

    /// Get all patterns with proper lazy loading from storage backends.
    pub async fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        super::queries::get_all_patterns(&self.patterns_fallback).await
    }

    /// List episodes with optional filtering, using proper lazy loading.
    pub async fn list_episodes(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        completed_only: Option<bool>,
    ) -> Result<Vec<Episode>> {
        super::queries::list_episodes(
            &self.episodes_fallback,
            self.cache_storage.as_ref(),
            self.turso_storage.as_ref(),
            limit,
            offset,
            completed_only,
        )
        .await
    }

    /// List episodes with advanced filtering support.
    ///
    /// Provides rich filtering capabilities including tags, date ranges,
    /// task types, outcomes, and more. Use `EpisodeFilter::builder()` for a fluent API.
    ///
    /// # Arguments
    ///
    /// * `filter` - Episode filter criteria
    /// * `limit` - Maximum number of episodes to return
    /// * `offset` - Number of episodes to skip (for pagination)
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, EpisodeFilter, TaskType};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    ///
    /// // Get successful episodes with specific tags
    /// let filter = EpisodeFilter::builder()
    ///     .with_any_tags(vec!["async".to_string()])
    ///     .success_only(true)
    ///     .build();
    ///
    /// let episodes = memory.list_episodes_filtered(filter, Some(10), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_episodes_filtered(
        &self,
        filter: super::filters::EpisodeFilter,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Episode>> {
        super::queries::list_episodes_filtered(
            &self.episodes_fallback,
            self.cache_storage.as_ref(),
            self.turso_storage.as_ref(),
            filter,
            limit,
            offset,
        )
        .await
    }

    /// Get patterns extracted from a specific episode
    #[allow(clippy::unused_async)]
    pub async fn get_episode_patterns(&self, episode_id: Uuid) -> Result<Vec<Pattern>> {
        super::queries::get_episode_patterns(episode_id, &self.patterns_fallback).await
    }
}

// ============================================================================
// Pattern Search and Discovery
// ============================================================================

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
    /// use memory_core::{SelfLearningMemory, TaskContext};
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
    ) -> Result<Vec<super::pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        super::pattern_search::search_patterns_semantic(
            query,
            patterns,
            &context,
            self.semantic_service.as_ref(),
            super::pattern_search::SearchConfig::default(),
            limit,
        )
        .await
    }

    /// Search patterns with custom configuration
    pub async fn search_patterns_with_config(
        &self,
        query: &str,
        context: TaskContext,
        config: super::pattern_search::SearchConfig,
        limit: usize,
    ) -> Result<Vec<super::pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        super::pattern_search::search_patterns_semantic(
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
    /// * `task_description` - Description of the task you're working on
    /// * `context` - Task context
    /// * `limit` - Maximum number of recommendations
    ///
    /// # Example
    /// ```no_run
    /// use memory_core::{SelfLearningMemory, TaskContext};
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
    ) -> Result<Vec<super::pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        super::pattern_search::recommend_patterns_for_task(
            task_description,
            context,
            patterns,
            self.semantic_service.as_ref(),
            limit,
        )
        .await
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
    /// use memory_core::{SelfLearningMemory, TaskContext};
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
    ) -> Result<Vec<super::pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        super::pattern_search::discover_analogous_patterns(
            source_domain,
            target_context,
            patterns,
            self.semantic_service.as_ref(),
            limit,
        )
        .await
    }
}
