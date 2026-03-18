//! Episode tag management operations
//!
//! This module provides tag CRUD operations for episodes, including
//! adding, removing, setting, and querying tags.

use crate::error::{Error, Result};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::SelfLearningMemory;

impl SelfLearningMemory {
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
}
