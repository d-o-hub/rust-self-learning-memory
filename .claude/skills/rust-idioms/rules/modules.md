# Module & Visibility Patterns

## mod-tests-submodule

Tests as submodule in source.

```rust
// src/parser.rs
fn parse_internal(input: &str) -> Vec<Token> { ... }

#[cfg(test)]
mod tests {
    use super::*; // Access private items
    
    #[test]
    fn test_parse_internal() {
        let tokens = parse_internal("hello");
        assert!(!tokens.is_empty());
    }
}
```

## mod-pub-crate

pub(crate) for internal APIs.

```rust
mod internal {
    // Visible throughout crate
    pub(crate) fn helper() { ... }
}

// Not visible outside crate
use crate::internal::helper;
```

## mod-pub-super

pub(super) for parent-only.

```rust
mod submodule {
    // Only visible to parent module
    pub(super) fn internal() { ... }
}
```

## mod-pub-use-reexport

Clean public API with reexports.

```rust
// lib.rs
mod client;
mod error;
mod types;

pub use client::Client;
pub use error::{Error, Result};
pub use types::{User, Order};

// Users see flat API
use my_crate::{Client, User, Result};
```

## mod-prelude

Prelude module for common imports.

```rust
// prelude.rs
pub use crate::client::Client;
pub use crate::error::{Error, Result};
pub use crate::types::{User, Order};

// Usage
use my_crate::prelude::*;
```

## mod-sealed-trait

Seal traits to prevent external impls.

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait MyTrait: sealed::Sealed {
    fn method(&self);
}

// Only implementable within crate
impl sealed::Sealed for MyType {}
impl MyTrait for MyType {
    fn method(&self) { ... }
}
```
