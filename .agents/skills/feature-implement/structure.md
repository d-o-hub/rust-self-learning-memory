# Module Structure

## Standard Layout

```
src/
└── feature_name/
    ├── mod.rs          # Public exports + docs
    ├── core.rs         # Core business logic
    ├── storage.rs      # Database operations
    ├── types.rs        # Data structures
    └── tests/          # Integration tests
        └── mod.rs
```

## mod.rs Template

```rust
//! Feature name module
//!
//! Brief description of what this module provides.

pub mod core;
pub mod storage;
pub mod types;

pub use types::{FeatureData, FeatureConfig};
pub use core::Feature;
```

## File Size Rules

- Each file: ≤ 500 LOC
- Split larger files into submodules
- Single responsibility per module

## Cargo.toml Updates

```toml
[package]
# ... existing config

[features]
# Add feature flags if needed

[dependencies]
# Add new dependencies
```

## lib.rs Updates

```rust
// src/lib.rs

pub mod feature_name;  // Add this line
```
