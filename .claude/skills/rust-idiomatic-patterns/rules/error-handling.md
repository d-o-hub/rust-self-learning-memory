# Error Handling Rules

## err-thiserror-lib

Use `thiserror` for library error types.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("invalid data: {0}")]
    Invalid(String),
    #[error("not found")]
    NotFound,
}
```

## err-anyhow-app

Use `anyhow` for application error handling.

```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.toml")
        .context("failed to read config file")?;
    Ok(toml::from_str(&content)?)
}
```

## err-result-over-panic

Return `Result`, don't panic on expected errors.

**Bad:**
```rust
fn parse_id(s: &str) -> u64 {
    s.parse().unwrap() // Panics on invalid input
}
```

**Good:**
```rust
fn parse_id(s: &str) -> Result<u64, ParseIntError> {
    s.parse() // Caller handles the error
}
```

## err-context-chain

Add context with `.context()` or `.with_context()`.

```rust
use anyhow::Context;

let user = db.find_user(id)
    .with_context(|| format!("failed to find user {}", id))?;
```

## err-no-unwrap-prod

Never use `.unwrap()` in production code.

**Bad:**
```rust
let config = load_config().unwrap();
```

**Good:**
```rust
let config = load_config()
    .context("failed to load config")?;
```

## err-expect-bugs-only

Use `.expect()` only for programming errors.

```rust
// OK: This is a bug if it fails
let mutex = Mutex::new(data);
let guard = mutex.lock().expect("mutex poisoned");
```

## err-question-mark

Use `?` operator for clean propagation.

```rust
fn load_and_process() -> Result<Data> {
    let raw = load_file()?;      // Propagates error
    let parsed = parse(raw)?;    // Propagates error
    Ok(process(parsed)?)
}
```

## err-from-impl

Use `#[from]` for automatic error conversion.

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error")]
    #[from]
    Io(#[from] std::io::Error),
    
    #[error("parse error")]
    #[from]
    Parse(#[from] serde_json::Error),
}
```

## err-source-chain

Use `#[source]` to chain underlying errors.

```rust
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("database error")]
    Database {
        #[source]
        source: sqlx::Error,
    },
}
```

## err-lowercase-msg

Error messages: lowercase, no trailing punctuation.

**Bad:**
```rust
#[error("Invalid input!")]
```

**Good:**
```rust
#[error("invalid input")]
```

## err-doc-errors

Document errors with `# Errors` section.

```rust
/// Parses a configuration file.
///
/// # Errors
///
/// Returns `ConfigError` if the file is not found or contains invalid TOML.
pub fn parse_config(path: &str) -> Result<Config, ConfigError> {
    // ...
}
```

## err-custom-type

Create custom error types, not `Box<dyn Error>`.

**Bad:**
```rust
fn do_something() -> Result<(), Box<dyn std::error::Error>> {
    // ...
}
```

**Good:**
```rust
#[derive(Error, Debug)]
pub enum MyError { ... }

fn do_something() -> Result<(), MyError> {
    // ...
}
```
