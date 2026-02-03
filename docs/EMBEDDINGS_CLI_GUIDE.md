# Embeddings CLI Integration - Usage Guide

## Overview

The memory-cli now supports semantic embeddings for enhanced search capabilities. This guide explains how to configure and use embeddings in the CLI.

## Configuration

### Basic Configuration

Embeddings are configured in the `[embeddings]` section of your config file:

```toml
[embeddings]
enabled = true
provider = "local"
model = "sentence-transformers/all-MiniLM-L6-v2"
dimension = 384
similarity_threshold = 0.7
batch_size = 32
cache_embeddings = true
timeout_seconds = 30
```

### Provider Options

#### Local Provider (Default)

```toml
[embeddings]
enabled = true
provider = "local"
model = "sentence-transformers/all-MiniLM-L6-v2"
dimension = 384
```

- **Cost**: Free (runs on your CPU)
- **Speed**: Fast for small batches
- **Setup**: Requires `--features local-embeddings`
- **Privacy**: All data stays local

#### OpenAI Provider

```toml
[embeddings]
enabled = true
provider = "openai"
model = "text-embedding-3-small"
dimension = 1536
api_key_env = "OPENAI_API_KEY"
```

- **Cost**: $0.02 per 1M tokens
- **Speed**: Very fast (API-based)
- **Setup**: Requires `OPENAI_API_KEY` environment variable
- **Models**:
  - `text-embedding-3-small` (1536 dimensions, default)
  - `text-embedding-3-large` (3072 dimensions, highest quality)
  - `text-embedding-ada-002` (1536 dimensions, legacy)

#### Mistral Provider

```toml
[embeddings]
enabled = true
provider = "mistral"
model = "mistral-embed"
dimension = 1024
api_key_env = "MISTRAL_API_KEY"
```

- **Cost**: See Mistral pricing
- **Speed**: Fast (API-based)
- **Setup**: Requires `MISTRAL_API_KEY` environment variable
- **Models**:
  - `mistral-embed` (1024 dimensions, general text)
  - `codestral-embed` (1536 dimensions, code-specific)

#### Azure OpenAI Provider

```toml
[embeddings]
enabled = true
provider = "azure"
model = "your-deployment-name"
dimension = 1536
api_key_env = "AZURE_OPENAI_API_KEY"
```

- **Setup**: Requires additional environment variables:
  - `AZURE_DEPLOYMENT` - Your deployment name
  - `AZURE_RESOURCE` - Your Azure resource name
  - `AZURE_API_VERSION` - API version (default: 2023-05-15)

#### Custom Provider

```toml
[embeddings]
enabled = true
provider = "custom"
model = "custom-model-name"
dimension = 768
base_url = "https://api.example.com/v1"
```

- Use any OpenAI-compatible embedding API
- Provide base_url and model name

## CLI Commands

### Test Embedding Configuration

```bash
memory-cli embedding test
```

Tests your embedding provider configuration:
- Verifies provider connectivity
- Generates test embeddings
- Measures performance
- Tests similarity calculations

### Show Configuration

```bash
memory-cli embedding config
```

Displays current embedding configuration:
- Provider status (enabled/disabled)
- Provider type and model
- Settings (thresholds, batch size, etc.)
- API key status

### List Available Providers

```bash
memory-cli embedding list-providers
```

Shows all available embedding providers with details:
- Provider names and models
- Dimensions
- Cost information
- Setup requirements

### Benchmark Performance

```bash
memory-cli embedding benchmark
```

Runs performance benchmarks:
- Single embedding generation speed
- Batch embedding generation speed
- Similarity calculation speed

### Enable/Disable Embeddings

```bash
# Enable (session-based)
memory-cli embedding enable

# Disable (session-based)
memory-cli embedding disable
```

Note: To persist changes, edit your config file and set `enabled = true` in the `[embeddings]` section.

## Semantic Search

### Episode Search with Embeddings

```bash
# Basic semantic search
memory-cli episode search --semantic "user authentication"

# With similarity threshold
memory-cli episode search --semantic --similarity 0.8 "API design"

# With limit
memory-cli episode search --semantic --limit 5 "database optimization"
```

### Episode List with Semantic Search

```bash
# List episodes semantically similar to query
memory-cli episode list --semantic-search "REST API"

# With domain filter
memory-cli episode list --semantic-search "web-api" --domain "web-api"

# With outcome filter
memory-cli episode list --semantic-search "testing" --outcome success
```

### Pattern Search

```bash
# Search patterns by semantic similarity
memory-cli pattern list --semantic-search "error handling"
```

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `false` | Enable/disable embeddings |
| `provider` | string | `"local"` | Provider name (local, openai, mistral, azure, custom) |
| `model` | string | `"sentence-transformers/all-MiniLM-L6-v2"` | Model name/identifier |
| `dimension` | int | `384` | Embedding dimension |
| `api_key_env` | string | `null` | Environment variable containing API key |
| `base_url` | string | `null` | Base URL for custom providers |
| `similarity_threshold` | float | `0.7` | Minimum similarity score (0.0-1.0) |
| `batch_size` | int | `32` | Batch size for embedding generation |
| `cache_embeddings` | bool | `true` | Cache embeddings to avoid regeneration |
| `timeout_seconds` | int | `30` | Timeout for embedding requests |

