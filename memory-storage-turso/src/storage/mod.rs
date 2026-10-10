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
mod capacity_cleanup;
pub(crate) mod capacity_intents;
mod embedding_backend;
mod embedding_tables;
mod embeddings_internal;
pub mod episodes;
pub mod heuristics;
pub mod monitoring;
pub mod patterns;
pub mod procedural;
pub mod query_builder;
pub mod recommendations;
pub mod search;
pub mod tag_operations;
mod transaction_scope;

// Multi-dimensional embedding storage (feature-gated)
#[cfg(feature = "turso_multi_dimension")]
mod embeddings_multi;

// Capacity-eviction durable outbox type (issue #1070)
pub use capacity_intents::CapacityEvictionIntent;

pub use batch::episode_batch::BatchConfig;
pub use episodes::EpisodeQuery;
pub use episodes::raw_query::EPISODE_SELECT_COLUMNS;
pub use episodes::raw_query::RawEpisodeQuery;
pub use patterns::PATTERN_SELECT_COLUMNS;
#[allow(unused)]
pub use patterns::PatternMetadata;
pub use patterns::PatternQuery;
pub use patterns::RawPatternQuery;
pub use query_builder::{
    EpisodeColumn, EpisodeQueryBuilder, FilterOp, PatternColumn, PatternQueryBuilder, QueryBuilder,
    QueryColumn,
};
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

    /// List embeddings stored through the generic backend API.
    ///
    /// Returns the `item_id`s of every row tagged `item_type = 'embedding'`.
    /// Both layouts are covered: the single-table build reads `embeddings`, and
    /// the `turso_multi_dimension` build walks the dimension-specific
    /// `embeddings_<dimension>` tables that [`Self::store_embedding_backend`]
    /// routes writes into. The result is sorted and deduplicated so callers get
    /// a deterministic candidate order.
    ///
    /// # Errors
    ///
    /// Returns error if the embeddings cannot be queried.
    pub async fn list_embedding_ids_backend(&self) -> Result<Vec<String>> {
        #[cfg(feature = "turso_multi_dimension")]
        {
            // The generic API routes writes into the dimension-specific tables
            // under this feature (`_store_embedding_internal` ->
            // `store_embedding_dimension_aware`), so the listing must walk the
            // same tables instead of returning an empty base namespace.
            const TABLES_SQL: &str = "SELECT name FROM sqlite_master \
                 WHERE type = 'table' \
                   AND name LIKE 'embeddings\\_%' ESCAPE '\\' \
                 ORDER BY name";

            let (conn, _conn_id) = self.get_connection_with_id().await?;
            let mut table_rows = conn.query(TABLES_SQL, ()).await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to list embedding tables: {}", e))
            })?;

            let mut tables = Vec::new();
            while let Some(row) = table_rows.next().await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to read embedding table row: {}", e))
            })? {
                let name: String = row.get(0).map_err(|e| {
                    do_memory_core::Error::Storage(format!(
                        "Failed to parse embedding table name: {}",
                        e
                    ))
                })?;
                tables.push(name);
            }

            let mut ids = Vec::new();
            for table in tables {
                // SAFETY: `table` comes from `sqlite_master` (schema-owned, never
                // user input), so it cannot inject SQL; identifiers cannot be
                // parameterized.
                let sql = format!("SELECT item_id FROM {table} WHERE item_type = 'embedding'");
                let mut rows = conn.query(&sql, ()).await.map_err(|e| {
                    do_memory_core::Error::Storage(format!(
                        "Failed to list embeddings from {table}: {}",
                        e
                    ))
                })?;
                while let Some(row) = rows.next().await.map_err(|e| {
                    do_memory_core::Error::Storage(format!(
                        "Failed to read embedding row from {table}: {}",
                        e
                    ))
                })? {
                    let id: String = row
                        .get(0)
                        .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
                    ids.push(id);
                }
            }
            ids.sort();
            ids.dedup();
            Ok(ids)
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            let (conn, _conn_id) = self.get_connection_with_id().await?;
            let mut rows = conn
                .query(
                    "SELECT item_id FROM embeddings WHERE item_type = 'embedding'",
                    (),
                )
                .await
                .map_err(|e| {
                    do_memory_core::Error::Storage(format!("Failed to list embeddings: {}", e))
                })?;

            let mut ids = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to read embedding row: {}", e))
            })? {
                let id: String = row
                    .get(0)
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
                ids.push(id);
            }
            ids.sort();
            ids.dedup();
            Ok(ids)
        }
    }

    /// Migrate existing embeddings to populate embedding_vector column
    pub async fn migrate_embeddings_to_vector_format(&self) -> Result<usize> {
        use tracing::info;
        info!("Starting embedding vector migration...");
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        let sql = r#"
            UPDATE embeddings
            SET embedding_vector = vector32(embedding_data)
            WHERE embedding_vector IS NULL AND embedding_data IS NOT NULL
        "#;

        let result = conn.execute(sql, ()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to migrate embeddings: {}", e))
        })?;

        info!("Migrated {} embeddings to vector format", result);
        Ok(result as usize)
    }

    /// Check if embedding vector column is populated for vector_top_k search
    pub async fn has_vector_embeddings(&self) -> Result<bool> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let sql = "SELECT COUNT(*) FROM embeddings WHERE embedding_vector IS NOT NULL LIMIT 1";

        let mut rows = conn.query(sql, ()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to check vector embeddings: {}", e))
        })?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?
        {
            let count: i64 = row
                .get(0)
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            return Ok(count > 0);
        }

        Ok(false)
    }
}
