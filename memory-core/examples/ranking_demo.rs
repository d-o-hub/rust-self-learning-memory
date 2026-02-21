//! Demonstration of search result ranking
//!
//! This example shows how search results are ranked based on multiple signals:
//! - Relevance (match quality)
//! - Recency (how recent)
//! - Success rate (outcome)
//! - Completeness (finished or not)
//! - Field importance (where matched)
//!
//! Run with:
//! ```bash
//! cargo run --example ranking_demo
//! ```

use chrono::{Duration, Utc};
use memory_core::search::{RankingWeights, SearchField, SearchMode, calculate_ranking_score};
use memory_core::{Episode, ExecutionStep, TaskContext, TaskOutcome, TaskType};

fn create_episode(
    description: &str,
    domain: &str,
    days_ago: i64,
    outcome: Option<TaskOutcome>,
    steps: usize,
) -> Episode {
    let context = TaskContext {
        domain: domain.to_string(),
        ..Default::default()
    };

    let mut episode = Episode::new(description.to_string(), context, TaskType::CodeGeneration);

    // Set start time in the past
    episode.start_time = Utc::now() - Duration::days(days_ago);

    // Add steps
    for i in 1..=steps {
        let step = ExecutionStep::new(i, "tool".to_string(), format!("action {i}"));
        episode.steps.push(step);
    }

    // Set outcome
    episode.outcome = outcome;
    if episode.outcome.is_some() {
        episode.end_time = Some(episode.start_time + Duration::hours(2));
    }

    episode
}

fn main() {
    println!("🏆 Search Result Ranking Demo\n");
    println!("Creating test episodes with different characteristics...\n");

    // Create episodes with varying qualities
    let episodes = [
        (
            create_episode(
                "Fix critical database bug",
                "backend",
                1, // 1 day ago
                Some(TaskOutcome::Success {
                    verdict: "Fixed".to_string(),
                    artifacts: vec!["fix.rs".to_string()],
                }),
                5,
            ),
            "Recent, successful, complete",
        ),
        (
            create_episode(
                "Implement database connection pool",
                "backend",
                30, // 30 days ago
                Some(TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec!["pool.rs".to_string()],
                }),
                10,
            ),
            "Old, successful, complete",
        ),
        (
            create_episode(
                "Try to fix database timeout",
                "backend",
                7, // 7 days ago
                Some(TaskOutcome::Failure {
                    reason: "Could not reproduce".to_string(),
                    error_details: None,
                }),
                3,
            ),
            "Week old, failed",
        ),
        (
            create_episode(
                "Update database schema",
                "backend",
                2, // 2 days ago
                Some(TaskOutcome::PartialSuccess {
                    verdict: "Partial".to_string(),
                    completed: vec!["migrations".to_string()],
                    failed: vec!["rollback".to_string()],
                }),
                8,
            ),
            "Recent, partial success",
        ),
        (
            create_episode(
                "Start database optimization",
                "backend",
                0,    // Today
                None, // In progress
                2,
            ),
            "Today, in progress",
        ),
    ];

    // Display episodes
    for (i, (ep, desc)) in episodes.iter().enumerate() {
        println!("Episode {}: {}", i + 1, ep.task_description);
        println!("  Description: {desc}");
        println!("  Age: {} days", (Utc::now() - ep.start_time).num_days());
        println!("  Steps: {}", ep.steps.len());
        println!(
            "  Status: {}",
            if ep.is_complete() {
                "Complete"
            } else {
                "In Progress"
            }
        );
        println!();
    }

    println!("{}", "=".repeat(60));
    println!("Ranking with Default Weights");
    println!("{}", "=".repeat(60));
    println!("Weights: Relevance=40%, Recency=20%, Success=20%,");
    println!("         Completeness=10%, Field=10%\n");

    let weights = RankingWeights::default();
    let mode = SearchMode::Fuzzy { threshold: 0.8 };
    let field = SearchField::Description;

    let mut scored: Vec<_> = episodes
        .iter()
        .map(|(ep, desc)| {
            let score = calculate_ranking_score(ep, &mode, 0.95, &field, &weights);
            (ep, desc, score)
        })
        .collect();

    // Sort by score
    scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    for (rank, (ep, desc, score)) in scored.iter().enumerate() {
        println!("Rank {}: {:.3} - {}", rank + 1, score, ep.task_description);
        println!("        ({desc})");
    }

    println!("\n{}", "=".repeat(60));
    println!("Ranking with Recency-Focused Weights");
    println!("{}", "=".repeat(60));
    println!("Weights: Relevance=20%, Recency=50%, Success=15%,");
    println!("         Completeness=10%, Field=5%\n");

    let recency_weights = RankingWeights::new(0.20, 0.50, 0.15, 0.10, 0.05);

    let mut scored: Vec<_> = episodes
        .iter()
        .map(|(ep, desc)| {
            let score = calculate_ranking_score(ep, &mode, 0.95, &field, &recency_weights);
            (ep, desc, score)
        })
        .collect();

    scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    for (rank, (ep, desc, score)) in scored.iter().enumerate() {
        println!("Rank {}: {:.3} - {}", rank + 1, score, ep.task_description);
        println!("        ({desc})");
    }

    println!("\n{}", "=".repeat(60));
    println!("Ranking with Success-Focused Weights");
    println!("{}", "=".repeat(60));
    println!("Weights: Relevance=20%, Recency=10%, Success=50%,");
    println!("         Completeness=15%, Field=5%\n");

    let success_weights = RankingWeights::new(0.20, 0.10, 0.50, 0.15, 0.05);

    let mut scored: Vec<_> = episodes
        .iter()
        .map(|(ep, desc)| {
            let score = calculate_ranking_score(ep, &mode, 0.95, &field, &success_weights);
            (ep, desc, score)
        })
        .collect();

    scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    for (rank, (ep, desc, score)) in scored.iter().enumerate() {
        println!("Rank {}: {:.3} - {}", rank + 1, score, ep.task_description);
        println!("        ({desc})");
    }

    println!("\n{}", "=".repeat(60));
    println!("Key Insights:");
    println!("{}", "=".repeat(60));
    println!("• Default weights balance all factors");
    println!("• Recency-focused ranks recent episodes higher");
    println!("• Success-focused ranks successful episodes higher");
    println!("• Rankings change based on what you value most");
    println!("\n✨ Demo Complete!\n");
}
