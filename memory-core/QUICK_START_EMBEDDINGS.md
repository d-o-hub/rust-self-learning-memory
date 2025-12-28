# Quick Start: Embedding Providers

## TL;DR - Copy & Paste Examples

### OpenAI
```rust
use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig};

let api_key = std::env::var("OPENAI_API_KEY")?;
let config = ModelConfig::openai_3_small();
let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
let embedding = provider.embed_text("Hello world").await?;
```

### Mistral AI
```rust
use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig};

let api_key = std::env::var("MISTRAL_API_KEY")?;
let config = ModelConfig::mistral_embed();
let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
let embedding = provider.embed_text("Hello world").await?;
```

### Azure OpenAI
```rust
use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig};

let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
let config = ModelConfig::azure_openai(
    "my-deployment",  // deployment name
    "my-resource",    // resource name
    "2023-05-15",    // API version
    1536             // dimension
);
let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
let embedding = provider.embed_text("Hello world").await?;
```

### Local Server (LM Studio, Ollama, etc.)
```rust
use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig};

let config = ModelConfig::custom(
    "text-embedding-model",
    768,
    "http://localhost:1234/v1",
    None
);
let provider = OpenAIEmbeddingProvider::new("not-needed".to_string(), config)?;
let embedding = provider.embed_text("Hello world").await?;
```

## Model Comparison

| Provider | Model | Dimensions | Cost per 1M tokens |
|----------|-------|------------|--------------------|
| OpenAI | text-embedding-3-small | 1536 | $0.02 |
| OpenAI | text-embedding-3-large | 3072 | $0.13 |
| OpenAI | text-embedding-ada-002 | 1536 | $0.10 |
| Mistral | mistral-embed | 1024 | See Mistral pricing |
| Local | (varies) | (varies) | Free |

## Environment Setup

```bash
# OpenAI
export OPENAI_API_KEY="sk-your-key"

# Mistral
export MISTRAL_API_KEY="your-key"

# Azure OpenAI
export AZURE_OPENAI_API_KEY="your-key"
export AZURE_DEPLOYMENT="your-deployment"
export AZURE_RESOURCE="your-resource"
```

## Cargo.toml

```toml
[dependencies]
memory-core = { version = "0.1.7", features = ["openai"] }
```

## Full Documentation

- **Configuration Guide**: `memory-core/EMBEDDING_PROVIDERS.md`
- **API Reference**: `memory-core/README_SEMANTIC_EMBEDDINGS.md`
- **Example Code**: `memory-core/examples/multi_provider_embeddings.rs`

## Common Issues

**"Failed to create HTTP client"** → Check internet connection and firewall

**"API error 401"** → Verify API key is correct and not expired

**"API error 429"** → Rate limit hit, use batch processing or upgrade plan

**Timeout errors** → Check network latency, consider local provider

## Need Help?

```bash
# Run the example
cargo run --example multi_provider_embeddings --features openai

# Check tests
cargo test --lib embeddings --features openai
```
