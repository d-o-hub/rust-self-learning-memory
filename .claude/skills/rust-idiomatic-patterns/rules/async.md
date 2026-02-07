# Async/Await Rules

## async-tokio-runtime

Use Tokio for production async runtime.

```rust
#[tokio::main]
async fn main() {
    // Your async code here
}
```

## async-no-lock-await

Never hold `Mutex`/`RwLock` across `.await`.

**Bad:**
```rust
async fn bad() {
    let guard = mutex.lock().unwrap();
    some_async_op().await; // Holding lock across await!
}
```

**Good:**
```rust
async fn good() {
    let data = {
        let guard = mutex.lock().unwrap();
        guard.clone() // Copy data out
    };
    some_async_op().await; // Lock released
}
```

## async-spawn-blocking

Use `spawn_blocking` for CPU-intensive work.

```rust
let result = tokio::task::spawn_blocking(|| {
    // CPU-intensive computation
    expensive_calculation(data)
}).await?;
```

## async-tokio-fs

Use `tokio::fs` not `std::fs` in async code.

**Bad:**
```rust
async fn bad() {
    let content = std::fs::read_to_string("file.txt"); // Blocks!
}
```

**Good:**
```rust
async fn good() {
    let content = tokio::fs::read_to_string("file.txt").await;
}
```

## async-cancellation-token

Use `CancellationToken` for graceful shutdown.

```rust
use tokio_util::sync::CancellationToken;

let token = CancellationToken::new();
let child_token = token.child_token();

// In worker task
loop {
    tokio::select! {
        _ = child_token.cancelled() => break,
        _ = work() => {},
    }
}

// To shutdown
token.cancel();
```

## async-join-parallel

Use `tokio::join!` for parallel operations.

```rust
let (result1, result2) = tokio::join!(
    fetch_data1(),
    fetch_data2(),
);
```

## async-try-join

Use `tokio::try_join!` for fallible parallel ops.

```rust
let (user, orders) = tokio::try_join!(
    fetch_user(id),
    fetch_orders(id),
)?;
```

## async-select-racing

Use `tokio::select!` for racing/timeouts.

```rust
tokio::select! {
    result = long_operation() => {
        println!("Completed: {:?}", result);
    }
    _ = tokio::time::sleep(Duration::from_secs(5)) => {
        println!("Timeout!");
    }
}
```

## async-bounded-channel

Use bounded channels for backpressure.

```rust
let (tx, mut rx) = tokio::sync::mpsc::channel(100); // Bounded

// Sender blocks when buffer full (backpressure)
tx.send(msg).await?;
```

## async-mpsc-queue

Use `mpsc` for work queues.

```rust
let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

// Spawn workers
for _ in 0..4 {
    let mut rx = rx.clone();
    tokio::spawn(async move {
        while let Some(work) = rx.recv().await {
            process(work).await;
        }
    });
}
```

## async-broadcast-pubsub

Use `broadcast` for pub/sub patterns.

```rust
let (tx, mut rx1) = tokio::sync::broadcast::channel(100);
let mut rx2 = tx.subscribe();

// Both receivers get the message
tx.send("hello")?;
```

## async-watch-latest

Use `watch` for latest-value sharing.

```rust
let (tx, mut rx) = tokio::sync::watch::channel(0);

// Receiver always has latest value
assert_eq!(*rx.borrow(), 0);
tx.send(1)?;
assert_eq!(*rx.borrow(), 1);
```

## async-oneshot-response

Use `oneshot` for request/response.

```rust
let (tx, rx) = tokio::sync::oneshot::channel();

tokio::spawn(async move {
    let result = do_work().await;
    tx.send(result).unwrap();
});

let result = rx.await?;
```

## async-joinset-structured

Use `JoinSet` for dynamic task groups.

```rust
let mut set = tokio::task::JoinSet::new();

for id in ids {
    set.spawn(fetch_user(id));
}

while let Some(result) = set.join_next().await {
    println!("Got: {:?}", result?);
}
```

## async-clone-before-await

Clone data before await, release locks.

**Bad:**
```rust
async fn bad() {
    let guard = mutex.lock().unwrap();
    some_op(&guard.data).await; // Holding lock!
}
```

**Good:**
```rust
async fn good() {
    let data = {
        let guard = mutex.lock().unwrap();
        guard.data.clone() // Clone before await
    };
    some_op(&data).await; // Lock released
}
```
