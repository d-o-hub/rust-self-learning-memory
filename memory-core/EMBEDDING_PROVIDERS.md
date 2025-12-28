# Embedding Provider Configuration Guide

This guide explains how to configure different embedding providers for the memory-core system.

## Overview

The embedding system now supports configurable API endpoints, allowing you to use any OpenAI-compatible API provider without hardcoded URLs. This includes:

- **OpenAI** - Standard OpenAI API
- **Mistral AI** - Mistral's embedding models
- **Azure OpenAI** - Enterprise Microsoft Azure deployment
- **Custom Providers** - Any OpenAI-compatible API (local or cloud)

## Quick Reference

| Provider | Model | Dimensions | Base URL |
|----------|-------|------------|----------|
| OpenAI | text-embedding-3-small | 1536 | https://api.openai.com/v1 |
| OpenAI | text-embedding-3-large | 3072 | https://api.openai.com/v1 |
| Mistral AI | mistral-embed | 1024 | https://api.mistral.ai/v1 |
| Azure OpenAI | (deployment-specific) | varies | https://{resource}.openai.azure.com |
| Custom | (model-specific) | varies | (configurable) |

## Configuration Examples

### 1. OpenAI (Standard)

```rust
use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get API key from environment
    let api_key = std::env::var("OPENAI_API_KEY")?;
    
    // Choose a model configuration
    let config = ModelConfig::openai_3_small(); // 1536 dimensions
    // or: ModelConfig::openai_3_large();  // 3072 dimensions
    // or: ModelConfig::openai_ada_002();  // 1536 dimensions (legacy)
    
    // Create provider
    let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
    
    // Generate embeddings
    let embedding = provider.embed_text("Hello, world!").await?;
    println!("Generated {} dimensional embedding", embedding.len());
    
    Ok(())
}
```

**Environment Setup:**
```bash
export OPENAI_API_KEY="sk-your-api-key-here"
```

### 2. Mistral AI

```rust
use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("MISTRAL_API_KEY")?;
    let config = ModelConfig::mistral_embed(); // 1024 dimensions
    let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
    
    let embedding = provider.embed_text("Hello, world!").await?;
    println!("Generated {} dimensional embedding", embedding.len());
    
    Ok(())
}
```

**Environment Setup:**
```bash
export MISTRAL_API_KEY="your-mistral-api-key"
```

### 3. Azure OpenAI Service

```rust
use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
    
    // Configure Azure-specific settings
    let config = ModelConfig::azure_openai(
        "my-embedding-deployment",  // Your Azure deployment name
        "my-openai-resource",       // Your Azure resource name
        "2023-05-15",              // API version
        1536                       // Embedding dimension of your model
    );
    
    let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
    let embedding = provider.embed_text("Hello, world!").await?;
    
    Ok(())
}
```

**Environment Setup:**
```bash
export AZURE_OPENAI_API_KEY="your-azure-api-key"
```

**Azure Endpoint Format:**
The provider automatically constructs the Azure endpoint:
```
https://{resource}.openai.azure.com/openai/deployments/{deployment}/embeddings?api-version={version}
```

### 4. Custom Provider (Local or Self-Hosted)

For local embedding servers like LM Studio, Ollama with OpenAI compatibility, or self-hosted solutions:

```rust
use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // API key might not be needed for local servers
    let api_key = "not-needed".to_string();
    
    // Configure for local LM Studio
    let config = ModelConfig::custom(
        "text-embedding-model",          // Model name
        768,                             // Embedding dimension
        "http://localhost:1234/v1",     // Base URL
        None                             // Use default /embeddings endpoint
    );
    
    let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
    let embedding = provider.embed_text("Hello, world!").await?;
    
    Ok(())
}
```

**Common Local Endpoints:**
- **LM Studio**: `http://localhost:1234/v1`
- **Ollama** (with OpenAI compatibility): `http://localhost:11434/v1`
- **LocalAI**: `http://localhost:8080/v1`

### 5. Custom Provider with Custom Endpoint

For APIs with non-standard endpoint paths:

```rust
let config = ModelConfig::custom(
    "custom-model",
    1024,
    "https://api.example.com/ml",
    Some("/api/v2/embeddings")  // Custom endpoint path
);
```

This will use: `https://api.example.com/ml/api/v2/embeddings`

## Optimization Features

All embedding providers include built-in optimizations:

- **Automatic Retry**: Exponential backoff for transient failures
- **Smart Batching**: Provider-specific batch size limits
- **Connection Pooling**: Reuse HTTP connections for better performance
- **Adaptive Timeouts**: Provider-specific timeout values

See [EMBEDDING_OPTIMIZATION_GUIDE.md](EMBEDDING_OPTIMIZATION_GUIDE.md) for detailed optimization strategies.

### Quick Optimization Examples

**OpenAI - High Volume:**
```rust
let mut config = ModelConfig::openai_3_small();
// Automatically configured with:
// - 60s timeout
// - 2048 batch size
// - 3 retries with exponential backoff
// - 20 connection pool
```

**Mistral AI - Balanced:**
```rust
let mut config = ModelConfig::mistral_embed();
// Automatically configured with:
// - 30s timeout
// - 128 batch size
// - Faster retry delays
```

**Custom Tuning:**
```rust
let mut config = ModelConfig::openai_3_small();
config.optimization.max_retries = 5;
config.optimization.max_batch_size = Some(1000);
config.optimization.timeout_seconds = Some(120);
```

## Advanced Usage

### Batch Processing

