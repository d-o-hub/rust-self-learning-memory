# Code Conventions

**Note**: The codebase follows Rust idioms automatically. Focus on learning from examples rather than memorizing rules.

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

## Formatting & Linting

### Automatic Formatting
```bash
# Format all code
cargo fmt --all

# Check formatting (fails if not formatted)
cargo fmt --all -- --check
```

### Linting
```bash
# Check with clippy (zero warnings required)
cargo clippy --all -- -D warnings

# Fix clippy suggestions automatically
cargo clippy --fix --allow-dirty --all-targets --all-features

# Check specific crate
cd memory-core && cargo clippy -- -D warnings
```

### Config Files
- `rustfmt.toml`: Formatting configuration
- `rust-toolchain.toml`: Rust version
- `.clippy.toml`: Linting rules
- `cargo/config.toml`: Build configuration

### Zero Warnings Policy
The project enforces **zero clippy warnings**:
- CI pipeline fails on any clippy warning
- Use `#[allow(...)]` sparingly with justification
- Fix warnings before committing
- Run `cargo clippy --all -- -D warnings` before PR

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

## Quality Gates

The project enforces quality gates:

```bash
# Run all quality checks
./scripts/quality-gates.sh

# Individual checks
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
cargo test --all
cargo llvm-cov --html --output-dir coverage
```

**Thresholds:**
- Coverage: >90% target
- Test pass rate: >99% target
- Clippy warnings: 0 (strictly enforced)
- File size: <500 LOC (strictly enforced)

## Key Principles

1. **Learn from Examples**: Study existing code patterns
2. **Use Tools**: Let formatters and linters do the work
3. **Keep Files Small**: Split large files (>500 lines)
4. **Document Public APIs**: Use rustdoc comments with backticks for code elements
5. **Test Everything**: Follow test conventions above, maintain >90% coverage
6. **Follow Rust Idioms**: Use Result, Option, iterators, etc.
7. **Apply 2025 Best Practices**: Modern format strings, From trait conversions, range contains
8. **Zero Warnings**: Strictly enforced, fix all clippy warnings
9. **Postcard Serialization**: Use Postcard, not bincode (v0.1.7 breaking change)
10. **Safe Database Access**: Always use parameterized queries

The best way to learn conventions is to examine the codebase structure and follow the patterns you see in similar files. For recent changes, see [CLAUDE.md](../../.claude/CLAUDE.md).