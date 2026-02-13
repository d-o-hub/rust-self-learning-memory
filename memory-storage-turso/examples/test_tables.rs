//! Test which tables are created during schema initialization

use memory_storage_turso::TursoStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = libsql::Builder::new_local(":memory:").build().await?;
    let storage = TursoStorage::from_database(db)?;

    println!("Initializing schema...");
    match storage.initialize_schema().await {
        Ok(_) => println!("Schema initialized successfully"),
        Err(e) => println!("Schema initialization failed: {}", e),
    }

    let conn = storage.get_connection().await?;
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
