# Test Fix Examples

Real-world examples of fixing common test failures in the Rust self-learning memory project.

## Example 1: Missing .await in async test

**Failure:**
```
error[E0277]: `impl Future<Output = Result<Episode>>` is not a future in the current context
   --> src/episode_test.rs:123
    |
123 |     let episode = store.create_episode(data);
    |                    --------------------------- ^^^^^^^^^^^^^^^^^^ future is not a type that awaits
    |
note: futures do nothing unless you `.await` or poll them
```

**Diagnosis:**
- Test creates an episode using async storage method
- Missing `.await` causes compilation error
- Simple typo/forgetful developer pattern

**Fix:**
```rust
// Before
let episode = store.create_episode(data);

// After
let episode = store.create_episode(data).await.expect("Failed to create episode");
```

**Verification:**
```bash
cargo test test_create_episode -- --exact
```

---

## Example 2: Race condition in concurrent test

**Failure (intermittent):**
```
test test_concurrent_steps ... FAILED
failures:
    test_concurrent_steps
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

**Symptoms:**
- Test fails ~30% of time
- No consistent error message
- Assertion failures with varying values

**Diagnosis:**
1. Run with single thread: `cargo test -- --test-threads=1` - PASS
2. Test spawns multiple tasks that write shared state
3. Missing synchronization causes data races

**Fix:**
```rust
use tokio::sync::Mutex;
use std::sync::Arc;

// Before
let shared_data = vec![];
let mut handles = vec![];
for i in 0..10 {
    let data = shared_data.clone(); // Still races
    handles.push(tokio::spawn(async move {
        data.push(i);
    }));
}

// After
let shared_data = Arc::new(Mutex::new(vec![]));
let mut handles = vec![];
for i in 0..10 {
    let data = shared_data.clone();
    handles.push(tokio::spawn(async move {
        let mut guard = data.lock().await;
        guard.push(i);
    }));
}
```

**Verification:**
```bash
# Run multiple times to prove fix
for i in {1..10}; do cargo test test_concurrent_steps -- --exact || break; done
```

---

## Example 3: Database connection issue

**Failure:**
```
thread 'test_episode_retrieval' panicked at 'Database error: Connection refused'
```

**Diagnosis:**
1. Test connects to Turso database
2. TURSO_DATABASE_URL not set in test environment
3. Test should use mock storage or test database

**Fix:**
```rust
use memory_storage_turso::TursoStorage;

#[tokio::test]
async fn test_episode_retrieval() {
    // Before - uses real database, requires TURSO_DATABASE_URL
    // let storage = TursoStorage::new(&std::env::var("TURSO_DATABASE_URL").unwrap()).await;
    
    // After - uses test database or mock storage
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "file::memory:?mode=memory&cache=shared".to_string());
    let storage = TursoStorage::new(&database_url).await.unwrap();
    
    // Test logic...
}
```

**Setup:**
```bash
# In .env.test (not committed)
TEST_DATABASE_URL=file:./test-cache.db

# Or use in-memory database
export TEST_DATABASE_URL="file::memory:?mode=memory&cache=shared"
```

---

## Example 4: Lifetime error in async closure

**Failure:**
```
error[E0597]: `data` does not live long enough
   --> src/episode_test.rs:89
    |
89  |     let task = tokio::spawn(async move {
    |                     ^^^^^^^^^^^- argument requires that `data` is borrowed for `'static`
90  |         process_episode(&data).await;
    |                         ---- borrow of `data` occurs here
91  |     });
92  |     drop(data);  // <-- `data` dropped here while still borrowed
```

**Diagnosis:**
- `tokio::spawn` requires 'static futures
- Data borrowed and then dropped
- Need to move ownership into async block

**Fix:**
```rust
use std::sync::Arc;

// Before
let task = tokio::spawn(async move {
    process_episode(&data).await; // Error: borrows data
});

