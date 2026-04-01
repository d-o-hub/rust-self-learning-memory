//! Demonstration of regex pattern search capabilities
//!
//! This example shows how to use regex patterns for advanced search queries.
//!
//! Run with:
//! ```bash
//! cargo run --example regex_search_demo
//! ```

use do_memory_core::search::SearchMode;
use do_memory_core::{EpisodeFilter, SelfLearningMemory, TaskContext, TaskType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize memory system
    let memory = SelfLearningMemory::new();

    println!("🔍 Regex Search Demo\n");
    println!("Creating sample episodes...\n");

    // Create some sample episodes
    let episodes = vec![
        ("Fix bug in API endpoint /users/123", "bugfix"),
        ("Implement new feature for authentication", "development"),
        ("Error: database connection timeout at 10:30:45", "incident"),
        ("Update documentation for API v2.0", "documentation"),
        ("Refactor HTTP client code in src/client.rs", "refactoring"),
        ("Deploy version 1.2.3 to production", "deployment"),
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
    println!("Test 1: Find API endpoints with regex");
    println!("{}", "=".repeat(60));

    let filter = EpisodeFilter::builder()
        .search_text(r"/\w+/\d+".to_string()) // Match /word/number pattern
        .search_mode(SearchMode::Regex)
        .build();

    let results = memory.list_episodes_filtered(filter, None, None).await?;

    println!("Query: r\"/\\w+/\\d+\" (matches /users/123)");
    println!("Results: {} episodes found", results.len());
    for ep in &results {
        println!("  ✓ {}", ep.task_description);
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 2: Find error messages with timestamps");
    println!("{}", "=".repeat(60));

    let filter = EpisodeFilter::builder()
        .search_text(r"error.*\d{2}:\d{2}:\d{2}".to_string()) // error followed by time
        .search_mode(SearchMode::Regex)
        .build();

    let results = memory.list_episodes_filtered(filter, None, None).await?;

    println!("Query: r\"error.*\\d{{2}}:\\d{{2}}:\\d{{2}}\"");
    println!("Results: {} episodes found", results.len());
    for ep in &results {
        println!("  ✓ {}", ep.task_description);
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 3: Find version numbers");
    println!("{}", "=".repeat(60));

    let filter = EpisodeFilter::builder()
        .search_text(r"v?\d+\.\d+\.\d+".to_string()) // Semantic versions
        .search_mode(SearchMode::Regex)
        .build();

    let results = memory.list_episodes_filtered(filter, None, None).await?;

    println!("Query: r\"v?\\d+\\.\\d+\\.\\d+\" (matches version numbers)");
    println!("Results: {} episodes found", results.len());
    for ep in &results {
        println!("  ✓ {}", ep.task_description);
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 4: Find file paths");
    println!("{}", "=".repeat(60));

    let filter = EpisodeFilter::builder()
        .search_text(r"src/\w+\.\w+".to_string()) // src/filename.ext
        .search_mode(SearchMode::Regex)
        .build();

    let results = memory.list_episodes_filtered(filter, None, None).await?;

    println!("Query: r\"src/\\w+\\.\\w+\" (matches file paths)");
    println!("Results: {} episodes found", results.len());
    for ep in &results {
        println!("  ✓ {}", ep.task_description);
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 5: Case-insensitive search with OR operator");
    println!("{}", "=".repeat(60));

    let filter = EpisodeFilter::builder()
        .search_text(r"(?i)(bug|error|fix)".to_string()) // Case-insensitive OR
        .search_mode(SearchMode::Regex)
        .build();

    let results = memory.list_episodes_filtered(filter, None, None).await?;

    println!("Query: r\"(?i)(bug|error|fix)\" (case-insensitive)");
    println!("Results: {} episodes found", results.len());
    for ep in &results {
        println!("  ✓ {}", ep.task_description);
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 6: Combine regex with domain filter");
    println!("{}", "=".repeat(60));

    let filter = EpisodeFilter::builder()
        .search_text(r"API|api".to_string())
        .search_mode(SearchMode::Regex)
        .domains(vec!["bugfix".to_string(), "documentation".to_string()])
        .build();

    let results = memory.list_episodes_filtered(filter, None, None).await?;

    println!("Query: r\"API|api\" + domain filter");
    println!("Results: {} episodes found", results.len());
    for ep in &results {
        println!(
            "  ✓ {} (domain: {})",
            ep.task_description, ep.context.domain
        );
    }

    println!("\n{}", "=".repeat(60));
    println!("Test 7: ReDoS protection (invalid pattern)");
    println!("{}", "=".repeat(60));

    let dangerous_pattern = "(a+)+b"; // Catastrophic backtracking
    println!("Attempting dangerous pattern: {dangerous_pattern}");

    match do_memory_core::search::validate_regex_pattern(dangerous_pattern) {
        Ok(()) => println!("  ❌ Pattern was allowed (should be blocked)"),
        Err(e) => println!("  ✅ Pattern blocked: {e}"),
    }

    println!("\n{}", "=".repeat(60));
    println!("✨ Demo Complete!");
    println!("{}\n", "=".repeat(60));

    Ok(())
}
