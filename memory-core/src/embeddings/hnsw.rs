//! HNSW-based vector indexing implementation.

use crate::embeddings::index::{VectorHit, VectorIndex};
use crate::error::Result;
#[cfg(feature = "hnsw")]
use hnsw_rs::prelude::{DistCosine, Hnsw};
use std::path::Path;

#[cfg(feature = "hnsw")]
use std::collections::HashMap;

/// Serializable snapshot of an [`HnswVectorIndex`] written by [`VectorIndex::save`].
///
/// Vectors are stored in the sidecar so the index can be rebuilt on load without
/// re-querying an embedding provider.  The HNSW graph is reconstructed by
/// re-inserting all vectors, which is O(n log n) on startup.
#[cfg(feature = "hnsw")]
#[derive(serde::Serialize, serde::Deserialize)]
struct HnswSnapshot {
    max_nb_conn: usize,
    max_layer: usize,
    ef_construction: usize,
    dim: usize,
    max_elements: Option<usize>,
    // Ordered by ascending internal_id so FIFO insertion order is preserved.
    entries: Vec<SnapshotEntry>,
}

#[cfg(feature = "hnsw")]
#[derive(serde::Serialize, serde::Deserialize)]
struct SnapshotEntry {
    internal_id: usize,
    external_id: String,
    embedding: Vec<f32>,
}

#[cfg(feature = "hnsw")]
pub struct HnswVectorIndex {
    hnsw: Hnsw<'static, f32, DistCosine>,
    id_map: HashMap<usize, String>,
    rev_map: HashMap<String, usize>,
    vectors: HashMap<usize, Vec<f32>>,
    next_id: usize,
    max_elements: Option<usize>,
    max_nb_conn: usize,
    max_layer: usize,
    ef_construction: usize,
    dim: usize,
}

#[cfg(feature = "hnsw")]
impl HnswVectorIndex {
    pub fn new(max_nb_conn: usize, max_layer: usize, ef_construction: usize, dim: usize) -> Self {
        Self {
            hnsw: Hnsw::new(max_nb_conn, max_layer, ef_construction, dim, DistCosine {}),
            id_map: HashMap::new(),
            rev_map: HashMap::new(),
            vectors: HashMap::new(),
            next_id: 0,
            max_elements: None,
            max_nb_conn,
            max_layer,
            ef_construction,
            dim,
        }
    }

    /// Set a hard capacity limit.  When the index reaches `max_elements` the
    /// oldest inserted vector (FIFO by insertion order) is evicted before each
    /// new insert.
    #[must_use]
    pub fn with_capacity(mut self, max_elements: usize) -> Self {
        self.max_elements = Some(max_elements);
        self
    }

    /// Reconstruct an index from a snapshot written by [`VectorIndex::save`].
    pub fn load(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let snap: HnswSnapshot = serde_json::from_reader(file)?;

        let mut index = Self::new(
            snap.max_nb_conn,
            snap.max_layer,
            snap.ef_construction,
            snap.dim,
        );
        index.max_elements = snap.max_elements;

        for entry in snap.entries {
            index
                .id_map
                .insert(entry.internal_id, entry.external_id.clone());
            index.rev_map.insert(entry.external_id, entry.internal_id);
            index.hnsw.insert((&entry.embedding, entry.internal_id));
            index.vectors.insert(entry.internal_id, entry.embedding);
            if entry.internal_id >= index.next_id {
                index.next_id = entry.internal_id + 1;
            }
        }

        Ok(index)
    }

    /// Evict the entry with the lowest internal_id (FIFO order).
    fn evict_oldest(&mut self) {
        if let Some(oldest_id) = self.id_map.keys().copied().min() {
            if let Some(ext_id) = self.id_map.remove(&oldest_id) {
                self.rev_map.remove(&ext_id);
            }
            self.vectors.remove(&oldest_id);
            // hnsw_rs has no remove; the internal graph node becomes orphaned.
            // Searches will still return it but id_map.get() will return None,
            // causing the filter_map in search() to drop it from results.
        }
    }
}

#[cfg(feature = "hnsw")]
impl VectorIndex for HnswVectorIndex {
    fn upsert(&mut self, id: &str, embedding: &[f32]) -> Result<()> {
        if let Some(&existing_id) = self.rev_map.get(id) {
            // Update vector data; hnsw_rs doesn't support in-place update so we
            // re-insert with the same internal_id (the graph keeps both nodes but
            // the id_map still points to this one, so the old node becomes orphaned).
            self.hnsw.insert((embedding, existing_id));
            self.vectors.insert(existing_id, embedding.to_vec());
            return Ok(());
        }

        // Evict when at capacity.
        if let Some(max) = self.max_elements {
            if self.id_map.len() >= max {
                self.evict_oldest();
            }
        }

        let new_id = self.next_id;
        self.next_id += 1;
        self.id_map.insert(new_id, id.to_string());
        self.rev_map.insert(id.to_string(), new_id);
        self.vectors.insert(new_id, embedding.to_vec());
        self.hnsw.insert((embedding, new_id));
        Ok(())
    }

    fn remove(&mut self, id: &str) -> Result<()> {
        if let Some(internal_id) = self.rev_map.remove(id) {
            self.id_map.remove(&internal_id);
            self.vectors.remove(&internal_id);
        }
        Ok(())
    }

    fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<VectorHit>> {
        let ef_search = top_k * 2;
        let neighbors = self.hnsw.search(query, top_k, ef_search);

        let hits = neighbors
            .into_iter()
            .filter_map(|neighbor| {
                let internal_id = neighbor.d_id;
                self.id_map.get(&internal_id).map(|id| VectorHit {
                    id: id.clone(),
                    score: 1.0 - neighbor.distance,
                })
            })
            .collect();

        Ok(hits)
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut entries: Vec<SnapshotEntry> = self
            .vectors
            .iter()
            .filter_map(|(&internal_id, embedding)| {
                self.id_map.get(&internal_id).map(|ext_id| SnapshotEntry {
                    internal_id,
                    external_id: ext_id.clone(),
                    embedding: embedding.clone(),
                })
            })
            .collect();

        // Sort by internal_id so load() preserves insertion order.
        entries.sort_by_key(|e| e.internal_id);

        let snap = HnswSnapshot {
            max_nb_conn: self.max_nb_conn,
            max_layer: self.max_layer,
            ef_construction: self.ef_construction,
            dim: self.dim,
            max_elements: self.max_elements,
            entries,
        };

        let file = std::fs::File::create(path)?;
        serde_json::to_writer(file, &snap)?;
        Ok(())
    }

    fn len(&self) -> usize {
        self.id_map.len()
    }
}

#[cfg(not(feature = "hnsw"))]
#[derive(Default)]
pub struct HnswVectorIndex {}

#[cfg(not(feature = "hnsw"))]
impl VectorIndex for HnswVectorIndex {
    fn upsert(&mut self, _id: &str, _embedding: &[f32]) -> Result<()> {
        Err(crate::error::Error::Configuration(
            "HNSW feature not enabled".to_string(),
        ))
    }
    fn remove(&mut self, _id: &str) -> Result<()> {
        Err(crate::error::Error::Configuration(
            "HNSW feature not enabled".to_string(),
        ))
    }
    fn search(&self, _query: &[f32], _top_k: usize) -> Result<Vec<VectorHit>> {
        Err(crate::error::Error::Configuration(
            "HNSW feature not enabled".to_string(),
        ))
    }
    fn save(&self, _path: &Path) -> Result<()> {
        Err(crate::error::Error::Configuration(
            "HNSW feature not enabled".to_string(),
        ))
    }
    fn len(&self) -> usize {
        0
    }
}

#[cfg(feature = "hnsw")]
impl Default for HnswVectorIndex {
    fn default() -> Self {
        Self::new(16, 16, 200, 384)
    }
}

impl HnswVectorIndex {
    /// Create a new HnswVectorIndex (if feature enabled)
    #[allow(dead_code)]
    pub fn new_enabled(dim: usize) -> Self {
        #[cfg(feature = "hnsw")]
        {
            Self::new(16, 16, 200, dim)
        }
        #[cfg(not(feature = "hnsw"))]
        {
            let _ = dim;
            Self {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::index::VectorIndex;

    #[cfg(feature = "hnsw")]
    #[test]
    fn test_hnsw_vector_index_search() {
        let mut index = HnswVectorIndex::new(16, 16, 200, 3);
        index.upsert("1", &[1.0, 0.0, 0.0]).unwrap();
        index.upsert("2", &[0.0, 1.0, 0.0]).unwrap();
        index.upsert("3", &[0.5, 0.5, 0.0]).unwrap();

        let query = [1.0, 0.1, 0.0];
        let hits = index.search(&query, 2).unwrap();

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id, "1");
        assert_eq!(hits[1].id, "3");
    }

    #[cfg(feature = "hnsw")]
    #[test]
    fn test_hnsw_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hnsw_index.json");

        let mut index = HnswVectorIndex::new(16, 16, 200, 3);
        index.upsert("a", &[1.0, 0.0, 0.0]).unwrap();
        index.upsert("b", &[0.0, 1.0, 0.0]).unwrap();
        index.upsert("c", &[0.5, 0.5, 0.0]).unwrap();
        index.save(&path).unwrap();

        let loaded = HnswVectorIndex::load(&path).unwrap();
        assert_eq!(loaded.len(), 3);

        let hits = loaded.search(&[1.0, 0.0, 0.0], 1).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "a");
    }

    #[cfg(feature = "hnsw")]
    #[test]
    fn test_hnsw_capacity_eviction() {
        let mut index = HnswVectorIndex::new(16, 16, 200, 2).with_capacity(2);
        index.upsert("x", &[1.0, 0.0]).unwrap();
        index.upsert("y", &[0.0, 1.0]).unwrap();
        assert_eq!(index.len(), 2);

        // Inserting a third entry must evict the oldest ("x").
        index.upsert("z", &[0.5, 0.5]).unwrap();
        assert_eq!(index.len(), 2);
        assert!(
            !index.rev_map.contains_key("x"),
            "oldest entry should be evicted"
        );
        assert!(index.rev_map.contains_key("y"));
        assert!(index.rev_map.contains_key("z"));
    }

    #[cfg(feature = "hnsw")]
    #[test]
    fn test_hnsw_remove() {
        let mut index = HnswVectorIndex::new(16, 16, 200, 2);
        index.upsert("p", &[1.0, 0.0]).unwrap();
        assert_eq!(index.len(), 1);
        index.remove("p").unwrap();
        assert_eq!(index.len(), 0);
    }

    #[cfg(not(feature = "hnsw"))]
    #[test]
    fn test_hnsw_disabled_errors() {
        let mut index = HnswVectorIndex::default();
        assert!(index.upsert("1", &[1.0]).is_err());
        assert!(index.search(&[1.0], 1).is_err());
        assert!(index.remove("1").is_err());
        assert_eq!(index.len(), 0);
    }
}
