//! Example of using spatiotemporal retrieval for task replay/guidance.
//!
//! This example shows how an agent can look up similar past tasks
//! to retrieve successful execution patterns and avoid past mistakes.

use chrono::{Duration, Utc};
use do_memory_core::episode::Episode;
use do_memory_core::spatiotemporal::{HierarchicalRetriever, RetrievalQuery};
use do_memory_core::types::{TaskContext, TaskOutcome, TaskType};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("--- Task Replay & Guidance Example ---");

    // 1. Setup past memories (some successful, some failed)
    let now = Utc::now();
    let mut history = Vec::new();

    // Successful past task
    let mut ep1 = create_episode(
        "Deploy to production environment",
        "devops",
        TaskType::Other,
        now - Duration::days(30),
    );
    ep1.outcome = Some(TaskOutcome::Success {
        verdict: "Deployed successfully after setting correct env vars".to_string(),
        artifacts: vec!["deploy-log.txt".to_string()],
    });
    history.push(Arc::new(ep1));

    // Failed past task (same domain)
    let mut ep2 = create_episode(
        "Deploy to production environment",
        "devops",
        TaskType::Other,
        now - Duration::days(45),
    );
    ep2.outcome = Some(TaskOutcome::Failure {
        reason: "Database migration failed: missing permissions".to_string(),
        error_details: Some("Tried to run migrations with read-only user".to_string()),
    });
    history.push(Arc::new(ep2));

    // 2. Current task we need guidance for
    let current_task = "Deploying the new analytics service to prod";
    println!("Current Task: {current_task}");

    // 3. Use spatiotemporal retrieval to find relevant guidance
    let retriever = HierarchicalRetriever::new();

    let query = RetrievalQuery {
        query_text: current_task.to_string(),
        domain: Some("devops".to_string()),
        task_type: Some(TaskType::Other),
        limit: 2,
        ..Default::default()
    };

    println!("Searching for relevant past experiences...");
    let matches = retriever.retrieve(&query, &history).await?;

    // 4. Extract guidance from the top match
    if let Some(best_match) = matches.first() {
        let ep = history
            .iter()
            .find(|e| e.episode_id == best_match.episode_id)
            .unwrap();
        println!("\nFound relevant past episode: {}", ep.task_description);
        println!("Relevance Score: {:.2}", best_match.relevance_score);

        match &ep.outcome {
            Some(TaskOutcome::Success { verdict, .. }) => {
                println!("Outcome: SUCCESS ✅");
                println!("Guidance: {verdict}");
            }
            Some(TaskOutcome::Failure {
                reason,
                error_details,
            }) => {
                println!("Outcome: FAILURE ❌");
                println!("Warning: Last time this was tried, it failed because: {reason}");
                if let Some(details) = error_details {
                    println!("Details: {details}");
                }
            }
            _ => println!("Outcome: Unknown"),
        }
    } else {
        println!("No relevant past experiences found.");
    }

    Ok(())
}

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
