//! Vector index abstractions and implementations.

use crate::embeddings::similarity::cosine_similarity;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// A hit from a vector search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorHit {
    /// The ID of the vector.
    pub id: String,
    /// The similarity score.
    pub score: f32,
}

/// Trait for vector indexing and similarity search.
pub trait VectorIndex: Send + Sync {
    /// Add or update a vector in the index.
    fn upsert(&mut self, id: &str, embedding: &[f32]) -> Result<()>;

    /// Remove a vector from the index.
    fn remove(&mut self, id: &str) -> Result<()>;

    /// Search for the top-k most similar vectors.
    fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<VectorHit>>;

    /// Save the index to a file.
    fn save(&self, path: &Path) -> Result<()>;

    /// Get the number of vectors in the index.
    fn len(&self) -> usize;

    /// Check if the index is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A simple brute-force vector index.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SimpleVectorIndex {
    vectors: HashMap<String, Vec<f32>>,
}

impl SimpleVectorIndex {
    /// Create a new empty SimpleVectorIndex.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a SimpleVectorIndex from a file.
    pub fn load(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let index: Self = serde_json::from_reader(file)?;
        Ok(index)
    }
}

impl VectorIndex for SimpleVectorIndex {
    fn upsert(&mut self, id: &str, embedding: &[f32]) -> Result<()> {
        self.vectors.insert(id.to_string(), embedding.to_vec());
        Ok(())
    }

    fn remove(&mut self, id: &str) -> Result<()> {
        self.vectors.remove(id);
        Ok(())
    }

    fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<VectorHit>> {
        let mut hits: Vec<VectorHit> = self
            .vectors
            .iter()
            .map(|(id, vec)| {
                let score = cosine_similarity(query, vec);
                VectorHit {
                    id: id.clone(),
                    score,
                }
            })
            .collect();

        // Sort by score descending
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top-k
        hits.truncate(top_k);

        Ok(hits)
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_simple_vector_index_search() {
        let mut index = SimpleVectorIndex::new();
        index.upsert("1", &[1.0, 0.0, 0.0]).unwrap();
        index.upsert("2", &[0.0, 1.0, 0.0]).unwrap();
        index.upsert("3", &[0.5, 0.5, 0.0]).unwrap();

        let query = [1.0, 0.1, 0.0];
        let hits = index.search(&query, 2).unwrap();

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id, "1");
        assert_eq!(hits[1].id, "3");
    }

    #[test]
    fn test_simple_vector_index_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("index.json");

        let mut index = SimpleVectorIndex::new();
        index.upsert("1", &[1.0, 0.0]).unwrap();
        index.save(&path).unwrap();

        let loaded = SimpleVectorIndex::load(&path).unwrap();
        assert_eq!(loaded.len(), 1);

        let hits = loaded.search(&[1.0, 0.0], 1).unwrap();
        assert_eq!(hits[0].id, "1");
    }
}
