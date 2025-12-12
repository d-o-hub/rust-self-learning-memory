use memory_storage_redb::RedbStorage;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Open redb storage
    let redb = RedbStorage::open("./data/cache.redb", 1000, 3600).await?;

    let episode_id = Uuid::parse_str("8e1e917e-7f56-4d59-9ff7-40cd44da541a")?;

    // Try to get episode from redb
    let episode = redb.get_episode(episode_id).await?;

    match episode {
        Some(ep) => {
            println!("✓ Episode found in redb cache!");
            println!("  ID: {}", ep.episode_id);
            println!("  Task: {}", ep.task_description);
            println!("  Type: {}", ep.task_type);
            println!("  Started: {}", ep.start_time);
            println!("  Status: {}", if ep.is_complete() { "completed" } else { "in_progress" });
        }
        None => {
            println!("✗ Episode NOT found in redb cache");
        }
    }

    // Try Turso storage
    println!("\nChecking Turso storage...");
    let turso = memory_storage_turso::TursoStorage::new("file:./data/memory.db", "").await?;
    let turso_episode = turso.get_episode(episode_id).await?;

    match turso_episode {
        Some(ep) => {
            println!("✓ Episode found in Turso DB!");
            println!("  ID: {}", ep.episode_id);
            println!("  Task: {}", ep.task_description);
            println!("  Type: {}", ep.task_type);
            println!("  Started: {}", ep.start_time);
            println!("  Status: {}", if ep.is_complete() { "completed" } else { "in_progress" });
        }
        None => {
            println!("✗ Episode NOT found in Turso DB");
        }
    }

    Ok(())
}
