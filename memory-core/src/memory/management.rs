//! Episode management operations
//!
//! This module provides episode lifecycle management operations including
//! deletion and archival functionality.

use crate::error::{Error, Result};
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Delete an episode permanently from all storage backends.
    ///
    /// This operation removes an episode from:
    /// - In-memory cache
    /// - Cache storage (redb)
    /// - Durable storage (Turso)
    ///
    /// **Warning**: This operation cannot be undone. The episode and all associated
    /// data will be permanently deleted.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - UUID of the episode to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` if deletion succeeds, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the episode doesn't exist in any storage backend.
    /// Returns `Error::Storage` if deletion fails in any backend.
    ///
    /// # Examples
    ///
    /// ```
    /// use do_memory_core::{SelfLearningMemory, TaskContext, TaskType};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    ///
    /// // Create an episode
    /// let episode_id = memory.start_episode(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// ).await;
    ///
    /// // Delete it
    /// memory.delete_episode(episode_id).await?;
    ///
    /// // Verify it's gone
    /// assert!(memory.get_episode(episode_id).await.is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_episode(&self, episode_id: Uuid) -> Result<()> {
        debug!("Deleting episode: {}", episode_id);

        // Check if episode exists first
        let exists = {
            let episodes = self.episodes_fallback.read().await;
            episodes.contains_key(&episode_id)
        };

        if !exists {
            // Try to load from storage to verify it doesn't exist
            if self.get_episode(episode_id).await.is_err() {
                return Err(Error::NotFound(episode_id));
            }
        }

        // Delete from in-memory cache first
        {
            let mut episodes = self.episodes_fallback.write().await;
            episodes.remove(&episode_id);
        }

        // Delete from step buffers if present
        {
            let mut buffers = self.step_buffers.write().await;
            buffers.remove(&episode_id);
        }

        // Delete from cache storage (redb)
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.delete_episode(episode_id).await {
                warn!("Failed to delete episode from cache storage: {}", e);
                // Continue with deletion from other backends
            }
        }

        // Delete from durable storage (Turso)
        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.delete_episode(episode_id).await {
                warn!("Failed to delete episode from durable storage: {}", e);
                return Err(e);
            }
        }

        info!("Successfully deleted episode: {}", episode_id);
        Ok(())
    }

    /// Delete a pattern permanently from every storage backend.
    ///
    /// Removes the pattern from the durable backend, the cache layer (when it is
    /// a distinct backend), and the in-memory fallback, then invalidates the
    /// derived ranking weights so no learned boost outlives the record.
    ///
    /// Historical episodes that still list the pattern ID in
    /// [`Episode::patterns`](crate::Episode) are left untouched; lookups such as
    /// [`Self::get_episode_patterns`] omit dangling IDs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Storage`] if a backend fails to delete. Deletion is
    /// idempotent, so the caller can retry a failed attempt.
    pub async fn delete_pattern(&self, pattern_id: crate::episode::PatternId) -> Result<()> {
        debug!("Deleting pattern: {}", pattern_id);

        if let Some(turso) = &self.turso_storage {
            turso.delete_pattern(pattern_id).await?;
        }

        if let Some(cache) = &self.cache_storage {
            let same_backend = self
                .turso_storage
                .as_ref()
                .is_some_and(|turso| Arc::ptr_eq(turso, cache));
            if !same_backend {
                cache.delete_pattern(pattern_id).await?;
            }
        }

        {
            let mut patterns = self.patterns_fallback.write().await;
            patterns.remove(&pattern_id);
        }

        self.ranking_index
            .write()
            .await
            .replace_with(super::attribution::RankingIndex::default());
        self.ranking_loaded
            .store(false, std::sync::atomic::Ordering::Release);

        info!("Successfully deleted pattern: {}", pattern_id);
        Ok(())
    }

    /// Archive an episode by marking it as archived.
    ///
    /// This sets the `archived_at` timestamp in both the episode's metadata
    /// and the database, allowing episodes to be preserved rather than deleted.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - UUID of the episode to archive
    ///
    /// # Returns
    ///
    /// `Ok(())` if archival succeeds, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the episode doesn't exist.
    /// Returns `Error::Storage` if the update fails in any storage backend.
    pub async fn archive_episode(&self, episode_id: Uuid) -> Result<()> {
        debug!("Archiving episode: {}", episode_id);

        // Get the episode first
        let episode = self.get_episode(episode_id).await?;

        // Set archived_at timestamp in metadata
        let archived_timestamp = Utc::now().timestamp();
        let mut episode = episode;
        episode
            .metadata
            .insert("archived_at".to_string(), archived_timestamp.to_string());

        // Update in all storage backends
        // In-memory cache - re-insert with updated episode (can't mutate through Arc)
        {
            let mut episodes = self.episodes_fallback.write().await;
            episodes.insert(episode_id, Arc::new(episode.clone()));
        }

        // Cache storage (redb)
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(&episode).await {
                warn!("Failed to update episode in cache storage: {}", e);
            }
        }

        // Durable storage (Turso)
        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(&episode).await {
                warn!("Failed to update episode in durable storage: {}", e);
                return Err(e);
            }
        }

        info!("Successfully archived episode: {}", episode_id);
        Ok(())
    }

    /// Restore an archived episode by clearing the archived status.
    ///
    /// This removes the `archived_at` timestamp, making the episode
    /// active again for retrieval and analysis.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - UUID of the episode to restore
    ///
    /// # Returns
    ///
    /// `Ok(())` if restoration succeeds, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the episode doesn't exist.
    /// Returns `Error::Storage` if the episode is not archived or update fails.
    pub async fn restore_episode(&self, episode_id: Uuid) -> Result<()> {
        debug!("Restoring episode: {}", episode_id);

        // Get the episode first
        let episode = self.get_episode(episode_id).await?;

        // Verify it's archived
        if !episode.metadata.contains_key("archived_at") {
            return Err(Error::Storage("Episode is not archived".to_string()));
        }

        // Remove archived_at from metadata
        let mut episode = episode;
        episode.metadata.remove("archived_at");

        // Update in all storage backends
        // In-memory cache - re-insert with updated episode (can't mutate through Arc)
        {
            let mut episodes = self.episodes_fallback.write().await;
            episodes.insert(episode_id, Arc::new(episode.clone()));
        }

        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(&episode).await {
                warn!("Failed to update episode in cache storage: {}", e);
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(&episode).await {
                warn!("Failed to update episode in durable storage: {}", e);
                return Err(e);
            }
        }

        info!("Successfully restored episode: {}", episode_id);
        Ok(())
    }

    /// Update episode fields
    ///
    /// Updates specified fields of an episode. Only provided fields are updated.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - Episode to update
    /// * `description` - Optional new task description
    /// * `metadata` - Optional metadata to merge with existing metadata
    ///
    /// # Returns
    ///
    /// `Ok(())` if update succeeds
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if episode doesn't exist
    /// Returns `Error::Storage` if update fails
    pub async fn update_episode(
        &self,
        episode_id: Uuid,
        description: Option<String>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<()> {
        debug!("Updating episode: {}", episode_id);

        // Get the episode
        let mut episode = self.get_episode(episode_id).await?;

        // Track if any changes were made
        let mut changes_made = false;

        // Update description if provided
        if let Some(new_description) = description {
            if episode.task_description != new_description {
                episode.task_description = new_description;
                changes_made = true;
                debug!("Updated task description for episode: {}", episode_id);
            }
        }

        // Merge metadata if provided
        if let Some(new_metadata) = metadata {
            if !new_metadata.is_empty() {
                episode.metadata.extend(new_metadata);
                changes_made = true;
                debug!("Updated metadata for episode: {}", episode_id);
            }
        }

        if !changes_made {
            debug!("No changes to apply for episode: {}", episode_id);
            return Ok(());
        }

        // Update in all storage backends
        self.update_episode_in_storage(&episode).await?;

        info!("Successfully updated episode: {}", episode_id);
        Ok(())
    }

    /// Helper to update episode in all storage backends
    pub(super) async fn update_episode_in_storage(&self, episode: &crate::Episode) -> Result<()> {
        let episode_id = episode.episode_id;

        // Update in-memory cache
        {
            let mut episodes = self.episodes_fallback.write().await;
            episodes.insert(episode_id, Arc::new(episode.clone()));
        }

        // Update cache storage
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(episode).await {
                warn!("Failed to update episode in cache storage: {}", e);
            }
        }

        // Update durable storage
        if let Some(turso) = &self.turso_storage {
            turso.store_episode(episode).await?;
        }

        Ok(())
    }

    /// Update an episode directly with the episode struct.
    ///
    /// This is used internally for operations like adding checkpoints.
    /// Updates all storage backends.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to update (will be stored with its episode_id)
    ///
    /// # Returns
    ///
    /// `Ok(())` if update succeeds
    ///
    /// # Errors
    ///
    /// Returns `Error::Storage` if update fails
    pub async fn update_episode_full(&self, episode: &crate::Episode) -> Result<()> {
        self.update_episode_in_storage(episode).await
    }

    /// Get all heuristics from memory.
    ///
    /// Returns all stored heuristics for use in handoff packs and recommendations.
    ///
    /// # Returns
    ///
    /// Vector of all heuristics
    pub async fn get_all_heuristics(&self) -> Result<Vec<crate::patterns::Heuristic>> {
        let heuristics = self.heuristics_fallback.read().await;
        Ok(heuristics.values().cloned().collect())
    }

    /// Search for patterns using multi-signal ranking.
    ///
    /// This is a simplified pattern search interface for checkpoint operations.
    ///
    /// # Arguments
    ///
    /// * `query` - Natural language query
    /// * `context` - Task context for filtering
    /// * `config` - Search configuration
    ///
    /// # Returns
    ///
    /// Vector of pattern search results ranked by relevance
    pub async fn search_patterns(
        &self,
        query: &str,
        context: &crate::types::TaskContext,
        config: super::pattern_search::SearchConfig,
    ) -> Result<Vec<super::pattern_search::PatternSearchResult>> {
        // Snapshot the live provider before the embedding awaits so the
        // management search path uses the runtime-activated provider rather
        // than the construction-time field (issue #1074).
        let semantic_service = self.live_semantic_service().await;
        let patterns = self.get_all_patterns().await?;
        super::pattern_search::search_patterns_semantic(
            query,
            patterns,
            context,
            semantic_service.as_ref(),
            config,
            10, // default limit
        )
        .await
    }
}
