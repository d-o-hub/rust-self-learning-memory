//! Example demonstrating semantic summarization of episodes.
//!
//! Run with:
//! ```bash
//! cargo run --example semantic_summarization
//! ```

use memory_core::pre_storage::SalientFeatures;
use memory_core::semantic::SemanticSummarizer;
use memory_core::{
    ComplexityLevel, Episode, ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Semantic Summarization Demo ===\n");

    // Create a summarizer
    let summarizer = SemanticSummarizer::new();
    println!("Created SemanticSummarizer with default config:");
    println!("  - Min summary length: 100 words");
    println!("  - Max summary length: 200 words");
    println!("  - Max key steps: 5\n");

    // Create a sample episode
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "authentication".to_string(),
        tags: vec!["security".to_string(), "async".to_string()],
    };

    let mut episode = Episode::new(
        "Implement JWT-based user authentication with Redis session storage".to_string(),
        context,
        TaskType::CodeGeneration,
    );

    println!("Created episode: {}", episode.task_description);
    println!("Episode ID: {}\n", episode.episode_id);

    // Add execution steps
    println!("Adding execution steps...");

    let mut step1 = ExecutionStep::new(
        1,
        "planner".to_string(),
        "Analyze authentication requirements".to_string(),
    );
    step1.result = Some(ExecutionResult::Success {
        output: "Requirements analyzed: JWT tokens, Redis storage, async handlers".to_string(),
    });
    episode.add_step(step1);

    let mut step2 = ExecutionStep::new(
        2,
        "code_generator".to_string(),
        "Generate JWT token handling code".to_string(),
    );
    step2.result = Some(ExecutionResult::Success {
        output: "JWT encoder/decoder implemented".to_string(),
    });
    episode.add_step(step2);

    let mut step3 = ExecutionStep::new(
        3,
        "code_generator".to_string(),
        "Implement Redis session store".to_string(),
    );
    step3.result = Some(ExecutionResult::Success {
        output: "Redis client integrated with async pool".to_string(),
    });
    episode.add_step(step3);

    let mut step4 = ExecutionStep::new(
        4,
        "validator".to_string(),
        "Validate token expiry logic".to_string(),
    );
    step4.result = Some(ExecutionResult::Error {
        message: "Token expiry check failed for edge case".to_string(),
    });
    episode.add_step(step4);

    let mut step5 = ExecutionStep::new(
        5,
        "code_generator".to_string(),
        "Fix token expiry validation".to_string(),
    );
    step5.result = Some(ExecutionResult::Success {
        output: "Added proper timestamp comparison".to_string(),
    });
    episode.add_step(step5);

    let mut step6 = ExecutionStep::new(
        6,
        "tester".to_string(),
        "Run comprehensive security tests".to_string(),
    );
    step6.result = Some(ExecutionResult::Success {
        output: "All tests passed (98% coverage)".to_string(),
    });
    episode.add_step(step6);

    println!("Added {} execution steps\n", episode.steps.len());

    // Add salient features (from PREMem analysis)
    let mut features = SalientFeatures::new();
    features
        .critical_decisions
        .push("Chose JWT over session cookies for stateless authentication".to_string());
    features
        .critical_decisions
        .push("Selected Redis for fast session lookup".to_string());
    features
        .tool_combinations
        .push(vec!["code_generator".to_string(), "validator".to_string()]);
    features
        .error_recovery_patterns
        .push("Token expiry validation failed -> Added timestamp comparison".to_string());
    features
        .key_insights
        .push("Async Redis pool significantly improves authentication performance".to_string());
    features
        .key_insights
        .push("Edge cases in token expiry require explicit timestamp handling".to_string());

    episode.salient_features = Some(features);
    println!("Added salient features from PREMem analysis\n");

    // Complete the episode
    episode.complete(TaskOutcome::Success {
        verdict: "JWT authentication implemented with Redis storage and 98% test coverage"
            .to_string(),
        artifacts: vec![
            "auth/jwt.rs".to_string(),
            "auth/session.rs".to_string(),
            "auth/middleware.rs".to_string(),
            "tests/auth_test.rs".to_string(),
        ],
    });

    println!("Episode completed successfully\n");

    // Generate semantic summary
    println!("=== Generating Semantic Summary ===\n");

    let summary = summarizer.summarize_episode(&episode).await?;

    println!("Summary Text:");
    println!("-------------");
    println!("{}\n", summary.summary_text);

    println!(
        "Word Count: {}",
        summary.summary_text.split_whitespace().count()
    );
    println!();

    println!("Key Concepts ({} total):", summary.key_concepts.len());
    println!("-------------");
    for (i, concept) in summary.key_concepts.iter().enumerate() {
        println!("  {}. {}", i + 1, concept);
    }
    println!();

    println!("Key Steps ({} total):", summary.key_steps.len());
    println!("-------------");
    for step in &summary.key_steps {
        println!("  - {}", step);
    }
    println!();

    println!("Summary Metadata:");
    println!("-------------");
    println!("  Episode ID: {}", summary.episode_id);
    println!("  Created At: {}", summary.created_at);
    println!("  Has Embedding: {}", summary.summary_embedding.is_some());
    println!();

    // Demonstrate custom configuration
    println!("=== Custom Configuration Demo ===\n");

    let custom_summarizer = SemanticSummarizer::with_config(50, 100, 3);
    println!("Created custom summarizer:");
    println!("  - Min length: 50 words");
    println!("  - Max length: 100 words");
    println!("  - Max key steps: 3\n");

    let custom_summary = custom_summarizer.summarize_episode(&episode).await?;

    println!("Custom Summary (truncated):");
    println!(
        "Word Count: {}",
        custom_summary.summary_text.split_whitespace().count()
    );
    println!("Key Steps: {}", custom_summary.key_steps.len());
    println!();

    // Show serialization
    println!("=== Serialization Demo ===\n");

    let json = serde_json::to_string_pretty(&summary)?;
    println!("JSON representation (first 500 chars):");
    println!("{}", &json[..json.len().min(500)]);
    if json.len() > 500 {
        println!("...");
    }
    println!();

    println!("=== Summary ===");
    println!("Successfully demonstrated semantic summarization!");
    println!("- Generated concise 100-200 word summary");
    println!("- Extracted {} key concepts", summary.key_concepts.len());
    println!("- Identified {} critical steps", summary.key_steps.len());
    println!("- Ready for semantic search and retrieval");

    Ok(())
}
