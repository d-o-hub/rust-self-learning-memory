//! HNSW-based vector indexing implementation.

use crate::embeddings::index::{VectorHit, VectorIndex};
use crate::error::Result;
#[cfg(feature = "hnsw")]
use hnsw_rs::prelude::*;
use std::path::Path;

#[cfg(feature = "hnsw")]
use std::collections::HashMap;

#[cfg(feature = "hnsw")]
pub struct HnswVectorIndex {
    hnsw: Hnsw<'static, f32, DistCosine>,
    id_map: HashMap<usize, String>,
    rev_map: HashMap<String, usize>,
    next_id: usize,
}

#[cfg(feature = "hnsw")]
impl HnswVectorIndex {
    pub fn new(max_nb_conn: usize, max_layer: usize, ef_construction: usize, dim: usize) -> Self {
        Self {
            hnsw: Hnsw::new(max_nb_conn, max_layer, ef_construction, dim, DistCosine {}),
            id_map: HashMap::new(),
            rev_map: HashMap::new(),
            next_id: 0,
        }
    }
}

#[cfg(feature = "hnsw")]
impl VectorIndex for HnswVectorIndex {
    fn upsert(&mut self, id: &str, embedding: &[f32]) -> Result<()> {
        let internal_id = if let Some(&existing_id) = self.rev_map.get(id) {
            existing_id
        } else {
            let new_id = self.next_id;
            self.next_id += 1;
            self.id_map.insert(new_id, id.to_string());
            self.rev_map.insert(id.to_string(), new_id);
            new_id
        };

        self.hnsw.insert((embedding, internal_id));
        Ok(())
    }

    fn remove(&mut self, id: &str) -> Result<()> {
        if let Some(internal_id) = self.rev_map.remove(id) {
            self.id_map.remove(&internal_id);
        }
        Ok(())
    }

    fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<VectorHit>> {
        let ef_search = top_k * 2;
        let neighbors = self.hnsw.search(query, top_k, ef_search);

        let hits = neighbors
            .into_iter()
            .filter_map(|neighbor| {
                let internal_id = neighbor.p_id;

                self.id_map.get(&internal_id).map(|id| VectorHit {
                    id: id.clone(),
                    score: 1.0 - neighbor.distance,
                })
            })
            .collect();

        Ok(hits)
    }

    fn save(&self, _path: &Path) -> Result<()> {
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
