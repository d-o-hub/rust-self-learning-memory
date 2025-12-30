use std::sync::Arc;
use memory_core::StorageBackend;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Storage Analysis ===");

    // Check memory.db (SQLite)
    println!("\n1. Memory.db (SQLite):");
    match memory_storage_turso::TursoStorage::new("file:./data/memory.db", "").await {
        Ok(storage) => {
            let episodes = storage.get_all_episodes().await.unwrap();
            println!("   Episodes: {}", episodes.len());
            for ep in &episodes {
                println!("   - {}: {}", ep.episode_id, ep.task_description);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Check memory.redb
    println!("\n2. Memory.redb:");
    match memory_storage_redb::RedbStorage::new(std::path::Path::new("./data/memory.redb")).await {
        Ok(storage) => {
            let episodes = storage.get_all_episodes().await.unwrap();
            println!("   Episodes: {}", episodes.len());
            for ep in &episodes {
                println!("   - {}: {}", ep.episode_id, ep.task_description);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Check cache.redb
    println!("\n3. Cache.redb:");
    match memory_storage_redb::RedbStorage::new(std::path::Path::new("./data/cache.redb")).await {
        Ok(storage) => {
            let episodes = storage.get_all_episodes().await.unwrap();
            println!("   Episodes: {}", episodes.len());
            for ep in &episodes {
                println!("   - {}: {}", ep.episode_id, ep.task_description);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Check memory-cli/data/memory.redb
    println!("\n4. Memory-CLI memory.redb:");
    match memory_storage_redb::RedbStorage::new(std::path::Path::new("./memory-cli/data/memory.redb")).await {
        Ok(storage) => {
            let episodes = storage.get_all_episodes().await.unwrap();
            println!("   Episodes: {}", episodes.len());
            for ep in &episodes {
                println!("   - {}: {}", ep.episode_id, ep.task_description);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    Ok(())
}