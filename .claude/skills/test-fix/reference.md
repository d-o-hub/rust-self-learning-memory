# Test Fix Reference

Detailed reference for diagnosing and fixing Rust test failures in async/memory systems.

## Error Pattern Classification

### Async/Await Errors

#### Missing .await
**Error:** `"future cannot be sent"`, `"expected Implement Future, got impl Future"`

**Diagnosis Steps:**
1. Search for async function calls without `.await`
2. Look for return type `impl Future` in signatures
3. Check if function returns `Result<Future>` instead of awaiting

**Common Locations:**
- Storage operations: `storage.get_episode(id)` → needs `.await`
- Database queries: `conn.execute(query)` → needs `.await`
- Network calls: `client.send(req)` → needs `.await`

**Fix Pattern:**
```rust
// Pattern: async calls in async functions
async fn foo() -> Result<T> {
    // ✓ Correct
    let result = operation().await?;
    
    // ✗ Wrong - returns Future, not T
    // let result = operation()?;
    
    Ok(result)
}
```

#### Send/Sync Bounds
**Error:** `"future cannot be sent between threads safely"`

**Diagnosis:**
- Future contains non-thread-safe data
- Using `tokio::spawn` with non-Send data
- Rc<RefCell> or internal mutability in async context

**Fix Options:**
```rust
// Option 1: Use Arc<Mutex<T>>
use std::sync::Arc;
use tokio::sync::Mutex;

let data = Arc::new(Mutex::new(value));

// Option 2: Clone moveable data
let value = value.clone();
tokio::spawn(async move {
    use_value(value);
});

// Option 3: Use local runtime (no thread pool)
#[tokio::test]
async fn test_local() {
    tokio::task::spawn_local(async move {
        // OK for !Send types
    }).await;
}
```

### Database Errors

#### Connection Issues
**Error:** `"Connection refused"`, `"No such file or directory"`

**Diagnosis Steps:**
```bash
# Check environment variable
echo $TURSO_DATABASE_URL

# Test connectivity
curl -I "https://your-turso-url.turso.io"

# Verify libSQL local file
ls -la ~/.local/share/libsql/
```

**Common Causes:**
- Missing `TURSO_DATABASE_URL`
- Wrong database path for local libSQL
- Network connectivity issues for remote Turso
- Database permissions wrong

**Fix Strategies:**

1. **Environment variables:**
```bash
# In .env (gitignored)
TURSO_DATABASE_URL="file:./local-cache.db"

# Test database
TEST_DATABASE_URL="file::memory:?mode=memory"
```

2. **Graceful fallback:**
```rust
let url = std::env::var("TURSO_DATABASE_URL")
    .or_else(|_| std::env::var("TEST_DATABASE_URL"))
    .unwrap_or_else(|_| "file::memory:".to_string());
```

3. **Test isolation:**
```rust
#[tokio::test]
async fn test() {
    let storage = TursoStorage::new("file::memory:?cache=shared")
        .await
        .unwrap();
}
```

#### Query Errors
**Error:** `"Query returned no rows"`, `"column not found"`, `"syntax error"`

**Diagnosis:**
```bash
# Enable SQL logging
RUST_LOG=sqlx=debug cargo test

# Check schema
sqlite3 path/to/db.db ".schema episodes"
```

**Common Causes:**
- Schema mismatch between code and actual database
- Missing migrations
- Wrong column names
- Type mismatch (e.g., TEXT vs INTEGER)

**Fix Strategies:**

1. **Run migrations:**
```bash
# Before tests
cargo run --bin migrate

# In test setup
async fn setup_db() -> TursoStorage {
    let storage = TursoStorage::new(":memory:").await.unwrap();
    storage.run_migrations().await.unwrap();
    storage
}
```

2. **Type alignment:**
```rust
// Ensure Rust types match SQL types
#[derive(Debug)]
struct Episode {
    // SQLite TEXT maps to String
    id: String,
    // SQLite INTEGER maps to i64
    created_at: i64,
    // SQLite BLOB maps to Vec<u8>
    data: Vec<u8>,
}
```

3. **Query validation:**
```rust
use sqlx::query_as;

let episode = query_as!(
    Episode,
    "SELECT id, created_at, data FROM episodes WHERE id = ?",
    id
)
.fetch_one(&pool)
.await
.unwrap();

// Type-safe - won't compile if columns don't matchEpisode struct
```

### Race Conditions

#### Detection Patterns

**Symptoms:**
- Test passes sometimes, fails sometimes (deterministic failures)
- Failed assertion values vary between runs
- "data race" warnings from tools

