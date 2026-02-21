# ADR-035: Rust 2024 Edition Migration

- **Status**: Proposed
- **Date**: 2026-02-21
- **Deciders**: Project maintainers
- **Supersedes**: None
- **Related**: ADR-032 (Disk Space), ADR-033 (Testing)

## Context

The workspace currently uses **Rust 2021 edition**. The **Rust 2024 edition** stabilized with Rust 1.85.0 (February 2025) and is now the recommended production default. As of February 2026, it has been stable for a full year.

### Why Migrate

1. **Improved diagnostics**: Better error messages and lint coverage
2. **Language improvements**: `unsafe_op_in_unsafe_fn` lint enabled by default, better lifetime elision rules
3. **Ecosystem alignment**: Major crates and tools assume 2024 edition
4. **Resolver v3**: Better feature unification in workspaces (eventual)
5. **Signal of maintenance**: Staying on 2021 signals stale project
6. **Future-proofing**: New APIs and patterns target 2024+

### Risk Assessment

- **Low risk**: Edition migrations are backwards-compatible by design
- Mixed-edition builds are fully supported (migrate crate-by-crate)
- `cargo fix --edition` handles most mechanical changes
- One year of ecosystem validation since stabilization

## Decision

Migrate all workspace crates from **edition = "2021"** to **edition = "2024"**.

### Migration Plan

#### Step 1: Preparation
```bash
rustup update stable
rustc --version  # Confirm ≥ 1.85.0
```

#### Step 2: Automated Fix
```bash
# Run edition migration tool
cargo +stable fix --edition --workspace --allow-dirty

# Format after migration
cargo fmt --all

# Check for new warnings
cargo clippy --all-targets --all-features -- -D warnings
```

#### Step 3: Workspace Update
```toml
# Cargo.toml
[workspace.package]
edition = "2024"    # was "2021"
```

#### Step 4: Verification
```bash
cargo build --all
cargo test --all
cargo clippy --all-targets -- -D warnings
./scripts/quality-gates.sh
```

#### Step 5: CI Validation
- Push to feature branch
- Verify all CI workflows pass
- Merge to develop → main

### Key 2024 Edition Changes to Watch

| Change | Impact | Action |
|--------|--------|--------|
| `unsafe_op_in_unsafe_fn` default | May surface new warnings | Add explicit `unsafe {}` blocks |
| `rust_2024_compatibility` lints | May flag patterns | Fix per lint guidance |
| Lifetime capture rules | Rare edge cases | Test thoroughly |
| `gen` keyword reservation | Unlikely conflict | Rename if needed |

## Consequences

### Positive
- Access to latest language features and improved diagnostics
- Better lint coverage catches bugs earlier
- Ecosystem alignment with modern Rust
- Signals active maintenance

### Negative
- One-time migration effort (~1-2 hours)
- Requires minimum Rust 1.85.0 (already exceeded by stable)

### Risks
- Near zero: edition migrations are the safest major Rust change
- CI may need `rustup update` if pinned to old toolchain

## References

- [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
- [Rust 1.85.0 Release](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
- [Rust in 2026 Trends](https://medium.com/@blogs-world/rust-in-2026)