// After - move data into Arc and clone
let data = Arc::new(data);
let data_clone = data.clone();
let task = tokio::spawn(async move {
    process_episode(&data_clone).await;
});
```

---

## Example 5: Type mismatch in storage layer

**Failure:**
```
error[E0308]: mismatched types
   --> src/storage_test.rs:45
    |
45  |     let episodes: Vec<Episode> = storage.get_all().await.unwrap();
    |                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `Episode`, found struct `StoredEpisode`
```

**Diagnosis:**
- Storage returns `StoredEpisode` (with metadata)
- Test expects `Episode` (domain type)
- Need conversion or API update

**Fix:**
```rust
// Option 1: Update test to use correct type
let stored_episodes: Vec<StoredEpisode> = storage.get_all().await.unwrap();
let episodes: Vec<Episode> = stored_episodes.into_iter().map(|se| se.episode).collect();

// Option 2: Add convenience method to storage (better ergonomics)
impl Storage {
    pub async fn get_all_episodes(&self) -> Result<Vec<Episode>> {
        let stored = self.get_all().await?;
        Ok(stored.into_iter().map(|se| se.episode).collect())
    }
}

// Then in test
let episodes = storage.get_all_episodes().await.unwrap();
```

---

## Example 6: Test leaves database in dirty state

**Failure (only observed when running full suite):**
```
test suite result: ok. 5 passed; 0 failed; 0 ignored
test test_next_ep_isolation ... FAILED
thread 'main' panicked at 'assertion failed: `(left == right)`
  left: `3`,
 right: `2`', src/episode_test.rs:72
```

**Diagnosis:**
- Test assumes clean database state
- Previous test didn't clean up after running
- Need proper setup/teardown

**Fix:**
```rust
use tempfile::TempDir;
use sqlx::SqlitePool;

#[tokio::test]
async fn test_ep_isolation() {
    // Before - no isolation
    // let storage = get_shared_storage();
    
    // After - isolated test database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = create_storage(&db_path).await;
    
    // Test logic...
    
    // Cleanup is automatic when temp_dir goes out of scope
}
```

Alternative with cleanup function:
```rust
async fn cleanup_storage(storage: &TursoStorage) -> Result<()> {
    storage.execute("DELETE FROM episodes").await?;
    Ok(())
}

#[tokio::test]
async fn test_with_cleanup() {
    let storage = get_test_storage().await;
    
    // Test...
    
    // Cleanup
    cleanup_storage(&storage).await.unwrap();
}
```

---

## Example 7: Panic in async function not propagated

**Failure (silent failure):**
```
test should_fail ... ok
```
But test should have failed!

**Diagnosis:**
- Tokio task spawned with `tokio::spawn` doesn't propagate panics
- Test doesn't await the task
- Panic occurs in background, test sees nothing

**Fix:**
```rust
#[tokio::test]
async fn test_should_fail() {
    // Before - silent failure
    let _ = tokio::spawn(async {
        panic!("This should fail the test");
    });
    
    // After - propagate panic
    let task = tokio::spawn(async {
        panic!("This will fail the test");
    });
    task.await.unwrap(); // This will panic
    
    // Or use join_all for multiple tasks
    let results = join_all(vec![
        tokio::spawn(task1()),
        tokio::spawn(task2()),
    ]).await;
    for result in results {
        result.unwrap(); // Propagates panics
    }
}
```

---

## Example 8: Mocking async trait methods

**Failure:**
```
error[E0391]: cycle detected when computing predicates of trait `Storage`
```

**Diagnosis:**
- Trying to mock async trait directly
- Async traits don't support simple mocking
- Need type erasure or testing approach change

**Fix approaches:**

**Approach 1: Use in-memory implementation for testing**
```rust
#[cfg(test)]
pub struct MockStorage {
    episodes: Arc<Mutex<Vec<Episode>>>,
}

