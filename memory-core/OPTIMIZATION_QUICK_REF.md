# Embedding Optimization Quick Reference

## TL;DR

All embedding providers now have automatic optimizations:
- ✅ Retry with exponential backoff
- ✅ Smart batching (provider-specific limits)
- ✅ Connection pooling
- ✅ Adaptive timeouts

**No configuration needed** - works optimally by default!

## Provider Comparison

| Feature | OpenAI | Mistral | Azure | Local |
|---------|--------|---------|-------|-------|
| Timeout | 60s | 30s | 90s | 10s |
| Retries | 3 | 3 | 4 | 2 |
| Batch Size | 2048 | 128 | 2048 | 32 |
| Pool Size | 20 | 10 | 15 | 5 |
| Best For | Production | Balanced | Enterprise | Dev/Testing |

## Quick Start

```rust
// Just use it - already optimized!
let config = ModelConfig::openai_3_small();
let provider = OpenAIEmbeddingProvider::new(api_key, config)?;
let embeddings = provider.embed_batch(&texts).await?;
```

## Customize If Needed

```rust
// High-reliability
config.optimization.max_retries = 5;
config.optimization.timeout_seconds = Some(120);

// Cost-optimized
config.optimization.max_retries = 2;
config.optimization.max_batch_size = Some(2048);

// Development
config.optimization.timeout_seconds = Some(10);
config.optimization.max_batch_size = Some(10);
```

## When to Customize

| Scenario | Setting | Value |
|----------|---------|-------|
| Unreliable network | `max_retries` | 5 |
| Large batches | `timeout_seconds` | 120 |
| Rate limit issues | `max_batch_size` | 500 |
| Fast feedback | `timeout_seconds` | 10 |
| GPU inference | `max_batch_size` | 64 |
| CPU inference | `max_batch_size` | 16 |

## Debug Retry Behavior

```bash
RUST_LOG=memory_core=debug cargo run
```

Output shows:
```
DEBUG Retry attempt 1/3, waiting 1000ms
DEBUG Retryable error 429: Rate limit exceeded
```

## Documentation

- **[EMBEDDING_OPTIMIZATION_GUIDE.md](EMBEDDING_OPTIMIZATION_GUIDE.md)** - Complete guide
- **[EMBEDDING_PROVIDERS.md](EMBEDDING_PROVIDERS.md)** - Provider configuration
- **[QUICK_START_EMBEDDINGS.md](QUICK_START_EMBEDDINGS.md)** - Getting started

## Run Demo

```bash
cargo run --example embedding_optimization_demo
```

Shows:
- Default configurations for all providers
- Retry behavior simulation
- Batch size calculations
- Connection pool recommendations
