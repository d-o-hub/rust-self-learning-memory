# How to Run Turso Locally with Vector Extensions

## Problem

The Phase 1 benchmarks failed because they used local SQLite which doesn't have Turso's vector extensions:
- ❌ No `vector32()` function
- ❌ No `vector_top_k()` table function
- ❌ No DiskANN vector index support

## Solution: Use Turso CLI `dev` Mode

Turso CLI provides a local libSQL server with full Turso extensions including vector search.

### Installation

```bash
# Install Turso CLI
curl -sSfL https://get.turso.dev | sh

# Or using Homebrew (macOS)
brew install tursodatabase/turso/turso

# Or using Cargo
cargo install turso-cli
```

### Starting Local Server

```bash
# Start local libSQL server (in-memory, changes lost on stop)
turso dev

# Start with persistent database file
turso dev --db-file local.db
```

The server will start on `libsql://127.0.0.1:8080`

### Connecting to Local Server

```rust
use libsql::{Connection, Database};

// Connect to local Turso server
let url = "libsql://127.0.0.1:8080";
let db = Database::open(url).await?;
```

In benchmarks, update the URL from:

```rust
// OLD: Uses SQLite without vector extensions
let storage = TursoStorage::new(&format!("file:{}", db_path.to_string_lossy()), "")?;
```

To:

```rust
// NEW: Uses Turso libSQL server with vector extensions
let storage = TursoStorage::new("libsql://127.0.0.1:8080", "")?;
```

### Verifying Vector Extensions

Connect to the local server and test vector functions:

```sql
-- Test vector32 function
SELECT vector32('0.1,0.2,0.3') AS vector;

-- Test vector index creation
CREATE TABLE test (
    id INTEGER PRIMARY KEY,
    v F32_BLOB(3)
);
CREATE INDEX idx_test_vector ON test(libsql_vector_idx(v));

-- Test vector search
INSERT INTO test VALUES (1, vector32('0.1,0.2,0.3'));
SELECT * FROM vector_top_k('idx_test_vector', vector32('0.1,0.2,0.3'), 10);
```

If these work, you have full vector support!

---

## Updated Benchmark Instructions

### Step 1: Start Turso Dev Server

```bash
# In one terminal
turso dev --db-file /tmp/turso_benchmark.db
```

Keep this terminal running. The server will display connection URL.

### Step 2: Update Benchmark to Use Local Server

Create a new benchmark file or modify existing:

```rust
// In benches/turso_vector_performance.rs

async fn setup_storage_with_data(
    dimension: usize,
    count: usize,
) -> Result<(Arc<TursoStorage>, TempDir, Vec<f32>)> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Use Turso dev server instead of file URL
    // Note: Uses libsql:// protocol, NOT http://
    let storage = TursoStorage::new("libsql://127.0.0.1:8080", "")
        .await
        .expect("Failed to create turso storage");
    storage.initialize_schema().await.expect("Failed to initialize schema");

    // ... rest of setup code
}
```

### Step 3: Run Benchmarks

```bash
# In another terminal
cd benches
cargo bench --bench turso_vector_performance --features memory-storage-turso/turso_multi_dimension
```

---

## Alternative: Use Turso Cloud

If local dev server has issues, use Turso cloud:

```bash
# Login to Turso
turso auth login

# Create a test database
turso db create benchmark-test

# Get connection URL
turso db show benchmark-test

# Set environment variables
export TURSO_DATABASE_URL="libsql://<your-database-url>"
export TURSO_AUTH_TOKEN="<your-auth-token>"
```

Then update benchmarks to use environment variable:

```rust
let url = std::env::var("TURSO_DATABASE_URL")
    .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
let token = std::env::var("TURSO_AUTH_TOKEN")
    .unwrap_or_else(|_| String::new());

let storage = TursoStorage::new(&url, &token)?;
```

---

## Troubleshooting

### Error: "no such function: vector32"

**Cause**: Using file:// URL with local SQLite instead of Turso server.

**Fix**: Use `http://127.0.0.1:8080` or `libsql://` cloud URL.

### Error: "connection refused"

**Cause**: Turso dev server not running or on different port.

**Fix**:
1. Start `turso dev` in a separate terminal
2. Check the port displayed (may not be 8080)
3. Update connection URL to match actual port

### Error: "no such table: vector_top_k"

**Cause**: Not using Turso server with vector extensions loaded.

**Fix**: Ensure you're connecting to Turso server (libsql://), not file:// or http://

### Slow Performance Even with Turso Server

**Possible Causes**:
1. Vector index not yet built (first query slower)
2. Dataset too small for DiskANN overhead
3. Index parameters not optimal

**Fixes**:
1. Run queries twice (second uses index)
2. Test with larger datasets (>1000 embeddings)
3. Tune DiskANN parameters (see TURSO_AI_CONCRETE_RECOMMENDATIONS.md)

---

## Quick Reference

| Connection Type | URL Format | Vector Support |
|----------------|-------------|-----------------|
| Local SQLite | `file:path.db` | ❌ No vector functions |
| Turso Local Server | `libsql://127.0.0.1:8080` | ✅ Full vector support |
| Turso Cloud | `libsql://<db-url>` | ✅ Full vector support |

**Recommendation**: Use Turso local server for development and benchmarks.
**Note**: Always use `libsql://` protocol, NOT `http://` protocol.

---

## Additional Resources

- [Turso CLI Docs](https://docs.turso.tech/cli/introduction)
- [Local Development Guide](https://docs.turso.tech/local-development)
- [libSQL Docs](https://docs.turso.tech/libsql)
- [Vector Search Blog](https://turso.tech/blog/turso-brings-native-vector-search-to-sqlite)
