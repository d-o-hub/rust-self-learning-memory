// Prototype routing logic for multi-dimension embedding storage
// This file shows the proposed changes to memory-storage-turso/src/storage.rs

// ========== HELPER FUNCTION ==========

/// Get table name and native vector support flag for a given dimension
fn get_table_for_dimension(dimension: usize) -> (&'static str, bool) {
    match dimension {
        384 => ("embeddings_384", true),
        1024 => ("embeddings_1024", true),
        1536 => ("embeddings_1536", true),
        3072 => ("embeddings_3072", true),
        _ => ("embeddings_other", false),
    }
}

/// Get the appropriate vector index name for a dimension
fn get_vector_index_for_dimension(dimension: usize) -> Option<&'static str> {
    match dimension {
        384 => Some("idx_embeddings_384_vector"),
        1024 => Some("idx_embeddings_1024_vector"),
        1536 => Some("idx_embeddings_1536_vector"),
        3072 => Some("idx_embeddings_3072_vector"),
        _ => None,
    }
}

// ========== UPDATED store_embedding METHOD ==========

/// Store an embedding in the appropriate dimension-specific table
async fn store_embedding(
    &self,
    item_id: &str,
    item_type: &str,
    embedding: &[f32],
) -> Result<()> {
    let dimension = embedding.len();
    let (table_name, native_support) = get_table_for_dimension(dimension);
    
    let embedding_id = format!("{}_{}", item_type, item_id);
    let embedding_json = serde_json::to_string(embedding).map_err(Error::Serialization)?;
    
    let conn = self.get_connection().await?;
    
    let sql = if native_support {
        // Convert embedding to vector string for native storage
        let vector_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        format!(
            r#"
            INSERT OR REPLACE INTO {} (
                embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model
            ) VALUES (?, ?, ?, ?, vector32(?), ?, ?)
            "#,
            table_name
        )
    } else {
        // Store as BLOB for non-native dimensions
        let blob_data = embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect::<Vec<u8>>();
        
        format!(
            r#"
            INSERT OR REPLACE INTO {} (
                embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            table_name
        )
    };
    
    // Execute with appropriate parameters
    if native_support {
        let vector_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        conn.execute(
            &sql,
            params![
                embedding_id,
                item_id,
                item_type,
                embedding_json,
                vector_str,
                dimension as i64,
                "unknown",
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store embedding: {}", e)))?;
    } else {
        let blob_data = embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect::<Vec<u8>>();
        
        conn.execute(
            &sql,
            params![
                embedding_id,
                item_id,
                item_type,
                embedding_json,
                blob_data,
                dimension as i64,
                "unknown",
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store embedding: {}", e)))?;
    }
    
    Ok(())
}

// ========== UPDATED find_similar_episodes_native METHOD ==========

/// Find similar episodes using dimension-specific native vector search
async fn find_similar_episodes_native(
    &self,
    conn: &libsql::Connection,
    query_embedding: &[f32],
    limit: usize,
    threshold: f32,
) -> Result<Vec<SimilaritySearchResult<Episode>>> {
    let dimension = query_embedding.len();
    let (table_name, native_support) = get_table_for_dimension(dimension);
    
    // If no native support for this dimension, return error to trigger fallback
    if !native_support {
        return Err(Error::Storage(
            "No native vector support for this dimension".to_string(),
        ));
    }
    
    // Get vector index name
    let vector_index = get_vector_index_for_dimension(dimension)
        .expect("Native support but no vector index");
    
    // Convert embedding to vector string for SQL
    let vector_str = format!(
        "[{}]",
        query_embedding
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    
    // Use Turso's native vector_top_k function with dimension-specific index
    let sql = format!(
        r#"
        SELECT
            e.item_id,
            e.item_type,
            e.model,
            e.created_at,
            vt.distance,
            (2 - vt.distance) / 2 AS similarity
        FROM vector_top_k('{}', vector32(?1), ?2) vt
        JOIN {} e ON e.rowid = vt.id
        WHERE e.item_type = 'episode'
          AND (2 - vt.distance) / 2 >= ?3
        ORDER BY similarity DESC
        LIMIT ?4
        "#,
        vector_index, table_name
    );
    
    // Request more results to filter by threshold
    let fetch_limit = limit * 2;
    
    let mut rows = conn
        .query(
            &sql,
            params![vector_str, fetch_limit as i64, threshold, limit as i64],
        )
        .await
        .map_err(|e| {
            // Check if this is a "no such table" or "no such index" error
            // indicating the migration hasn't been applied
            if e.to_string().contains("no such index")
                || e.to_string().contains("no such table")
                || e.to_string().contains("no such function")
            {
                return Error::Storage(format!("Vector search not available: {}", e));
            }
            Error::Storage(format!("Failed to query similar episodes: {}", e))
        })?;
    
    // ... rest of the method remains the same ...
    
    Ok(results)
}

// ========== UPDATED get_embedding METHOD ==========

/// Retrieve an embedding from the appropriate dimension-specific table
async fn get_embedding(&self, item_id: &str, item_type: &str) -> Result<Option<Vec<f32>>> {
    let embedding_id = format!("{}_{}", item_type, item_id);
    
    // Challenge: We don't know which table contains the embedding
    // Options:
    // 1. Search across all tables (inefficient)
    // 2. Maintain a mapping table (embedding_id -> dimension)
    // 3. Store dimension in embedding_id format (e.g., "episode_{id}_{dim}")
    // 4. Require dimension parameter (breaking change)
    
    // Implementation option 1: Search all tables
    let tables = [
        "embeddings_384",
        "embeddings_1024",
        "embeddings_1536",
        "embeddings_3072",
        "embeddings_other",
        "embeddings", // Legacy table for backward compatibility
    ];
    
    for table in tables.iter() {
        let sql = format!("SELECT embedding_data FROM {} WHERE embedding_id = ?", table);
        
        let conn = self.get_connection().await?;
        let mut rows = conn
            .query(&sql, params![embedding_id])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query embedding: {}", e)))?;
        
        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch embedding row: {}", e)))?
        {
            let embedding_json: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to get embedding_data: {}", e)))?;
            let embedding: Vec<f32> =
                serde_json::from_str(&embedding_json).map_err(Error::Serialization)?;
            return Ok(Some(embedding));
        }
    }
    
    Ok(None)
}

// ========== BATCH OPERATIONS ==========

/// Store multiple embeddings in batch, grouping by dimension
async fn store_embeddings_batch_backend(
    &self,
    embeddings: Vec<(String, Vec<f32>)>,
) -> Result<()> {
    if embeddings.is_empty() {
        return Ok(());
    }
    
    // Group embeddings by dimension
    let mut grouped: std::collections::HashMap<usize, Vec<(String, Vec<f32>)>> =
        std::collections::HashMap::new();
    
    for (id, embedding) in embeddings {
        grouped
            .entry(embedding.len())
            .or_insert_with(Vec::new)
            .push((id, embedding));
    }
    
    // Store each group in its dimension-specific table
    for (dimension, group) in grouped {
        let (table_name, native_support) = get_table_for_dimension(dimension);
        
        let conn = self.get_connection().await?;
        
        for (id, embedding) in group {
            // Similar to store_embedding logic but optimized for batch
            // Could use prepared statements or transaction
            // ...
        }
    }
    
    Ok(())
}

// ========== SCHEMA INITIALIZATION UPDATES ==========

// Add to initialize_schema() in lib.rs:
/*
// Create dimension-specific embedding tables
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_384_TABLE).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1024_TABLE).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1536_TABLE).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_3072_TABLE).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_OTHER_TABLE).await?;

// Create vector indexes for dimension-specific tables
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_384_VECTOR_INDEX).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1024_VECTOR_INDEX).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1536_VECTOR_INDEX).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_3072_VECTOR_INDEX).await?;

// Create item indexes for dimension-specific tables
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_384_ITEM_INDEX).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1024_ITEM_INDEX).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1536_ITEM_INDEX).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_3072_ITEM_INDEX).await?;
self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_OTHER_ITEM_INDEX).await?;
*/