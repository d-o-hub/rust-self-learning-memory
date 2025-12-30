//! End-to-end example of semantic embeddings in memory-core
//!
//! This example demonstrates the complete workflow:
//! 1. Initialize embedding provider (local or OpenAI)
//! 2. Generate embeddings for text
//! 3. Store embeddings with episodes
//! 4. Perform semantic similarity search
//! 5. Retrieve relevant episodes based on meaning
//!
//! Run with:
//! ```bash
//! # With local embeddings (CPU-based, no API key needed)
//! cargo run --example embeddings_end_to_end --features local-embeddings
//!
//! # With OpenAI (requires API key)
//! export OPENAI_API_KEY="sk-your-key"
//! cargo run --example embeddings_end_to_end --features openai
//! ```

use memory_core::embeddings::EmbeddingProvider;
use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};
use chrono::Utc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\nMemory-Core: End-to-End Embeddings Example");
    println!("{}", "=".repeat(60));
    println!();

    // Step 1: Initialize Memory System
    println!("Step 1: Initializing memory system...");
    let memory = SelfLearningMemory::new();
    println!("Memory system initialized\n");

    // Step 2: Initialize Embedding Provider
    println!("Step 2: Initializing embedding provider...");
    let provider = initialize_provider().await?;
    println!("Using provider: {}", provider_name(&provider));
    println!("Dimension: {}\n", provider.embedding_dimension());

    // Step 3: Create sample episodes with different domains
    println!("Step 3: Creating sample episodes with embeddings...");
    let episodes = vec![
        (
            "Implement REST API authentication",
            "web-api",
            vec!["rest", "auth", "jwt"],
        ),
        (
            "Build OAuth2 login flow",
            "web-api",
            vec!["oauth", "authentication", "security"],
        ),
        (
            "Optimize database query performance",
            "database",
            vec!["sql", "performance", "indexing"],
        ),
        (
            "Create React component for user profile",
            "frontend",
            vec!["react", "ui", "components"],
        ),
        (
            "Implement WebSocket real-time notifications",
            "web-api",
            vec!["websocket", "realtime", "notifications"],
        ),
    ];

    for (desc, domain, tags) in &episodes {
        // Generate embedding for task description
        let embedding = provider.embed_text(desc).await?;
        let embedding_len = embedding.len();
        println!(
            "  Created episode: '{}' (embedding: {} dims)",
            desc,
            embedding_len
        );

        // Start episode
        let context = TaskContext {
            domain: domain.to_string(),
            language: Some("rust".to_string()),
            framework: Some("axum".to_string()),
            complexity: ComplexityLevel::Moderate,
            tags: tags.iter().map(|s| s.to_string()).collect(),
        };

        let episode_id = memory
            .start_episode(desc.to_string(), context, TaskType::CodeGeneration)
            .await;

        // Log some steps
        let step1 = ExecutionStep {
            step_number: 1,
            timestamp: Utc::now(),
            tool: "analyze".to_string(),
            action: "Analyzing requirements".to_string(),
            parameters: serde_json::json!({}),
            result: Some(ExecutionResult::Success {
                output: "Requirements analyzed".to_string(),
            }),
            latency_ms: 10,
            tokens_used: None,
            metadata: std::collections::HashMap::new(),
        };
        memory.log_step(episode_id, step1).await;

        let step2 = ExecutionStep {
            step_number: 2,
            timestamp: Utc::now(),
            tool: "implement".to_string(),
            action: "Implementing solution".to_string(),
            parameters: serde_json::json!({}),
            result: Some(ExecutionResult::Success {
                output: "Implementation complete".to_string(),
            }),
            latency_ms: 10,
            tokens_used: None,
            metadata: std::collections::HashMap::new(),
        };
        memory.log_step(episode_id, step2).await;

        let step3 = ExecutionStep {
            step_number: 3,
            timestamp: Utc::now(),
            tool: "test".to_string(),
            action: "Testing solution".to_string(),
            parameters: serde_json::json!({}),
            result: Some(ExecutionResult::Success {
                output: "All tests passed".to_string(),
            }),
            latency_ms: 10,
            tokens_used: None,
            metadata: std::collections::HashMap::new(),
        };
        memory.log_step(episode_id, step3).await;

        // Complete episode
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Task completed successfully".to_string(),
                    artifacts: vec![],
                },
            )
            .await?;
    }
    println!("Created {} episodes\n", episodes.len());

    // Step 4: Demonstrate semantic search
    println!("Step 4: Performing semantic similarity searches...");
    println!();

    let queries = vec![
        ("user authentication", "web-api", vec!["auth"]),
        ("database optimization", "database", vec!["performance"]),
        ("frontend UI component", "frontend", vec!["ui"]),
    ];

    for (query, domain, tags) in &queries {
        println!("Query: \"{}\"", query);
        println!("{}", "-".repeat(60));

        // Generate embedding for query
        let query_embedding = provider.embed_text(query).await?;
        let query_embedding_len = query_embedding.len();
        println!("  Query embedding: {} dimensions", query_embedding_len);

        // Retrieve relevant episodes
        let context = TaskContext {
            domain: domain.to_string(),
            language: Some("rust".to_string()),
            framework: Some("axum".to_string()),
            complexity: ComplexityLevel::Moderate,
            tags: tags.iter().map(|s| s.to_string()).collect(),
        };

        let relevant = memory
            .retrieve_relevant_context(query.to_string(), context, 3)
            .await;

        println!("  Found {} relevant episodes:", relevant.len());
        for (i, episode) in relevant.iter().enumerate() {
            println!("    {}. {}", i + 1, episode.task_description);
            if let Some(reward) = &episode.reward {
                println!("       Reward: {:.2}", reward.total);
            }
        }
        println!();
    }

    // Step 5: Demonstrate similarity calculations
    println!("Step 5: Direct similarity calculations...");
    println!();

    let text_pairs = vec![
        ("REST API", "web service API"),
        ("OAuth authentication", "user login"),
        ("database indexing", "React components"),
    ];

    for (text1, text2) in text_pairs {
        let similarity = provider.similarity(text1, text2).await?;
        println!("  Similarity('{}', '{}') = {:.3}", text1, text2, similarity);
    }
    println!();

    // Step 6: Batch processing demonstration
    println!("Step 6: Batch embedding generation...");
    let batch_texts = vec![
        "Implement user authentication".to_string(),
        "Create database migration".to_string(),
        "Build API endpoint".to_string(),
        "Write unit tests".to_string(),
    ];

    let batch_results = provider.embed_batch(&batch_texts).await?;
    println!("  Generated {} embeddings in batch", batch_results.len());
    for (i, embedding) in batch_results.iter().enumerate() {
        println!(
            "    {}. '{}' → {} dims",
            i + 1,
            batch_texts[i],
            embedding.len()
        );
    }
    println!();

    // Summary
    println!("Example Complete!");
    println!();
    println!("Key Takeaways:");
    println!("  - Embeddings enable semantic (meaning-based) search");
    println!("  - Multiple providers supported (local, OpenAI, etc.)");
    println!("  - Batch processing available for efficiency");
    println!("  - Seamlessly integrates with memory system");
    println!();
    println!("Next Steps:");
    println!("  1. Try with different providers (see EMBEDDING_PROVIDERS.md)");
    println!("  2. Experiment with different similarity thresholds");
    println!("  3. Integrate with your own application");
    println!("  4. See memory-cli for command-line usage");

    Ok(())
}

