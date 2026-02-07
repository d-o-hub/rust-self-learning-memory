# Documentation Rules

## doc-all-public

Document all public items with `///`.

```rust
/// Represents a user in the system.
pub struct User {
    /// The user's unique identifier.
    pub id: UserId,
    /// The user's display name.
    pub name: String,
}
```

## doc-module-inner

Use `//!` for module-level documentation.

```rust
//! HTTP client implementation.
//!
//! This module provides an async HTTP client with connection pooling.

pub mod client;
pub mod error;
```

## doc-examples-section

Include `# Examples` with runnable code.

```rust
/// Parses a date string.
///
/// # Examples
///
/// ```
/// let date = my_crate::parse_date("2024-01-15").unwrap();
/// assert_eq!(date.year(), 2024);
/// ```
pub fn parse_date(s: &str) -> Result<Date, ParseError> {
    // ...
}
```

## doc-errors-section

Include `# Errors` for fallible functions.

```rust
/// Opens a configuration file.
///
/// # Errors
///
/// Returns `ConfigError` if:
/// - The file does not exist
/// - The file cannot be read
/// - The content is not valid TOML
pub fn open_config(path: &Path) -> Result<Config, ConfigError> {
    // ...
}
```

## doc-panics-section

Include `# Panics` for panicking functions.

```rust
/// Returns the element at the given index.
///
/// # Panics
///
/// Panics if the index is out of bounds.
pub fn get_unchecked(&self, index: usize) -> &T {
    // ...
}
```

## doc-safety-section

Include `# Safety` for unsafe functions.

```rust
/// Converts a raw pointer to a reference.
///
/// # Safety
///
/// The pointer must be:
/// - Non-null
/// - Properly aligned
/// - Point to valid memory
pub unsafe fn from_raw(ptr: *const T) -> &T {
    &*ptr
}
```

## doc-question-mark

Use `?` in examples, not `.unwrap()`.

**Bad:**
```rust
/// # Examples
///
/// ```
/// let config = parse_config("config.toml").unwrap();
/// ```
```

**Good:**
```rust
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = parse_config("config.toml")?;
/// # Ok(())
/// # }
/// ```
```

## doc-hidden-setup

Use `# ` prefix to hide example setup code.

```rust
/// # Examples
///
/// ```
/// # use my_crate::Client;
/// # async fn example() -> Result<(), Error> {
/// let client = Client::new().await?;
/// client.fetch_data().await?;
/// # Ok(())
/// # }
/// ```
```

## doc-intra-links

Use intra-doc links: `[Vec]`.

```rust
/// Similar to [`Vec`], but with a fixed capacity.
///
/// See also [`ArrayVec::new`](ArrayVec::new).
pub struct ArrayVec<T, const N: usize> { ... }
```

## doc-link-types

Link related types and functions in docs.

```rust
/// An error that occurred while parsing.
///
/// See [`parse`] for more details.
///
/// [`parse`]: crate::parse
pub struct ParseError { ... }
```

## doc-cargo-metadata

Fill `Cargo.toml` metadata.

```toml
[package]
name = "my-crate"
version = "1.0.0"
authors = ["Your Name <email@example.com>"]
description = "A brief description of the crate"
license = "MIT OR Apache-2.0"
repository = "https://github.com/user/repo"
documentation = "https://docs.rs/my-crate"
keywords = ["async", "http"]
categories = ["network-programming"]
```
