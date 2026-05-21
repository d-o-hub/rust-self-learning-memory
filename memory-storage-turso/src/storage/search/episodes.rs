//! Episode similarity search operations

use super::super::episodes::row_to_episode;
use crate::TursoStorage;
use do_memory_core::{
    Episode, Error, Result,
    embeddings::{SimilarityMetadata, SimilaritySearchResult, cosine_similarity},
};
use libsql::Connection;
use tracing::{debug, info, warn};

impl TursoStorage {
    /// Find similar episodes using native vector_top_k search (DiskANN-accelerated)
    ///
    /// This uses Turso's native `vector_top_k()` function with DiskANN index
    /// for O(log n) similarity search instead of O(n) brute-force.
    ///
    /// Requires:
    /// - `embedding_vector` column to be populated (via migration)
    /// - DiskANN index created on the embedding table
    ///
    /// Falls back to brute-force if vector column not populated.
    #[allow(clippy::excessive_nesting)]
    pub(crate) async fn find_similar_episodes_native(
        &self,
        conn: &Connection,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        debug!(
            "Finding similar episodes using native vector search (limit={}, threshold={})",
            limit, threshold
        );

        #[cfg(feature = "turso_multi_dimension")]
        {
            let table = self.get_embedding_table_for_dimension(query_embedding.len());
            let vector_index = self.get_vector_index_for_dimension(query_embedding.len());

            // Try native vector_top_k search if index exists for this dimension
            if let Some(index_name) = vector_index {
                match self
                    .find_episodes_vector_top_k(
                        conn,
                        table,
                        index_name,
                        query_embedding,
                        limit,
                        threshold,
                    )
                    .await
                {
                    Ok(results) => {
                        info!("Found {} episodes via vector_top_k", results.len());
                        return Ok(results);
                    }
                    Err(e) => {
                        // Log warning and fall through to brute-force
                        warn!(
                            "vector_top_k failed (vector column may not be populated): {}",
                            e
                        );
                    }
                }
            }

            // Fall back to brute-force if vector_top_k not available
            debug!("Falling back to brute-force search for episodes");
            self.find_similar_episodes_brute_force(query_embedding, limit, threshold)
                .await
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            // Try native vector_top_k with single embedding table
            match self
                .find_episodes_vector_top_k_single(conn, query_embedding, limit, threshold)
                .await
            {
                Ok(results) => {
                    info!("Found {} episodes via vector_top_k", results.len());
                    return Ok(results);
                }
                Err(e) => {
                    warn!("vector_top_k failed: {}", e);
                }
            }

            // Fall back to brute-force
            debug!("Falling back to brute-force search for episodes");
            self.find_similar_episodes_brute_force(query_embedding, limit, threshold)
                .await
        }
    }

