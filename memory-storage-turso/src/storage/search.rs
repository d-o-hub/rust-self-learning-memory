//! Vector similarity search operations for Turso

#![allow(unexpected_cfgs)]

#[allow(unused_imports)]
use super::episodes::row_to_episode;
#[allow(unused_imports)]
use super::patterns::row_to_pattern;
use crate::TursoStorage;
use do_memory_core::{
    Episode, Error, Pattern, Result,
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

        // Get all episodes with their embeddings
        let conn = self.get_connection().await?;

        #[cfg(feature = "turso_multi_dimension")]
        {
            // For multi-dimension, we need to search across all dimension tables
            let dimensions = [384, 1024, 1536, 3072];
            let mut all_results: Vec<SimilaritySearchResult<Episode>> = Vec::new();
            let mut seen_ids = std::collections::HashSet::new();

            for dim in dimensions {
                let table = self.get_embedding_table_for_dimension(dim);
                let episodes_sql = format!(
                    r#"
                    SELECT e.episode_id, e.task_type, e.task_description, e.context,
                           e.start_time, e.end_time, e.steps, e.outcome, e.reward,
                           e.reflection, e.patterns, e.heuristics, e.metadata, e.domain, e.language
                    FROM episodes e
                    INNER JOIN {} et ON e.episode_id = et.item_id
                    WHERE et.item_type = 'episode'
                "#,
                    table
                );

                let mut episode_rows = conn
                    .query(&episodes_sql, ())
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

                let query_emb = query_embedding.to_vec();

                while let Some(row) = episode_rows
                    .next()
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
                {
                    let episode_id: String =
                        row.get(0).map_err(|e| Error::Storage(e.to_string()))?;

                    if seen_ids.insert(episode_id.clone()) {
                        if let Some(episode) = self
                            .get_episode(
                                uuid::Uuid::parse_str(&episode_id)
                                    .map_err(|e| Error::Storage(e.to_string()))?,
                            )
                            .await?
                        {
                            if let Some(embedding) = self
                                ._get_embedding_internal(&episode.episode_id.to_string(), "episode")
                                .await?
                            {
                                let similarity = cosine_similarity(&query_emb, &embedding);

                                if similarity >= threshold {
                                    all_results.push(SimilaritySearchResult {
                                        item: episode,
                                        similarity,
                                        metadata: SimilarityMetadata {
                                            embedding_model: "turso".to_string(),
                                            embedding_timestamp: None,
                                            context: serde_json::json!({ "dimension": embedding.len() }),
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Sort by similarity and limit
            all_results.sort_by(|a, b| {
                b.similarity
                    .partial_cmp(&a.similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            all_results.truncate(limit);

            info!("Found {} similar episodes (brute-force)", all_results.len());
            Ok(all_results)
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            let episodes_sql = r#"
                SELECT e.episode_id, e.task_type, e.task_description, e.context,
                       e.start_time, e.end_time, e.steps, e.outcome, e.reward,
                       e.reflection, e.patterns, e.heuristics, e.metadata, e.domain, e.language
                FROM episodes e
                INNER JOIN embeddings et ON e.episode_id = et.item_id
                WHERE et.item_type = 'episode'
            "#;

            let mut episode_rows = conn
                .query(episodes_sql, ())
                .await
                .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

            let mut results = Vec::new();
            let query_emb = query_embedding.to_vec();

            while let Some(row) = episode_rows
                .next()
                .await
                .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
            {
                let episode = row_to_episode(&row)?;

                if let Some(embedding) = self
                    ._get_embedding_internal(&episode.episode_id.to_string(), "episode")
                    .await?
                {
                    let similarity = cosine_similarity(&query_emb, &embedding);

                    if similarity >= threshold {
                        results.push(SimilaritySearchResult {
                            item: episode,
                            similarity,
                            metadata: SimilarityMetadata {
                                embedding_model: "turso".to_string(),
                                embedding_timestamp: None,
                                context: serde_json::json!({ "dimension": embedding.len() }),
                            },
                        });
                    }
                }
            }

            // Sort by similarity and limit
            results.sort_by(|a, b| {
                b.similarity
                    .partial_cmp(&a.similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            results.truncate(limit);

            info!("Found {} similar episodes (brute-force)", results.len());
            Ok(results)
        }
    }

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

        let patterns = self.query_patterns(&Default::default()).await?;
        let mut results = Vec::new();
        let query_emb = query_embedding.to_vec();

        for pattern in patterns {
            if let Some(embedding) = self
                ._get_embedding_internal(&pattern.id().to_string(), "pattern")
                .await?
            {
                let similarity = cosine_similarity(&query_emb, &embedding);

                if similarity >= threshold {
                    results.push(SimilaritySearchResult {
                        item: pattern,
                        similarity,
                        metadata: SimilarityMetadata {
                            embedding_model: "turso".to_string(),
                            embedding_timestamp: None,
                            context: serde_json::json!({ "dimension": embedding.len() }),
                        },
                    });
                }
            }
        }

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        info!("Found {} similar patterns (brute-force)", results.len());
        Ok(results)
    }
}
