# Code Quality

Maintain high code quality through formatting, linting, and static analysis.

## Purpose
Ensure consistent code style, catch common mistakes, and enforce best practices across the codebase.

## Tools

### 1. Rustfmt (Formatting)

#### Format Code
```bash
# Format all code
cargo fmt

# Check formatting without changing files
cargo fmt -- --check

# Format specific file
rustfmt src/lib.rs
```

#### Configuration: rustfmt.toml
```toml
max_width = 100
tab_spaces = 4
edition = "2021"
use_small_heuristics = "Max"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

#### Pre-Commit Hook
Automatically format before commit (see hooks section).

### 2. Clippy (Linting)

#### Run Clippy
```bash
# Run all lints
cargo clippy

# Run on all targets
cargo clippy --all-targets

# Treat warnings as errors (CI)
cargo clippy --all -- -D warnings

# Fix automatically when possible
cargo clippy --fix
```

#### Common Lint Categories

**Correctness** (Deny by default):
- Type errors
- Logic errors
- Memory safety issues

**Performance**:
- Unnecessary clones
- Inefficient algorithms
- Boxing when unnecessary

**Style**:
- Idiomatic Rust patterns
- Naming conventions
- Code organization

**Complexity**:
- Overly complex functions
- Deep nesting
- Long parameter lists

#### Configure Clippy: .clippy.toml
```toml
# Deny these lints
warn-on-all-wildcard-imports = true

# Allow these lints
disallowed-names = ["foo", "bar", "baz"]

# Thresholds
cognitive-complexity-threshold = 30
```

#### Inline Lint Control
```rust
// Allow specific lint for function
#[allow(clippy::too_many_arguments)]
fn complex_function(...) {}

// Deny for module
#![deny(clippy::unwrap_used)]

// Warn for specific code
#[warn(clippy::cast_lossless)]
let x = value as u64;
```

### 3. Cargo Deny (Dependency Auditing)

#### Check Dependencies
```bash
# Install
cargo install cargo-deny

# Run all checks
cargo deny check

# Check licenses
cargo deny check licenses

# Check security advisories
cargo deny check advisories

# Check bans
cargo deny check bans
```

#### Configuration: deny.toml
```toml
[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
]

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"

[bans]
multiple-versions = "warn"
deny = [
    { name = "openssl", wrappers = ["native-tls"] },
]
```

### 4. Cargo Audit (Security)

#### Check for Vulnerabilities
```bash
# Install
cargo install cargo-audit

# Run audit
cargo audit

# Fix known issues
cargo audit fix
```

### 5. Cargo Outdated (Dependency Updates)

```bash
# Install
cargo install cargo-outdated

# Check for updates
cargo outdated

# With detail
cargo outdated -wR
```

## Quality Checklist

### Before Every Commit
- [ ] `cargo fmt` - Format code
- [ ] `cargo clippy -- -D warnings` - No lint warnings
- [ ] `cargo test --all` - All tests pass
- [ ] `cargo check --all` - Code compiles

### Before Every PR
- [ ] `cargo build --release` - Release build works
- [ ] `cargo doc --no-deps` - Documentation builds
- [ ] `cargo audit` - No security issues
- [ ] Code review passes
- [ ] Tests cover new code

### Weekly/Monthly
- [ ] `cargo outdated` - Check dependency updates
- [ ] `cargo deny check` - License/advisory check
- [ ] `cargo bloat --release` - Check binary size
- [ ] Review and update dependencies

## Common Issues and Fixes

### Issue 1: Unused Imports
```
warning: unused import: `HashMap`
```

**Fix**: Remove the import or use it
```rust
// BEFORE
use std::collections::HashMap;  // Unused

// AFTER - Remove if truly unused
// Or use it
```

### Issue 2: Unnecessary Clone
```
warning: using `clone` on type `Copy`
```

**Fix**: Remove clone for Copy types
```rust
// BEFORE
let x = y.clone();  // y is Copy

// AFTER
let x = y;
```

### Issue 3: Redundant Pattern Matching
```
warning: redundant pattern matching
```

**Fix**: Simplify
```rust
// BEFORE
if let Some(_) = option {
    true
} else {
    false
}

