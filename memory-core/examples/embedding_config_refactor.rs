//! Example demonstrating the refactored embedding configuration system
//!
//! This example shows how to:
//! - Configure OpenAI with custom dimensions
//! - Configure Mistral with codestral and output_dtype
//! - Configure codestral with binary embeddings
//! - Use the new ProviderConfig API
//!
//! Run with:
//! ```bash
//! # With OpenAI (requires API key)
//! export OPENAI_API_KEY="sk-your-key"
//! cargo run --example embedding_config_refactor --features openai
//! ```

#![allow(
    clippy::uninlined_format_args,
    clippy::doc_markdown,
    clippy::unnecessary_wraps
)]

use memory_core::embeddings::{
    config::mistral::{MistralConfig, OutputDtype},
    config::openai::OpenAIConfig,
    EmbeddingConfig, ProviderConfig,
};

fn main() {
    println!("🔧 Embedding Configuration Refactor Examples\n");
    println!("{}", "=".repeat(70));

    // ========================================================================
    // Example 1: OpenAI with Custom Dimensions
    // ========================================================================
    println!("\n📘 Example 1: OpenAI text-embedding-3-small with Custom Dimensions");
    println!("{}", "-".repeat(70));

    // Create OpenAI config with 512 dimensions (instead of default 1536)
    let openai_512 = OpenAIConfig::text_embedding_3_small().with_dimensions(512);

    println!("Configuration:");
    println!("  Model: text-embedding-3-small");
    println!("  Dimensions: {}", openai_512.effective_dimension());
    println!("  Encoding: Float");
    println!("  Default dim (full): 1536");
    println!("  Custom dim: 512");
    println!(
        "  Size reduction: {:.1}%",
        100.0 * (1536.0 - 512.0) / 1536.0
    );
    println!("\nBenefits:");
    println!("  • Reduces storage requirements by ~67%");
    println!("  • Faster similarity search (smaller vectors)");
    println!("  • Still captures semantic meaning well");

    println!("\nJSON representation:");
    let json = serde_json::to_string_pretty(&openai_512).unwrap();
    println!("{}", json);

    // ========================================================================
    // Example 2: Mistral with Codestral and Output Dtype
    // ========================================================================
    println!("\n🟣 Example 2: Mistral Codestral with Int8 Quantization");
    println!("{}", "-".repeat(70));

    let mistral_codestral = MistralConfig::codestral_embed()
        .with_output_dimension(512)
        .with_output_dtype(OutputDtype::Int8);

    println!("Configuration:");
    println!("  Model: codestral-embed");
    println!(
        "  Effective dimension: {}",
        mistral_codestral.effective_dimension()
    );
    println!(
        "  Output dimension: {:?}",
        mistral_codestral.output_dimension
    );
    println!("  Output dtype: {:?}", mistral_codestral.output_dtype);
    println!("\nBenefits:");
    println!("  • Int8 reduces memory footprint by 4x (vs float32)");
    println!("  • 512 dim output for faster search");
    println!("  • Codestral optimized for code semantics");

    println!("\nJSON representation:");
    let json = serde_json::to_string_pretty(&mistral_codestral).unwrap();
    println!("{}", json);

    // ========================================================================
    // Example 3: Codestral with Binary Embeddings
    // ========================================================================
    println!("\n🔢 Example 3: Mistral Codestral with Binary Embeddings");
    println!("{}", "-".repeat(70));

    let mistral_binary = MistralConfig::codestral_binary();

    println!("Configuration:");
    println!("  Model: codestral-embed");
    println!(
        "  Effective dimension: {}",
        mistral_binary.effective_dimension()
    );
    println!("  Output dimension: {:?}", mistral_binary.output_dimension);
    println!("  Output dtype: {:?}", mistral_binary.output_dtype);
    println!("\nBenefits:");
    println!("  • Binary: 8x reduction vs float32");
    println!("  • Hamming distance for ultra-fast search");
    println!("  • Perfect for large-scale semantic search");
    println!("  • Minimal accuracy loss for many use cases");

    println!("\nJSON representation:");
    let json = serde_json::to_string_pretty(&mistral_binary).unwrap();
    println!("{}", json);

    // ========================================================================
    // Example 4: ProviderConfig Enum Usage
    // ========================================================================
    println!("\n🏗️  Example 4: Using ProviderConfig Enum");
    println!("{}", "-".repeat(70));

    // Create provider configs using enum variants
    let openai_provider = ProviderConfig::OpenAI(openai_512);
    let mistral_provider = ProviderConfig::Mistral(mistral_codestral.clone());
    let binary_provider = ProviderConfig::Mistral(mistral_binary.clone());

    println!("ProviderConfig Enum Variants:");
    println!("\n  OpenAI Provider:");
    println!("    Variant: {:?}", openai_provider);
    println!("    Dimension: {}", openai_provider.effective_dimension());

    println!("\n  Mistral Provider:");
    println!("    Variant: {:?}", mistral_provider);
    println!("    Dimension: {}", mistral_provider.effective_dimension());

    println!("\n  Binary Provider:");
    println!("    Variant: {:?}", binary_provider);
    println!("    Dimension: {}", binary_provider.effective_dimension());

    // ========================================================================
    // Example 5: Serialization/Deserialization
    // ========================================================================
    println!("\n💾 Example 5: Serialization & Deserialization");
    println!("{}", "-".repeat(70));

    // Serialize to JSON
    let provider =
        ProviderConfig::OpenAI(OpenAIConfig::text_embedding_3_small().with_dimensions(512));

    let json = serde_json::to_string_pretty(&provider).unwrap();
    println!("Serialized ProviderConfig:");
    println!("{}", json);

    // Deserialize from JSON
    let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
    println!("\nDeserialized successfully!");
    println!("  Dimension: {}", deserialized.effective_dimension());
    println!(
        "  Match: {}",
        provider.effective_dimension() == deserialized.effective_dimension()
    );

    // ========================================================================
    // Example 6: Convenience Constructors
    // ========================================================================
    println!("\n⚡ Example 6: Convenience Constructors");
    println!("{}", "-".repeat(70));

    // Quick access to common configurations
    println!("Quick constructors:");

    let openai_default = ProviderConfig::openai_3_small();
    println!(
        "  OpenAI 3-small: {} dims",
        openai_default.effective_dimension()
    );

    let openai_large = ProviderConfig::openai_3_large();
    println!(
        "  OpenAI 3-large: {} dims",
        openai_large.effective_dimension()
    );

    let mistral_default = ProviderConfig::mistral_embed();
    println!(
        "  Mistral embed: {} dims",
        mistral_default.effective_dimension()
    );

    let codestral = ProviderConfig::codestral_embed();
    println!(
        "  Codestral embed: {} dims",
        codestral.effective_dimension()
    );

    let codestral_binary = ProviderConfig::codestral_binary();
    println!(
        "  Codestral binary: {} dims (binary)",
        codestral_binary.effective_dimension()
    );

    let local = ProviderConfig::local_default();
    println!("  Local default: {} dims", local.effective_dimension());

    // ========================================================================
    // Example 7: Integration with EmbeddingConfig
    // ========================================================================
    println!("\n🔗 Example 7: Integration with EmbeddingConfig");
    println!("{}", "-".repeat(70));

    // Create full embedding configuration
    let embedding_config = EmbeddingConfig {
        provider: ProviderConfig::OpenAI(
            OpenAIConfig::text_embedding_3_small().with_dimensions(512),
        ),
        similarity_threshold: 0.7,
        cache_embeddings: true,
        batch_size: 100,
        timeout_seconds: 30,
    };

    println!("Complete EmbeddingConfig:");
    println!("  Provider: OpenAI text-embedding-3-small");
    println!(
        "  Dimensions: {}",
        embedding_config.provider.effective_dimension()
    );
    println!(
        "  Similarity threshold: {}",
        embedding_config.similarity_threshold
    );
    println!("  Caching enabled: {}", embedding_config.cache_embeddings);
    println!("  Batch size: {}", embedding_config.batch_size);

    println!("\nJSON representation:");
    let json = serde_json::to_string_pretty(&embedding_config).unwrap();
    println!("{}", json);

    // ========================================================================
    // Example 8: Optimization Configuration
    // ========================================================================
    println!("\n🚀 Example 8: Optimization Configuration");
    println!("{}", "-".repeat(70));

    let optimized_config = MistralConfig::codestral_embed()
        .with_output_dimension(512)
        .with_output_dtype(OutputDtype::Int8);

    println!("Optimized Mistral Config:");
    println!("  Model: codestral-embed");
    println!(
        "  Output: {} dims (int8)",
        optimized_config.effective_dimension()
    );
    println!("  Timeout: 30s (from optimization config)");
    println!("  Max retries: 3 (from optimization config)");
    println!("  Batch size: 128 (from optimization config)");
    println!("  Compression: enabled (from optimization config)");
    println!("  Rate limit: 100 RPM, 10k TPM (from optimization config)");
    println!("  Connection pool: 10 (from optimization config)");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n✅ Refactor Complete!");
    println!("{}", "=".repeat(70));
    println!("\nKey Changes from ModelConfig:");
    println!("  1. Type-safe enum variants for each provider");
    println!("  2. Provider-specific configuration fields");
    println!("  3. Support for dimensions, encoding_format (OpenAI)");
    println!("  4. Support for output_dimension, output_dtype (Mistral)");
    println!("  5. Better serialization with tagged enums");
    println!("  6. Convenience constructors for common configs");
    println!("  7. Clear separation of concerns");
    println!("\nMigration Guide:");
    println!("  • ModelConfig::openai_3_small() → ProviderConfig::openai_3_small()");
    println!("  • ModelConfig::mistral_embed() → ProviderConfig::mistral_embed()");
    println!("  • ModelConfig::local_sentence_transformer() → LocalConfig::new()");
    println!("  • ModelConfig::azure_openai() → AzureOpenAIConfig::new()");
    println!("  • ModelConfig::custom() → CustomConfig::new()");
}
