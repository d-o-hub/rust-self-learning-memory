# Anti-patterns

## anti-unwrap-abuse

Don't use `.unwrap()` in production code.

**Bad:**
```rust
let config = std::fs::read_to_string("config.toml").unwrap();
```

**Good:**
```rust
let config = std::fs::read_to_string("config.toml")
    .context("failed to read config")?;
```

## anti-expect-lazy

Don't use `.expect()` for recoverable errors.

**Bad:**
```rust
let file = File::open("data.txt").expect("file not found");
```

**Good:**
```rust
let file = File::open("data.txt")
    .with_context(|| format!("failed to open {}", path))?;
```

## anti-clone-excessive

Don't clone when borrowing works.

**Bad:**
```rust
fn process(items: Vec<i32>) {
    for item in &items {
        println!("{}", item);
    }
}
process(items.clone());
```

**Good:**
```rust
fn process(items: &[ i32]) {
    for item in items {
        println!("{}", item);
    }
}
process(&items);
```

## anti-lock-across-await

Don't hold locks across `.await`.

**Bad:**
```rust
async fn bad() {
    let guard = mutex.lock().unwrap();
    some_async().await; // Deadlock risk!
}
```

**Good:**
```rust
async fn good() {
    let data = {
        let guard = mutex.lock().unwrap();
        guard.data.clone()
    };
    some_async().await;
}
```

## anti-string-for-str

Don't accept `&String` when `&str` works.

**Bad:**
```rust
fn greet(name: &String) { ... }
```

**Good:**
```rust
fn greet(name: &str) { ... }
```

## anti-vec-for-slice

Don't accept `&Vec<T>` when `&[T]` works.

**Bad:**
```rust
fn sum(numbers: &Vec<i32>) -> i32 { ... }
```

**Good:**
```rust
fn sum(numbers: &[i32]) -> i32 { ... }
```

## anti-index-over-iter

Don't use indexing when iterators work.

**Bad:**
```rust
for i in 0..vec.len() {
    process(&vec[i]);
}
```

**Good:**
```rust
for item in &vec {
    process(item);
}
```

## anti-panic-expected

Don't panic on expected/recoverable errors.

**Bad:**
```rust
fn parse_id(s: &str) -> u64 {
    s.parse().unwrap() // Panics on bad input
}
```

**Good:**
```rust
fn parse_id(s: &str) -> Result<u64, ParseIntError> {
    s.parse() // Caller decides
}
```

## anti-empty-catch

Don't use empty `if let Err(_) = ...` blocks.

**Bad:**
```rust
if let Err(_) = operation() {
    // Empty!
}
```

**Good:**
```rust
if let Err(e) = operation() {
    log::error!("Operation failed: {}", e);
}
// Or use let _ = ... to explicitly ignore
let _ = operation();
```

## anti-over-abstraction

Don't over-abstract with excessive generics.

**Bad:**
```rust
fn process<T, U, V, F, G>(
    input: T,
    f: F,
    g: G,
) -> V
where
    F: Fn(T) -> U,
    G: Fn(U) -> V,
{ ... }
```

**Good:**
```rust
fn process(input: Input) -> Output {
    let intermediate = transform(input);
    finalize(intermediate)
}
```

## anti-premature-optimize

Don't optimize before profiling.

**Bad:**
```rust
// Complex unsafe code for "performance"
// without measuring first
```

**Good:**
```rust
// Simple, clear implementation first
// Profile, then optimize if needed
```

## anti-type-erasure

Don't use `Box<dyn Trait>` when `impl Trait` works.

**Bad:**
```rust
fn get_iter() -> Box<dyn Iterator<Item = i32>> {
    Box::new(0..100)
}
```

**Good:**
```rust
fn get_iter() -> impl Iterator<Item = i32> {
    0..100
}
```

## anti-format-hot-path

Don't use `format!()` in hot paths.

**Bad:**
```rust
for i in 0..1_000_000 {
    log::debug!("{}", format!("item {}", i));
}
```

**Good:**
```rust
for i in 0..1_000_000 {
    log::debug!("item {}", i);
}
```

## anti-collect-intermediate

Don't `collect()` intermediate iterators.

**Bad:**
```rust
let temp: Vec<_> = data.iter().map(|x| x * 2).collect();
let result = temp.iter().filter(|x| x > &10).count();
```

**Good:**
```rust
let result = data.iter()
    .map(|x| x * 2)
    .filter(|x| x > &10)
    .count();
```

## anti-stringly-typed

Don't use strings for structured data.

**Bad:**
```rust
fn set_status(status: &str) {
    match status {
        "active" => ..., // Typo-prone
        "inactive" => ...,
        _ => ...,
    }
}
```

**Good:**
```rust
enum Status {
    Active,
    Inactive,
}

fn set_status(status: Status) {
    match status {
        Status::Active => ...,
        Status::Inactive => ...,
    }
}
```
