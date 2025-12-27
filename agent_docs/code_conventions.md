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
}
```

## Code Organization

### File Structure
- **Maximum 500 lines** per file
- Group related functionality
- Use modules for organization

### Naming Conventions
- **Variables/functions**: `snake_case`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`

### 2025 Rust Best Practices

#### Format Strings
Use captured identifiers in format strings (Rust 1.58+):

```rust
// ✅ Modern: Direct variable capture
let message = format!("Processing {items} items in {seconds} seconds");

// ❌ Legacy: Explicit formatting
let message = format!("Processing {} items in {} seconds", items, seconds);
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

## Formatting & Linting

### Automatic Formatting
```bash
# Format all code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting
```bash
# Check with clippy
cargo clippy -- -D warnings

# Fix clippy suggestions
cargo clippy --fix
```

### Config Files
- `rustfmt.toml`: Formatting configuration
- `rust-toolchain.toml`: Rust version
- `.clippy.toml`: Linting rules

## Testing Conventions

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange-Act-Assert pattern
    }
}
```

### Async Tests
```rust
#[tokio::test]
async fn test_async_feature() {
    // Async test implementation
}
```

## Key Principles

1. **Learn from Examples**: Study existing code patterns
2. **Use Tools**: Let formatters and linters do the work
3. **Keep Files Small**: Split large files (>500 lines)
4. **Document Public APIs**: Use rustdoc comments with backticks for code elements
5. **Test Everything**: Follow test conventions above
6. **Follow Rust Idioms**: Use Result, Option, iterators, etc.
7. **Apply 2025 Best Practices**: Modern format strings, From trait conversions, range contains

The best way to learn conventions is to examine the codebase structure and follow the patterns you see in similar files. For recent changes, see [CLIPPY_FIX_PLAN.md](../plans/CLIPPY_FIX_PLAN.md).