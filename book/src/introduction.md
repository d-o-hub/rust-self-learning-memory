# Self-Learning Memory System

A Rust-based self-learning memory system with episodic memory, pattern extraction, and intelligent retrieval capabilities.

## Overview

The Self-Learning Memory System provides:

- **Episodic Memory**: Store and retrieve task execution episodes with context
- **Pattern Extraction**: Automatically extract patterns from successful task completions
- **Intelligent Retrieval**: Semantic search with multiple embedding providers
- **MCP Server**: Model Context Protocol server for AI assistant integration
- **CLI Tools**: Command-line interface for all operations

## Key Features

| Feature | Description |
|---------|-------------|
| Episode Management | Create, log steps, and complete episodes |
| Pattern Recognition | Extract and use patterns for recommendations |
| Multi-provider Embeddings | OpenAI, Cohere, Ollama, local models |
| Dual Storage | Turso (persistent) + redb (cache) |
| MCP Protocol | Full MCP 2025-11-25 support |

## Quick Start

```bash
# Install the CLI
cargo install do-memory-cli

# Start the MCP server
do-memory-mcp-server

# Create an episode
do-memory-cli episode create --task "My task"
```

## Architecture

The system consists of several crates:

- `do-memory-core`: Core types and logic
- `do-memory-storage-turso`: Turso/libSQL backend
- `do-memory-storage-redb`: redb cache backend
- `do-memory-mcp`: MCP server
- `do-memory-cli`: Command-line interface

See the [Architecture](./architecture.md) chapter for details.