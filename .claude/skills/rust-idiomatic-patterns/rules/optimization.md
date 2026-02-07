# Compiler Optimization Rules

## opt-inline-small

Use `#[inline]` for small hot functions.

```rust
#[inline]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

## opt-inline-always-rare

Use `#[inline(always)]` sparingly.

```rust
// Only when profiling proves it's needed
#[inline(always)]
fn critical_path() { ... }
```

## opt-inline-never-cold

Use `#[inline(never)]` for cold paths.

```rust
#[inline(never)]
fn handle_error(err: Error) { // Cold path
    log::error!("{}", err);
}
```

## opt-cold-unlikely

Use `#[cold]` for error/unlikely paths.

```rust
#[cold]
fn report_rare_event() { ... }
```

## opt-likely-hint

Use `likely()`/`unlikely()` for branch hints.

```rust
if likely!(condition) {
    // Hot path
} else {
    // Cold path
}
```

## opt-lto-release

Enable LTO in release builds.

```toml
[profile.release]
lto = "fat"  # or "thin" for faster builds
```

## opt-codegen-units

Use `codegen-units = 1` for max optimization.

```toml
[profile.release]
codegen-units = 1  # Slower compile, better optimization
```

## opt-pgo-profile

Use PGO for production builds.

```bash
# Compile with instrumentation
cargo build --release

# Run workload to generate profile
./target/release/myapp --benchmark

# Recompile with profile data
# (requires llvm-profdata)
```

## opt-target-cpu

Set `target-cpu=native` for local builds.

```toml
[profile.release]
rustflags = ["-C", "target-cpu=native"]
```

## opt-bounds-check

Use iterators to avoid bounds checks.

**Bad:**
```rust
for i in 0..vec.len() {
    println!("{}", vec[i]); // Bounds check every iteration
}
```

**Good:**
```rust
for item in &vec {
    println!("{}", item); // No bounds check
}
```

## opt-simd-portable

Use portable SIMD for data-parallel ops.

```rust
use std::simd::*;

let a = f32x4::splat(1.0);
let b = f32x4::splat(2.0);
let c = a + b; // SIMD addition
```

## opt-cache-friendly

Design cache-friendly data layouts (SoA).

**Bad (AoS):**
```rust
struct Particle {
    x: f32, y: f32, z: f32,
    vx: f32, vy: f32, vz: f32,
}
```

**Good (SoA):**
```rust
struct Particles {
    x: Vec<f32>, y: Vec<f32>, z: Vec<f32>,
    vx: Vec<f32>, vy: Vec<f32>, vz: Vec<f32>,
}
```
