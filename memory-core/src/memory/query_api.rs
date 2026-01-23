use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::storage::StorageBackend;
use crate::Result;
use std::sync::Arc;
use uuid::Uuid;

use super::queries;
use super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Check if Turso storage is configured
    #[must_use]
    pub fn has_turso_storage(&self) -> bool {
        queries::has_turso_storage(&self.turso_storage)
    }

    /// Check if cache storage is configured
    #[must_use]
    pub fn has_cache_storage(&self) -> bool {
        queries::has_cache_storage(&self.cache_storage)
    }

    /// Get a reference to Turso storage backend (if configured)
    #[must_use]
    pub fn turso_storage(&self) -> Option<&Arc<dyn StorageBackend>> {
        queries::turso_storage(&self.turso_storage)
    }

    /// Get a reference to cache storage backend (if configured)
    #[must_use]
    pub fn cache_storage(&self) -> Option<&Arc<dyn StorageBackend>> {
        queries::cache_storage(&self.cache_storage)
    }

    /// Get all episodes with proper lazy loading from storage backends.
    pub async fn get_all_episodes(&self) -> Result<Vec<Episode>> {
        queries::get_all_episodes(
            &self.episodes_fallback,
            self.cache_storage.as_ref(),
            self.turso_storage.as_ref(),
        )
        .await
    }

    /// Get all patterns with proper lazy loading from storage backends.
    pub async fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        queries::get_all_patterns(&self.patterns_fallback).await
    }

    /// List episodes with optional filtering, using proper lazy loading.
    pub async fn list_episodes(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        completed_only: Option<bool>,
    ) -> Result<Vec<Episode>> {
        queries::list_episodes(
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
        queries::list_episodes_filtered(
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
        queries::get_episode_patterns(episode_id, &self.patterns_fallback).await
    }

    /// Get multiple episodes by their IDs in a single operation.
    ///
    /// This is more efficient than calling `get_episode()` multiple times,
    /// as it can batch storage queries and reduce round trips.
    ///
    /// Uses the standard lazy-loading pattern for each episode and attempts
    /// to fetch missing episodes from storage backends in batches where possible.
    ///
    /// # Arguments
    ///
    /// * `episode_ids` - Collection of episode UUIDs to retrieve
    ///
    /// # Returns
    ///
    /// Vector of episodes that were found. Episodes that don't exist are silently
    /// omitted (no error is raised for missing episodes).
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::SelfLearningMemory;
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    ///
    /// let ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    /// let episodes = memory.get_episodes_by_ids(&ids).await?;
    ///
    /// println!("Found {} out of {} episodes", episodes.len(), ids.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_episodes_by_ids(&self, episode_ids: &[Uuid]) -> Result<Vec<Episode>> {
        queries::get_episodes_by_ids(
            episode_ids,
            &self.episodes_fallback,
            self.cache_storage.as_ref(),
            self.turso_storage.as_ref(),
        )
        .await
    }
}
