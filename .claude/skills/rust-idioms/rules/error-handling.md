# Error Handling Patterns

## err-thiserror-lib

Use thiserror for libraries.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("not found: {0}")]
    NotFound(String),
    
    #[error("invalid format")]
    #[from]
    Parse(serde_json::Error),
    
    #[error("io error")]
    #[from]
    Io(#[from] std::io::Error),
}
```

## err-anyhow-app

Use anyhow for applications.

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("failed to load config")?;
    
    run_app(config)?;
    Ok(())
}
```

## err-question-mark

Use ? for error propagation.

```rust
fn load_and_process(path: &str) -> Result<Data, Error> {
    let content = std::fs::read_to_string(path)?;
    let data = serde_json::from_str(&content)?;
    Ok(process(data)?)
}
```

## err-result-over-panic

Return Result for expected errors.

**Bad:**
```rust
fn parse_id(s: &str) -> u64 {
    s.parse().unwrap() // Panics!
}
```

**Good:**
```rust
fn parse_id(s: &str) -> Result<u64, ParseIntError> {
    s.parse() // Caller handles error
}
```

## err-from-impl

Implement From for automatic conversion.

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("io error")]
    #[from]
    Io(std::io::Error),
    
    #[error("parse error")]
    #[from]
    Parse(serde_json::Error),
}

// Now ? works automatically
fn load() -> Result<(), AppError> {
    let file = std::fs::read_to_string("data.json")?; // Converts to AppError
    let data: Value = serde_json::from_str(&file)?;    // Converts to AppError
    Ok(())
}
```