// AFTER
option.is_some()
```

### Issue 4: Missing Error Propagation
```
warning: called `unwrap` on a `Result` value
```

**Fix**: Proper error handling
```rust
// BEFORE
let data = read_file().unwrap();  // Avoid in library code

// AFTER
let data = read_file()?;  // Propagate error
```

### Issue 5: Large Stack Structures
```
warning: large size difference between variants
```

**Fix**: Box large variants
```rust
// BEFORE
enum Message {
    Small(u8),
    Large([u8; 1024]),  // Large!
}

// AFTER
enum Message {
    Small(u8),
    Large(Box<[u8; 1024]>),  // Heap allocated
}
```

## Code Organization

### File Size Limit
Keep each file ≤ 500 LOC (as per AGENTS.md).

**When file grows**:
```
src/
├── storage/
│   ├── mod.rs          # Public interface
│   ├── turso.rs        # Turso implementation
│   └── redb.rs         # redb implementation
```

### Module Structure
```rust
// src/lib.rs
pub mod storage;
pub mod patterns;
pub mod retrieval;

// Clear public API
pub use storage::SelfLearningMemory;
pub use patterns::{Pattern, PatternType};
```

## Documentation Standards

### Public API Documentation
```rust
/// Start a new learning episode.
///
/// # Arguments
///
/// * `task_description` - Clear description of the task
/// * `context` - Task context with language, domain, tags
///
/// # Returns
///
/// Episode ID for subsequent logging
///
/// # Errors
///
/// Returns error if database write fails
///
/// # Example
///
/// ```
/// let id = memory.start_episode("implement feature", ctx).await?;
/// ```
pub async fn start_episode(
    &self,
    task_description: &str,
    context: TaskContext,
) -> Result<String> {
    // ...
}
```

### Generate Documentation
```bash
# Build docs
cargo doc --no-deps

# Build and open
cargo doc --no-deps --open

# Check for broken links
cargo doc --no-deps 2>&1 | grep warning
```

## Performance Linting

### Cargo Flamegraph
```bash
# Install
cargo install flamegraph

# Profile
cargo flamegraph --dev

# View flamegraph.svg
```

### Cargo Bench
```bash
# Run benchmarks
cargo bench

# Specific benchmark
cargo bench pattern_extraction
```

## CI Integration

### GitHub Actions Workflow
```yaml
- name: Format check
  run: cargo fmt -- --check

- name: Clippy
  run: cargo clippy --all-targets -- -D warnings

- name: Audit
  run: cargo audit

- name: Test
  run: cargo test --all
```

## Editor Integration

### VS Code (rust-analyzer)
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.rustfmt.overrideCommand": ["rustfmt"],
  "[rust]": {
    "editor.formatOnSave": true
  }
}
```

### Neovim (rust-tools)
```lua
require('rust-tools').setup({
  tools = {
    inlay_hints = { auto = true },
    hover_actions = { auto_focus = true },
  },
  server = {
    settings = {
      ["rust-analyzer"] = {
        checkOnSave = {
          command = "clippy"
        }
      }
    }
  }
})
```

## Code Review Checklist

- [ ] Follows rustfmt style
- [ ] No clippy warnings
- [ ] Functions are < 50 LOC
- [ ] Files are < 500 LOC
- [ ] Public items are documented
- [ ] Error handling is proper (no unwrap in library)
- [ ] Tests are included
- [ ] No TODO/FIXME without issue reference
- [ ] Async functions use `.await` correctly
- [ ] No unnecessary clones or allocations

## Best Practices

1. **Run clippy regularly** during development
2. **Fix all warnings** before committing
3. **Use `rustfmt`** automatically (editor or hook)
4. **Document public API** thoroughly
5. **Keep functions small** (< 50 LOC)
6. **Keep files modular** (< 500 LOC)
7. **Prefer `?` over `unwrap`** for error handling
8. **Use `#[must_use]`** for important return values
9. **Add examples** to complex APIs
10. **Review code yourself** before requesting review
