# memory-storage-redb

[![Crates.io](https://img.shields.io/crates/v/memory-storage-redb.svg)](https://crates.io/crates/memory-storage-redb)
[![Documentation](https://docs.rs/memory-storage-redb/badge.svg)](https://docs.rs/memory-storage-redb)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

redb embedded storage backend for memory-core episodic learning system providing fast key-value caching.

## Overview

`memory-storage-redb` implements the high-performance cache layer for AI agent episodic memory using [redb](https://www.redb.org), an embedded key-value database. It provides:

- Extremely fast reads for hot-path operations (< 10µs)
- Embedded database with no external dependencies
- LRU eviction with configurable cache size limits
- TTL-based expiration for automatic cleanup
- ACID transactions with crash recovery

## Features

- **Blazing Fast**: Sub-microsecond read latency for cached data
- **Embedded**: No separate server process required
- **LRU Caching**: Automatic eviction of least recently used entries
- **TTL Support**: Configurable time-to-live for cache entries
- **Safe Concurrency**: Multiple readers with single writer (MVCC)
- **Crash Safe**: ACID guarantees with automatic recovery
- **Zero Copy**: Direct memory mapping for minimal overhead

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
memory-storage-redb = "0.1"
```

## Quick Start

```rust
use memory_storage_redb::RedbStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create cache with default settings
    let cache = RedbStorage::new("./data/cache.redb")?;

    // Configure cache size (max episodes)
    let cache = RedbStorage::with_config(
        "./data/cache.redb",
        1000,  // max_cache_size
    )?;

    // Cache is ready to use
    Ok(())
}
```

## Configuration

### Basic Configuration

```rust
use memory_storage_redb::{RedbStorage, RedbConfig};

let config = RedbConfig {
    max_cache_size: 1000,        // Maximum episodes to cache
    ttl_seconds: Some(3600),      // 1 hour TTL
    enable_compression: true,     // Enable value compression
};

let cache = RedbStorage::with_full_config("./data/cache.redb", config)?;
```

### Environment Variables

```bash
# Optional: Configure cache settings
export REDB_CACHE_PATH="./data/cache.redb"
export REDB_MAX_CACHE_SIZE="1000"
export REDB_TTL_SECONDS="3600"
```

## Cache Tables

The cache maintains several tables for different data types:

- **episodes**: Full episode data for fast retrieval
- **patterns**: Extracted patterns with similarity keys
- **embeddings**: Vector embeddings (if enabled)
- **metadata**: Cache statistics and health info

## Performance

Optimized for extremely fast read operations:

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Cache Read | < 10µs | 100K+ ops/s |
| Cache Write | < 100µs | 10K+ ops/s |
| LRU Eviction | < 1ms | Background |
| TTL Cleanup | < 10ms | Background |

## LRU Cache Strategy

The cache uses an LRU (Least Recently Used) eviction policy:

1. Cache fills up to `max_cache_size`
2. New entries trigger eviction of oldest entries
3. Access updates entry timestamp
4. TTL expiration runs in background

## Usage with memory-core

```rust
use memory_core::SelfLearningMemory;
use memory_storage_turso::TursoStorage;
use memory_storage_redb::RedbStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Turso for durable storage
    let turso = TursoStorage::new(
        std::env::var("TURSO_DATABASE_URL")?,
        std::env::var("TURSO_AUTH_TOKEN")?,
    ).await?;

    // redb for fast caching
    let redb = RedbStorage::new("./data/cache.redb")?;

    // Hybrid storage for best performance
    let memory = SelfLearningMemory::with_storage(turso, redb).await?;

    Ok(())
}
```

## Cache Synchronization

The cache automatically synchronizes with Turso:

- **Write-through**: Writes go to both Turso and redb
- **Read-aside**: Reads check cache first, then Turso
- **Invalidation**: Cache can be refreshed from Turso
- **Reconciliation**: Periodic sync ensures consistency

```rust
// Manual cache refresh
memory.sync_memories().await?;

// Get cache statistics
let stats = cache.get_stats()?;
println!("Cache hit rate: {:.2}%", stats.hit_rate * 100.0);
```

## Background Tasks

The cache runs background tasks for maintenance:

- **TTL Cleanup**: Removes expired entries (every 60s)
- **LRU Eviction**: Maintains size limits (on write)
- **Compression**: Optional value compression (configurable)
- **Health Checks**: Monitors cache health (on demand)

## File Management

redb stores all data in a single file:

```bash
# Check cache file size
ls -lh ./data/cache.redb

# Backup cache
cp ./data/cache.redb ./backups/cache-$(date +%Y%m%d).redb

# Clear cache
rm ./data/cache.redb
```

## Testing

Run tests with an in-memory database:

```bash
cargo test -p memory-storage-redb
```

## Documentation

Full API documentation: [docs.rs/memory-storage-redb](https://docs.rs/memory-storage-redb)

## License

Licensed under the MIT License. See [LICENSE](../LICENSE) for details.

## Project

Part of the [rust-self-learning-memory](https://github.com/d-o-hub/rust-self-learning-memory) project.
