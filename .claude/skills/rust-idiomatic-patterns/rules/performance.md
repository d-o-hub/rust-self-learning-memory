# Performance Patterns

## perf-iter-over-index

Prefer iterators over manual indexing.

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

## perf-iter-lazy

Keep iterators lazy, collect() only when needed.

**Bad:**
```rust
vec.iter()
    .map(|x| x * 2)
    .collect::<Vec<_>>()
    .iter()
    .filter(|x| x > &10)
    .collect::<Vec<_>>();
```

**Good:**
```rust
vec.iter()
    .map(|x| x * 2)
    .filter(|x| x > &10)
    .collect::<Vec<_>>();
```

## perf-collect-once

Don't `collect()` intermediate iterators.

**Bad:**
```rust
let temp: Vec<_> = data.iter().map(...).collect();
let result: Vec<_> = temp.iter().filter(...).collect();
```

**Good:**
```rust
let result: Vec<_> = data.iter().map(...).filter(...).collect();
```

## perf-entry-api

Use `entry()` API for map insert-or-update.

```rust
// Insert or update
counter.entry(key).and_modify(|v| *v += 1).or_insert(1);
```

## perf-drain-reuse

Use `drain()` to reuse allocations.

```rust
let mut buf = Vec::with_capacity(1000);
for chunk in chunks {
    buf.clear();
    buf.extend_from_slice(chunk);
    process(&buf);
}
```

## perf-extend-batch

Use `extend()` for batch insertions.

```rust
// Single allocation
vec.extend(other.iter().map(|x| x * 2));

// vs multiple push() calls
for x in other {
    vec.push(x * 2);
}
```

## perf-chain-avoid

Avoid `chain()` in hot loops.

**Bad:**
```rust
for item in list1.iter().chain(list2.iter()) {
    // Process
}
```

**Good:**
```rust
for item in list1.iter() {
    // Process
}
for item in list2.iter() {
    // Process
}
```

## perf-collect-into

Use `collect_into()` for reusing containers.

```rust
let mut result = Vec::with_capacity(1000);
(0..1000).map(|x| x * 2).collect_into(&mut result);
```

## perf-black-box-bench

Use `black_box()` in benchmarks.

```rust
use criterion::black_box;

fn bench(c: &mut Criterion) {
    c.bench_function("fib", |b| {
        b.iter(|| fibonacci(black_box(20)))
    });
}
```

## perf-release-profile

Optimize release profile settings.

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

## perf-profile-first

Profile before optimizing.

```bash
# Build with debug symbols
cargo build --release

# Profile
cargo flamegraph
# or
cargo instruments -t time
```
