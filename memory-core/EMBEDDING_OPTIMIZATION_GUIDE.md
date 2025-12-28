# Embedding Provider Optimization Guide

This guide explains the optimization features built into the embedding provider system and how to tune them for different use cases.

## Overview

Each embedding provider has specific characteristics that affect performance, reliability, and cost. The system includes provider-specific optimizations that are automatically configured but can be customized for your needs.

## Optimization Features

### 1. Automatic Retry with Exponential Backoff

**What it does:** Automatically retries failed requests with increasing delays between attempts.

**Default behavior:**
- Retries on network errors, rate limits (429), and server errors (5xx)
- Exponential backoff: 1s, 2s, 4s, 8s...
- Non-retryable errors (4xx except 429) fail immediately

**Configuration:**
```rust
use memory_core::embeddings::{ModelConfig, OptimizationConfig};

let mut config = ModelConfig::openai_3_small();
config.optimization.max_retries = 5;           // Increase retry attempts
config.optimization.retry_delay_ms = 500;      // Faster initial retry
```

### 2. Provider-Specific Timeouts

**What it does:** Sets appropriate timeout values based on provider characteristics.

**Defaults:**
- **OpenAI**: 60 seconds (cloud API with moderate latency)
- **Mistral AI**: 30 seconds (faster response times)
- **Azure OpenAI**: 90 seconds (higher latency due to enterprise routing)
- **Local/Custom**: 10 seconds (local inference, should be fast)

**Configuration:**
```rust
let mut config = ModelConfig::openai_3_small();
config.optimization.timeout_seconds = Some(120);  // Custom timeout
```

### 3. Adaptive Batch Sizing

**What it does:** Automatically splits large batches into provider-appropriate chunk sizes.

**Defaults:**
- **OpenAI**: 2048 items per batch (API limit)
- **Mistral AI**: 128 items per batch (recommended)
- **Azure OpenAI**: 2048 items per batch
- **Local/Custom**: 32 items per batch (avoid OOM)

**Example:**
```rust
let texts = vec!["text1", "text2", /* ... 5000 texts ... */];

// Automatically splits into multiple batches based on provider
let embeddings = provider.embed_batch(&texts).await?;
```

**Configuration:**
```rust
let mut config = ModelConfig::openai_3_small();
config.optimization.max_batch_size = Some(500);  // Custom batch size
```

### 4. Connection Pooling

**What it does:** Reuses HTTP connections for better performance.

**Defaults:**
- **OpenAI**: 20 connections (high throughput)
- **Mistral AI**: 10 connections (moderate throughput)
- **Azure OpenAI**: 15 connections
- **Local/Custom**: 5 connections (low overhead)

**Configuration:**
```rust
let mut config = ModelConfig::openai_3_small();
config.optimization.connection_pool_size = 30;  // More connections
```

### 5. Rate Limiting Information

**What it does:** Provides rate limit information for planning and monitoring.

**Rate Limits (reference):**
- **OpenAI**: 3,000 RPM / 1,000,000 TPM
- **Mistral AI**: 100 RPM / 100,000 TPM
- **Azure OpenAI**: 300 RPM (varies by deployment)
- **Local/Custom**: No limits

**Note:** Rate limits are informational. The system doesn't enforce them but retries on 429 errors.

## Provider-Specific Optimization Profiles

### OpenAI (High Throughput)

**Best for:** Production workloads with high volume

```rust
use memory_core::embeddings::{ModelConfig, OptimizationConfig};

let config = ModelConfig::openai_3_small();
// Already optimized with:
// - 60s timeout
// - 3 retries with 1s base delay
// - 2048 batch size
// - 20 connection pool
// - Compression enabled
```

**When to customize:**
- **Increase timeout** for large batches: `timeout_seconds = Some(120)`
- **Reduce batch size** if hitting rate limits: `max_batch_size = Some(1000)`
- **More retries** for unreliable networks: `max_retries = 5`

### Mistral AI (Balanced)

**Best for:** Cost-effective embeddings with good quality

```rust
let config = ModelConfig::mistral_embed();
// Optimized with:
// - 30s timeout (faster responses)
// - 3 retries with 500ms base delay
// - 128 batch size (recommended limit)
// - 10 connection pool
```

**When to customize:**
- **Increase timeout** for complex texts: `timeout_seconds = Some(60)`
- **Faster retries** for quick feedback: `retry_delay_ms = 250`

### Azure OpenAI (Enterprise)

**Best for:** Enterprise deployments with compliance needs

```rust
let config = ModelConfig::azure_openai(
    "my-deployment", 
    "my-resource", 
    "2023-05-15", 
    1536
);
// Optimized with:
// - 90s timeout (account for enterprise routing)
// - 4 retries with 2s base delay
// - 2048 batch size
// - 15 connection pool
```

**When to customize:**
- **Adjust for your deployment's rate limits**: `rate_limit_rpm = Some(500)`
- **Regional considerations**: `timeout_seconds = Some(120)` for cross-region

### Local/Custom (Low Latency)

**Best for:** Development, testing, or self-hosted models

```rust
let config = ModelConfig::custom(
    "local-model",
    768,
    "http://localhost:1234/v1",
    None
);
// Optimized with:
// - 10s timeout (local should be fast)
// - 2 retries with 100ms delay
// - 32 batch size (avoid OOM)
// - 5 connection pool
// - No compression
```

**When to customize:**
- **GPU inference**: `max_batch_size = Some(64)` if you have VRAM
- **CPU inference**: `max_batch_size = Some(16)` to avoid slowdowns
- **Fast hardware**: `timeout_seconds = Some(5)`

## Custom Optimization Configurations

### High-Reliability Configuration

For critical applications where reliability is paramount:

