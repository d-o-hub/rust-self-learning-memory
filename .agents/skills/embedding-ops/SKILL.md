---
name: embedding-ops
description: Configure, test, benchmark, and manage embedding providers for semantic search and vector operations
---

# Embedding Operations

## When to Use

- Configuring embedding providers (OpenAI, Cohere, Ollama, local)
- Testing embedding connectivity and quality
- Benchmarking provider performance
- Rebuilding embedding indices
- Debugging semantic search issues

## CLI Commands

| Command | Purpose |
|---------|---------|
| `do-memory-cli embedding config` | Show current embedding configuration |
| `do-memory-cli embedding test` | Test embedding provider connectivity |
| `do-memory-cli embedding list-providers` | List available providers |
| `do-memory-cli embedding benchmark` | Benchmark provider latency/throughput |
| `do-memory-cli embedding rebuild` | Rebuild embedding index from episodes |
| `do-memory-cli embedding enable` | Enable embeddings (session only) |
| `do-memory-cli embedding disable` | Disable embeddings (session only) |

## MCP Tools

| Tool | Parameters | Purpose |
|------|-----------|---------|
| `configure_embeddings` | provider, model, dimension, api_key_env | Configure embedding provider |
| `query_semantic_memory` | query, limit, threshold | Semantic similarity search |
| `test_embeddings` | text | Test embedding generation |
| `generate_embedding` | text, provider | Generate embedding vector |
| `search_by_embedding` | embedding, limit | Search by raw vector |
| `embedding_provider_status` | - | Check provider health |

## Configuration

Set in config file or environment:

```toml
[embeddings]
enabled = true
provider = "openai"  # openai | cohere | ollama | local
model = "text-embedding-3-small"
dimension = 1536
api_key_env = "OPENAI_API_KEY"
```

## Provider Setup

### OpenAI
```bash
export OPENAI_API_KEY="sk-..."
do-memory-cli embedding config
do-memory-cli embedding test
```

### Ollama (local)
```bash
ollama pull nomic-embed-text
# Config: provider = "ollama", model = "nomic-embed-text", dimension = 768
```

## Troubleshooting

1. **Test fails**: Check API key env var is set, provider is reachable
2. **Slow search**: Run `benchmark` to identify latency; consider local provider
3. **Poor results**: Check dimension matches model; rebuild index after provider change

## Known Limitations

- `enable`/`disable` commands affect current session only (no persistence)
- `--semantic-search` flag on `episode list` is declared but not yet wired
- HNSW index `save()` is a no-op when `hnsw` feature is enabled
