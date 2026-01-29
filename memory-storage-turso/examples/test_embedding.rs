#![allow(clippy::expect_used)]

use tempfile::TempDir;

#[tokio::main]
async fn main() {
    let dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = dir.path().join("test.db");

    // Use Builder::new_local for file-based databases
    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .expect("Failed to build database");

    let conn = db.connect().expect("Failed to connect to database");

    // Create table with F32_BLOB(384)
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS embeddings (
            embedding_id TEXT PRIMARY KEY NOT NULL,
            item_id TEXT NOT NULL,
            item_type TEXT NOT NULL,
            embedding_data TEXT NOT NULL,
            embedding_vector F32_BLOB(384),
            dimension INTEGER NOT NULL,
            model TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )
        "#,
        (),
    )
    .await
    .expect("Failed to create embeddings table");

    // Try to insert 1024 dimension embedding
    let embedding: Vec<f32> = (0..1024).map(|i| i as f32 / 1024.0).collect();
    let embedding_json = serde_json::to_string(&embedding).expect("Failed to serialize embedding");

    println!("Inserting 1024-dimension embedding...");
    let result = conn.execute(
        "INSERT INTO embeddings (embedding_id, item_id, item_type, embedding_data, dimension, model) VALUES (?, ?, ?, ?, ?, ?)",
        libsql::params!["test_1024", "dim_1024", "embedding", embedding_json, 1024i64, "default"],
    ).await;

    match result {
        Ok(_) => println!("Insert succeeded!"),
        Err(e) => println!("Insert failed: {}", e),
    }

    // Try to retrieve
    let mut rows = conn
        .query(
            "SELECT embedding_data FROM embeddings WHERE item_id = ? AND item_type = ?",
            libsql::params!["dim_1024", "embedding"],
        )
        .await
        .expect("Failed to query embeddings");

    if let Some(row) = rows.next().await.expect("Failed to get next row") {
        let data: String = row.get(0).expect("Failed to get embedding data");
        let parsed: Vec<f32> = serde_json::from_str(&data).expect("Failed to parse embedding JSON");
        println!("Retrieved embedding with {} dimensions", parsed.len());
    } else {
        println!("No embedding found!");
    }
}
