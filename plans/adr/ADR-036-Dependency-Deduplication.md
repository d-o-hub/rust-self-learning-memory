# ADR-036: Dependency Deduplication and Optimization

- **Status**: Proposed
- **Date**: 2026-02-21
- **Deciders**: Project maintainers
- **Supersedes**: None
- **Related**: ADR-032 (Disk Space), ADR-031 (Cargo Lock Integrity)

## Context

The workspace has **863 packages** in `Cargo.lock` with **120 duplicate dependency roots**. Key duplicates:

| Dependency | Versions | Source |
|-----------|----------|--------|
| `rand` | 0.7, 0.8, 0.9 (3 versions) | rv, changepoint, direct |
| `getrandom` | 0.1, 0.2, 0.3, 0.4 (4 versions) | rand chain |
| `hashbrown` | 0.12, 0.13, 0.14, 0.15, 0.16 (5 versions) | wasmtime, libsql, indexmap |
| `nalgebra` | 0.32, 0.34 (2 versions) | argmin chain, direct |
| `argmin`/`argmin-math` | 0.8/0.3, 0.11/0.5 (2 versions) | rv, direct |
| `changepoint` | 0.14, 0.15 (2 versions) | augurs-changepoint, direct |
| `itertools` | 0.12, 0.13, 0.14 (3 versions) | criterion, prost, rv |
| `thiserror` | 1.x, 2.x (2 versions) | mixed ecosystem |
| `syn` | 1.x, 2.x (2 versions) | proc-macros |
| `zerocopy` | 0.7, 0.8 (2 versions) | wasmtime chain |

### Impact

- Each duplicate dependency is compiled separately
- Inflates `target/debug/deps/` (currently 3.9 GB)
- Increases compile time linearly
- Increases CI cache size and restore time
- Some duplicates are **unavoidable** (transitive from wasmtime, libsql)

## Decision

### Tier 1: Direct Dependency Cleanup (Immediate)

Use `cargo-machete` and `cargo-shear` to identify and remove unused dependencies:

```bash
cargo install --locked cargo-machete cargo-shear
cargo machete
cargo shear
```

### Tier 2: Feature Pruning (Short-term)

Audit dependency features to reduce transitive dependency count:

```bash
cargo install --locked cargo-unused-features
cargo unused-features analyze
```

Key candidates:
- `tokio = { features = ["full"] }` → use only needed features
- `wasmtime` → evaluate if all features are needed
- `serde = { features = ["derive"] }` → already minimal

### Tier 3: Upstream Alignment (Medium-term)

Track upstream releases that unify major version splits:

| Duplicate | Resolution | Tracking |
|-----------|-----------|----------|
| `changepoint` 0.14/0.15 | Wait for augurs-changepoint to use 0.15 | augurs repo |
| `argmin` 0.8/0.11 | Wait for rv to use argmin 0.11 | rv repo |
| `rand` 0.7/0.8/0.9 | Wait for rv to move to rand 0.9 | rv repo |
| `thiserror` 1.x/2.x | Ecosystem convergence (mostly done) | Cargo.lock |
| `hashbrown` (5 versions) | Unavoidable (wasmtime, libsql) | Monitor |

### Tier 4: Heavy Dependency Audit (Medium-term)

Review whether heavy dependencies justify their compile cost:

| Dependency | Build Cost | Alternative |
|-----------|-----------|-------------|
| `wasmtime` (41.x) | Very high (cranelift, etc.) | Feature-gate behind `wasm` flag |
| `rquickjs` | High | Already feature-gated |
| `augurs-changepoint` | Medium (brings rv, ndarray) | Consider lighter alternatives |
| `nalgebra` | Medium | Only needed for argmin chain |

### Tier 5: Periodic Monitoring

Add `cargo tree -d | wc -l` to quality-gates.sh as a metric:

```bash
# Track duplicate dependency count
DUPES=$(cargo tree -d 2>/dev/null | grep -cE "^[a-z]")
echo "Duplicate dependency roots: $DUPES"
if [ "$DUPES" -gt 130 ]; then
  echo "WARNING: Duplicate dependencies increasing (was 120)"
fi
```

## Consequences

### Positive
- **Reduced compile times** from fewer crates to build
- **Smaller target/** directory
- **Faster CI** from less compilation work
- **Smaller attack surface** from fewer dependencies

### Negative
- Some duplicates are unavoidable (transitive, different semver)
- Removing features may break edge-case functionality
- Upstream tracking is ongoing maintenance

### Risks
- Aggressive pruning may break features
- Upstream crates may lag in version updates
- `cargo-machete` can produce false positives

## References

- [cargo-machete](https://github.com/bnjbvr/cargo-machete) — Detect unused dependencies
- [cargo-shear](https://github.com/Boshen/cargo-shear) — Another unused dependency detector
- [cargo-unused-features](https://github.com/TimonPost/cargo-unused-features) — Find unused features
- [Cargo Book: Reducing Built Code](https://doc.rust-lang.org/cargo/guide/build-performance.html)