**Verification:**
```bash
# Run test multiple times
for i in {1..20}; do
  cargo test flaky_test -- --exact || echo "Failed at run $i"
done

# Use race detector (experimental)
RUSTFLAGS="-Z sanitize=thread" cargo test

# Force single-threaded to confirm
cargo test flaky_test -- --test-threads=1
```

**Diagnostic Tools:**
```bash
# Detailed logging
RUST_LOG=debug cargo test flaky_test

# Add test instrumentation
cargo add --dev loom
```

#### Common Race Patterns

**Pattern 1: Concurrent writes without synchronization**
```rust
// ✗ Wrong
let data = vec![];
tokio::spawn(async { data.push(1); });
tokio::spawn(async { data.push(2); });

// ✓ Fixed - use Arc<Mutex>
let data = Arc::new(Mutex::new(vec![]));
let d1 = data.clone();
tokio::spawn(async move {
    let mut guard = d1.lock().await;
    guard.push(1);
});
```

**Pattern 2: Read-modify-write race**
```rust
// ✗ Wrong - check-then-act
if !storage.exists(id).await {
    storage.create(id, data).await;
}
// Another task could create between check and write!

// ✓ Fixed - use insert or handle error
match storage.insert_if_new(id, data).await {
    Ok(_) => {},
    Err(Error::AlreadyExists) => {},
    Err(e) => return Err(e),
}
```

**Pattern 3: Ordering dependency**
```rust
// ✗ Wrong - tasks may complete in any order
let t1 = tokio::spawn(task1());
let t2 = tokio::spawn(task2());
// No guarantee which finishes first

// ✓ Fixed - explicit ordering
task1().await;
task2().await;

// Or collect results
let results = join_all(vec![task1(), task2()]).await;
```

### Type/Lifetime Errors

#### Lifetime Elision Failures

**Error:** `"borrowed value does not live long enough"`, `"explicit lifetime required"`

**Diagnosis:**
- Look for references in async functions
- Check for `.await` while holding borrows
- Identify closures capturing references

**Common Patterns:**

**Pattern 1: .await holding borrow**
```rust
// ✗ Wrong
let data = get_data();
storage.save(data).await?;

// ✓ Fixed - clone or move
let data = get_data();
storage.save(data.clone()).await?;

// or
let data = get_data();
let data_clone = data.clone();
tokio::spawn(async move {
    storage.save(data_clone).await;
});
```

**Pattern 2: Async closure issues**
```rust
async fn process(data: &str) -> Result<String> {
    Ok(data.to_uppercase())
}

// ✗ Wrong - closure with async function
let data = String::from("test");
let task = tokio::spawn(async move {
    process(&data).await  // Error: data doesn't live long enough
});

// ✓ Fixed - move data
let data = String::from("test");
let task = tokio::spawn(async move {
    process(&data.clone()).await
});
```

#### Type Mismatches

**Error:** `"expected X, found Y"`, `"mismatched types"`

**Common Storage Types:**
```rust
// Domain types
struct Episode { id: String, name: String }

// Storage types (with metadata)
struct StoredEpisode {
    episode: Episode,
    created_at: i64,
    updated_at: i64,
}

// Conversion pattern
impl From<StoredEpisode> for Episode {
    fn from(stored: StoredEpisode) -> Self {
        stored.episode
    }
}
```

**Convenience methods:**
```rust
impl Storage {
    // Return domain type
    async fn get_episode(&self, id: &str) -> Result<Episode> {
        let stored: StoredEpisode = self.query(id).await?;
        Ok(stored.into())
    }
    
    // Raw storage access for advanced use
    async fn get_raw(&self, id: &str) -> Result<StoredEpisode> {
        self.query(id).await
    }
}
```

### Memory Management

#### Clone vs Copy

**When to clone:**
- Moving value out of shared context
- Need separate ownership across async tasks
- Type doesn't implement Copy

**When to use references:**
- Large data structures (avoid copies)
- Temporary access pattern
- No async/await while holding reference

**Pattern:**
```rust
// Large struct - use Arc
#[derive(Clone)]
struct LargeData { /* lots of fields */ }

async fn process(data: Arc<LargeData>) -> Result<()> {
    // Clone Arc (cheap), not data
    let data2 = data.clone();
    task1(data2).await?;
    task2(data).await
}
```

#### Drop Issues

**Error:** Value dropped while still in use, panic on double-drop

**Common causes:**
- Early return without cleanup
- Task cancelled without cleanup
- Panic in async task

