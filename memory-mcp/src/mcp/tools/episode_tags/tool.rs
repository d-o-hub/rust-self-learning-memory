//! Episode tagging tool implementations.

use crate::mcp::tools::episode_tags::types::{
    AddEpisodeTagsInput, AddEpisodeTagsOutput, EpisodeTagResult, GetEpisodeTagsInput,
    GetEpisodeTagsOutput, RemoveEpisodeTagsInput, RemoveEpisodeTagsOutput,
    SearchEpisodesByTagsInput, SearchEpisodesByTagsOutput, SetEpisodeTagsInput,
    SetEpisodeTagsOutput,
};
use anyhow::{Result, anyhow};
use memory_core::SelfLearningMemory;
use std::sync::Arc;
use tracing::{debug, info, instrument};
use uuid::Uuid;

/// Episode tagging tools
#[derive(Clone)]
pub struct EpisodeTagTools {
    memory: Arc<SelfLearningMemory>,
}

impl EpisodeTagTools {
    /// Create a new episode tag tools instance
    pub fn new(memory: Arc<SelfLearningMemory>) -> Self {
        Self { memory }
    }

    /// Add tags to an episode
    ///
    /// Adds one or more tags to the specified episode. Tags are validated and normalized
    /// (lowercase, trimmed). Duplicate tags are ignored.
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing episode ID and tags to add
    ///
    /// # Returns
    ///
    /// Returns the number of tags added and the current tag list.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Episode ID is invalid (not a UUID)
    /// - Episode does not exist
    /// - Tag validation fails (invalid characters, length)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_mcp::mcp::tools::episode_tags::{EpisodeTagTools, AddEpisodeTagsInput};
    /// # use memory_core::SelfLearningMemory;
    /// # use std::sync::Arc;
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = Arc::new(SelfLearningMemory::new());
    /// let tools = EpisodeTagTools::new(memory);
    ///
    /// let input = AddEpisodeTagsInput {
    ///     episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    ///     tags: vec!["bug-fix".to_string(), "critical".to_string()],
    /// };
    ///
    /// let output = tools.add_tags(input).await?;
    /// println!("Added {} tags", output.tags_added);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id, tag_count = input.tags.len()))]
    pub async fn add_tags(&self, input: AddEpisodeTagsInput) -> Result<AddEpisodeTagsOutput> {
        info!(
            "Adding {} tags to episode: {}",
            input.tags.len(),
            input.episode_id
        );

        // Parse episode ID
        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode ID: {}", e))?;

        // Validate tags
        if input.tags.is_empty() {
            return Ok(AddEpisodeTagsOutput {
                success: false,
                episode_id: input.episode_id.clone(),
                tags_added: 0,
                current_tags: vec![],
                message: "No tags provided".to_string(),
            });
        }

        // Get current tags before adding
        let tags_before = self.memory.get_episode_tags(episode_id).await?;
        let before_count = tags_before.len();

        // Add tags using the memory API
        self.memory
            .add_episode_tags(episode_id, input.tags.clone())
            .await?;

        // Get updated tags
        let current_tags = self.memory.get_episode_tags(episode_id).await?;
        let tags_added = current_tags.len() - before_count;

        info!(
            "Successfully added {} tags to episode: {}",
            tags_added, episode_id
        );

        Ok(AddEpisodeTagsOutput {
            success: true,
            episode_id: input.episode_id,
            tags_added,
            current_tags,
            message: format!("Added {} tag(s) to episode", tags_added),
        })
    }

