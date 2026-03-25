# Code Conventions

**Note**: The codebase follows Rust idioms automatically. Focus on learning from examples rather than memorizing rules.

> For core invariants, change workflow, common pitfalls, tool selection, security, env vars, and performance targets, see `../AGENTS.md`.
> For quality gates and formatting/linting commands, see `building_the_project.md`.

## Learning from Code

Instead of memorizing conventions, examine existing code:

### Import Organization
```rust
// Standard library first
use std::collections::HashMap;
use std::sync::Arc;

// External crates
use anyhow::Result;
use tokio::sync::Mutex;

// Local modules
use crate::embeddings::*;
use memory_core::Episode;
```

### Async Patterns
```rust
// Use Tokio for async
#[tokio::main]
async fn main() -> Result<()> {
    let worker_count = 4;
    println!("Starting async operation with {worker_count} workers");
    Ok(())
}

// Avoid blocking in async
async fn process_data(id: u64) -> Result<Data> {
    // Use async versions of blocking operations
    let result = async_operation(id).await?;
    Ok(result)
}

// For blocking operations, use spawn_blocking
let result = tokio::task::spawn_blocking(|| {
    blocking_operation()
}).await?;
```

### Error Handling
```rust
// Public APIs: anyhow::Result
pub async fn public_function() -> Result<ReturnType> {
    // Business logic
    Ok(success_value)
}

// Domain errors: thiserror
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid episode state")]
    InvalidState,

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

// Use ? for error propagation
pub async fn process_data(data: &str) -> Result<ProcessedData> {
    let parsed = parse_data(data)?;
    let validated = validate_data(&parsed)?;
    Ok(validated)
}
```

## Code Organization

### File Structure
- **Maximum 500 lines** per file (strictly enforced)
- Group related functionality
- Use modules for organization
- Split large modules into sub-modules

### Module Structure Example
```
memory-core/src/
├── patterns/          # Pattern extraction (5319 LOC - split into multiple files)
│   ├── mod.rs
│   ├── optimized_validator.rs
│   ├── clustering.rs
│   └── ...
├── embeddings/        # Embedding providers (5250 LOC)
├── memory/            # Core memory operations (4457 LOC)
├── spatiotemporal/    # Spatial-temporal indexing (3377 LOC)
└── ...
```

### Naming Conventions
- **Variables/functions**: `snake_case`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### 2025 Rust Best Practices

#### Format Strings
Use captured identifiers in format strings (Rust 1.58+):

```rust
// ✅ Modern: Direct variable capture
let message = format!("Processing {items} items in {seconds} seconds");
println!("User {user_id} performed {action}");

// ❌ Legacy: Explicit formatting
let message = format!("Processing {} items in {} seconds", items, seconds);
println!("User {} performed {}", user_id, action);
```

#### Type Conversions
Use `From` trait for infallible conversions instead of `as` casts:

```rust
// ✅ Modern: Type-safe conversion
let id: i64 = i64::from(u32_id);
// or
let id: i64 = u32_id.into();

// ❌ Legacy: Unchecked cast
let id: i64 = u32_id as i64;
```

For API response structs where fields are unused but part of external contracts:
```rust
#[allow(dead_code)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
    // model field kept for API contract even if unused internally
    model: String,
}
```

#### Range Checks
Use idiomatic range contains method:

```rust
// ✅ Modern: Range contains
if (0.0..=1.0).contains(&value) {
    // Valid value
}

// ❌ Legacy: Explicit comparisons
if value >= 0.0 && value <= 1.0 {
    // Valid value
}
```

#### Documentation Formatting
Use backticks for code elements in documentation:

