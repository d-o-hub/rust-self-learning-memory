//! Post-completion embedding, summary persistence, and ANN index maintenance.
//!
//! Split out of [`completion`](super::completion) to keep that module under
//! the repository's per-file LOC ceiling.

use super::SelfLearningMemory;
use crate::Episode;
use crate::error::Result;
use crate::semantic::EpisodeSummary;
use tracing::{debug, info, warn};
use uuid::Uuid;

impl SelfLearningMemory {
    /// Persist a generated semantic summary across fallback, cache, and durable storage.
    ///
    /// Capability-gated via [`StorageBackendCapabilities::supports_episode_summaries`](crate::storage::StorageBackendCapabilities::supports_episode_summaries)
    /// so non-supporting backends are skipped rather than raising `CapabilityUnavailable`.
    pub(super) async fn persist_completion_summary(&self, summary: Option<&EpisodeSummary>) {
        let Some(summary) = summary else {
            return;
        };
        let episode_id = summary.episode_id;

        {
            let mut summaries = self.summaries_fallback.write().await;
            summaries.insert(episode_id, summary.clone());
        }

        self.mirror_summary_to_cache(summary).await;

        if let Some(turso) = &self.turso_storage {
            if turso.supports_episode_summaries() {
                if let Err(e) = turso.store_episode_summary(summary).await {
                    warn!(
                        episode_id = %episode_id,
                        error = %e,
                        "Failed to store semantic summary in durable storage"
                    );
                }
            }
        }

        info!(
            episode_id = %episode_id,
            summary_words = summary.summary_text.split_whitespace().count(),
            key_concepts = summary.key_concepts.len(),
            "Stored semantic summary"
        );
    }

    /// Best-effort cache write for an episode summary when the cache backend
    /// advertises `supports_episode_summaries()`.
    async fn mirror_summary_to_cache(&self, summary: &EpisodeSummary) {
        let Some(cache) = &self.cache_storage else {
            return;
        };
        if !cache.supports_episode_summaries() {
            return;
        }
        if let Err(e) = cache.store_episode_summary(summary).await {
            warn!(
                episode_id = %summary.episode_id,
                error = %e,
                "Failed to store semantic summary in cache storage"
            );
        }
    }

    /// Retrieve an episode's semantic summary from memory, cache, or durable storage.
    ///
    /// Hydrates `summaries_fallback` (and mirrors from durable to cache storage when
    /// supported) on a storage hit so subsequent lookups are served in-memory.
    pub async fn get_episode_summary(&self, episode_id: Uuid) -> Result<Option<EpisodeSummary>> {
        if let Some(summary) = self
            .summaries_fallback
            .read()
            .await
            .get(&episode_id)
            .cloned()
        {
            return Ok(Some(summary));
        }

        if let Some(cache) = &self.cache_storage {
            if cache.supports_episode_summaries() {
                if let Ok(Some(summary)) = cache.get_episode_summary(episode_id).await {
                    self.summaries_fallback
                        .write()
                        .await
                        .insert(episode_id, summary.clone());
                    return Ok(Some(summary));
                }
            }
        }

        if let Some(turso) = &self.turso_storage {
            if turso.supports_episode_summaries() {
                match turso.get_episode_summary(episode_id).await {
                    Ok(Some(summary)) => {
                        self.mirror_summary_to_cache(&summary).await;
                        self.summaries_fallback
                            .write()
                            .await
                            .insert(episode_id, summary.clone());
                        return Ok(Some(summary));
                    }
                    Ok(None) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(None)
    }

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
