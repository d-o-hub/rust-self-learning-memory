//! Example demonstrating embedding provider optimization features
//!
//! This example shows how to:
//! - Use provider-specific optimizations
//! - Customize retry and timeout settings
//! - Optimize batch processing
//! - Handle large datasets efficiently
//!
//! Run with: cargo run --example `embedding_optimization_demo`

#![allow(
    clippy::uninlined_format_args,
    clippy::doc_markdown,
    clippy::unnecessary_wraps
)]

use memory_core::embeddings::{MistralConfig, OpenAIConfig, OptimizationConfig, ProviderConfig};

fn main() {
    println!("🚀 Embedding Provider Optimization Demo\n");
    println!("{}", "=".repeat(70));

    // Example 1: Default Provider Optimizations
    println!("\n📊 Example 1: Default Provider Optimizations");
    println!("{}", "-".repeat(70));

    let openai_config = OpenAIConfig::text_embedding_3_small();
    print_optimization_config("OpenAI (3-small)", &openai_config.optimization);

    let mistral_config = MistralConfig::mistral_embed();
    print_optimization_config("Mistral AI", &mistral_config.optimization);

    let azure_config =
        ProviderConfig::AzureOpenAI(memory_core::embeddings::AzureOpenAIConfig::new(
            "deployment",
            "resource",
            "2023-05-15",
            1536,
        ));
    print_optimization_config("Azure OpenAI", &azure_config.optimization());

    let custom_config = ProviderConfig::Custom(memory_core::embeddings::CustomConfig::new(
        "model",
        768,
        "http://localhost:1234/v1",
    ));
    print_optimization_config("Local/Custom", &custom_config.optimization());

    // Example 2: Custom High-Reliability Configuration
    println!("\n🛡️  Example 2: High-Reliability Configuration");
    println!("{}", "-".repeat(70));

    let mut high_reliability = OpenAIConfig::text_embedding_3_small();
    high_reliability.optimization = OptimizationConfig {
        timeout_seconds: Some(120),
        max_retries: 5,
        retry_delay_ms: 2000,
        max_batch_size: Some(500),
        rate_limit_rpm: Some(1000),
        rate_limit_tpm: Some(500_000),
        compression_enabled: true,
        connection_pool_size: 15,
        ..Default::default()
    };

    println!("Configuration for mission-critical applications:");
    print_optimization_config("High-Reliability", &high_reliability.optimization);
    println!("\nBenefits:");
    println!("  • Generous timeout (120s) for complex operations");
    println!("  • More retry attempts (5) for transient failures");
    println!("  • Conservative batching (500) to avoid rate limits");
    println!("  • Slower backoff (2s) gives servers time to recover");

    // Example 3: Cost-Optimized Configuration
    println!("\n💰 Example 3: Cost-Optimized Configuration");
    println!("{}", "-".repeat(70));

    let mut cost_optimized = OpenAIConfig::text_embedding_3_small();
    cost_optimized.optimization = OptimizationConfig {
        timeout_seconds: Some(30),
        max_retries: 2,
        retry_delay_ms: 500,
        max_batch_size: Some(2048),
        rate_limit_rpm: Some(3000),
        rate_limit_tpm: Some(1_000_000),
        compression_enabled: true,
        connection_pool_size: 10,
        ..Default::default()
    };

    println!("Configuration to minimize API costs:");
    print_optimization_config("Cost-Optimized", &cost_optimized.optimization);
    println!("\nBenefits:");
    println!("  • Shorter timeout (30s) fails fast on issues");
    println!("  • Fewer retries (2) reduces redundant API calls");
    println!("  • Maximum batching (2048) reduces request count");
    println!("  • Compression enabled reduces bandwidth costs");

    // Example 4: Development/Testing Configuration
    println!("\n🔧 Example 4: Development/Testing Configuration");
    println!("{}", "-".repeat(70));

    let mut dev_config = OpenAIConfig::text_embedding_3_small();
    dev_config.optimization = OptimizationConfig {
        timeout_seconds: Some(10),
        max_retries: 1,
        retry_delay_ms: 100,
        max_batch_size: Some(10),
        rate_limit_rpm: None,
        rate_limit_tpm: None,
        compression_enabled: false,
        connection_pool_size: 2,
        ..Default::default()
    };

    println!("Configuration for rapid development iteration:");
    print_optimization_config("Development", &dev_config.optimization);
    println!("\nBenefits:");
    println!("  • Very short timeout (10s) for quick feedback");
    println!("  • Minimal retries (1) to avoid long waits");
    println!("  • Small batches (10) for easier debugging");
    println!("  • No compression for simpler HTTP debugging");

    // Example 5: Batch Size Comparison
    println!("\n📦 Example 5: Batch Size Comparison");
    println!("{}", "-".repeat(70));

    let total_texts = 5000;

    println!(
        "Processing {} texts with different providers:\n",
        total_texts
    );

    let providers = vec![
        ("OpenAI (2048/batch)", 2048),
        ("Mistral (128/batch)", 128),
        ("Azure OpenAI (2048/batch)", 2048),
        ("Local (32/batch)", 32),
    ];

    for (name, batch_size) in providers {
        let num_batches = (total_texts + batch_size - 1) / batch_size;
        let last_batch_size = total_texts % batch_size;
        let last_batch_size = if last_batch_size == 0 {
            batch_size
        } else {
            last_batch_size
        };

        println!(
            "{:30} → {} batches ({} items each, last batch: {})",
            name, num_batches, batch_size, last_batch_size
        );
    }

    // Example 6: Retry Behavior Simulation
    println!("\n🔄 Example 6: Retry Behavior Simulation");
    println!("{}", "-".repeat(70));

    println!("Simulating exponential backoff for different configurations:\n");

    let configs = vec![
        ("OpenAI Default", 3, 1000),
        ("Mistral Default", 3, 500),
        ("High-Reliability", 5, 2000),
        ("Cost-Optimized", 2, 500),
    ];

    for (name, max_retries, base_delay_ms) in configs {
        println!(
            "{} (max_retries={}, base_delay={}ms):",
            name, max_retries, base_delay_ms
        );

        let mut total_time_ms = 0;
        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay_ms = base_delay_ms * 2u64.pow(attempt - 1);
                total_time_ms += delay_ms;
                println!(
                    "  Attempt {}: wait {}ms (total: {}ms)",
                    attempt, delay_ms, total_time_ms
                );
            } else {
                println!("  Attempt {}: immediate", attempt);
            }
        }
        println!("  Total time if all retries needed: {}ms\n", total_time_ms);
    }

    // Example 7: Connection Pool Utilization
    println!("\n🔌 Example 7: Connection Pool Sizing");
    println!("{}", "-".repeat(70));

    println!("Recommended pool sizes for concurrent operations:\n");

    let scenarios = vec![
        ("Single-threaded sequential", 2, "Minimal overhead"),
        ("Low concurrency (2-5 workers)", 5, "Local/custom providers"),
        (
            "Medium concurrency (5-10 workers)",
            10,
            "Mistral AI default",
        ),
        ("High concurrency (10-20 workers)", 20, "OpenAI default"),
        (
            "Very high concurrency (20+ workers)",
            30,
            "Enterprise workloads",
        ),
    ];

    for (scenario, pool_size, notes) in scenarios {
        println!("{:35} → Pool size: {:2}  ({})", scenario, pool_size, notes);
    }

    // Summary
    println!("\n✅ Optimization Features Summary");
    println!("{}", "=".repeat(70));
    println!(
        "
1. **Automatic Retry**: Exponential backoff handles transient failures
2. **Smart Batching**: Provider-specific limits maximize efficiency
3. **Connection Pooling**: Reuse connections for better performance
4. **Adaptive Timeouts**: Each provider gets appropriate timeout values
5. **Configurable**: Fine-tune every parameter for your use case

See EMBEDDING_OPTIMIZATION_GUIDE.md for detailed strategies.
"
    );
}

fn print_optimization_config(name: &str, config: &OptimizationConfig) {
    println!("\n{} Optimization Settings:", name);
    println!(
        "  Timeout:         {} seconds",
        config.get_timeout_seconds()
    );
    println!("  Max Retries:     {}", config.max_retries);
    println!(
        "  Retry Delay:     {}ms (exponential backoff)",
        config.retry_delay_ms
    );
    println!("  Max Batch Size:  {}", config.get_max_batch_size());
    println!(
        "  Rate Limit RPM:  {}",
        format_optional(config.rate_limit_rpm)
    );
    println!(
        "  Rate Limit TPM:  {}",
        format_optional(config.rate_limit_tpm)
    );
    println!(
        "  Compression:     {}",
        if config.compression_enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!(
        "  Pool Size:       {} connections",
        config.connection_pool_size
    );
}

fn format_optional<T: std::fmt::Display>(opt: Option<T>) -> String {
    match opt {
        Some(val) => format!("{}", val),
        None => "No limit".to_string(),
    }
}
