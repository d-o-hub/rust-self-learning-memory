---
name: rust-idioms
description: >
  Rust refactoring and idiomatic patterns with 44 rules across 8 categories.
  Use when refactoring code, writing idiomatic Rust, or learning common patterns.
  Covers type safety, ownership, error handling, API design, module organization,
  conversions, patterns, and iterators. Complements rust-idiomatic-patterns.
---

# Rust Idioms

44 idiomatic patterns for writing clean, maintainable Rust code.

## Quick Reference

| Priority | Category | Rules | Prefix |
|----------|----------|-------|--------|
| CRITICAL | [Type Safety](#1-type-safety--newtypes) | 6 | `type-` |
| CRITICAL | [Ownership](#2-ownership--borrowing) | 6 | `own-` |
| CRITICAL | [Error Handling](#3-error-handling) | 5 | `err-` |
| HIGH | [API Design](#4-api-design) | 6 | `api-` |
| HIGH | [Module Organization](#5-module-visibility) | 6 | `mod-` |
| MEDIUM | [Conversions](#6-conversion-traits) | 5 | `conv-` |
| MEDIUM | [Idioms](#7-idiomatic-patterns) | 5 | `idiom-` |
| MEDIUM | [Iterators](#8-iterator-patterns) | 5 | `iter-` |

## Category Summaries

### 1. Type Safety & Newtypes

**6 rules for compile-time guarantees and domain modeling**

- **Newtype for units** - `Meters(f64)`, `Seconds(u64)` for type-safe arithmetic
- **Newtype for IDs** - `UserId(u64)`, `OrderId(u64)` to prevent mixing IDs
- **Validated newtypes** - `Email`, `Phone` with parse-time validation
- **Enum for states** - Use enums instead of booleans for state machines
- **Option for nullable** - Explicit null handling vs magic values
- **PhantomData markers** - Type-level distinctions without runtime cost

**[See detailed rules →](rules/type-safety.md)**

### 2. Ownership & Borrowing

**6 rules for memory safety and performance**

- **Borrow over clone** - `&T` instead of `T` to avoid allocations
- **Slice over Vec** - `&[T]` accepts arrays, vectors, and slices
- **Cow for conditional** - Avoid cloning when not needed
- **Box for recursion** - Indirection for recursive types
- **Rc for shared data** - Reference counting for shared ownership
- **Clone explicit** - Make expensive operations visible

**[See detailed rules →](rules/ownership.md)**

### 3. Error Handling

**5 rules for robust error management**

- **thiserror for libraries** - Derive `Error` without boilerplate
- **anyhow for apps** - Ergonomic error handling in applications
- **? operator** - Clean error propagation
- **Result over panic** - Expected errors are part of the API
- **From impls** - Automatic error conversions

**[See detailed rules →](rules/error-handling.md)**

### 4. API Design

**6 rules for clean public interfaces**

- **Builder pattern** - Step-by-step construction for complex types
- **Typestate pattern** - Compile-time state verification
- **Extension traits** - Add methods to foreign types
- **impl Trait** - Hide implementation details
- **impl Into** - Flexible argument types
- **impl AsRef** - Accept borrowed inputs flexibly

**[See detailed rules →](rules/api-design.md)**

### 5. Module & Visibility

**6 rules for code organization**

- **Tests submodule** - `#[cfg(test)] mod tests` in source files
- **pub(crate)** - Internal APIs visible across crate
- **pub(super)** - Parent-only visibility
- **pub use reexport** - Clean public API
- **prelude module** - Common imports grouped
- **Sealed traits** - Prevent external implementations

**[See detailed rules →](rules/modules.md)**

### 6. Conversion Traits

**5 rules for flexible type conversions**

- **From over Into** - Implement `From`, get `Into` for free
- **TryFrom for fallible** - Fallible conversions with `Result`
- **AsRef for borrowing** - Cheap reference conversions
- **AsMut for mutation** - Mutable reference conversions
- **Borrow for collections** - Hash/Eq-compatible borrowing

**[See detailed rules →](rules/conversions.md)**

### 7. Idiomatic Patterns

**5 common Rust patterns**

- **RAII cleanup** - Drop trait for resource management
- **Deref for smart pointers** - Transparent delegation
- **Default for config** - Sensible defaults
- **Display for user output** - Human-readable formatting
- **Debug for dev output** - Developer-friendly debugging

**[See detailed rules →](rules/idioms.md)**

### 8. Iterator Patterns

**5 patterns for iterator chains**

- **filter_map combo** - Filter and map in one pass
- **fold for accumulation** - Reduce to single value
- **collect with turbofish** - Explicit type annotation
- **Iterator trait** - Custom iterator types
- **IntoIterator for loops** - Make types iterable

**[See detailed rules →](rules/iterators.md)**

## Usage Examples

### Refactoring a Function

**Before:**
```rust
fn process(data: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    for item in data {
        if item > 0 {
            result.push(item * 2);
        }
    }
    result
}
```

**After:**
```rust
fn process(data: &[i32]) -> impl Iterator<Item = i32> + '_ {
    data.iter()
        .filter(|&x| x > 0)
        .map(|x| x * 2)
}
```

### Adding a Newtype

**Before:**
```rust
fn process(user_id: u64, order_id: u64) {
    // Easy to swap arguments by mistake
}
```

**After:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct OrderId(u64);

fn process(user_id: UserId, order_id: OrderId) {
    // Compiler prevents mixing them up
}
```

### Error Handling Setup

**Library:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}
```

**Application:**
```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.toml")
        .context("failed to read config file")?;
    Ok(toml::from_str(&content)?)
}
```

## When to Apply

| Situation | Rules to Check |
|-----------|----------------|
| Writing new code | Ownership, Type Safety, API Design |
| Refactoring existing code | Idioms, Iterators, Conversions |
| Adding error handling | Error Handling patterns |
| Organizing modules | Module patterns |
| Reviewing PRs | All categories |

## Relationship to Other Skills

- **rust-idiomatic-patterns**: This skill focuses on refactoring and idiom usage; idiomatic-patterns covers the full 179-rule comprehensive guide
- **quality-unit-testing**: Use together when writing tests for refactored code
- **rust-code-quality**: Use together for code reviews
