//! Checkpoint projection query (ADR-085 Phase 1, HY-03).
//!
//! Serves checkpoint listings without loading the full episode: the in-memory
//! tier projects the list straight from the fallback episode map, and a durable
//! backend advertising
//! [`StorageBackendCapabilities::supports_checkpoints_query`] answers through
//! a dedicated projection query (Turso stores checkpoints in their own
//! column). Backends without the projection fall back to the standard
//! full-episode read.

use crate::error::{Error, Result};
use crate::memory::SelfLearningMemory;
use crate::memory::checkpoint::types::CheckpointMeta;
use uuid::Uuid;

impl SelfLearningMemory {
    /// Retrieve the checkpoint list of an episode without loading the full episode.
    ///
    /// Tier order: in-memory fallback projection → durable projection query
    /// (capability-gated by
    /// [`supports_checkpoints_query`](crate::storage::StorageBackendCapabilities::supports_checkpoints_query))
    /// → standard full-episode read. A `None` projection falls through so a
    /// hybrid setup whose cache tier still holds the episode keeps working.
    /// Returns [`Error::NotFound`] when no tier holds the episode, matching
    /// `get_episode`.
    pub async fn get_checkpoints(&self, episode_id: Uuid) -> Result<Vec<CheckpointMeta>> {
        {
            let fallback = self.episodes_fallback.read().await;
            if let Some(checkpoints) = fallback.get(&episode_id).map(|ep| ep.checkpoints.clone()) {
                return Ok(checkpoints);
            }
        }

        if let Some(durable) = &self.turso_storage {
            if durable.supports_checkpoints_query() {
                match durable.get_checkpoints(episode_id).await {
                    Ok(Some(list)) => return Ok(list),
                    Ok(None) => {}
                    Err(Error::NotFound(_)) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        let episode = self.get_episode(episode_id).await?;
        Ok(episode.checkpoints)
    }
}

#[cfg(test)]
#[path = "query_tests.rs"]
mod tests;
