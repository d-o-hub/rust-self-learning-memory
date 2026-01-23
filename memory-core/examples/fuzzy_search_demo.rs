//! Demonstration of fuzzy search capabilities
//!
//! This example shows how to use fuzzy search to find episodes with typo-tolerant matching.
//!
//! Run with:
//! ```bash
//! cargo run --example fuzzy_search_demo
//! ```

use memory_core::search::SearchMode;
use memory_core::{EpisodeFilter, SelfLearningMemory, TaskContext, TaskType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize memory system
    let memory = SelfLearningMemory::new();

    println!("🔍 Fuzzy Search Demo\n");
    println!("Creating sample episodes...\n");

    // Create some sample episodes
    let episodes = vec![
        ("Build a database connection pool", "backend"),
        ("Implement HTTP client with async", "networking"),
        ("Create authentication middleware", "security"),
        ("Design PostgreSQL schema", "database"),
        ("Write unit tests for API", "testing"),
    ];

    for (description, domain) in episodes {
        let context = TaskContext {
            domain: domain.to_string(),
            ..Default::default()
        };
        memory
            .start_episode(description.to_string(), context, TaskType::CodeGeneration)
            .await;
        println!("✓ Created: {description}");
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 1: Exact Search (typo = no match)");
    println!("{}", "=".repeat(60));

    let filter_exact = EpisodeFilter::builder()
        .search_text("databse".to_string()) // Typo: missing 'a'
        .search_mode(SearchMode::Exact)
        .build();

    let results = memory
        .list_episodes_filtered(filter_exact, None, None)
        .await?;

    println!("Query: 'databse' (exact match)");
    println!("Results: {} episodes found", results.len());
    if results.is_empty() {
        println!("❌ No matches (expected - exact search doesn't tolerate typos)");
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 2: Fuzzy Search (typo = match!)");
    println!("{}", "=".repeat(60));

    let filter_fuzzy = EpisodeFilter::builder()
        .search_text("databse".to_string()) // Same typo
        .search_mode(SearchMode::Fuzzy { threshold: 0.8 })
        .build();

    let results = memory
        .list_episodes_filtered(filter_fuzzy, None, None)
        .await?;

    println!("Query: 'databse' (fuzzy match, threshold: 0.8)");
    println!("Results: {} episodes found", results.len());
    println!("✅ Fuzzy search found episodes despite typo:\n");
    for ep in &results {
        println!("  - {}", ep.task_description);
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 3: Fuzzy Search with Domain Filter");
    println!("{}", "=".repeat(60));

    let filter_combined = EpisodeFilter::builder()
        .search_text("databse".to_string())
        .search_mode(SearchMode::Fuzzy { threshold: 0.8 })
        .domains(vec!["backend".to_string()])
        .build();

    let results = memory
        .list_episodes_filtered(filter_combined, None, None)
        .await?;

    println!("Query: 'databse' + domain='backend'");
    println!("Results: {} episodes found", results.len());
    println!("✅ Found:\n");
    for ep in &results {
        println!(
            "  - {} (domain: {})",
            ep.task_description, ep.context.domain
        );
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 4: Different Threshold Levels");
    println!("{}", "=".repeat(60));

    let queries = vec![
        ("async", 0.9, "High threshold (0.9) - strict matching"),
        ("asnyc", 0.8, "Medium threshold (0.8) - 1 letter swap"),
        ("asinc", 0.6, "Low threshold (0.6) - 2 letter changes"),
    ];

    for (query, threshold, description) in queries {
        let filter = EpisodeFilter::builder()
            .search_text(query.to_string())
            .search_mode(SearchMode::Fuzzy { threshold })
            .build();

        let results = memory.list_episodes_filtered(filter, None, None).await?;

        println!("\n{description}");
        println!("Query: '{query}' with threshold {threshold}");
        println!("Results: {} episodes", results.len());

        if results.is_empty() {
            println!("  (no matches)");
        } else {
            for ep in results {
                println!("  ✓ {}", ep.task_description);
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("✨ Demo Complete!");
    println!("{}\n", "=".repeat(60));

    Ok(())
}