#[cfg(test)]
impl Storage for MockStorage {
    async fn create_episode(&self, episode: Episode) -> Result<Episode> {
        let mut episodes = self.episodes.lock().await;
        episodes.push(episode.clone());
        Ok(episode)
    }
    
    async fn get_episode(&self, id: &str) -> Result<Episode> {
        let episodes = self.episodes.lock().await;
        episodes.iter()
            .find(|e| e.id == id)
            .cloned()
            .ok_or_else(|| Error::NotFound(id.to_string()))
    }
}

#[tokio::test]
async fn test_with_mock() {
    let storage = MockStorage::new();
    // Test with mock storage...
}
```

**Approach 2: Integration test with real storage**
```rust
#[tokio::test]
async fn test_integration() {
    let storage = TursoStorage::new(":memory:").await.unwrap();
    // Test with real but ephemeral storage...
}
```

---

## Example 9: Time-based test flakiness

**Failure:**
```
test test_timeout ... FAILED
thread 'test_timeout' panicked at 'assertion failed: elapsed.as_millis() < 1000'
  left: `1002`,
 right: `1000`
```

**Diagnosis:**
- Test assumes operation completes within time
- System load varies, causing flaky timing
- Need tolerance or different approach

**Fix:**
```rust
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_timing() {
    // Before - exact timing (flaky)
    let start = Instant::now();
    operation().await;
    assert!(start.elapsed() < Duration::from_millis(1000));
    
    // After - tolerance + use sleep for timing tests
    let start = Instant::now();
    operation().await;
    let elapsed = start.elapsed();
    // Allow 50% buffer for system load
    assert!(elapsed < Duration::from_millis(1500), 
            "Operation took {:?}", elapsed);
}

// Better: test behavior, not timing
#[tokio::test]
async fn test_behavior() {
    // Test that it completes, not how long it takes
    let result = operation().await;
    assert!(result.is_ok());
}
```

---

## Example 10: Clippy warning in test code

**Failure:**
```
warning: unnecessary `unwrap` usage on `Result`
   --> src/episode_test.rs:45
    |
45  |     let episode = storage.get_episode(id).unwrap();
    |                                             ^^^^^^^ help: use `.expect()` to provide a better panic message: `.expect("...")`
```

**Diagnosis:**
- CI enforces `cargo clippy --all -- -D warnings`
- Test uses `.unwrap()` which linter dislikes
- Need more informative error messages

**Fix:**
```rust
// Before
let episode = storage.get_episode(id).unwrap();

// After - use expect with context
let episode = storage.get_episode(id)
    .expect("Failed to retrieve episode in test");

// Or handle error explicitly
let episode = storage.get_episode(id)
    .expect("Episode should exist after creation");
```

---

## Checklist for Test Fixes

Before considering a fix complete:

- [ ] Root cause identified and documented
- [ ] Minimal fix applied (no unnecessary changes)
- [ ] Fix runs locally successfully
- [ ] Fix tested multiple times if intermittent
- [ ] No regressions (cargo test --all passes)
- [ ] Code quality passes (cargo fmt, cargo clippy)
- [ ] Comments added for non-obvious fixes
- [ ] If common pattern, consider adding to test documentation
- [ ] Environment variables documented if used
- [ ] Cleanup/teardown verified

## Common Anti-Patterns to Avoid

1. **Suppressing errors** instead of fixing them
2. **Changing assertions** to pass failing tests
3. **Adding sleep()** as "fix" for race conditions
4. **Ignoring tests** without documenting why
5. **Making test-only changes** when production code has the bug
6. **Skipping clippy checks** to "make tests pass"

## Getting Help

If you're stuck after trying these patterns:

1. Check existing test patterns in the codebase: `find . -name "*test*.rs"`
2. Review project documentation: `cat AGENTS.md | grep -A 10 test`
3. Search for similar errors in the repo: `git log --all --grep="async test"`
4. Consult async Rust patterns: `cargo doc --open`
