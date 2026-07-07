//! Semantic retriever for embedding-based episode lookup

use crate::embeddings::{SemanticService, VectorIndex};
use crate::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

/// Retriever that uses semantic embeddings and a vector index
pub struct SemanticRetriever {
    semantic_service: Arc<SemanticService>,
    index: Arc<RwLock<dyn VectorIndex>>,
}

impl SemanticRetriever {
    /// Create a new semantic retriever
    pub fn new(semantic_service: Arc<SemanticService>, index: Arc<RwLock<dyn VectorIndex>>) -> Self {
        Self {
            semantic_service,
            index,
        }
    }

    /// Retrieve episodes similar to the given text
    pub async fn retrieve(&self, query_text: &str, top_k: usize) -> Result<Vec<(Uuid, f32)>> {
        // 1. Generate embedding for query text
        let embedding = self.semantic_service.provider.embed_text(query_text).await?;

        // 2. Search index
        let hits = self.index.read().search(&embedding, top_k)?;

        // 3. Map to UUIDs
        let mut results = Vec::new();
        for hit in hits {
            if let Ok(id) = Uuid::parse_str(&hit.id) {
                results.push((id, hit.score));
            }
        }

        Ok(results)
    }

    /// Add an episode to the semantic index
    pub async fn index_episode(&self, episode_id: Uuid, text: &str) -> Result<()> {
        let embedding = self.semantic_service.provider.embed_text(text).await?;
        self.index.write().upsert(&episode_id.to_string(), &embedding)?;
        Ok(())
    }

    /// Remove an episode from the semantic index
    pub fn remove_episode(&self, episode_id: Uuid) -> Result<()> {
        self.index.write().remove(&episode_id.to_string())?;
        Ok(())
    }
}
