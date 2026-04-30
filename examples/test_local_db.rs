//! Test local database example - verifies storage backends work correctly.

use do_memory_core::{
    ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome, TaskType,
};
use do_memory_storage_redb::RedbStorage;
use do_memory_storage_turso::TursoStorage;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("✓ Starting local database test");

    // Create local SQLite storage
    let turso_storage = TursoStorage::new("file:./data/memory.db", "").await?;
    turso_storage.initialize_schema().await?;
    println!("✓ Local SQLite storage initialized");

    // Create redb cache storage
    let redb_storage = RedbStorage::new(std::path::Path::new("./data/memory.redb")).await?;
    println!("✓ redb cache storage initialized");

    // Create memory system with local storage
    let memory_config = MemoryConfig::default();
    let memory = SelfLearningMemory::with_storage(
        memory_config,
        Arc::new(turso_storage) as Arc<dyn do_memory_core::StorageBackend>,
        Arc::new(redb_storage) as Arc<dyn do_memory_core::StorageBackend>,
    );
    println!("✓ Memory system created with local database");

    // Start an episode
    let episode_id = memory
        .start_episode(
            "Test local database programmatically".to_string(),
            TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: do_memory_core::ComplexityLevel::Simple,
                domain: "testing".to_string(),
                tags: vec!["local-db".to_string()],
            },
            TaskType::CodeGeneration,
        )
        .await;

    println!("✓ Episode created: {}", episode_id);

    // Log a step
    let step = ExecutionStep::new(
        1,
        "test".to_string(),
        "Verify local database setup".to_string(),
    );
    memory.log_step(episode_id, step).await;
    println!("✓ Step logged");

    // Complete episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Local database setup verified programmatically".to_string(),
                artifacts: vec!["test_local_db.rs".to_string()],
            },
        )
        .await?;

    println!("✓ Episode completed");

    // Retrieve relevant context
    let relevant = memory
        .retrieve_relevant_context(
            "Test database functionality".to_string(),
            TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: do_memory_core::ComplexityLevel::Simple,
                domain: "testing".to_string(),
                tags: vec![],
            },
            5,
        )
        .await;

    println!("✓ Retrieved {} relevant episodes", relevant.len());

    for (i, episode) in relevant.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            i + 1,
            episode.task_description,
            episode.episode_id
        );
    }

    println!("\n🎉 Local database configuration test completed successfully!");

    Ok(())
}
