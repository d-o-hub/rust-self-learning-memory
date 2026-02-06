//! Vector similarity search operations for Turso

#![allow(unexpected_cfgs)]

#[allow(unused_imports)]
use super::episodes::row_to_episode;
#[allow(unused_imports)]
use super::patterns::row_to_pattern;
use crate::TursoStorage;
use libsql::Connection;
use memory_core::{
    embeddings::{cosine_similarity, SimilarityMetadata, SimilaritySearchResult},
    Episode, Error, Pattern, Result,
};
use tracing::{debug, info};

impl TursoStorage {
    /// Find similar episodes using native vector search (if available)
    #[allow(clippy::excessive_nesting, unused)]
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
            let _vector_index = self.get_vector_index_for_dimension(query_embedding.len());

            // SAFETY: Table name comes from a fixed whitelist in get_embedding_table_for_dimension().
            // The function only returns predefined table names (embeddings_384, embeddings_1024,
            // embeddings_1536, embeddings_3072, or embeddings_other) based on dimension size.
            // No user input can influence the table name, preventing SQL injection.
            // CodeQL may flag this as a potential SQL injection, but it is a false positive.
            #[allow(clippy::literal_string_with_formatting_args)]
            let check_sql = format!("SELECT COUNT(*) FROM {}", table);
            let mut check_rows = conn
                .query(&check_sql, ())
                .await
                .map_err(|e| Error::Storage(format!("Failed to check table: {}", e)))?;

            if let Some(row) = check_rows
                .next()
                .await
                .map_err(|e| Error::Storage(e.to_string()))?
            {
                let count: i64 = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                if count == 0 {
                    debug!("No embeddings in table {}, skipping", table);
                    return Err(Error::Storage("No embeddings found".to_string()));
                }
            }

            // Query episodes with their embeddings and compute similarity
            // SAFETY: Table name comes from a fixed whitelist in get_embedding_table_for_dimension().
            // The function only returns predefined table names based on dimension size.
            // No user input can influence the table name, preventing SQL injection.
            // CodeQL may flag this as a potential SQL injection, but it is a false positive.
            #[allow(clippy::literal_string_with_formatting_args)]
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

            let mut results = Vec::new();
            let query_emb = query_embedding.to_vec();

            while let Some(row) = episode_rows
                .next()
                .await
                .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
            {
                let episode = row_to_episode(&row)?;

                // Get embedding for this episode
                // SAFETY: Table name comes from a fixed whitelist in get_embedding_table_for_dimension().
                // The function only returns predefined table names based on dimension size.
                // No user input can influence the table name, preventing SQL injection.
                // CodeQL may flag this as a potential SQL injection, but it is a false positive.
                #[allow(clippy::literal_string_with_formatting_args)]
                let embedding_sql = format!(
                    "SELECT embedding FROM {} WHERE item_id = ? AND item_type = 'episode'",
                    table
                );
                let mut emb_rows = conn
                    .query(
                        &embedding_sql,
                        libsql::params![episode.episode_id.to_string()],
                    )
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to query embedding: {}", e)))?;

                if let Some(emb_row) = emb_rows
                    .next()
                    .await
                    .map_err(|e| Error::Storage(e.to_string()))?
                {
                    let embedding_json: String =
                        emb_row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                    let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                        .map_err(|e| Error::Storage(format!("Failed to parse embedding: {}", e)))?;

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

            info!("Found {} similar episodes", results.len());
            Ok(results)
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            Err(Error::Storage(
                "Native vector search requires turso_multi_dimension feature".to_string(),
            ))
        }
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

    /// Find similar patterns using native vector search
    #[allow(clippy::excessive_nesting, unused)]
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

            let patterns_sql = format!(
                r#"
                SELECT p.pattern_id, p.pattern_type, p.description, p.context,
                       p.heuristic, p.success_rate, p.occurrence_count,
                       p.created_at, p.updated_at, p.domain, p.language, p.metadata
                FROM patterns p
                INNER JOIN {} et ON p.pattern_id = et.item_id::TEXT
                WHERE et.item_type = 'pattern'
            "#,
                table
            );

            let mut pattern_rows = conn
                .query(&patterns_sql, ())
                .await
                .map_err(|e| Error::Storage(format!("Failed to query patterns: {}", e)))?;

            let mut results = Vec::new();
            let query_emb = query_embedding.to_vec();

            while let Some(row) = pattern_rows
                .next()
                .await
                .map_err(|e| Error::Storage(format!("Failed to fetch pattern row: {}", e)))?
            {
                let pattern = row_to_pattern(&row)?;

                let embedding_sql = format!(
                    "SELECT embedding FROM {} WHERE item_id = ? AND item_type = 'pattern'",
                    table
                );
                let mut emb_rows = conn
                    .query(&embedding_sql, libsql::params![pattern.id().to_string()])
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to query embedding: {}", e)))?;

                if let Some(emb_row) = emb_rows
                    .next()
                    .await
                    .map_err(|e| Error::Storage(e.to_string()))?
                {
                    let embedding_json: String =
                        emb_row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                    let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                        .map_err(|e| Error::Storage(format!("Failed to parse embedding: {}", e)))?;

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

            info!("Found {} similar patterns", results.len());
            Ok(results)
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            Err(Error::Storage(
                "Native vector search requires turso_multi_dimension feature".to_string(),
            ))
        }
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
