//! Direct test of CREATE_EMBEDDINGS_TABLE SQL

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = libsql::Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    println!("Creating embeddings table...");
    let create_sql = r#"
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
"#;

    match conn.execute(create_sql, ()).await {
        Ok(_) => println!("Table created successfully"),
        Err(e) => println!("Failed to create table: {}", e),
    }

    let mut tables = conn
        .query(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;",
            (),
        )
        .await?;

    println!("Tables created:");
    let mut count = 0;
    while let Ok(Some(row)) = tables.next().await {
        let name: String = row.get(0)?;
        println!("  - {}", name);
        count += 1;
    }

    if count == 0 {
        println!("  (No tables found!)");
    }

    Ok(())
}
