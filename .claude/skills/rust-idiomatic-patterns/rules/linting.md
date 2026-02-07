# Clippy & Linting Rules

## lint-deny-correctness

`#![deny(clippy::correctness)]`

```rust
#![deny(clippy::correctness)]
```

## lint-warn-suspicious

`#![warn(clippy::suspicious)]`

```rust
#![warn(clippy::suspicious)]
```

## lint-warn-style

`#![warn(clippy::style)]`

```rust
#![warn(clippy::style)]
```

## lint-warn-complexity

`#![warn(clippy::complexity)]`

```rust
#![warn(clippy::complexity)]
```

## lint-warn-perf

`#![warn(clippy::perf)]`

```rust
#![warn(clippy::perf)]
```

## lint-pedantic-selective

Enable `clippy::pedantic` selectively.

```rust
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
```

## lint-missing-docs

`#![warn(missing_docs)]`

```rust
#![warn(missing_docs)]
```

## lint-unsafe-doc

`#![warn(clippy::undocumented_unsafe_blocks)]`

```rust
#![warn(clippy::undocumented_unsafe_blocks)]
```

## lint-cargo-metadata

`#![warn(clippy::cargo)]` for published crates.

```rust
#![warn(clippy::cargo)]
```

## lint-rustfmt-check

Run `cargo fmt --check` in CI.

```yaml
# .github/workflows/ci.yml
- name: Check formatting
  run: cargo fmt -- --check
```

## lint-workspace-lints

Configure lints at workspace level.

```toml
# Cargo.toml (workspace root)
[workspace.lints.clippy]
correctness = "deny"
suspicious = "warn"
style = "warn"
complexity = "warn"
perf = "warn"

# In member Cargo.toml
[lints]
workspace = true
```
