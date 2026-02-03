# Connection Lifecycle Integration for Prepared Statement Cache

## Overview

This document describes the integration between the adaptive connection pool and the prepared statement cache, enabling automatic cache cleanup when connections are returned to the pool.

## Problem Statement

The prepared statement cache tracks statements per connection ID to ensure proper isolation and avoid using statements from one connection on another. However, when connections are returned to the pool or closed, the cached statements need to be cleaned up to prevent memory leaks.

Previously, there was no mechanism to notify the cache when connections were closed, leading to:

1. Memory leaks from abandoned connection caches
2. Potential performance degradation from tracking too many connections
3. No automatic cleanup of stale prepared statement metadata

## Solution

The adaptive connection pool now supports **connection lifecycle callbacks** that are triggered when connections are dropped. This allows the prepared statement cache (and other components) to clean up resources associated with specific connections.

## Architecture

### Connection ID

Each connection in the pool has a unique `ConnectionId` (type alias for `u64`):

```rust
use memory_storage_turso::pool::ConnectionId;

let conn_id = connection.connection_id();
```

The connection ID is:
- Unique across all connections in the pool
- Monotonically increasing
- Stable for the lifetime of the connection
- Exposed via `AdaptivePooledConnection::connection_id()`

### Cleanup Callback

The pool accepts a cleanup callback of type:

```rust
use memory_storage_turso::pool::ConnectionCleanupCallback;
use std::sync::Arc;

pub type ConnectionCleanupCallback = Arc<dyn Fn(ConnectionId) + Send + Sync>;
```

The callback is called with the connection ID when the connection is dropped.

## Usage

### Basic Integration

```rust
use memory_storage_turso::pool::{AdaptiveConnectionPool, ConnectionId};
use memory_storage_turso::prepared::PreparedStatementCache;
use std::sync::Arc;

// Create the pool and cache
let pool = AdaptiveConnectionPool::new(db, config).await?;
let cache = Arc::new(PreparedStatementCache::new(100));
let cache_clone = Arc::clone(&cache);

// Register the cleanup callback
pool.set_cleanup_callback(Arc::new(move |conn_id: ConnectionId| {
    cache_clone.clear_connection(conn_id);
}));

// Use the pool normally
let conn = pool.get().await?;
let conn_id = conn.connection_id();

// Record cache entries
cache.record_miss(conn_id, "SELECT 1", 100);

// When `conn` is dropped, the cache is automatically cleared
```

### Removing the Callback

```rust
// Remove the callback (disables automatic cleanup)
pool.remove_cleanup_callback();
```

### Multiple Components

Multiple components can coordinate cleanup by using a shared callback:

```rust
use std::sync::Arc;

let cache1 = Arc::new(Cache1::new());
let cache2 = Arc::new(Cache2::new());
let cache1_clone = Arc::clone(&cache1);
let cache2_clone = Arc::clone(&cache2);

pool.set_cleanup_callback(Arc::new(move |conn_id| {
    // Clean up both caches
    cache1_clone.clear_connection(conn_id);
    cache2_clone.clear_connection(conn_id);
}));
```

## Implementation Details

### Thread Safety

- The cleanup callback is wrapped in `Arc<dyn Fn(ConnectionId) + Send + Sync>`
- This ensures thread-safe access to captured variables
- The callback is stored in `RwLock<Option<ConnectionCleanupCallback>>` in the pool

### Drop Order

When a connection is dropped:

1. The permit is released (returns to pool)
2. Metrics are updated (decrement active count)
3. The cleanup callback is invoked (if registered)
4. The connection object is destroyed

### Performance

- Minimal overhead: callback invocation is a simple function call
- No locks held during callback execution
- Callback execution time does not block pool operations

## Testing

### Unit Tests

See `memory-storage-turso/src/pool/adaptive.rs` tests:
- `test_connection_id_uniqueness` - Verifies IDs are unique and increasing
- `test_cleanup_callback_on_connection_drop` - Tests callback is invoked
- `test_cleanup_callback_tracks_correct_connection_id` - Tests correct ID is passed
- `test_cleanup_callback_removal` - Tests callback can be removed

