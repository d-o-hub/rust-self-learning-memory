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

use memory_core::embeddings::{EmbeddingProvider, EmbeddingResult, MockEmbeddingProvider};
use memory_core::{ComplexityLevel, SelfLearningMemory, TaskContext, TaskOutcome, TaskType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Memory-Core: End-to-End Embeddings Example");
    println!("{}", "=".repeat(60));
    println!();

    // Step 1: Initialize Memory System
    println!("📦 Step 1: Initializing memory system...");
    let memory = SelfLearningMemory::new();
    println!("✅ Memory system initialized\n");

    // Step 2: Initialize Embedding Provider
    println!("🧠 Step 2: Initializing embedding provider...");
    let provider = initialize_provider().await?;
    println!("✅ Using provider: {}", provider_name(&provider));
    println!("   Dimension: {}\n", provider.dimension());

    // Step 3: Create sample episodes with different domains
    println!("📝 Step 3: Creating sample episodes with embeddings...");
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

    for (desc, domain, tags) in episodes {
        // Generate embedding for task description
        let result = provider.embed_text(desc).await?;
        println!(
            "  • Created episode: '{}' (embedding: {} dims)",
            desc,
            result.embedding.len()
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
            .start_episode(desc.to_string(), context, TaskType::Feature)
            .await?;

        // Log some steps
        memory
            .log_step(episode_id, format!("Analyzing requirements for {}", desc))
            .await?;
        memory
            .log_step(episode_id, format!("Implementing {}", desc))
            .await?;
        memory
            .log_step(episode_id, format!("Testing {}", desc))
            .await?;

        // Complete episode
        memory
            .complete_episode(episode_id, TaskOutcome::Success, None)
            .await?;
    }
    println!("✅ Created {} episodes\n", episodes.len());

    // Step 4: Demonstrate semantic search
    println!("🔍 Step 4: Performing semantic similarity searches...");
    println!();

    let queries = vec![
        ("user authentication", "web-api", vec!["auth"]),
        ("database optimization", "database", vec!["performance"]),
        ("frontend UI component", "frontend", vec!["ui"]),
    ];

    for (query, domain, tags) in queries {
        println!("Query: \"{}\"", query);
        println!("{}", "-".repeat(60));

        // Generate embedding for query
        let query_result = provider.embed_text(query).await?;
        println!(
            "  Query embedding: {} dimensions",
            query_result.embedding.len()
        );

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
    println!("📊 Step 5: Direct similarity calculations...");
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
    println!("⚡ Step 6: Batch embedding generation...");
    let batch_texts = vec![
        "Implement user authentication",
        "Create database migration",
        "Build API endpoint",
        "Write unit tests",
    ];

    let batch_results = provider.embed_batch(&batch_texts).await?;
    println!("  Generated {} embeddings in batch", batch_results.len());
    for (i, result) in batch_results.iter().enumerate() {
        println!(
            "    {}. '{}' → {} dims",
            i + 1,
            batch_texts[i],
            result.embedding.len()
        );
    }
    println!();

    // Summary
    println!("✨ Example Complete!");
    println!();
    println!("Key Takeaways:");
    println!("  • Embeddings enable semantic (meaning-based) search");
    println!("  • Multiple providers supported (local, OpenAI, etc.)");
    println!("  • Batch processing available for efficiency");
    println!("  • Seamlessly integrates with memory system");
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
                println!("  ℹ️  Using Local Embedding Provider (CPU-based)");
                return Ok(Box::new(provider));
            }
            Err(e) => {
                println!("  ⚠️  Local provider failed: {}", e);
            }
        }
    }

    #[cfg(feature = "openai")]
    {
        use memory_core::embeddings::{ModelConfig, OpenAIEmbeddingProvider};
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            match OpenAIEmbeddingProvider::new(api_key, ModelConfig::openai_3_small()) {
                Ok(provider) => {
                    println!("  ℹ️  Using OpenAI Embedding Provider");
                    return Ok(Box::new(provider));
                }
                Err(e) => {
                    println!("  ⚠️  OpenAI provider failed: {}", e);
                }
            }
        }
    }

    // Fallback to mock provider
    println!("  ⚠️  Using Mock Provider (random embeddings - not semantically meaningful)");
    println!("  ℹ️  For production use, enable 'openai' or 'local-embeddings' feature");
    Ok(Box::new(MockEmbeddingProvider))
}

/// Get provider name for display
fn provider_name(provider: &Box<dyn EmbeddingProvider>) -> &str {
    // This is a simplified check - in production you'd use proper type checking
    let dim = provider.dimension();
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
    async fn embed_text(&self, text: &str) -> anyhow::Result<EmbeddingResult> {
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

        Ok(EmbeddingResult::new(embedding, "mock-provider".to_string()))
    }

    async fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<EmbeddingResult>> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed_text(text).await?);
        }
        Ok(results)
    }

    async fn similarity(&self, text1: &str, text2: &str) -> anyhow::Result<f32> {
        let emb1 = self.embed_text(text1).await?;
        let emb2 = self.embed_text(text2).await?;

        let dot_product: f32 = emb1
            .embedding
            .iter()
            .zip(emb2.embedding.iter())
            .map(|(a, b)| a * b)
            .sum();

        Ok(dot_product)
    }

    fn dimension(&self) -> usize {
        768
    }

    fn model_name(&self) -> &str {
        "mock-provider"
    }
}
