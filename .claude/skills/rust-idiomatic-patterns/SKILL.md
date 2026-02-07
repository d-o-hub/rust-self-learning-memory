---
name: rust-idiomatic-patterns
description: >
  Comprehensive Rust idiomatic patterns with 179 rules across 14 categories.
  Use when writing, reviewing, or refactoring Rust code. Covers ownership,
  error handling, async patterns, API design, memory optimization, performance,
  testing, and anti-patterns. Invoke with /rust-idiomatic-patterns or specific
  category like "check ownership patterns" or "review error handling".
---

# Rust Idiomatic Patterns

179 battle-tested rules for writing high-quality, idiomatic Rust code across 14 categories.

## Quick Reference

| Priority | Category | Rules | Prefix | When to Apply |
|----------|----------|-------|--------|---------------|
| CRITICAL | [Ownership](#1-ownership--borrowing) | 12 | `own-` | Writing functions, handling data |
| CRITICAL | [Error Handling](#2-error-handling) | 12 | `err-` | Error types, propagation |
| CRITICAL | [Memory](#3-memory-optimization) | 15 | `mem-` | Collections, allocations |
| HIGH | [API Design](#4-api-design) | 15 | `api-` | Public interfaces, builders |
| HIGH | [Async](#5-asyncawait) | 15 | `async-` | Tokio, concurrency |
| HIGH | [Optimization](#6-compiler-optimization) | 12 | `opt-` | Release builds, hot paths |
| MEDIUM | [Naming](#7-naming-conventions) | 16 | `name-` | Functions, types, variables |
| MEDIUM | [Type Safety](#8-type-safety) | 10 | `type-` | Newtypes, enums |
| MEDIUM | [Testing](#9-testing) | 13 | `test-` | Unit, integration, property |
| MEDIUM | [Documentation](#10-documentation) | 11 | `doc-` | Public APIs, examples |
| MEDIUM | [Performance](#11-performance-patterns) | 11 | `perf-` | Iterators, hot loops |
| LOW | [Project Structure](#12-project-structure) | 11 | `proj-` | Modules, workspaces |
| LOW | [Linting](#13-clippy--linting) | 11 | `lint-` | CI configuration |
| REF | [Anti-patterns](#14-anti-patterns) | 15 | `anti-` | Code review |

## Rule Categories

### 1. Ownership & Borrowing
**Critical rules for memory safety and zero-cost abstractions**

- **Prefer borrowing over cloning** - Accept `&T` instead of cloning
- **Slice over Vec** - Accept `&[T]` not `&Vec<T>`, `&str` not `&String`
- **Cow for conditional ownership** - Use `Cow<'a, T>` when ownership varies
- **Arc for shared data** - Use `Arc<T>` for thread-safe sharing
- **Rc for single-threaded** - Use `Rc<T>` when not Send
- **RefCell for interior mutability** - Use `RefCell<T>` (single-thread)
- **Mutex for thread-safe** - Use `Mutex<T>` for multi-thread interior mut
- **RwLock for read-heavy** - Use `RwLock<T>` when reads dominate
- **Derive Copy for small types** - Types ≤ 16 bytes that are trivial
- **Explicit Clone** - Avoid implicit copies, make Clone visible
- **Move large data** - Transfer ownership instead of cloning large types
- **Lifetime elision** - Rely on elision before explicit lifetimes

**[See detailed rules →](rules/ownership.md)**

### 2. Error Handling
**Critical for robust applications and libraries**

- **thiserror for libraries** - Use `thiserror` for library error types
- **anyhow for apps** - Use `anyhow` for application error handling
- **Result over panic** - Return `Result`, don't panic on expected errors
- **Context chaining** - Use `.context()` for error messages
- **No unwrap in prod** - Never use `.unwrap()` in production code
- **Expect for bugs only** - Use `.expect()` only for programming errors
- **? operator** - Use `?` for clean error propagation
- **#[from] attribute** - Use `#[from]` for automatic conversion
- **#[source] chaining** - Chain underlying errors with `#[source]`
- **Lowercase messages** - Error messages: lowercase, no trailing punctuation
- **Document errors** - Include `# Errors` section in docs
- **Custom error types** - Create custom types, not `Box<dyn Error>`

**[See detailed rules →](rules/error-handling.md)**

### 3. Memory Optimization
**Critical for performance-critical code**

- **with_capacity()** - Preallocate when size is known
- **SmallVec** - Use for usually-small collections
- **ArrayVec** - Use for bounded-size collections
- **Box large variants** - Box large enum variants
- **Box<[T]> over Vec** - Use when size is fixed after creation
- **clone_from()** - Reuse allocations with `clone_from()`
- **clear() and reuse** - Reuse collections with `clear()` in loops
- **Avoid format!()** - Don't use `format!()` when literals work
- **write!() macro** - Use `write!()` instead of `format!()`
- **Arena allocators** - Use for batch allocations
- **Zero-copy** - Use slices and `Bytes` for zero-copy
- **CompactString** - Use for small string optimization
- **Smallest integer** - Use smallest type that fits (u8, u16, etc.)
- **Assert type sizes** - Assert hot type sizes to prevent regressions

**[See detailed rules →](rules/memory.md)**

### 4. API Design
**High-impact patterns for public interfaces**

- **Builder pattern** - Use for complex construction
- **#[must_use] on builders** - Prevent ignoring builder results
- **Newtypes for safety** - Wrap IDs and validated data
- **Typestate pattern** - Use for compile-time state machines
- **Sealed traits** - Seal traits to prevent external impls
- **Extension traits** - Add methods to foreign types
- **Parse don't validate** - Parse into validated types at boundaries
- **impl Into<T>** - Accept for flexible string inputs
- **impl AsRef<T>** - Accept for borrowed inputs
- **#[must_use] on Result** - Add to Result-returning functions
- **#[non_exhaustive]** - For future-proof enums/structs
- **Implement From** - Implement `From`, not `Into` (auto-derived)
- **Default impl** - Implement `Default` for sensible defaults
- **Common traits** - Derive `Debug`, `Clone`, `PartialEq` eagerly
- **Serde optional** - Gate behind feature flag

**[See detailed rules →](rules/api-design.md)**

### 5. Async/Await
**High-impact patterns for concurrent code**

- **Tokio runtime** - Use Tokio for production async
- **No locks across await** - Never hold Mutex/RwLock across `.await`
- **spawn_blocking** - Use for CPU-intensive work
- **tokio::fs** - Use async fs operations
- **CancellationToken** - For graceful shutdown
- **tokio::join!** - For parallel operations
- **tokio::try_join!** - For fallible parallel ops
- **tokio::select!** - For racing/timeouts
- **Bounded channels** - Use bounded for backpressure
- **mpsc for queues** - Use for work queues
- **broadcast for pub/sub** - Use for publish/subscribe
- **watch for latest** - Use for latest-value sharing
- **oneshot for response** - Use for request/response
- **JoinSet** - For dynamic task groups
- **Clone before await** - Clone data before await, release locks

**[See detailed rules →](rules/async.md)**

### 6. Compiler Optimization
**High-impact for release performance**

- **#[inline] small** - Use for small hot functions
- **#[inline(always)] sparingly** - Use rarely, only when proven
- **#[inline(never)] cold** - For cold error paths
- **#[cold] attribute** - Mark unlikely paths
- **likely()/unlikely()** - Branch hints for hot paths
- **LTO in release** - Enable `lto = "fat"`
- **codegen-units = 1** - For max optimization
- **PGO profiling** - Use profile-guided optimization
- **target-cpu=native** - For local builds only
- **Iterators avoid bounds** - Use iterators over indexing
- **Portable SIMD** - Use for data-parallel operations
- **Cache-friendly layout** - Structure of Arrays (SoA) for hot data

**[See detailed rules →](rules/optimization.md)**

### 7-14. Additional Categories

**[Naming Conventions →](rules/naming.md)** | **[Type Safety →](rules/type-safety.md)** | **[Testing →](rules/testing.md)**

**[Documentation →](rules/documentation.md)** | **[Performance →](rules/performance.md)** | **[Project Structure →](rules/project-structure.md)**

**[Linting →](rules/linting.md)** | **[Anti-patterns →](rules/anti-patterns.md)**

## Recommended Cargo.toml

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.dev.package."*"]
opt-level = 3  # Optimize dependencies in dev
```

## Usage by Task

| Task | Primary Categories |
|------|-------------------|
| New function | Ownership, Error Handling, Naming |
| New struct/API | API Design, Type Safety, Documentation |
| Async code | Async, Ownership |
| Error handling | Error Handling, API Design |
| Memory optimization | Memory, Ownership, Performance |
| Performance tuning | Optimization, Memory, Performance |
| Code review | Anti-patterns, Linting |

## Sources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- Production codebases: ripgrep, tokio, serde, polars, axum
