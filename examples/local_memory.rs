//! Local memory example demonstrating zero-credential SQLite storage.
#![allow(clippy::doc_markdown)]

use do_memory_core::SelfLearningMemory;
use do_memory_core::{ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("Starting local memory example");

    // 1. Initialize memory with a local SQLite database file
    // No Turso account or credentials are required for this mode.
    let db_path = "./data/example_memory.db";

    // Ensure the data directory exists
    if let Some(parent) = Path::new(db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("Connecting to local database at {db_path}");
    // Use the new named constructor from do_memory_storage_turso
    let storage = do_memory_storage_turso::TursoStorage::new_local(db_path).await?;
    storage.initialize_schema().await?;

    // Use an in-memory redb for caching
    let cache = std::sync::Arc::new(
        do_memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:")).await?,
    );

    let config = do_memory_core::MemoryConfig {
        quality_threshold: 0.0, // Lower threshold for demonstration
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_storage(config, std::sync::Arc::new(storage), cache);

    // 2. Start an episode for a task
    let context = TaskContext::default();
    let episode_id = memory
        .start_episode(
            "Demonstrate local-only memory".to_string(),
            context,
            TaskType::Testing,
        )
        .await;

    println!("Started episode: {episode_id}");

    // 3. Log an execution step
    let mut step = ExecutionStep::new(
        1,
        "example_tool".to_string(),
        "Perform a local operation".to_string(),
    );
    step.result = Some(ExecutionResult::Success {
        output: "Local operation completed successfully".to_string(),
    });
    memory.log_step(episode_id, step).await;

    // 4. Complete the episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Example completed without any cloud credentials".to_string(),
                artifacts: vec![],
            },
        )
        .await?;

    println!("Episode completed and stored locally");

    // 5. Retrieve the episode to verify it was stored
    let episodes = memory.get_all_episodes().await?;
    let episodes_len = episodes.len();
    println!("Total episodes in local storage: {episodes_len}");

    Ok(())
}
