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

    /// Provider identity (`kind:model:dims`) that produced the stored vectors.
    ///
    /// Returns `None` when the index does not track provider provenance, which
    /// includes snapshots written before identity stamping. Callers must treat
    /// `None` as "unknown provenance" and never assume compatibility with a
    /// specific provider.
    fn provider_identity(&self) -> Option<&str> {
        None
    }

    /// Adopt `identity` as the producer of future vectors.
    ///
    /// Implementations that track provenance MUST drop every stored vector when
    /// the recorded producer differs (vectors from another provider are not
    /// comparable) and then stamp `identity`. Implementations that cannot clear
    /// themselves MUST leave the index untouched; callers then quarantine it by
    /// comparing [`Self::provider_identity`] with the required identity instead
    /// of querying stale vectors.
    fn reconcile_provider_identity(&mut self, identity: &str) {
        let _ = identity;
    }
}

/// A simple brute-force vector index.
///
/// The snapshot records the [`VectorIndex::provider_identity`] that produced
/// the stored vectors so a reloaded index can be checked against the provider
/// that is actually active, instead of being queried blindly.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SimpleVectorIndex {
    /// Provider identity that produced the stored vectors. Empty for snapshots
    /// written before identity stamping, which are treated as unknown.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    provider_identity: String,
    vectors: HashMap<String, Vec<f32>>,
}

impl SimpleVectorIndex {
    /// Create a new empty SimpleVectorIndex.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty index stamped with the provider `identity`.
    #[must_use]
    pub fn with_provider_identity(identity: impl Into<String>) -> Self {
        Self {
            provider_identity: identity.into().trim().to_string(),
            vectors: HashMap::new(),
        }
    }

    /// Whether this index records `identity` as its vector producer.
    ///
    /// An unstamped (legacy) index never matches, because its provenance is
    /// unknown.
    #[must_use]
    pub fn has_provider_identity(&self, identity: &str) -> bool {
        !self.provider_identity.is_empty() && self.provider_identity == identity.trim()
    }

    /// Load a SimpleVectorIndex from a file.
    ///
    /// Snapshots written before identity stamping load with an empty provider
    /// identity; use [`Self::has_provider_identity`] to reject them.
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

    fn provider_identity(&self) -> Option<&str> {
        if self.provider_identity.is_empty() {
            None
        } else {
            Some(self.provider_identity.as_str())
        }
    }

    fn reconcile_provider_identity(&mut self, identity: &str) {
        let identity = identity.trim();
        if identity == self.provider_identity {
            return;
        }
        // Vectors from a different provider (or unknown legacy provenance) are
        // not comparable with the new provider; drop them rather than query
        // mismatched embeddings.
        self.vectors.clear();
        identity.clone_into(&mut self.provider_identity);
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

    #[test]
    fn test_simple_vector_index_remove() {
        let mut index = SimpleVectorIndex::new();
        index.upsert("1", &[1.0, 0.0]).unwrap();
        assert_eq!(index.len(), 1);
        index.remove("1").unwrap();
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_provider_identity_round_trips_through_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("index.json");

        let mut index = SimpleVectorIndex::with_provider_identity("local:all-MiniLM:384");
        index.upsert("1", &[1.0, 0.0]).unwrap();
        index.save(&path).unwrap();

        let loaded = SimpleVectorIndex::load(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.provider_identity(), Some("local:all-MiniLM:384"));
        assert!(loaded.has_provider_identity("local:all-MiniLM:384"));
        assert!(!loaded.has_provider_identity("openai:text-embedding-3-small:1536"));
    }

    #[test]
    fn test_legacy_snapshot_without_identity_has_unknown_provenance() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("legacy.json");
        std::fs::write(&path, r#"{"vectors":{"1":[1.0,0.0]}}"#).unwrap();

        let loaded = SimpleVectorIndex::load(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.provider_identity(), None);
        assert!(!loaded.has_provider_identity("local:any:1"));
    }

    #[test]
    fn test_reconcile_drops_vectors_from_another_provider() {
        let mut index = SimpleVectorIndex::with_provider_identity("local:a:4");
        index.upsert("1", &[1.0, 0.0, 0.0, 0.0]).unwrap();
        assert_eq!(index.len(), 1);

        // Same identity: vectors stay.
        index.reconcile_provider_identity("local:a:4");
        assert_eq!(index.len(), 1);

        // Different provider: incomparable vectors are dropped, identity stamped.
        index.reconcile_provider_identity("openai:text-embedding-3-small:1536");
        assert_eq!(index.len(), 0);
        assert_eq!(
            index.provider_identity(),
            Some("openai:text-embedding-3-small:1536")
        );
    }

    #[test]
    fn test_vector_index_identity_is_trimmed_and_blank_never_matches() {
        let index = SimpleVectorIndex::with_provider_identity("  local:a:4  ");
        assert_eq!(index.provider_identity(), Some("local:a:4"));
        assert!(index.has_provider_identity("local:a:4"));
        assert!(index.has_provider_identity("  local:a:4 "));

        let blank = SimpleVectorIndex::with_provider_identity("   ");
        assert_eq!(blank.provider_identity(), None);
        assert!(!blank.has_provider_identity(""));
        assert!(!blank.has_provider_identity("local:a:4"));
    }

    #[test]
    fn test_vector_index_unstamped_snapshot_omits_identity_field() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("index.json");

        // An empty identity must serialize exactly like a legacy snapshot so
        // the on-disk format stays backward compatible.
        let mut index = SimpleVectorIndex::new();
        index.upsert("1", &[1.0, 0.0]).unwrap();
        index.save(&path).unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(
            !raw.contains("provider_identity"),
            "unstamped snapshot must not gain an identity field: {raw}"
        );
        assert_eq!(
            SimpleVectorIndex::load(&path).unwrap().provider_identity(),
            None
        );
    }

    #[test]
    fn test_vector_index_load_rejects_malformed_snapshot() {
        let dir = tempdir().unwrap();
        let malformed = dir.path().join("malformed.json");
        std::fs::write(&malformed, "{ not json").unwrap();

        assert!(
            SimpleVectorIndex::load(&malformed).is_err(),
            "a corrupt snapshot must surface an error instead of panicking"
        );
        assert!(SimpleVectorIndex::load(&dir.path().join("missing.json")).is_err());
    }
}