```rust
/// Provides access to `SelfLearningMemory` for managing episodes.
///
/// The `MemoryConfig` struct allows customization of storage backends.
///
/// # Examples
///
/// ```rust
/// use memory_core::{SelfLearningMemory, MemoryConfig};
///
/// let config = MemoryConfig::default();
/// let memory = SelfLearningMemory::new(config).await?;
/// ```
pub async fn create_memory(config: MemoryConfig) -> Result<SelfLearningMemory> {
    // Implementation
}
```

#### Postcard Serialization (v0.1.7+)
**IMPORTANT**: Use Postcard for serialization, NOT bincode:

```rust
// ✅ Modern: Postcard serialization (required since v0.1.7)
use postcard::{from_bytes, to_allocvec};

// Serialize
let serialized = postcard::to_allocvec(&data)?;

// Deserialize
let deserialized: DataType = postcard::from_bytes(&serialized)?;

// Storage
use postcard::to_stdvec;
let bytes = postcard::to_stdvec(&episode)?;
storage.store(&id, &bytes).await?;
```

**Why Postcard over Bincode?**
- Safer: Uses `#[no_std]` compatible format
- Smaller: More compact binary representation
- No unsafe code: Bincode uses unsafe operations
- Breaking change in v0.1.7: All storage must use postcard

## Testing Conventions

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = create_test_input();

        // Act
        let result = process(input);

        // Assert
        assert!(result.is_ok());
    }
}
```

### Async Tests
```rust
#[tokio::test]
async fn test_async_feature() {
    // Async test implementation
    let memory = create_test_memory().await;
    let result = memory.create_episode("test").await;
    assert!(result.is_ok());
}
```

### Error Testing
```rust
#[tokio::test]
async fn test_error_handling() {
    let result = invalid_operation().await;
    assert!(result.is_err());

    // Check specific error
    match result {
        Err(MemoryError::InvalidInput(msg)) => {
            assert_eq!(msg, "expected non-empty input");
        }
        _ => panic!("Expected InvalidInput error"),
    }
}
```

## Storage Conventions

### Database Access
```rust
// ✅ Use parameterized queries (prevents SQL injection)
let rows = db.query(
    "SELECT * FROM episodes WHERE id = ?1",
    &[episode_id]
).await?;

// ❌ Never string interpolation
let rows = db.query(&format!(
    "SELECT * FROM episodes WHERE id = '{}'",
    episode_id
), &[]).await?;
```

### Connection Pooling
```rust
// Semaphore-based connection limiting (default: 10)
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(10));
let permit = semaphore.acquire().await?;
let conn = pool.get_connection().await?;
// Use connection
drop(permit);
```

### Transaction Handling
```rust
// Keep transactions short-lived
let result = {
    let txn = db.begin_write()?;
    let mut table = txn.open_table(TABLE)?;
    table.insert(key, value)?;
    txn.commit()
}?;
```

## Dependency Version Management

### Major Version Upgrades

**Always check docs.rs for breaking changes before upgrading major versions.**

#### redb 3.x (from 2.x)
- `begin_read()` moved to `ReadableDatabase` trait:
  ```rust
  // OLD (redb 2.x)
  let txn = db.begin_read()?;

  // NEW (redb 3.x) - must import trait
  use redb::ReadableDatabase;
  let txn = db.begin_read()?;
  ```
- `begin_write()` unchanged (still on `Database` struct)

#### rand 0.10 (from 0.8/0.9)
- Function renames:
  ```rust
  // OLD
  let mut rng = rand::thread_rng();
  let value: f64 = rng.gen();
  let range_val = rng.gen_range(0.0..1.0);

  // NEW
  let mut rng = rand::rng();
  let value: f64 = rng.random();
  let range_val = rng.random_range(0.0..1.0);
  ```
- Trait import change:
  ```rust
  // OLD
  use rand::Rng;

  // NEW - use RngExt for user-level methods
  use rand::RngExt;
  ```
- Keep `rand` and `rand_chacha` versions aligned (both 0.10)

### Upgrade Checklist
1. Check docs.rs for breaking changes
2. Run `cargo build` to identify errors
3. Fix imports and API changes
4. Run `./scripts/code-quality.sh clippy --workspace`
5. Run `cargo nextest run --all`
6. Run `./scripts/quality-gates.sh`
