//! Turso storage configuration and schema initialization.
//!
//! This module provides schema initialization methods to keep lib.rs under 500 LOC.

use crate::{schema, Result, TursoStorage};
use tracing::info;

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
        self.execute_with_retry(&conn, schema::CREATE_PATTERNS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_HEURISTICS_TABLE)
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    async fn initialize_vector_tables(&self, _conn: &libsql::Connection) -> Result<()> {
        Ok(())
    }
}