**Cleanup pattern:**
```rust
use tokio::sync::Semaphore;

async fn with_cleanup(storage: &Storage) -> Result<()> {
    let semaphore = Semaphore::new(1);
    let _permit = semaphore.acquire().await?;
    
    let result = storage.do_work().await;
    
    // Cleanup happens when _permit drops
    result
}
```

**Guard pattern:**
```rust
struct ConnectionGuard<'a> {
    conn: &'a mut Connection,
}

impl<'a> Drop for ConnectionGuard<'a> {
    fn drop(&mut self) {
        // Auto rollback or close
    }
}
```

## Diagnostic Commands

### Test Execution

```bash
# All tests
cargo test --all

# Specific test
cargo test test_name -- --exact

# With output
cargo test test_name -- --exact --nocapture

# Multiple runs (for flaky tests)
cargo test -- --ignored --test-threads=1
```

### Debugging

```bash
# Debug logging (all modules)
RUST_LOG=debug cargo test

# Specific module
RUST_LOG=memory_core=debug cargo test

# Specific filter
RUST_LOG=memory_storage=trace cargo test

# SQL query logging
RUST_LOG=sqlx=debug cargo test

# Panic backtrace
RUST_BACKTRACE=full cargo test

# Pretty-printed errors
RUST_BACKTRACE=1 cargo test
```

### Code Analysis

```bash
# Check what tests exist
cargo test -- --list

# Find test files
find . -name "*test*.rs"

# Grep for async patterns
grep -r "\.await" tests/
grep -r "tokio::spawn" tests/

# Check for TODO/FIXME in tests
grep -r "TODO\|FIXME" tests/
```

### Database Tools

```bash
# SQLite tools
sqlite3 path/to/db.db ".schema"
sqlite3 path/to/db.db "SELECT * FROM episodes LIMIT 5;"

# libSQL query
libsql-cli path/to/db.db

# Turso CLI (if available)
turso db shell my-db
```

## Test Organization Patterns

### Unit Tests in Code
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic() {
        let result = function_to_test().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
use memory_core::Episode;
use memory_storage_turso::TursoStorage;

#[tokio::test]
async fn test_full_workflow() {
    let storage = setup_test_storage().await;
    let episode = Episode::new("test");
    let created = storage.create_episode(episode).await.unwrap();
    assert_eq!(created.id, "test");
}
```

### Common Test Utilities

```rust
// tests/common/mod.rs
use tempfile::TempDir;

pub async fn create_temp_storage() -> TursoStorage {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let path = temp.path().join("test.db");
    TursoStorage::new(path.to_str().unwrap()).await.expect("Failed to create storage")
}

pub struct TestCtx {
    pub storage: TursoStorage,
    _temp: TempDir, // Keeps temp alive
}
```

## Performance Testing

### Benchmark Tests
```rust
#[tokio::test]
async fn test_performance() {
    let start = std::time::Instant::now();
    let result = expensive_operation().await;
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    assert!(elapsed < std::time::Duration::from_secs(1));
}
```

### Cargo Benchmarks
```rust
// benches/operations.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("create_episode", |b| {
        b.iter(|| {
            // Benchmark code
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

## Best Practices Reference

### Test Naming
- ✓ `test_create_episode_success`
- ✓ `test_create_episode_duplicate_fails`
- ✗ `test1`, `test_stuff`, `good_test`

### Test Structure (AAA)
```rust
#[tokio::test]
async fn test_pattern() {
    // Arrange - setup
    let storage = create_test_storage().await;
    let input = Episode::new("test-001");
    
    // Act - execute
    let result = storage.create_episode(input.clone()).await;
    
    // Assert - verify
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, input.id);
}
```

### Async Test Setup
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_single_threaded() {
    // Test with single worker thread
}

#[tokio::test]
async fn test_isolated() {
    // Isolated runtime per test
}
```

## Project-Specific Notes

### Memory Storage Architecture
- **Storage traits**: async trait pattern in memory-core
- **Dual storage**: Turso (durable) + redb (cache)
- **Serialization**: postcard for efficient binary format
- **Compression**: optional for large payloads

### Common Test Scenarios
1. Episode creation and retrieval
2. Pattern extraction and learning
3. Cache synchronization
4. Schema migrations
5. Concurrent access patterns

### Environment Setup
```bash
# Required for Turso tests
export TURSO_DATABASE_URL="...your-database-url..."
export TURSO_AUTH_TOKEN="...your-token..."

# Optional for local-only tests
export USE_LOCAL_DB=true
export LOCAL_DB_PATH="./dev/cache.db"
```
