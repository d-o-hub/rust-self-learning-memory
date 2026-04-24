//! Storage operations for episodes, patterns, and heuristics
//!
//! This module is organized into submodules for different storage concerns:
//! - `episodes`: Episode CRUD operations
//! - `patterns`: Pattern CRUD operations
//! - `heuristics`: Heuristic CRUD operations
//! - `monitoring`: Monitoring and metrics storage
//! - `embeddings`: Embedding storage and retrieval
//! - `search`: Vector similarity search
//! - `capacity`: Capacity-constrained storage

use crate::TursoStorage;
use do_memory_core::Result;

// Re-export submodules
pub mod batch;
pub mod capacity;
pub mod embedding_backend;
mod embedding_tables;
mod embeddings_internal;
pub mod episodes;
pub mod heuristics;
pub mod monitoring;
pub mod patterns;
pub mod recommendations;
pub mod search;
pub mod tag_operations;

// Multi-dimensional embedding storage (feature-gated)
#[cfg(feature = "turso_multi_dimension")]
mod embeddings_multi;

pub use batch::episode_batch::BatchConfig;
pub use episodes::EpisodeQuery;
pub use episodes::raw_query::EPISODE_SELECT_COLUMNS;
pub use episodes::raw_query::RawEpisodeQuery;
pub use patterns::PATTERN_SELECT_COLUMNS;
#[allow(unused)]
pub use patterns::PatternMetadata;
pub use patterns::PatternQuery;
pub use patterns::RawPatternQuery;
pub use tag_operations::TagStats;

// Re-export dimension stats when multi-dimension feature is enabled
#[cfg(feature = "turso_multi_dimension")]
pub use embeddings_multi::DimensionStats;

impl TursoStorage {
    // ========== Backend-compatible embedding methods ==========

    /// Store an embedding (backend API)
    pub async fn store_embedding_backend(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        self._store_embedding_internal(id, "embedding", &embedding)
            .await
    }

    /// Get an embedding (backend API)
    pub async fn get_embedding_backend(&self, id: &str) -> Result<Option<Vec<f32>>> {
        self._get_embedding_internal(id, "embedding").await
    }

    /// Delete an embedding (backend API)
    pub async fn delete_embedding_backend(&self, id: &str) -> Result<bool> {
        self._delete_embedding_internal(id).await
    }

    /// Store embeddings in batch (backend API)
    pub async fn store_embeddings_batch_backend(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        self._store_embeddings_batch_internal(embeddings).await
    }

    /// Get embeddings in batch (backend API)
    pub async fn get_embeddings_batch_backend(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<Vec<f32>>>> {
        self._get_embeddings_batch_internal(ids).await
    }
}
