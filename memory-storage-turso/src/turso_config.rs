//! Turso storage configuration and schema initialization.
//!
//! This module provides schema initialization methods to keep lib.rs under 500 LOC.

use crate::{Result, TursoStorage, schema};
use tracing::{debug, info};

impl TursoStorage {
    /// Initialize the database schema
    ///
    /// Creates tables and indexes if they don't exist.
    /// Safe to call multiple times.
    pub async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing Turso database schema");
        let conn = self.get_connection().await?;

        // Enable WAL mode for better concurrent access
        let _ = self.execute_pragmas(&conn).await;

        // Create tables
        self.execute_with_retry(&conn, schema::CREATE_EPISODES_TABLE)
            .await?;
        self.ensure_episodes_checkpoints_column(&conn).await?;
        self.execute_with_retry(&conn, schema::CREATE_PATTERNS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_HEURISTICS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_RECOMMENDATION_SESSIONS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_RECOMMENDATION_FEEDBACK_TABLE)
            .await?;

        // Create legacy embeddings table only when multi-dimension feature is NOT enabled
        #[cfg(not(feature = "turso_multi_dimension"))]
        self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_TABLE)
            .await?;

        // Create monitoring tables
        self.execute_with_retry(&conn, schema::CREATE_EXECUTION_RECORDS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_AGENT_METRICS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_TASK_METRICS_TABLE)
            .await?;

        // Create indexes
        self.execute_with_retry(&conn, schema::CREATE_EPISODES_TASK_TYPE_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODES_TIMESTAMP_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODES_DOMAIN_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODES_ARCHIVED_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_PATTERNS_CONTEXT_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_HEURISTICS_CONFIDENCE_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_RECOMMENDATION_SESSIONS_EPISODE_INDEX)
            .await?;

        // Create legacy embeddings indexes
        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_ITEM_INDEX)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_VECTOR_INDEX)
                .await?;
        }

        // Create monitoring indexes
        self.execute_with_retry(&conn, schema::CREATE_EXECUTION_RECORDS_TIME_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EXECUTION_RECORDS_AGENT_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_AGENT_METRICS_TYPE_INDEX)
            .await?;

        // Create Phase 2 (GENESIS) tables and indexes
        self.execute_with_retry(&conn, schema::CREATE_EPISODE_SUMMARIES_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_SUMMARIES_CREATED_AT_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_METADATA_TABLE)
            .await?;

        // Create Episode Tags tables and indexes
        self.execute_with_retry(&conn, schema::CREATE_EPISODE_TAGS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODE_TAGS_TAG_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODE_TAGS_EPISODE_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_TAG_METADATA_TABLE)
            .await?;

        // Create Episode Relationships table and indexes
        self.execute_with_retry(&conn, schema::CREATE_EPISODE_RELATIONSHIPS_TABLE)
            .await?;
        self.ensure_relationships_weight_column(&conn).await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODE_PATTERN_RELATIONSHIPS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODE_PATTERN_REL_EPISODE_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODE_PATTERN_REL_PATTERN_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_PROCEDURAL_MEMORY_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_PROCEDURAL_MEMORY_NAME_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_PROCEDURAL_MEMORY_UPDATED_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_RELATIONSHIPS_FROM_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_RELATIONSHIPS_TO_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_RELATIONSHIPS_TYPE_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_RELATIONSHIPS_BIDIRECTIONAL_INDEX)
            .await?;

        // Create FTS5 tables for hybrid search (feature-gated)
        #[cfg(feature = "hybrid_search")]
        self.initialize_fts5_schema(&conn).await?;

        // Create dimension-specific vector tables (Phase 0)
        #[cfg(feature = "turso_multi_dimension")]
        self.initialize_vector_tables(&conn).await?;

        info!("Schema initialization complete");
        Ok(())
    }

    /// Initialize FTS5 schema for hybrid search
    #[cfg(feature = "hybrid_search")]
    async fn initialize_fts5_schema(&self, conn: &libsql::Connection) -> Result<()> {
        use crate::fts5_schema;
        info!("Initializing FTS5 schema for hybrid search");
        self.execute_with_retry(conn, fts5_schema::CREATE_EPISODES_FTS_TABLE)
            .await?;
        self.execute_with_retry(conn, fts5_schema::CREATE_PATTERNS_FTS_TABLE)
            .await?;
        self.execute_with_retry(conn, fts5_schema::CREATE_EPISODES_FTS_TRIGGERS)
            .await?;
        self.execute_with_retry(conn, fts5_schema::CREATE_PATTERNS_FTS_TRIGGERS)
            .await?;
        info!("FTS5 schema initialization complete");
        Ok(())
    }

    #[cfg(not(feature = "hybrid_search"))]
    #[allow(dead_code)] // Feature-gated stub: empty implementation when hybrid_search disabled
    async fn initialize_fts5_schema(&self, _conn: &libsql::Connection) -> Result<()> {
        Ok(())
    }

    /// Initialize dimension-specific vector tables
    #[cfg(feature = "turso_multi_dimension")]
    async fn initialize_vector_tables(&self, conn: &libsql::Connection) -> Result<()> {
        info!("Initializing dimension-specific vector tables");

        // Create tables
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_384_TABLE)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_1024_TABLE)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_1536_TABLE)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_3072_TABLE)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_OTHER_TABLE)
            .await?;

        // Create vector indexes
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_384_VECTOR_INDEX)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_1024_VECTOR_INDEX)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_1536_VECTOR_INDEX)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_3072_VECTOR_INDEX)
            .await?;

        // Create item indexes
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_384_ITEM_INDEX)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_1024_ITEM_INDEX)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_1536_ITEM_INDEX)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_3072_ITEM_INDEX)
            .await?;
        self.execute_with_retry(conn, schema::CREATE_EMBEDDINGS_OTHER_ITEM_INDEX)
            .await?;

        info!("Dimension-specific vector tables initialized");
        Ok(())
    }

    #[cfg(not(feature = "turso_multi_dimension"))]
    #[allow(dead_code)] // Feature-gated stub: empty implementation when turso_multi_dimension disabled
    async fn initialize_vector_tables(&self, _conn: &libsql::Connection) -> Result<()> {
        Ok(())
    }

    /// Ensure the episodes.checkpoints column exists for backward compatibility.
    async fn ensure_episodes_checkpoints_column(&self, conn: &libsql::Connection) -> Result<()> {
        let mut rows = conn
            .query("PRAGMA table_info(episodes)", ())
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to inspect episodes schema: {}", e))
            })?;

        let mut has_checkpoints = false;
        while let Some(row) = rows.next().await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to read episodes schema row: {}", e))
        })? {
            let column_name: String = row.get(1).map_err(|e| {
                do_memory_core::Error::Storage(format!(
                    "Failed to parse episodes schema column name: {}",
                    e
                ))
            })?;

            if column_name == "checkpoints" {
                has_checkpoints = true;
                break;
            }
        }

        if !has_checkpoints {
            debug!("Adding missing episodes.checkpoints column");
            self.execute_with_retry(conn, schema::ADD_EPISODES_CHECKPOINTS_COLUMN)
                .await?;
        }

        Ok(())
    }

    /// Ensure the episode_relationships.weight column exists.
    async fn ensure_relationships_weight_column(&self, conn: &libsql::Connection) -> Result<()> {
        let mut rows = conn
            .query("PRAGMA table_info(episode_relationships)", ())
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!(
                    "Failed to inspect episode_relationships schema: {}",
                    e
                ))
            })?;

        let mut has_weight = false;
        while let Some(row) = rows.next().await.map_err(|e| {
            do_memory_core::Error::Storage(format!(
                "Failed to read episode_relationships schema row: {}",
                e
            ))
        })? {
            let column_name: String = row.get(1).map_err(|e| {
                do_memory_core::Error::Storage(format!(
                    "Failed to parse episode_relationships schema column name: {}",
                    e
                ))
            })?;

            if column_name == "weight" {
                has_weight = true;
                break;
            }
        }

        if !has_weight {
            debug!("Adding missing episode_relationships.weight column");
            self.execute_with_retry(conn, schema::ADD_RELATIONSHIPS_WEIGHT_COLUMN)
                .await?;
        }

        Ok(())
    }
}
