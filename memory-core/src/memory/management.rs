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
    /// use memory_core::{SelfLearningMemory, TaskContext, TaskType};
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

    /// Add tags to an episode
    ///
    /// Tags are normalized (lowercase, trimmed) and validated.
    /// Duplicates are ignored. Tags persist across all storage backends.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - Episode to tag
    /// * `tags` - Tags to add (will be normalized)
    ///
    /// # Errors
    ///
    /// Returns error if episode doesn't exist or tag validation fails
    pub async fn add_episode_tags(&self, episode_id: Uuid, tags: Vec<String>) -> Result<()> {
        debug!("Adding {} tags to episode: {}", tags.len(), episode_id);

        // Get the episode
        let mut episode = self.get_episode(episode_id).await?;

        // Add tags using Episode's validation logic
        let mut added_count = 0;
        for tag in tags {
            match episode.add_tag(tag.clone()) {
                Ok(true) => added_count += 1,
                Ok(false) => debug!("Tag '{}' already exists on episode", tag),
                Err(e) => {
                    warn!("Failed to add tag '{}': {}", tag, e);
                    return Err(Error::Storage(e));
                }
            }
        }

        if added_count == 0 {
            debug!("No new tags added to episode");
            return Ok(());
        }

        // Update in all storage backends
        self.update_episode_in_storage(&episode).await?;

        info!("Added {} tags to episode: {}", added_count, episode_id);
        Ok(())
    }

    /// Remove tags from an episode
    ///
    /// Silently ignores tags that don't exist.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - Episode to modify
    /// * `tags` - Tags to remove
    pub async fn remove_episode_tags(&self, episode_id: Uuid, tags: Vec<String>) -> Result<()> {
        debug!("Removing {} tags from episode: {}", tags.len(), episode_id);

        // Get the episode
        let mut episode = self.get_episode(episode_id).await?;

        // Remove tags
        let mut removed_count = 0;
        for tag in tags {
            if episode.remove_tag(&tag) {
                removed_count += 1;
            }
        }

        if removed_count == 0 {
            debug!("No tags removed from episode");
            return Ok(());
        }

        // Update in all storage backends
        self.update_episode_in_storage(&episode).await?;

        info!(
            "Removed {} tags from episode: {}",
            removed_count, episode_id
        );
        Ok(())
    }

    /// Set episode tags (replace all existing tags)
    ///
    /// Replaces all existing tags with the provided set.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - Episode to modify
    /// * `tags` - New tags (will replace all existing)
    ///
    /// # Errors
    ///
    /// Returns error if episode doesn't exist or tag validation fails
    pub async fn set_episode_tags(&self, episode_id: Uuid, tags: Vec<String>) -> Result<()> {
        debug!("Setting {} tags on episode: {}", tags.len(), episode_id);

        // Get the episode
        let mut episode = self.get_episode(episode_id).await?;

        // Clear existing tags
        episode.clear_tags();

        // Add new tags
        for tag in tags {
            match episode.add_tag(tag.clone()) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to add tag '{}': {}", tag, e);
                    return Err(Error::Storage(e));
                }
            }
        }

        // Update in all storage backends
        self.update_episode_in_storage(&episode).await?;

        info!("Set {} tags on episode: {}", episode.tags.len(), episode_id);
        Ok(())
    }

    /// Get tags for an episode
    ///
    /// # Arguments
    ///
    /// * `episode_id` - Episode to query
    ///
    /// # Returns
    ///
    /// Vector of tags (normalized)
    pub async fn get_episode_tags(&self, episode_id: Uuid) -> Result<Vec<String>> {
        let episode = self.get_episode(episode_id).await?;
        Ok(episode.tags.clone())
    }

    /// Helper to update episode in all storage backends
    async fn update_episode_in_storage(&self, episode: &crate::Episode) -> Result<()> {
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
}
