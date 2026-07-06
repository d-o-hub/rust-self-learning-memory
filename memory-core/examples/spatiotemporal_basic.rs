//! Basic example of spatiotemporal indexing and hierarchical retrieval.
//!
//! This example demonstrates how to:
//! 1. Create a `SpatiotemporalIndex` and insert episodes.
//! 2. Use `HierarchicalRetriever` to search for episodes.
//! 3. Observe how domain, task type, and recency affect the results.

use chrono::{Duration, Utc};
use do_memory_core::episode::Episode;
use do_memory_core::spatiotemporal::{HierarchicalRetriever, RetrievalQuery, SpatiotemporalIndex};
use do_memory_core::types::{TaskContext, TaskType};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,do_memory_core::spatiotemporal=debug")
        .init();

    println!("--- Spatiotemporal Retrieval Basic Example ---");

    // 1. Create some sample episodes
    let now = Utc::now();
    let mut episodes = Vec::new();

    // Episode 1: Web API / Code Generation (Recent)
    let ep1 = create_episode(
        "Implement JWT authentication",
        "web-api",
        TaskType::CodeGeneration,
        now - Duration::hours(2),
    );
    episodes.push(Arc::new(ep1));

    // Episode 2: Web API / Debugging (Yesterday)
    let ep2 = create_episode(
        "Fix database connection leak",
        "web-api",
        TaskType::Debugging,
        now - Duration::days(1),
    );
    episodes.push(Arc::new(ep2));

    // Episode 3: Frontend / Code Generation (Last Week)
    let ep3 = create_episode(
        "Create React login component",
        "frontend",
        TaskType::CodeGeneration,
        now - Duration::days(7),
    );
    episodes.push(Arc::new(ep3));

    // 2. Build the Spatiotemporal Index
    let mut index = SpatiotemporalIndex::new();
    for ep in &episodes {
        index.insert(ep);
    }

    println!(
        "Indexed {} episodes across {} domains.",
        index.len(),
        index.num_domains()
    );

    // 3. Setup the Hierarchical Retriever
    // Weight recency heavily (0.5)
    let retriever = HierarchicalRetriever::with_config(0.5, 5);

    // 4. Perform a query
    println!("\nQuerying for: 'authentication' in 'web-api' domain...");

    let query = RetrievalQuery {
        query_text: "authentication".to_string(),
        domain: Some("web-api".to_string()),
        task_type: Some(TaskType::CodeGeneration),
        limit: 5,
        ..Default::default()
    };

    let results = retriever.retrieve(&query, &episodes).await?;

    println!("Found {} results:", results.len());
    for (i, score) in results.iter().enumerate() {
        let Some(ep) = episodes.iter().find(|e| e.episode_id == score.episode_id) else {
            continue;
        };
        println!(
            "{}. [{:.2}] {} (Domain: {}, Type: {:?}, Created: {})",
            i + 1,
            score.relevance_score,
            ep.task_description,
            ep.context.domain,
            ep.task_type,
            ep.start_time.format("%Y-%m-%d %H:%M")
        );
        println!(
            "   Scores: Domain: {:.2}, TaskType: {:.2}, Temporal: {:.2}, Similarity: {:.2}",
            score.level_1_score, score.level_2_score, score.level_3_score, score.level_4_score
        );
    }

    Ok(())
}

/// Helper to create an episode with a specific start time.
/// NOTE: duplicate of `spatiotemporal_task_replay` — consider a shared examples/utils
fn create_episode(
    description: &str,
    domain: &str,
    task_type: TaskType,
    start_time: chrono::DateTime<Utc>,
) -> Episode {
    let context = TaskContext {
        domain: domain.to_string(),
        ..Default::default()
    };
    let mut ep = Episode::new(description.to_string(), context, task_type);
    ep.start_time = start_time;
    ep
}
