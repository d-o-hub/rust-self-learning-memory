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
    // Async operations
    Ok(())
}

// Avoid blocking in async
async fn process_data() -> Result<Data> {
    // Use async versions of blocking operations
    let result = async_operation().await?;
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

### Documentation
```rust
/// Public API documentation with examples
/// 
/// # Examples
/// 
/// ```rust
/// let result = function_name(input);
/// assert!(result.is_ok());
/// ```
pub fn public_function(input: InputType) -> Result<OutputType> {
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
4. **Document Public APIs**: Use rustdoc comments
5. **Test Everything**: Follow test conventions above
6. **Follow Rust Idioms**: Use Result, Option, iterators, etc.

The best way to learn conventions is to examine the codebase structure and follow the patterns you see in similar files.