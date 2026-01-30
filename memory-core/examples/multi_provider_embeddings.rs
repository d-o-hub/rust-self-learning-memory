//! Example demonstrating multiple embedding provider configurations
//!
//! This example shows how to configure various embedding providers:
//! - `OpenAI` (standard API)
//! - Azure `OpenAI`
//! - Custom OpenAI-compatible APIs
//!
//! Run with: cargo run --example `multi_provider_embeddings` --features openai

#![allow(clippy::doc_markdown, clippy::unnecessary_wraps)]

use memory_core::embeddings::{AzureOpenAIConfig, CustomConfig, MistralConfig, OpenAIConfig};

fn main() {
    println!("🌐 Multi-Provider Embedding Configuration Examples\n");
    println!("{}", "=".repeat(60));

    // Example 1: OpenAI Standard API
    println!("\n📘 Example 1: OpenAI Standard API");
    println!("{}", "-".repeat(60));
    let openai_config = OpenAIConfig::text_embedding_3_small();
    println!("Model: {}", openai_config.model.model_name());
    println!("Dimension: {}", openai_config.effective_dimension());
    println!("Full Endpoint: {}", openai_config.embeddings_url());

    #[cfg(feature = "openai")]
    {
        // Uncomment to use (requires OPENAI_API_KEY):
        // let api_key = std::env::var("OPENAI_API_KEY")?;
        // let provider = OpenAIEmbeddingProvider::new(api_key, openai_config)?;
        // let embedding = provider.embed_text("Hello world").await?;
    }

    // Example 2: Mistral AI
    println!("\n🟣 Example 2: Mistral AI");
    println!("{}", "-".repeat(60));
    let mistral_config = MistralConfig::mistral_embed();
    println!("Model: {}", mistral_config.model.model_name());
    println!("Dimension: {}", mistral_config.effective_dimension());

    #[cfg(feature = "openai")]
    {
        // Uncomment to use (requires MISTRAL_API_KEY):
        // let api_key = std::env::var("MISTRAL_API_KEY")?;
        // let provider = OpenAIEmbeddingProvider::new(api_key, mistral_config)?;
        // let embedding = provider.embed_text("Hello world").await?;
    }

    // Example 3: Azure OpenAI
    println!("\n☁️  Example 3: Azure OpenAI Service");
    println!("{}", "-".repeat(60));
    let azure_config = AzureOpenAIConfig::new(
        "my-embedding-deployment",
        "my-openai-resource",
        "2023-05-15",
        1536,
    );
    println!("Deployment: {}", azure_config.deployment_name);
    println!("Dimension: {}", azure_config.embedding_dimension);
    println!("Full Endpoint: {}", azure_config.endpoint_url());

    #[cfg(feature = "openai")]
    {
        // Uncomment to use (requires AZURE_OPENAI_API_KEY):
        // let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
        // let provider = OpenAIEmbeddingProvider::new(api_key, azure_config)?;
        // let embedding = provider.embed_text("Hello world").await?;
    }

    // Example 4: Custom Provider (e.g., LM Studio, Ollama with OpenAI compatibility)
    println!("\n🔧 Example 4: Custom OpenAI-Compatible API");
    println!("{}", "-".repeat(60));
    let custom_config = CustomConfig::new(
        "text-embedding-model",
        768,
        "http://localhost:1234/v1", // LM Studio default
    );
    println!("Model: {}", custom_config.model_name);
    println!("Dimension: {}", custom_config.embedding_dimension);
    println!("Full Endpoint: {}", custom_config.embeddings_url());

    #[cfg(feature = "openai")]
    {
        // Uncomment to use (works with local LM Studio, Ollama, etc.):
        // let api_key = "not-needed-for-local".to_string();
        // let provider = OpenAIEmbeddingProvider::new(api_key, custom_config)?;
        // let embedding = provider.embed_text("Hello world").await?;
    }

    // Example 5: Custom Provider with Custom Endpoint
    println!("\n🛠️  Example 5: Custom API with Custom Endpoint Path");
    println!("{}", "-".repeat(60));
    let custom_endpoint_config =
        CustomConfig::new("custom-embed-model", 1024, "https://api.mycompany.com/ml")
            .with_endpoint("/api/v2/embeddings");
    println!("Model: {}", custom_endpoint_config.model_name);
    println!("Dimension: {}", custom_endpoint_config.embedding_dimension);
    println!("Full Endpoint: {}", custom_endpoint_config.embeddings_url());

    // Example 6: All OpenAI Models
    println!("\n📚 Example 6: All OpenAI Model Options");
    println!("{}", "-".repeat(60));

    let ada_002 = OpenAIConfig::ada_002();
    println!(
        "Legacy: {} ({} dims) - Most cost-effective",
        ada_002.model.model_name(),
        ada_002.effective_dimension()
    );

    let small = OpenAIConfig::text_embedding_3_small();
    println!(
        "Balanced: {} ({} dims) - Best price/performance",
        small.model.model_name(),
        small.effective_dimension()
    );

    let large = OpenAIConfig::text_embedding_3_large();
    println!(
        "Quality: {} ({} dims) - Highest quality",
        large.model.model_name(),
        large.effective_dimension()
    );

    println!("\n✅ Configuration examples complete!");
    println!("\nTo use these configurations:");
    println!("1. Set the appropriate API key environment variable");
    println!("2. Enable the 'openai' feature: cargo build --features openai");
    println!("3. Create a provider with your chosen config");
    println!("4. Call embed_text() or embed_batch() on the provider");

    #[cfg(not(feature = "openai"))]
    {
        println!("\n⚠️  Note: This example was compiled without the 'openai' feature.");
        println!("To see full functionality, run:");
        println!("  cargo run --example multi_provider_embeddings --features openai");
    }
}