/// Initialize embedding provider based on available features
async fn initialize_provider() -> anyhow::Result<Box<dyn EmbeddingProvider>> {
    #[cfg(feature = "local-embeddings")]
    {
        use memory_core::embeddings::{LocalEmbeddingProvider, ModelConfig};
        match LocalEmbeddingProvider::new(ModelConfig::default()).await {
            Ok(provider) => {
                println!("  Using Local Embedding Provider (CPU-based)");
                return Ok(Box::new(provider));
            }
            Err(e) => {
                println!("  Local provider failed: {}", e);
            }
        }
    }

    #[cfg(feature = "openai")]
    {
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            match memory_core::embeddings::OpenAIEmbeddingProvider::new(
                api_key,
                ModelConfig::openai_3_small(),
            ) {
                Ok(provider) => {
                    println!("  Using OpenAI Embedding Provider");
                    return Ok(Box::new(provider));
                }
                Err(e) => {
                    println!("  OpenAI provider failed: {}", e);
                }
            }
        }
    }

    // Fallback to mock provider
    println!("  Using Mock Provider (random embeddings - not semantically meaningful)");
    println!("  For production use, enable 'openai' or 'local-embeddings' feature");
    Ok(Box::new(MockEmbeddingProvider))
}

/// Get provider name for display
fn provider_name(provider: &Box<dyn EmbeddingProvider>) -> &str {
    let dim = provider.embedding_dimension();
    match dim {
        384 => "Local (sentence-transformers)",
        1536 => "OpenAI (text-embedding-3-small)",
        768 => "Mock Provider",
        _ => "Custom Provider",
    }
}

/// Mock provider for demonstration when no real provider available
struct MockEmbeddingProvider;

#[async_trait::async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Generate deterministic pseudo-random embedding
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let dimension = 768;
        let mut embedding = Vec::with_capacity(dimension);
        let mut seed = hash;

        for _ in 0..dimension {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let value = ((seed >> 16) as f32) / 32768.0 - 1.0;
            embedding.push(value);
        }

        // Normalize
        let magnitude = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for x in &mut embedding {
                *x /= magnitude;
            }
        }

        Ok(embedding)
    }

    fn embedding_dimension(&self) -> usize {
        768
    }

    fn model_name(&self) -> &str {
        "mock-provider"
    }
}