All providers support batch processing for better efficiency:

```rust
let texts = vec![
    "First text".to_string(),
    "Second text".to_string(),
    "Third text".to_string(),
];

let embeddings = provider.embed_batch(&texts).await?;
println!("Generated {} embeddings", embeddings.len());
```

### Provider Metadata

Get information about the configured provider:

```rust
let metadata = provider.metadata();
println!("Provider info: {}", serde_json::to_string_pretty(&metadata)?);
```

### Checking Availability

Test if the provider is accessible:

```rust
if provider.is_available().await {
    println!("Provider is ready!");
} else {
    println!("Provider is not available");
}
```

### Warming Up Connection

Pre-initialize the connection:

```rust
provider.warmup().await?;
println!("Provider warmed up and ready");
```

## Configuration via Environment Variables

You can create a configuration loader that reads from environment:

```rust
use memory_core::embeddings::{ModelConfig, OpenAIEmbeddingProvider};

fn create_provider_from_env() -> anyhow::Result<OpenAIEmbeddingProvider> {
    let provider_type = std::env::var("EMBEDDING_PROVIDER")
        .unwrap_or_else(|_| "openai".to_string());
    
    let api_key = match provider_type.as_str() {
        "openai" => std::env::var("OPENAI_API_KEY")?,
        "mistral" => std::env::var("MISTRAL_API_KEY")?,
        "azure" => std::env::var("AZURE_OPENAI_API_KEY")?,
        _ => std::env::var("CUSTOM_API_KEY")?,
    };
    
    let config = match provider_type.as_str() {
        "openai" => ModelConfig::openai_3_small(),
        "mistral" => ModelConfig::mistral_embed(),
        "azure" => {
            let deployment = std::env::var("AZURE_DEPLOYMENT")?;
            let resource = std::env::var("AZURE_RESOURCE")?;
            let version = std::env::var("AZURE_API_VERSION")
                .unwrap_or_else(|_| "2023-05-15".to_string());
            ModelConfig::azure_openai(&deployment, &resource, &version, 1536)
        },
        "custom" => {
            let base_url = std::env::var("CUSTOM_BASE_URL")?;
            let model = std::env::var("CUSTOM_MODEL")?;
            let dimension = std::env::var("CUSTOM_DIMENSION")?.parse()?;
            ModelConfig::custom(&model, dimension, &base_url, None)
        },
        _ => anyhow::bail!("Unknown provider type: {}", provider_type),
    };
    
    OpenAIEmbeddingProvider::new(api_key, config)
}
```

## Cost Considerations

### OpenAI Pricing (as of 2024)
- **text-embedding-ada-002**: $0.10 per 1M tokens
- **text-embedding-3-small**: $0.02 per 1M tokens (5x cheaper!)
- **text-embedding-3-large**: $0.13 per 1M tokens (highest quality)

### Mistral AI Pricing
- **mistral-embed**: Check [Mistral AI pricing](https://mistral.ai/pricing)

### Cost Optimization Tips
1. Use batch processing when possible
2. Cache embeddings for repeated content
3. Choose the right model for your use case
4. Consider local models for high-volume, non-critical workloads

## Troubleshooting

### Common Issues

**Issue: "Failed to create HTTP client"**
- Check your internet connection
- Verify firewall settings
- Ensure the base URL is accessible

**Issue: "OpenAI API error 401"**
- Verify your API key is correct
- Check that the API key has the necessary permissions
- Ensure the API key is not expired

**Issue: "OpenAI API error 429"**
- You've hit rate limits
- Implement exponential backoff
- Consider upgrading your API plan
- Use batch processing to reduce request count

**Issue: "Timeout errors"**
- Increase timeout in provider configuration
- Check network latency
- Consider using a local provider for better performance

## Migration Guide

### From Hardcoded URLs

**Before:**
```rust
// Old code with hardcoded URL
let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
// URL was always https://api.openai.com/v1
```

**After:**
```rust
// New code with flexible configuration
let config = ModelConfig::openai_3_small(); // URL included in config
let provider = OpenAIEmbeddingProvider::new(api_key, config)?;

// Or use a different provider
let config = ModelConfig::mistral_embed(); // Different URL
let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
```

### Backward Compatibility

All existing code continues to work! The default behavior is unchanged:
- `ModelConfig::openai_*()` methods now include the OpenAI base URL
- If no base URL is specified, it defaults to OpenAI's endpoint

## Examples

See the `memory-core/examples/` directory for complete examples:
- `multi_provider_embeddings.rs` - Configuration examples for all providers
- `semantic_embeddings_demo.rs` - Basic semantic search demo

## API Reference

For complete API documentation, see the inline documentation in:
- `memory-core/src/embeddings/config.rs` - Configuration structures
- `memory-core/src/embeddings/openai.rs` - OpenAI provider implementation
- `memory-core/src/embeddings/provider.rs` - Provider trait

## Related Documentation

- **[EMBEDDING_OPTIMIZATION_GUIDE.md](EMBEDDING_OPTIMIZATION_GUIDE.md)** - Performance tuning and optimization strategies
- **[QUICK_START_EMBEDDINGS.md](QUICK_START_EMBEDDINGS.md)** - Quick reference for getting started
- **[README_SEMANTIC_EMBEDDINGS.md](README_SEMANTIC_EMBEDDINGS.md)** - Complete semantic embeddings guide

## Support

For issues or questions:
- Check the [main README](../README_SEMANTIC_EMBEDDINGS.md)
- Open an issue on GitHub
- Review the example code
