//! Pattern Search and Recommendation Demo
//!
//! This example demonstrates the semantic pattern search and recommendation features.
//!
//! Run with: `cargo run --example pattern_search_demo`

use memory_core::{
    ComplexityLevel, ExecutionStep, SelfLearningMemory, TaskContext, TaskOutcome, TaskType,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize memory system
    let memory = SelfLearningMemory::new();

    println!("🧠 Pattern Search & Recommendation Demo");
    println!("========================================\n");

    // Create some example episodes to generate patterns
    println!("📝 Creating sample episodes...\n");

    // Episode 1: REST API creation
    let context1 = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        tags: vec!["rest".to_string(), "async".to_string()],
    };

    let ep1 = memory
        .start_episode(
            "Build REST API with authentication".to_string(),
            context1.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    memory
        .log_step(
            ep1,
            ExecutionStep::new(1, "create_router".to_string(), "Setup routes".to_string()),
        )
        .await;

    memory
        .complete_episode(
            ep1,
            TaskOutcome::Success {
                verdict: "API created with JWT auth".to_string(),
                artifacts: vec!["api.rs".to_string()],
            },
        )
        .await?;

    // Episode 2: CLI tool
    let context2 = TaskContext {
        domain: "cli".to_string(),
        language: Some("rust".to_string()),
        framework: Some("clap".to_string()),
        complexity: ComplexityLevel::Simple,
        tags: vec!["argparse".to_string()],
    };

    let ep2 = memory
        .start_episode(
            "Build CLI tool".to_string(),
            context2.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    memory
        .log_step(
            ep2,
            ExecutionStep::new(1, "parse_args".to_string(), "Parse CLI args".to_string()),
        )
        .await;

    memory
        .complete_episode(
            ep2,
            TaskOutcome::Success {
                verdict: "CLI tool working".to_string(),
                artifacts: vec!["main.rs".to_string()],
            },
        )
        .await?;

    println!("✅ Created 2 sample episodes\n");

    // Demo 1: Semantic Pattern Search
    println!("🔍 Demo 1: Semantic Pattern Search");
    println!("-----------------------------------");
    println!("Query: 'How to build a REST API with authentication'\n");

    let search_context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec!["rest".to_string()],
    };

    let results = memory
        .search_patterns_semantic(
            "How to build a REST API with authentication",
            search_context,
            5,
        )
        .await?;

    println!("Found {} patterns:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "  {}. Relevance: {:.2} | Success Rate: {:.1}%",
            i + 1,
            result.relevance_score,
            result.pattern.success_rate() * 100.0
        );
        println!("     Pattern ID: {}", result.pattern.id());
        if let Some(ctx) = result.pattern.context() {
            println!("     Domain: {}", ctx.domain);
        }
    }
    println!();

    // Demo 2: Pattern Recommendations
    println!("💡 Demo 2: Pattern Recommendations");
    println!("-----------------------------------");
    println!("Task: 'Build an async HTTP client with connection pooling'\n");

    let rec_context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Complex,
        tags: vec!["async".to_string(), "http".to_string()],
    };

    let recommendations = memory
        .recommend_patterns_for_task(
            "Build an async HTTP client with connection pooling",
            rec_context,
            3,
        )
        .await?;

    println!("Got {} recommendations:", recommendations.len());
    for (i, rec) in recommendations.iter().enumerate() {
        println!(
            "  {}. Relevance: {:.2} | Effectiveness: {:.2}",
            i + 1,
            rec.relevance_score,
            rec.score_breakdown.effectiveness
        );
        println!("     Pattern ID: {}", rec.pattern.id());
    }
    println!();

    // Demo 3: Cross-Domain Pattern Discovery
    println!("🌐 Demo 3: Cross-Domain Pattern Discovery");
    println!("------------------------------------------");
    println!("Finding CLI patterns applicable to web-api\n");

    let target_context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec![],
    };

    let analogous = memory
        .discover_analogous_patterns("cli", target_context, 5)
        .await?;

    println!("Found {} analogous patterns:", analogous.len());
    for (i, pattern) in analogous.iter().enumerate() {
        println!("  {}. Relevance: {:.2}", i + 1, pattern.relevance_score);
        if let Some(ctx) = pattern.pattern.context() {
            println!("     Original Domain: {}", ctx.domain);
        }
    }
    println!();

    println!("✨ Demo complete!");
    println!("\nKey Features:");
    println!("  • Multi-signal ranking (semantic + context + effectiveness + recency + success)");
    println!("  • Configurable search parameters");
    println!("  • Cross-domain pattern discovery");
    println!("  • High-quality recommendations for tasks");

    Ok(())
}
