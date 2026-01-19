//! Example demonstrating the new bulk episode operations API
//!
//! Run with: `cargo run --example bulk_episode_operations`

use memory_core::{ComplexityLevel, ExecutionStep, SelfLearningMemory, TaskContext, TaskType};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the memory system
    let memory = SelfLearningMemory::new();

    println!("=== Bulk Episode Operations Demo ===\n");

    // 1. Create multiple episodes
    println!("Creating 5 test episodes...");
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "async-programming".to_string(),
        tags: vec!["demo".to_string(), "bulk-ops".to_string()],
    };

    let mut episode_ids = Vec::new();
    for i in 1..=5 {
        let id = memory
            .start_episode(
                format!("Task {}: Implement async feature", i),
                context.clone(),
                TaskType::CodeGeneration,
            )
            .await;

        // Add some steps to each episode
        let step = ExecutionStep::new(1, "compiler".to_string(), "Build project".to_string());
        memory.log_step(id, step).await;

        episode_ids.push(id);
        println!("  Created episode {}: {}", i, id);
    }

    println!("\n--- Single Episode Retrieval ---");

    // 2. Retrieve a single episode by ID
    let first_id = episode_ids[0];
    match memory.get_episode(first_id).await {
        Ok(episode) => {
            println!("✓ Retrieved episode: {}", episode.episode_id);
            println!("  Task: {}", episode.task_description);
            println!("  Steps: {}", episode.steps.len());
            println!("  Domain: {}", episode.context.domain);
        }
        Err(e) => {
            println!("✗ Failed to retrieve episode: {}", e);
        }
    }

    // 3. Test retrieving a non-existent episode
    let non_existent = Uuid::new_v4();
    println!("\nTrying to fetch non-existent episode {}...", non_existent);
    match memory.get_episode(non_existent).await {
        Ok(_) => println!("✗ Unexpectedly found episode!"),
        Err(e) => println!("✓ Expected error: {}", e),
    }

    println!("\n--- Bulk Episode Retrieval ---");

    // 4. Bulk retrieve all episodes
    println!("\nRetrieving all {} episodes in bulk...", episode_ids.len());
    let episodes = memory.get_episodes_by_ids(&episode_ids).await?;
    println!("✓ Retrieved {} episodes", episodes.len());

    for (idx, episode) in episodes.iter().enumerate() {
        println!(
            "  [{}] {} - {} steps",
            idx + 1,
            episode.episode_id,
            episode.steps.len()
        );
    }

    // 5. Bulk retrieve with mixed existing/non-existing IDs
    println!("\n--- Partial Match Test ---");
    let mut mixed_ids = vec![episode_ids[0], episode_ids[2]]; // 2 existing
    mixed_ids.push(Uuid::new_v4()); // Non-existent
    mixed_ids.push(Uuid::new_v4()); // Non-existent

    println!("Requesting 4 episodes (2 exist, 2 don't exist)...");
    let partial_episodes = memory.get_episodes_by_ids(&mixed_ids).await?;
    println!("✓ Retrieved {} out of 4 episodes", partial_episodes.len());

    // 6. Performance comparison
    println!("\n--- Performance Comparison ---");

    // Individual lookups
    let start = std::time::Instant::now();
    for id in &episode_ids {
        let _ = memory.get_episode(*id).await?;
    }
    let individual_duration = start.elapsed();

    // Bulk lookup
    let start = std::time::Instant::now();
    let _ = memory.get_episodes_by_ids(&episode_ids).await?;
    let bulk_duration = start.elapsed();

    println!("Individual lookups (5 calls): {:?}", individual_duration);
    println!("Bulk lookup (1 call):          {:?}", bulk_duration);

    if bulk_duration < individual_duration {
        let speedup = individual_duration.as_micros() as f64 / bulk_duration.as_micros() as f64;
        println!("✓ Bulk lookup is {:.2}x faster!", speedup);
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
