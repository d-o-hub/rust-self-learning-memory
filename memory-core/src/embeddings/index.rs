//! Vector index trait and implementations for semantic search

use crate::Result;
use std::collections::HashMap;

/// Result of a vector search
#[derive(Debug, Clone)]
pub struct VectorHit {
    /// ID of the item
    pub id: String,
    /// Similarity score (0.0 to 1.0)
    pub score: f32,
}

/// Trait for vector indices supporting ANN or exact search
pub trait VectorIndex: Send + Sync {
    /// Add or update an item in the index
    fn upsert(&mut self, id: &str, embedding: &[f32]) -> Result<()>;

    /// Remove an item from the index
    fn remove(&mut self, id: &str) -> Result<()>;

    /// Search for the most similar items
    fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<VectorHit>>;
}

/// Simple in-memory vector index using linear scan
#[derive(Default)]
pub struct InMemoryVectorIndex {
    embeddings: HashMap<String, Vec<f32>>,
}

impl InMemoryVectorIndex {
    /// Create a new empty in-memory vector index
    #[must_use]
    pub fn new() -> Self {
        Self {
            embeddings: HashMap::new(),
        }
    }
}

impl VectorIndex for InMemoryVectorIndex {
    fn upsert(&mut self, id: &str, embedding: &[f32]) -> Result<()> {
        self.embeddings.insert(id.to_string(), embedding.to_vec());
        Ok(())
    }

    fn remove(&mut self, id: &str) -> Result<()> {
        self.embeddings.remove(id);
        Ok(())
    }

    fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<VectorHit>> {
        let mut hits: Vec<VectorHit> = self.embeddings.iter()
            .map(|(id, embedding)| {
                let score = crate::embeddings::cosine_similarity(query, embedding);
                VectorHit {
                    id: id.clone(),
                    score,
                }
            })
            .collect();

        // Sort by score descending
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Take top K
        hits.truncate(top_k);

        Ok(hits)
    }
}
