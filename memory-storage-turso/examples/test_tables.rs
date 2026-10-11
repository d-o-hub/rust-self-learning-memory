//! Test which tables are created during schema initialization

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::expect_used)]
#![allow(clippy::ignored_unit_patterns)]

use do_memory_storage_turso::TursoStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = libsql::Builder::new_local(":memory:").build().await?;
    let storage = TursoStorage::from_database(db)?;

    println!("Initializing schema...");
    match storage.initialize_schema().await {
        Ok(_) => println!("Schema initialized successfully"),
        Err(e) => println!("Schema initialization failed: {}", e),
    }

    storage
        .with_connection(async |conn| {
            let mut tables = conn
                .query(
                    "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;",
                    (),
                )
                .await
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;

            println!("Tables created:");
            let mut count = 0;
            while let Ok(Some(row)) = tables.next().await {
                let name: String = row
                    .get(0)
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
                println!("  - {}", name);
                count += 1;
            }

            if count == 0 {
                println!("  (No tables found!)");
            }

            Ok(())
        })
        .await?;

    Ok(())
}