```rust
use memory_core::embeddings::{ModelConfig, OptimizationConfig};

let mut config = ModelConfig::openai_3_small();
config.optimization = OptimizationConfig {
    timeout_seconds: Some(120),        // Generous timeout
    max_retries: 5,                    // More retry attempts
    retry_delay_ms: 2000,              // Conservative backoff
    max_batch_size: Some(500),         // Smaller batches
    rate_limit_rpm: Some(1000),        // Conservative rate limit
    rate_limit_tpm: Some(500_000),
    compression_enabled: true,
    connection_pool_size: 15,
};
```

### Cost-Optimized Configuration

Minimize costs while maintaining functionality:

```rust
let mut config = ModelConfig::openai_3_small();
config.optimization = OptimizationConfig {
    timeout_seconds: Some(30),         // Fail fast
    max_retries: 2,                    // Fewer retries
    retry_delay_ms: 500,               // Quick retry
    max_batch_size: Some(2048),        // Max batch size
    rate_limit_rpm: Some(3000),
    rate_limit_tpm: Some(1_000_000),
    compression_enabled: true,         // Reduce bandwidth
    connection_pool_size: 10,          // Fewer connections
};
```

### Development/Testing Configuration

Fast feedback during development:

```rust
let mut config = ModelConfig::openai_3_small();
config.optimization = OptimizationConfig {
    timeout_seconds: Some(10),         // Fail fast
    max_retries: 1,                    // Don't wait long
    retry_delay_ms: 100,               // Quick retry
    max_batch_size: Some(10),          // Small batches
    rate_limit_rpm: None,
    rate_limit_tpm: None,
    compression_enabled: false,
    connection_pool_size: 2,
};
```

## Performance Tips

### 1. Batch Processing

Always use `embed_batch()` for multiple texts:

```rust
// ❌ Slow - multiple API calls
for text in texts {
    let embedding = provider.embed_text(text).await?;
}

// ✅ Fast - single API call (or optimally chunked)
let embeddings = provider.embed_batch(&texts).await?;
```

### 2. Pre-warming Connections

For latency-sensitive applications:

```rust
// Warm up the connection pool before main workload
provider.warmup().await?;

// Now ready for fast requests
let embedding = provider.embed_text("important query").await?;
```

### 3. Monitoring Retry Behavior

Enable debug logging to see retry behavior:

```bash
RUST_LOG=memory_core=debug cargo run
```

You'll see logs like:
```
DEBUG Retry attempt 1/3, waiting 1000ms
DEBUG Retryable error 429: Rate limit exceeded
```

### 4. Handling Rate Limits Gracefully

The system automatically handles rate limits, but you can help:

```rust
// Process in smaller batches with delays
for chunk in texts.chunks(100) {
    let embeddings = provider.embed_batch(chunk).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}
```

### 5. Provider-Specific Batch Sizes

Match your batch size to the provider:

```rust
let batch_size = match provider_type {
    "openai" => 2000,
    "mistral" => 100,
    "local" => 32,
    _ => 100,
};

for chunk in texts.chunks(batch_size) {
    let embeddings = provider.embed_batch(chunk).await?;
}
```

## Troubleshooting

### Timeouts

**Problem:** Requests timing out frequently

**Solutions:**
- Increase timeout: `config.optimization.timeout_seconds = Some(120)`
- Reduce batch size: `config.optimization.max_batch_size = Some(500)`
- Check network latency
- Consider local provider for faster response

### Rate Limits

**Problem:** Getting 429 errors despite retries

**Solutions:**
- Reduce batch size to lower token usage
- Add delays between batches
- Upgrade your API plan
- Use multiple API keys with load balancing (not yet implemented)

### Memory Issues (Local Providers)

**Problem:** Out of memory errors with local inference

**Solutions:**
- Reduce batch size: `config.optimization.max_batch_size = Some(8)`
- Use a smaller model
- Increase system RAM or use GPU

### Connection Pool Exhaustion

**Problem:** Slow performance with many concurrent requests

**Solutions:**
- Increase pool size: `config.optimization.connection_pool_size = 30`
- Process requests in smaller concurrent batches
- Consider rate limiting on your side

## Best Practices

1. **Use default optimizations** - They're tuned for typical use cases
2. **Monitor performance** - Enable debug logging to understand behavior
3. **Batch when possible** - Dramatically reduces API calls and cost
4. **Test with production-like data** - Tune based on real workload
5. **Plan for failures** - Use appropriate retry settings for your reliability needs
6. **Consider costs** - Larger batches reduce API calls but may hit rate limits
7. **Profile your provider** - Local providers need different tuning than cloud
8. **Use warmup** - Pre-establish connections for latency-sensitive operations

## Example: Complete Configuration

```rust
use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig, OptimizationConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start with provider-optimized defaults
    let mut config = ModelConfig::openai_3_small();
    
    // Fine-tune for your use case
    config.optimization.max_retries = 5;
    config.optimization.max_batch_size = Some(1000);
    config.optimization.timeout_seconds = Some(90);
    
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
    
    // Warm up connection pool
    provider.warmup().await?;
    
    // Process large batch efficiently
    let texts: Vec<String> = /* your texts */;
    let embeddings = provider.embed_batch(&texts).await?;
    
    println!("Generated {} embeddings", embeddings.len());
    Ok(())
}
```

## See Also

- [EMBEDDING_PROVIDERS.md](EMBEDDING_PROVIDERS.md) - Provider configuration guide
- [README_SEMANTIC_EMBEDDINGS.md](README_SEMANTIC_EMBEDDINGS.md) - API documentation
- [QUICK_START_EMBEDDINGS.md](QUICK_START_EMBEDDINGS.md) - Quick start guide