    /// Remove tags from an episode
    ///
    /// Removes one or more tags from the specified episode. Non-existent tags are
    /// silently ignored. Matching is case-insensitive.
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing episode ID and tags to remove
    ///
    /// # Returns
    ///
    /// Returns the number of tags removed and the current tag list.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Episode ID is invalid (not a UUID)
    /// - Episode does not exist
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id, tag_count = input.tags.len()))]
    pub async fn remove_tags(
        &self,
        input: RemoveEpisodeTagsInput,
    ) -> Result<RemoveEpisodeTagsOutput> {
        info!(
            "Removing {} tags from episode: {}",
            input.tags.len(),
            input.episode_id
        );

        // Parse episode ID
        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode ID: {}", e))?;

        // Validate tags
        if input.tags.is_empty() {
            return Ok(RemoveEpisodeTagsOutput {
                success: false,
                episode_id: input.episode_id.clone(),
                tags_removed: 0,
                current_tags: vec![],
                message: "No tags provided".to_string(),
            });
        }

        // Get current tags before removing
        let tags_before = self.memory.get_episode_tags(episode_id).await?;
        let before_count = tags_before.len();

        // Remove tags using the memory API
        self.memory
            .remove_episode_tags(episode_id, input.tags.clone())
            .await?;

        // Get updated tags
        let current_tags = self.memory.get_episode_tags(episode_id).await?;
        let tags_removed = before_count - current_tags.len();

        info!(
            "Successfully removed {} tags from episode: {}",
            tags_removed, episode_id
        );

        Ok(RemoveEpisodeTagsOutput {
            success: true,
            episode_id: input.episode_id,
            tags_removed,
            current_tags,
            message: format!("Removed {} tag(s) from episode", tags_removed),
        })
    }

    /// Set tags on an episode (replace all existing tags)
    ///
    /// Replaces all existing tags with the provided set. Useful for complete
    /// tag reorganization. Empty tag list will clear all tags.
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing episode ID and new tags
    ///
    /// # Returns
    ///
    /// Returns the number of tags set and the current tag list.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Episode ID is invalid (not a UUID)
    /// - Episode does not exist
    /// - Tag validation fails
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id, tag_count = input.tags.len()))]
    pub async fn set_tags(&self, input: SetEpisodeTagsInput) -> Result<SetEpisodeTagsOutput> {
        info!(
            "Setting {} tags on episode: {}",
            input.tags.len(),
            input.episode_id
        );

        // Parse episode ID
        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode ID: {}", e))?;

        // Set tags using the memory API
        self.memory
            .set_episode_tags(episode_id, input.tags.clone())
            .await?;

        // Get updated tags
        let current_tags = self.memory.get_episode_tags(episode_id).await?;

        info!(
            "Successfully set {} tags on episode: {}",
            current_tags.len(),
            episode_id
        );

        Ok(SetEpisodeTagsOutput {
            success: true,
            episode_id: input.episode_id,
            tags_set: current_tags.len(),
            current_tags,
            message: format!("Set {} tag(s) on episode", input.tags.len()),
        })
    }

    /// Get tags for an episode
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id))]
    pub async fn get_tags(&self, input: GetEpisodeTagsInput) -> Result<GetEpisodeTagsOutput> {
        debug!("Getting tags for episode: {}", input.episode_id);

        // Parse episode ID
        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode ID: {}", e))?;

        // Get tags using the memory API
        let tags = self.memory.get_episode_tags(episode_id).await?;

        debug!("Found {} tags for episode: {}", tags.len(), episode_id);

        Ok(GetEpisodeTagsOutput {
            success: true,
            episode_id: input.episode_id,
            tags: tags.clone(),
            message: format!("Found {} tag(s)", tags.len()),
        })
    }

    /// Search episodes by tags
    ///
    /// Finds episodes that match the specified tag criteria using AND or OR logic.
    /// Matching is case-insensitive.
    ///
    /// # Arguments
    ///
    /// * `input` - Search criteria including tags, match mode (all/any), and limit
    ///
    /// # Returns
    ///
    /// Returns matching episodes with their metadata.
    ///
    /// # Search Modes
    ///
    /// - `require_all: true` - Episodes must have ALL specified tags (AND)
    /// - `require_all: false` - Episodes must have ANY specified tag (OR)
    ///
    /// # Performance Note
    ///
    /// This implementation loads all episodes into memory for filtering.
    /// For production use with large datasets, consider adding database indexes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_mcp::mcp::tools::episode_tags::{EpisodeTagTools, SearchEpisodesByTagsInput};
    /// # use memory_core::SelfLearningMemory;
    /// # use std::sync::Arc;
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = Arc::new(SelfLearningMemory::new());
    /// let tools = EpisodeTagTools::new(memory);
    ///
    /// // Find episodes with "bug-fix" OR "critical"
    /// let input = SearchEpisodesByTagsInput {
    ///     tags: vec!["bug-fix".to_string(), "critical".to_string()],
    ///     require_all: Some(false),
    ///     limit: Some(10),
    /// };
    ///
    /// let results = tools.search_by_tags(input).await?;
    /// println!("Found {} episodes", results.count);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, input), fields(tag_count = input.tags.len()))]
    pub async fn search_by_tags(
        &self,
        input: SearchEpisodesByTagsInput,
    ) -> Result<SearchEpisodesByTagsOutput> {
        info!("Searching episodes by {} tags", input.tags.len());

        // Validate tags
        if input.tags.is_empty() {
            return Ok(SearchEpisodesByTagsOutput {
                success: false,
                count: 0,
                episodes: vec![],
                search_criteria: "No tags provided".to_string(),
                message: "No tags provided for search".to_string(),
            });
        }

        let require_all = input.require_all.unwrap_or(false);
        let limit = input.limit.unwrap_or(100);

        // Build search criteria description
        let search_criteria = if require_all {
            format!("All of: [{}]", input.tags.join(", "))
        } else {
            format!("Any of: [{}]", input.tags.join(", "))
        };

        // Get all episodes and filter by tags
        // Note: This is a basic implementation. In production, you'd want
        // to use a database query for better performance
        let all_episodes = self.memory.get_all_episodes().await?;

        let mut matching_episodes = Vec::new();

        for episode in all_episodes {
            let episode_tags = &episode.tags;

            let matches = if require_all {
                // Check if episode has ALL requested tags
                input
                    .tags
                    .iter()
                    .all(|tag| episode_tags.iter().any(|et| et.eq_ignore_ascii_case(tag)))
            } else {
                // Check if episode has ANY requested tag
                input
                    .tags
                    .iter()
                    .any(|tag| episode_tags.iter().any(|et| et.eq_ignore_ascii_case(tag)))
            };

            if matches {
                matching_episodes.push(EpisodeTagResult {
                    episode_id: episode.episode_id.to_string(),
                    task_description: episode.task_description.clone(),
                    task_type: format!("{:?}", episode.task_type),
                    tags: episode.tags.clone(),
                    start_time: episode.start_time.timestamp(),
                    end_time: episode.end_time.map(|t| t.timestamp()),
                    outcome: episode.outcome.map(|o| format!("{:?}", o)),
                });

                if matching_episodes.len() >= limit {
                    break;
                }
            }
        }

        let count = matching_episodes.len();
        info!("Found {} episodes matching tag search", count);

        Ok(SearchEpisodesByTagsOutput {
            success: true,
            count,
            episodes: matching_episodes,
            search_criteria,
            message: format!("Found {} episode(s) matching tags", count),
        })
    }
}
