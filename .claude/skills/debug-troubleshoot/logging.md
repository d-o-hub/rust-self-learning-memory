# Logging Configuration

## Setup

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument]
async fn problematic_function(id: &str) -> Result<Data> {
    debug!("Starting operation for id: {}", id);
    // ...
}
```

## Run with Logging

```bash
# Info level
RUST_LOG=info cargo run

# Debug level
RUST_LOG=debug cargo run

# Trace level (verbose)
RUST_LOG=trace cargo run

# Specific module
RUST_LOG=memory_core::storage=debug cargo run

# Multiple modules
RUST_LOG=memory_core=debug,memory_storage_turso=trace cargo run
```

## Console Subscriber

```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
}
```

## Pretty Printing

```bash
# Colored output
RUST_LOG=debug cargo run 2>&1 | less -R

# JSON format (for parsing)
RUST_LOG_FORMAT=json RUST_LOG=debug cargo run
```
