# memory-storage-turso

[![Crates.io](https://img.shields.io/crates/v/memory-storage-turso.svg)](https://crates.io/crates/memory-storage-turso)
[![Documentation](https://docs.rs/memory-storage-turso/badge.svg)](https://docs.rs/memory-storage-turso)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Turso/libSQL storage backend for the memory-core episodic learning system providing durable, distributed SQL storage.

## Overview

`memory-storage-turso` implements the durable storage layer for AI agent episodic memory using [Turso](https://turso.tech) and libSQL. It provides:

- Distributed SQL database for production deployments
- Local libSQL support for development and testing
- Parameterized queries for SQL injection prevention
- Connection pooling and circuit breaker patterns
- Automatic schema initialization and migrations

## Features

- **Distributed Storage**: Deploy to Turso's edge network for global low-latency access
- **Local Development**: Use local libSQL files for offline development
- **Production Ready**: Connection pooling, circuit breakers, and resilient error handling
- **Secure**: Parameterized queries, TLS enforcement, credential validation
- **Analytics**: Complex SQL queries for pattern analysis and insights
- **Conflict Resolution**: Turso as source of truth with automatic reconciliation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
memory-storage-turso = "0.1"
```

## Quick Start

### With Turso (Production)

```rust
use memory_storage_turso::TursoStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set environment variables
    // TURSO_DATABASE_URL=libsql://your-database.turso.io
    // TURSO_AUTH_TOKEN=your-auth-token

    let storage = TursoStorage::new(
        std::env::var("TURSO_DATABASE_URL")?,
        std::env::var("TURSO_AUTH_TOKEN")?,
    ).await?;

    // Storage is ready to use
    Ok(())
}
```

### With Local libSQL (Development)

```rust
use memory_storage_turso::TursoStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let storage = TursoStorage::new_local("./data/memory.db").await?;

    // Use for development and testing
    Ok(())
}
```

## Configuration

### Environment Variables

```bash
# Production (Turso)
export TURSO_DATABASE_URL="libsql://your-database.turso.io"
export TURSO_AUTH_TOKEN="your-auth-token-here"

# Development (Local libSQL)
export LIBSQL_DATABASE_PATH="./data/memory.db"
```

### Getting Turso Credentials

1. Sign up at [turso.tech](https://turso.tech)
2. Install Turso CLI: `brew install tursodatabase/tap/turso` (macOS)
3. Create a database: `turso db create my-memory-db`
4. Get the URL: `turso db show my-memory-db --url`
5. Create a token: `turso db tokens create my-memory-db`

## Database Schema

The storage layer maintains three main tables:

- **episodes**: Stores complete episode data with JSON fields for steps, context, and artifacts
- **patterns**: Stores extracted patterns with metadata for retrieval
- **heuristics**: Stores learned condition→action rules (future use)

Schema is automatically initialized on first connection.

## Performance

Connection pooling and query optimization ensure excellent performance:

- Concurrent operations: 100+ ops/second
- Connection pool: Configurable size (default: 10)
- Circuit breaker: Automatic failure detection and recovery
- Query caching: Prepared statements for frequently used queries

## Resilience Features

- **Connection Pooling**: Semaphore-based pool management for efficient resource usage
- **Circuit Breaker**: Automatic failure detection with exponential backoff
- **Retry Logic**: Configurable retry strategies for transient failures
- **Health Checks**: Endpoint to verify database connectivity

## Security

- **Parameterized Queries**: All SQL queries use parameter binding to prevent injection
- **TLS Enforcement**: Encrypted connections to Turso (automatic)
- **Credential Validation**: Environment variable validation on startup
- **Input Sanitization**: JSON serialization validation before storage

## Usage with memory-core

```rust
use memory_core::SelfLearningMemory;
use memory_storage_turso::TursoStorage;
use memory_storage_redb::RedbStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let turso = TursoStorage::new(
        std::env::var("TURSO_DATABASE_URL")?,
        std::env::var("TURSO_AUTH_TOKEN")?,
    ).await?;

    let redb = RedbStorage::new("./data/cache.redb")?;

    let memory = SelfLearningMemory::with_storage(turso, redb).await?;

    // Use memory system
    Ok(())
}
```

## Testing

Run integration tests with a local libSQL database:

```bash
cargo test -p memory-storage-turso
```

For production testing against Turso, set environment variables first.

## Documentation

Full API documentation: [docs.rs/memory-storage-turso](https://docs.rs/memory-storage-turso)

## License

Licensed under the MIT License. See [LICENSE](../LICENSE) for details.

## Project

Part of the [rust-self-learning-memory](https://github.com/d-o-hub/rust-self-learning-memory) project.
