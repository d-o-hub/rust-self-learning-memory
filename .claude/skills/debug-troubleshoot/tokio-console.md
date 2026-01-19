# Tokio Console

## Enable

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full", "tracing"] }
console-subscriber = "0.1"
```

```rust
fn main() {
    console_subscriber::init();
    // Rest of code
}
```

## Run

```bash
# Terminal 1: Run app
cargo run --features tokio-console

# Terminal 2: Run console
tokio-console
```

## What to Look For

- Long-running tasks
- Tasks waiting on locks
- Task spawn rate
- Resource usage