    /// Native vector_top_k search for multi-dimension setup
    #[cfg(feature = "turso_multi_dimension")]
    async fn find_episodes_vector_top_k(
        &self,
        conn: &Connection,
        table: &str,
        index_name: &str,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        // Convert embedding to JSON array for vector32() function
        let query_json = serde_json::to_string(query_embedding)
            .map_err(|e| Error::Storage(format!("Failed to serialize embedding: {}", e)))?;

        // Use vector_top_k table-valued function for O(log n) DiskANN search
        // SAFETY: Table and index names come from fixed whitelists in get_embedding_table_for_dimension
        // and get_vector_index_for_dimension. No user input can influence these names.
        #[allow(clippy::literal_string_with_formatting_args)]
        let sql = format!(
            r#"
            SELECT
                e.episode_id, e.task_type, e.task_description, e.context,
                e.start_time, e.end_time, e.steps, e.outcome, e.reward,
                e.reflection, e.patterns, e.heuristics, e.metadata, e.domain, e.language,
                v.distance
            FROM vector_top_k(?, vector32(?), ?) AS v
            JOIN {} emb ON emb.rowid = v.id
            JOIN episodes e ON e.episode_id = emb.item_id
            WHERE emb.item_type = 'episode' AND v.distance <= ?
            "#,
            table
        );

        let mut rows = conn
            .query(
                &sql,
                libsql::params![index_name, query_json, limit as i64, 1.0 - threshold],
            )
            .await
            .map_err(|e| Error::Storage(format!("vector_top_k query failed: {}", e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            let episode = row_to_episode(&row)?;
            // vector_top_k returns distance, convert to similarity (1 - distance for cosine)
            let distance: f64 = row.get(16).map_err(|e| Error::Storage(e.to_string()))?;
            let similarity = (1.0 - distance) as f32;

            if similarity >= threshold {
                results.push(SimilaritySearchResult {
                    item: episode,
                    similarity,
                    metadata: SimilarityMetadata {
                        embedding_model: "turso-native".to_string(),
                        embedding_timestamp: None,
                        context: serde_json::json!({
                            "dimension": query_embedding.len(),
                            "search_method": "vector_top_k"
                        }),
                    },
                });
            }
        }

        Ok(results)
    }

    /// Native vector_top_k search for single embedding table
    #[cfg(not(feature = "turso_multi_dimension"))]
    async fn find_episodes_vector_top_k_single(
        &self,
        conn: &Connection,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        // Convert embedding to JSON array for vector32() function
        let query_json = serde_json::to_string(query_embedding)
            .map_err(|e| Error::Storage(format!("Failed to serialize embedding: {}", e)))?;

        let sql = r#"
            SELECT
                e.episode_id, e.task_type, e.task_description, e.context,
                e.start_time, e.end_time, e.steps, e.outcome, e.reward,
                e.reflection, e.patterns, e.heuristics, e.metadata, e.domain, e.language,
                v.distance
            FROM vector_top_k('idx_embeddings_vector', vector32(?), ?) AS v
            JOIN embeddings emb ON emb.rowid = v.id
            JOIN episodes e ON e.episode_id = emb.item_id
            WHERE emb.item_type = 'episode' AND v.distance <= ?
        "#;

        let mut rows = conn
            .query(
                sql,
                libsql::params![query_json, limit as i64, 1.0 - threshold],
            )
            .await
            .map_err(|e| Error::Storage(format!("vector_top_k query failed: {}", e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            let episode = row_to_episode(&row)?;
            let distance: f64 = row.get(16).map_err(|e| Error::Storage(e.to_string()))?;
            let similarity = (1.0 - distance) as f32;

            if similarity >= threshold {
                results.push(SimilaritySearchResult {
                    item: episode,
                    similarity,
                    metadata: SimilarityMetadata {
                        embedding_model: "turso-native".to_string(),
                        embedding_timestamp: None,
                        context: serde_json::json!({
                            "dimension": query_embedding.len(),
                            "search_method": "vector_top_k"
                        }),
                    },
                });
            }
        }

        Ok(results)
    }

    /// Find similar episodes using brute-force search (fallback)
    #[allow(clippy::excessive_nesting)]
    pub(crate) async fn find_similar_episodes_brute_force(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        debug!(
            "Finding similar episodes using brute-force search (limit={}, threshold={})",
            limit, threshold
        );

        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let mut matched_ids = Vec::new();

        #[cfg(feature = "turso_multi_dimension")]
        {
            let dimensions = [384, 1024, 1536, 3072];
            let mut seen_ids = std::collections::HashSet::new();

            for dim in dimensions {
                let table = self.get_embedding_table_for_dimension(dim);
                let sql = format!(
                    "SELECT item_id, embedding_data FROM {} WHERE item_type = 'episode'",
                    table
                );

                let mut rows = conn
                    .query(&sql, ())
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to query embeddings: {}", e)))?;

                while let Some(row) = rows
                    .next()
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to fetch embedding row: {}", e)))?
                {
                    let item_id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                    if !seen_ids.insert(item_id.clone()) {
                        continue;
                    }

                    let embedding_data: String =
                        row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                    let embedding = self.decode_embedding_data(&embedding_data)?;
                    let similarity = cosine_similarity(query_embedding, &embedding);

                    if similarity >= threshold {
                        matched_ids.push((item_id, similarity));
                    }
                }
            }
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            const SQL: &str =
                "SELECT item_id, embedding_data FROM embeddings WHERE item_type = 'episode'";
            let mut rows = conn
                .query(SQL, ())
                .await
                .map_err(|e| Error::Storage(format!("Failed to query embeddings: {}", e)))?;

            while let Some(row) = rows
                .next()
                .await
                .map_err(|e| Error::Storage(format!("Failed to fetch embedding row: {}", e)))?
            {
                let item_id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let embedding_data: String =
                    row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let embedding = self.decode_embedding_data(&embedding_data)?;
                let similarity = cosine_similarity(query_embedding, &embedding);

                if similarity >= threshold {
                    matched_ids.push((item_id, similarity));
                }
            }
        }

        // Optimization: select top-k matches before full episode retrieval
        let top_matches = do_memory_core::search::select_top_k(&mut matched_ids, limit, |a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        if top_matches.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch full episode data for the top matches in a single query
        let placeholders = top_matches
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let select_cols = crate::storage::episodes::raw_query::EPISODE_SELECT_COLUMNS;
        let episodes_sql = format!(
            "SELECT {} FROM episodes WHERE episode_id IN ({})",
            select_cols, placeholders
        );

        let params: Vec<libsql::Value> = top_matches
            .iter()
            .map(|(id, _)| id.clone().into())
            .collect();

        let mut episode_rows = conn
            .query(&episodes_sql, libsql::params_from_iter(params))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query top episodes: {}", e)))?;

        let mut episodes_map = std::collections::HashMap::new();
        while let Some(row) = episode_rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            let episode = row_to_episode(&row)?;
            episodes_map.insert(episode.episode_id.to_string(), episode);
        }

        let mut results = Vec::with_capacity(top_matches.len());
        for (item_id, similarity) in top_matches {
            if let Some(episode) = episodes_map.remove(&item_id) {
                results.push(SimilaritySearchResult {
                    item: episode,
                    similarity,
                    metadata: SimilarityMetadata {
                        embedding_model: "turso".to_string(),
                        embedding_timestamp: None,
                        context: serde_json::json!({}),
                    },
                });
            }
        }

        info!(
            "Found {} similar episodes (brute-force optimized)",
            results.len()
        );
        Ok(results)
    }
}
