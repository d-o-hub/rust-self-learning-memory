# CLI Turso Storage Segfault Issue

**Date**: 2026-04-02
**Severity**: P1 - Critical
**Status**: Under Investigation

## Problem

The CLI crashes with a segmentation fault when using file-based Turso databases:

```
SQL attempt 1 failed: SQLite failure: `bad parameter or other API misuse`, retrying...
Segmentation fault
```

## Root Cause Analysis

1. **Error occurs during schema initialization** - The `initialize_schema()` method fails with "bad parameter or other API misuse"
2. **Segfault during retry logic** - After the SQL error, the CLI crashes instead of handling the error gracefully

## Affected Code

- `memory-storage-turso/src/lib_impls/helpers.rs:173` - `execute_with_retry`
- `memory-storage-turso/src/turso_config.rs:13` - `initialize_schema`

## Potential Causes

1. **libSQL file URL handling** - May be an issue with how libSQL handles file: URLs vs http:// URLs
2. **Threading/async issue** - The segfault could be caused by a race condition or memory safety issue
3. **Resource cleanup** - Connection might not be properly closed after error

## Workaround

Use redb-only storage for local development:

```bash
# Build without turso feature
cargo build --release -p do-memory-cli

# Or use turso dev server with http:// URL
turso dev --db-file ./data/memory.db --port 8080
TURSO_URL="http://127.0.0.1:8080" TURSO_TOKEN="" ./target/release/do-memory-cli health check
```

## Next Steps

1. Add error handling to prevent segfault on SQL errors
2. Investigate libSQL file URL handling
3. Add integration tests for file-based storage
4. Consider using rusqlite directly for file-based databases