### Integration Tests

See `memory-storage-turso/src/pool/cache_integration_test.rs`:
- `test_cache_cleanup_on_connection_return` - Tests automatic cleanup
- `test_cache_tracks_multiple_connections` - Tests multiple connections
- `test_cache_statistics_with_cleanup` - Tests statistics accuracy
- `test_no_callback_registered` - Tests behavior without callback
- `test_callback_removal_during_runtime` - Tests dynamic removal

## Benefits

1. **Automatic Resource Management**: No manual cleanup needed
2. **Memory Leak Prevention**: Abandoned connection caches are cleaned up
3. **Performance Optimization**: Cache size stays bounded
4. **Extensibility**: Pattern can be used for other per-connection resources
5. **Thread Safety**: Safe to use from multiple threads

## Migration Guide

### Before (Manual Cleanup)

```rust
// Manual cleanup required
{
    let conn = pool.get().await?;
    let conn_id = conn.connection_id();
    cache.record_miss(conn_id, "SELECT 1", 100);
}

// Must remember to clean up manually
cache.clear_connection(conn_id);
```

### After (Automatic Cleanup)

```rust
// Automatic cleanup with callback
pool.set_cleanup_callback(Arc::new(move |conn_id| {
    cache.clear_connection(conn_id);
}));

{
    let conn = pool.get().await?;
    let conn_id = conn.connection_id();
    cache.record_miss(conn_id, "SELECT 1", 100);
} // Cleanup happens automatically on drop
```

## Future Enhancements

Potential future improvements:

1. **Per-connection resource tracking**: Generic interface for any per-connection resource
2. **Connection metadata**: Expose connection creation time, usage stats
3. **Lifecycle events**: Support for create/checkout/checkin/close events
4. **Callback chains**: Allow multiple callbacks to be registered
5. **Async callbacks**: Support async cleanup operations

## API Reference

### AdaptiveConnectionPool

#### `set_cleanup_callback`

```rust
pub fn set_cleanup_callback(&self, callback: ConnectionCleanupCallback)
```

Register a callback to be invoked when connections are dropped.

**Parameters:**
- `callback`: Function to call with connection ID

**Example:**
```rust
pool.set_cleanup_callback(Arc::new(|conn_id| {
    println!("Connection {} dropped", conn_id);
}));
```

#### `remove_cleanup_callback`

```rust
pub fn remove_cleanup_callback(&self)
```

Remove the cleanup callback, disabling automatic cleanup notifications.

### AdaptivePooledConnection

#### `connection_id`

```rust
pub fn connection_id(&self) -> ConnectionId
```

Get the unique connection identifier.

**Returns:**
- Unique connection ID (u64)

**Example:**
```rust
let conn = pool.get().await?;
let conn_id = conn.connection_id();
```

## Performance Impact

- **Memory**: Minimal overhead (one Arc pointer per connection)
- **CPU**: Negligible overhead (single function call on drop)
- **Latency**: No impact on connection acquisition
- **Throughput**: No impact on query performance

## Security Considerations

- Callbacks have access to captured variables (be careful with Arc::clone)
- Callbacks execute during Drop (avoid panics)
- Callbacks should be non-blocking
- Callbacks should not call back into the pool

## Troubleshooting

### Cleanup Not Invoked

**Symptom**: Cache entries not being cleared

**Possible Causes:**
1. Callback not registered
2. Connection not dropped (held somewhere)
3. Callback removed prematurely

**Solution**: Verify callback is registered and connection is dropped

### Callback Panics

**Symptom**: Panic during connection drop

**Possible Causes:**
1. Callback captures invalid state
2. Callback performs blocking operation
3. Callback calls back into pool

**Solution**: Ensure callback is panic-free and non-blocking

### Memory Still Growing

**Symptom**: Cache size still growing

**Possible Causes:**
1. Cache size limit too high
2. Connection ID reuse (unlikely)
3. Callback not removing entries

**Solution**: Monitor cache statistics and adjust limits
