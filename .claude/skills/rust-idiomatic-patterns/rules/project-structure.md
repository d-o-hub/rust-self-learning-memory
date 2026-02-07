# Project Structure

## proj-lib-main-split

Keep `main.rs` minimal, logic in `lib.rs`.

```
src/
  main.rs   # Just CLI parsing and call to lib
  lib.rs    # All the logic
```

```rust
// main.rs
fn main() {
    let args = Args::parse();
    my_crate::run(args).unwrap();
}
```

## proj-mod-by-feature

Organize modules by feature, not type.

**Bad:**
```
src/
  models/
  controllers/
  views/
```

**Good:**
```
src/
  user/
    mod.rs
    auth.rs
    profile.rs
  order/
    mod.rs
    cart.rs
    payment.rs
```

## proj-flat-small

Keep small projects flat.

```
src/
  lib.rs
  error.rs
  utils.rs
```

## proj-mod-rs-dir

Use `mod.rs` for multi-file modules.

```
src/
  user/
    mod.rs      # Public API
    auth.rs     # Private implementation
    db.rs       # Private implementation
```

## proj-pub-crate-internal

Use `pub(crate)` for internal APIs.

```rust
pub(crate) fn internal_helper() { ... }
```

## proj-pub-super-parent

Use `pub(super)` for parent-only visibility.

```rust
pub(super) fn parent_only() { ... }
```

## proj-pub-use-reexport

Use `pub use` for clean public API.

```rust
// lib.rs
pub use self::client::Client;
pub use self::error::{Error, Result};
```

## proj-prelude-module

Create `prelude` module for common imports.

```rust
// prelude.rs
pub use crate::client::Client;
pub use crate::error::{Error, Result};

// Usage
use my_crate::prelude::*;
```

## proj-bin-dir

Put multiple binaries in `src/bin/`.

```
src/
  bin/
    server.rs    # cargo run --bin server
    cli.rs       # cargo run --bin cli
  lib.rs
```

## proj-workspace-large

Use workspaces for large projects.

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["core", "api", "cli"]

[workspace.dependencies]
tokio = "1.0"
```

## proj-workspace-deps

Use workspace dependency inheritance.

```toml
# In member crate Cargo.toml
[dependencies]
tokio = { workspace = true }
serde = { workspace = true, optional = true }
```
