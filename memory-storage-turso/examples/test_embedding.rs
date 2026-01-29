use std::sync::Arc;
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    // Use Builder::new_local for file-based databases
    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

    let conn = db.connect().unwrap();

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
    .unwrap();

    // Try to insert 1024 dimension embedding
    let embedding: Vec<f32> = (0..1024).map(|i| i as f32 / 1024.0).collect();
    let embedding_json = serde_json::to_string(&embedding).unwrap();

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
        .unwrap();

    if let Some(row) = rows.next().await.unwrap() {
        let data: String = row.get(0).unwrap();
        let parsed: Vec<f32> = serde_json::from_str(&data).unwrap();
        println!("Retrieved embedding with {} dimensions", parsed.len());
    } else {
        println!("No embedding found!");
    }
}
