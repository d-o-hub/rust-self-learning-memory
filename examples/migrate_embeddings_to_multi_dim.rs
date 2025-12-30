//! Migration script to transfer embeddings from legacy schema to multi-dimension schema
//!
//! This script reads embeddings from the legacy `embeddings` table and migrates them
//! to the appropriate dimension-specific tables (embeddings_384, embeddings_1536, etc.).
//!
//! Usage:
//!   cargo run --example migrate_embeddings_to_multi_dim --features turso_multi_dimension
//!
//! Environment variables:
//!   TURSO_DB_URL - Database URL (default: file:memory.db)
//!   TURSO_AUTH_TOKEN - Auth token (default: empty)

use anyhow::Result;
use libsql::{params, Builder};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("Starting migration to multi-dimension schema...");

    let db_url = std::env::var("TURSO_DB_URL").unwrap_or_else(|_| "file:memory.db".to_string());
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").unwrap_or_default();

    println!("Connecting to database: {}", db_url);

    let db = if auth_token.is_empty() {
        Builder::new_local(&db_url).build().await?
    } else {
        Builder::new_remote(db_url, auth_token).build().await?
    };

    let conn = db.connect()?;

    // Read from old embeddings table
    let sql = "SELECT embedding_id, embedding_data, model FROM embeddings";
    let mut rows = conn.query(sql, ()).await?;

    let mut migrated_384 = 0;
    let mut migrated_1024 = 0;
    let mut migrated_1536 = 0;
    let mut migrated_3072 = 0;
    let mut migrated_other = 0;
    let mut errors = 0;

    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let embedding_json: String = row.get(1)?;
        let model: String = row.get(2)?;

        let embedding: Vec<f32> = serde_json::from_str(&embedding_json)?;
        let dimension = embedding.len();

        let table_name = match dimension {
            384 => {
                migrated_384 += 1;
                "embeddings_384"
            }
            1024 => {
                migrated_1024 += 1;
                "embeddings_1024"
            }
            1536 => {
                migrated_1536 += 1;
                "embeddings_1536"
            }
            3072 => {
                migrated_3072 += 1;
                "embeddings_3072"
            }
            _ => {
                migrated_other += 1;
                "embeddings_other"
            }
        };

        let vector_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let insert_sql = if [384, 1024, 1536, 3072].contains(&dimension) {
            format!(
                "INSERT INTO {} (embedding_id, item_id, item_type, embedding_data, embedding_vector, model) 
                 VALUES (?, ?, 'unknown', ?, vector32(?), ?)",
                table_name
            )
        } else {
            format!(
                "INSERT INTO {} (embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model) 
                 VALUES (?, ?, 'unknown', ?, NULL, ?, ?)",
                table_name
            )
        };

        let params = if [384, 1024, 1536, 3072].contains(&dimension) {
            params![
                id.clone(),
                id.clone(),
                embedding_json.clone(),
                vector_str,
                model.clone(),
            ]
        } else {
            params![
                id.clone(),
                id.clone(),
                embedding_json.clone(),
                dimension as i64,
                model.clone(),
            ]
        };

        if let Err(e) = conn.execute(&insert_sql, params).await {
            eprintln!("Failed to migrate {}: {}", id, e);
            errors += 1;
        }
    }

    println!("Migration complete:");
    println!("  - 384-dim: {}", migrated_384);
    println!("  - 1024-dim: {}", migrated_1024);
    println!("  - 1536-dim: {}", migrated_1536);
    println!("  - 3072-dim: {}", migrated_3072);
    println!("  - Other: {}", migrated_other);
    println!("  - Errors: {}", errors);

    Ok(())
}
