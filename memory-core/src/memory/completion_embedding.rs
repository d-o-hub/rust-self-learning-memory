//! Post-completion embedding generation and ANN index maintenance.
//!
//! Split out of [`completion`](super::completion) to keep that module under
//! the repository's per-file LOC ceiling.

use super::SelfLearningMemory;
use crate::Episode;
use tracing::{debug, warn};

impl SelfLearningMemory {
    /// Generate the episode embedding and refresh the ANN index.
    ///
    /// Uses the live provider snapshot so episodes are embedded with the
    /// runtime-activated provider (issue #1072). Embedding failures are
    /// logged and swallowed: completion must not fail because of the
    /// semantic layer.
    pub(super) async fn update_completion_embeddings(&self, episode: &Episode) {
        let episode_id = episode.episode_id;

        let Some(semantic) = self.live_semantic_service().await else {
            return;
        };

        if let Err(e) = semantic.embed_episode(episode).await {
            warn!(
                episode_id = %episode_id,
                error = %e,
                "Failed to generate embedding for episode. Continuing without embedding."
            );
            // Don't fail entire operation on embedding error
            return;
        }

        debug!(
            episode_id = %episode_id,
            "Successfully generated embedding for episode"
        );

        // Update ANN index for hybrid search (v0.1.34)
        let Some(retriever) = &self.semantic_retriever else {
            return;
        };
        let Ok(embeddings) = semantic.get_embeddings_batch(&[episode_id]).await else {
            return;
        };
        if let Some(Some(embedding)) = embeddings.first() {
            let _ = retriever.upsert(&episode_id.to_string(), embedding.clone());
        }
    }
}
