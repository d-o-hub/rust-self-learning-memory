//! Pattern similarity search operations

use super::super::patterns::row_to_pattern;
use crate::TursoStorage;
use do_memory_core::{
    Error, Pattern, Result,
    embeddings::{SimilarityMetadata, SimilaritySearchResult, cosine_similarity},
};
use libsql::Connection;
use tracing::{debug, info, warn};

impl TursoStorage {
    /// Find similar patterns using native vector_top_k search (DiskANN-accelerated)
    ///
    /// This uses Turso's native `vector_top_k()` function with DiskANN index
    /// for O(log n) similarity search instead of O(n) brute-force.
    #[allow(clippy::excessive_nesting)]
    pub(crate) async fn find_similar_patterns_native(
        &self,
        conn: &Connection,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        debug!(
            "Finding similar patterns using native vector search (limit={}, threshold={})",
            limit, threshold
        );

        #[cfg(feature = "turso_multi_dimension")]
        {
            let table = self.get_embedding_table_for_dimension(query_embedding.len());
            let vector_index = self.get_vector_index_for_dimension(query_embedding.len());

            // Try native vector_top_k search if index exists for this dimension
            if let Some(index_name) = vector_index {
                match self
                    .find_patterns_vector_top_k(
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
                        info!("Found {} patterns via vector_top_k", results.len());
                        return Ok(results);
                    }
                    Err(e) => {
                        // Log warning and fall through to brute-force
                        warn!("vector_top_k failed for patterns: {}", e);
                    }
                }
            }

            // Fall back to brute-force if vector_top_k not available
            debug!("Falling back to brute-force search for patterns");
            self.find_similar_patterns_brute_force(query_embedding, limit, threshold)
                .await
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            // Try native vector_top_k with single embedding table
            match self
                .find_patterns_vector_top_k_single(conn, query_embedding, limit, threshold)
                .await
            {
                Ok(results) => {
                    info!("Found {} patterns via vector_top_k", results.len());
                    return Ok(results);
                }
                Err(e) => {
                    warn!("vector_top_k failed for patterns: {}", e);
                }
            }

            // Fall back to brute-force
            debug!("Falling back to brute-force search for patterns");
            self.find_similar_patterns_brute_force(query_embedding, limit, threshold)
                .await
        }
    }

    /// Native vector_top_k search for patterns (multi-dimension)
    #[cfg(feature = "turso_multi_dimension")]
    async fn find_patterns_vector_top_k(
        &self,
        conn: &Connection,
        table: &str,
        index_name: &str,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        let query_json = serde_json::to_string(query_embedding)
            .map_err(|e| Error::Storage(format!("Failed to serialize embedding: {}", e)))?;

        #[allow(clippy::literal_string_with_formatting_args)]
        let sql = format!(
            r#"
            SELECT
                p.pattern_id, p.pattern_type, p.description, p.context,
                p.heuristic, p.success_rate, p.occurrence_count,
                p.created_at, p.updated_at, p.domain, p.language, p.metadata,
                v.distance
            FROM vector_top_k(?, vector32(?), ?) AS v
            JOIN {} emb ON emb.rowid = v.id
            JOIN patterns p ON p.pattern_id = emb.item_id
            WHERE emb.item_type = 'pattern' AND v.distance <= ?
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
            let pattern = row_to_pattern(&row)?;
            let distance: f64 = row.get(12).map_err(|e| Error::Storage(e.to_string()))?;
            let similarity = (1.0 - distance) as f32;

            if similarity >= threshold {
                results.push(SimilaritySearchResult {
                    item: pattern,
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

    /// Native vector_top_k search for patterns (single table)
    #[cfg(not(feature = "turso_multi_dimension"))]
    async fn find_patterns_vector_top_k_single(
        &self,
        conn: &Connection,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        let query_json = serde_json::to_string(query_embedding)
            .map_err(|e| Error::Storage(format!("Failed to serialize embedding: {}", e)))?;

        let sql = r#"
            SELECT
                p.pattern_id, p.pattern_type, p.description, p.context,
                p.heuristic, p.success_rate, p.occurrence_count,
                p.created_at, p.updated_at, p.domain, p.language, p.metadata,
                v.distance
            FROM vector_top_k('idx_embeddings_vector', vector32(?), ?) AS v
            JOIN embeddings emb ON emb.rowid = v.id
            JOIN patterns p ON p.pattern_id = emb.item_id
            WHERE emb.item_type = 'pattern' AND v.distance <= ?
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
            let pattern = row_to_pattern(&row)?;
            let distance: f64 = row.get(12).map_err(|e| Error::Storage(e.to_string()))?;
            let similarity = (1.0 - distance) as f32;

            if similarity >= threshold {
                results.push(SimilaritySearchResult {
                    item: pattern,
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

    /// Find similar patterns using brute-force search
    pub(crate) async fn find_similar_patterns_brute_force(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        debug!(
            "Finding similar patterns using brute-force search (limit={}, threshold={})",
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
                    "SELECT item_id, embedding_data, dimension FROM {} WHERE item_type = 'pattern'",
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
                    let dimension: i64 = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;

                    let embedding = self.decode_embedding_data(&embedding_data)?;
                    let similarity = cosine_similarity(query_embedding, &embedding);

                    if similarity >= threshold {
                        matched_ids.push((item_id, similarity, dimension));
                    }
                }
            }
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            const SQL: &str = "SELECT item_id, embedding_data, dimension FROM embeddings WHERE item_type = 'pattern'";
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
                let dimension: i64 = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;

                let embedding = self.decode_embedding_data(&embedding_data)?;
                let similarity = cosine_similarity(query_embedding, &embedding);

                if similarity >= threshold {
                    matched_ids.push((item_id, similarity, dimension));
                }
            }
        }

        // Optimization: select top-k matches before full pattern retrieval
        let top_matches = do_memory_core::search::select_top_k(&mut matched_ids, limit, |a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        if top_matches.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch full pattern data for the top matches in a single query
        let placeholders = top_matches
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let select_cols = crate::storage::patterns::PATTERN_SELECT_COLUMNS;
        let patterns_sql = format!(
            "SELECT {} FROM patterns WHERE pattern_id IN ({})",
            select_cols, placeholders
        );

        let params: Vec<libsql::Value> = top_matches
            .iter()
            .map(|(id, _, _)| id.clone().into())
            .collect();

        let mut pattern_rows = conn
            .query(&patterns_sql, libsql::params_from_iter(params))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query top patterns: {}", e)))?;

        let mut patterns_map = std::collections::HashMap::new();
        while let Some(row) = pattern_rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch pattern row: {}", e)))?
        {
            let pattern = row_to_pattern(&row)?;
            patterns_map.insert(pattern.id().to_string(), pattern);
        }

        let mut results = Vec::with_capacity(top_matches.len());
        for (item_id, similarity, dimension) in top_matches {
            if let Some(pattern) = patterns_map.remove(&item_id) {
                results.push(SimilaritySearchResult {
                    item: pattern,
                    similarity,
                    metadata: SimilarityMetadata {
                        embedding_model: "turso".to_string(),
                        embedding_timestamp: None,
                        context: serde_json::json!({ "dimension": dimension }),
                    },
                });
            }
        }

        info!(
            "Found {} similar patterns (brute-force optimized)",
            results.len()
        );
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use do_memory_core::embeddings::EmbeddingStorageBackend;
    use do_memory_core::{OutcomeStats, Pattern, TaskContext};
    use tempfile::TempDir;

    async fn create_test_storage() -> Result<(TursoStorage, TempDir)> {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_pattern_search.db");
        let storage = TursoStorage::new(&format!("file:{}", db_path.display()), "").await?;
        storage.initialize_schema().await?;
        Ok((storage, dir))
    }

    #[tokio::test]
    #[ignore = "libsql memory corruption bug - see ADR-027"]
    async fn test_find_similar_patterns_brute_force_direct() -> Result<()> {
        let (storage, _dir) = create_test_storage().await?;

        let pattern = Pattern::DecisionPoint {
            id: uuid::Uuid::new_v4(),
            condition: "test condition".to_string(),
            action: "test action".to_string(),
            outcome_stats: OutcomeStats {
                success_count: 0,
                failure_count: 0,
                total_count: 0,
                avg_duration_secs: 0.0,
            },
            context: TaskContext {
                domain: "test-domain".to_string(),
                ..Default::default()
            },
            effectiveness: Default::default(),
        };
        storage.store_pattern(&pattern).await?;

        let embedding = vec![0.1; 384];
        storage
            .store_pattern_embedding(pattern.id(), embedding.clone())
            .await?;

        let results = storage
            .find_similar_patterns_brute_force(&embedding, 10, 0.5)
            .await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item.id(), pattern.id());

        let dim = results[0]
            .metadata
            .context
            .get("dimension")
            .and_then(|v| v.as_i64())
            .expect("dimension should be in metadata");
        assert_eq!(dim, 384);

        Ok(())
    }
}