## Best Practices

### Choosing Similarity Threshold

- **0.9 - 1.0**: Very strict, only near-duplicates
- **0.7 - 0.9**: Good balance (recommended default)
- **0.5 - 0.7**: More permissive, broader results
- **0.0 - 0.5**: Very permissive, many weak matches

### Choosing a Provider

| Use Case | Recommended Provider | Reason |
|----------|---------------------|---------|
| Privacy-sensitive | Local | Data stays on your machine |
| High volume | Local | No API costs |
| Best quality | OpenAI (3-large) | State-of-the-art embeddings |
| Code-specific | Mistral (codestral) | Optimized for code |
| Cost-sensitive | Local | Free option available |

### Performance Optimization

1. **Enable caching**: Set `cache_embeddings = true` (default)
2. **Use appropriate batch size**: 32-64 for API providers, lower for local
3. **Adjust timeout**: 30s default, increase for slow connections
4. **Use appropriate dimensions**: Lower dimensions = faster computation

## Troubleshooting

### Embeddings Disabled Error

If you see "Embeddings are disabled", either:
1. Enable in config: Set `enabled = true` in `[embeddings]`
2. Use session flag: `memory-cli embedding enable`

### API Key Not Set Error

For API providers (OpenAI, Mistral, Azure):
```bash
export OPENAI_API_KEY="your-key-here"
# or
export MISTRAL_API_KEY="your-key-here"
# or
export AZURE_OPENAI_API_KEY="your-key-here"
```

### Local Provider Not Available

If using local provider, ensure:
1. Feature flag enabled: `--features local-embeddings`
2. Dependencies installed: Check `memory-core` build output
3. Model downloaded: First run may download model files

### Performance Issues

If embeddings are slow:
1. Check network connection (for API providers)
2. Reduce `batch_size` if memory constrained
3. Increase `timeout_seconds` for slow connections
4. Consider using local provider for privacy/no latency

## Examples

### Example 1: Setup with Local Provider

```bash
# 1. Create config file
cat > memory-cli.toml << EOF
[embeddings]
enabled = true
provider = "local"
model = "sentence-transformers/all-MiniLM-L6-v2"
dimension = 384
similarity_threshold = 0.7
EOF

# 2. Test configuration
memory-cli embedding test

# 3. Search semantically
memory-cli episode search --semantic "database migration"
```

### Example 2: Setup with OpenAI

```bash
# 1. Set API key
export OPENAI_API_KEY="sk-..."

# 2. Create config file
cat > memory-cli.toml << EOF
[embeddings]
enabled = true
provider = "openai"
model = "text-embedding-3-small"
dimension = 1536
api_key_env = "OPENAI_API_KEY"
similarity_threshold = 0.75
EOF

# 3. Test configuration
memory-cli embedding test

# 4. Benchmark performance
memory-cli embedding benchmark
```

### Example 3: Semantic Search in CI/CD

```bash
# Use embeddings in CI with OpenAI
export OPENAI_API_KEY="${{ secrets.OPENAI_API_KEY }}"

memory-cli episode list \
  --semantic-search "authentication" \
  --limit 10 \
  --format json \
  > results.json
```

## Migration from Keyword Search

To migrate from keyword search to semantic search:

**Before:**
```bash
memory-cli episode search "authentication"
```

**After:**
```bash
memory-cli episode search --semantic "authentication"
```

The semantic search will find episodes related to:
- "user login"
- "access control"
- "identity verification"
- "authorization systems"

Even if the exact word "authentication" doesn't appear!

## Integration with Memory System

Embeddings integrate with the memory system in several ways:

1. **Episode Embedding**: Automatically generated when episodes are completed
2. **Pattern Embedding**: Generated for extracted patterns
3. **Semantic Retrieval**: Used in context retrieval for task execution
4. **Similarity Scoring**: Used in hierarchical retrieval system

## API Integration

The CLI embedding commands use the same embedding providers as the memory-core API:

```rust
use memory_core::embeddings::{
    SemanticService,
    EmbeddingConfig,
    LocalEmbeddingProvider,
};

let service = SemanticService::default(storage).await?;
let results = service.find_similar_episodes(
    "database optimization",
    &context,
    10
).await?;
```

## Future Enhancements

Planned features:
- [ ] Hybrid search (keyword + semantic)
- [ ] Re-ranking results
- [ ] Multi-lingual embeddings
- [ ] Custom embedding models
- [ ] Embedding versioning
- [ ] A/B testing different providers
