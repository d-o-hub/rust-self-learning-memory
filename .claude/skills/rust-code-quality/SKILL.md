---
skill_name: rust-code-quality
description: Perform comprehensive Rust code quality reviews against best practices for async Rust, error handling, testing, and project structure
version: 1.0.0
tags: [rust, code-quality, best-practices, review, project]
tools: [Read, Glob, Grep, Bash]
---

# Rust Code Quality Review Skill

Systematically review Rust code quality against industry best practices, focusing on async/Tokio patterns, error handling, module organization, testing, and documentation.

## Purpose

Ensure the Rust self-learning memory project follows:
- **Rust idioms** and best practices
- **Clean code principles** (readable, maintainable, testable)
- **Async/await patterns** with Tokio
- **Error handling** with Result types
- **Module organization** (<500 LOC per file)
- **Testing** (unit, integration, benchmarks)
- **Documentation** (rustdoc, examples)
- **Performance** (zero-copy, minimal allocations)
- **Security** (memory safety, input validation)

## Quality Dimensions

### 1. Project Structure & Organization

**Criteria**:
- [ ] Workspace organization (Cargo.toml with proper dependencies)
- [ ] Crate separation (core, storage, mcp, test-utils)
- [ ] Module hierarchy (lib.rs, mod.rs structure)
- [ ] File size limits (<500 LOC per file)
- [ ] Naming conventions (snake_case, consistency)

**Check**:
```bash
# Verify workspace structure
cat Cargo.toml

# Check file sizes
find . -name "*.rs" -not -path "*/target/*" -exec wc -l {} + | sort -rn | head -20

# Check module organization
find . -name "lib.rs" -o -name "mod.rs" | head -10
```

**Best Practices**:
```rust
// ✅ GOOD: Clear module hierarchy
// lib.rs
pub mod episode;
pub mod pattern;
pub mod memory;

pub use episode::Episode;
pub use pattern::Pattern;
pub use memory::SelfLearningMemory;

// ❌ BAD: Everything in one file
// lib.rs (5000 lines)
pub struct Episode { ... }
pub struct Pattern { ... }
impl Episode { ... }  // 1000+ lines
impl Pattern { ... }  // 1000+ lines
```

### 2. Error Handling

**Criteria**:
- [ ] Custom Error enum with thiserror
- [ ] Result<T> return types for fallible operations
- [ ] Error propagation with `?` operator
- [ ] Meaningful error messages with context
- [ ] No unwrap() in production code (only in tests)
- [ ] Error conversion implementations

**Check**:
```bash
# Find unwrap usage (should be minimal, tests only)
rg "\.unwrap\(\)" --glob "!*/tests/*" --glob "*.rs"

# Find expect usage
rg "\.expect\(" --glob "!*/tests/*" --glob "*.rs"

# Check error.rs files
find . -name "error.rs" -exec cat {} \;
```

**Best Practices**:
```rust
// ✅ GOOD: Proper error handling
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Episode not found: {0}")]
    EpisodeNotFound(Uuid),

    #[error("Database connection failed: {0}")]
    DatabaseConnection(#[from] libsql::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn get_episode(&self, id: Uuid) -> Result<Episode> {
    let episode = self.storage
        .get(id)
        .await?  // Propagate error
        .ok_or_else(|| Error::EpisodeNotFound(id))?;
    Ok(episode)
}

// ❌ BAD: Unwrap everywhere
pub async fn get_episode(&self, id: Uuid) -> Episode {
    self.storage.get(id).await.unwrap().unwrap()  // PANIC!
}
```

### 3. Async/Await Patterns

**Criteria**:
- [ ] Proper async fn signatures
- [ ] Tokio runtime usage (#[tokio::main], #[tokio::test])
- [ ] spawn_blocking for CPU-bound or sync operations
- [ ] No blocking calls in async context
- [ ] Proper Future trait usage
- [ ] Stream usage for iterators
- [ ] Concurrent operations with join! or join_all

**Check**:
```bash
# Find async functions
rg "async fn" --glob "*.rs"

# Check Tokio usage
rg "#\[tokio::" --glob "*.rs"

# Find spawn_blocking usage
rg "spawn_blocking" --glob "*.rs"

# Check for problematic blocking calls
rg "std::thread::sleep|std::fs::" --glob "*.rs"
```

**Best Practices**:
```rust
// ✅ GOOD: Proper async patterns
use tokio::task;

pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
    // Async operation
    self.turso.store(episode).await?;

    // Sync redb wrapped in spawn_blocking
    let episode_clone = episode.clone();
    let redb = Arc::clone(&self.redb);
    task::spawn_blocking(move || {
        redb.store(&episode_clone)
    })
    .await
    .map_err(|e| Error::TaskJoin(e.to_string()))??;

    Ok(())
}

// Concurrent operations
pub async fn sync_all(&self) -> Result<()> {
    let episodes_future = self.sync_episodes();
    let patterns_future = self.sync_patterns();

    tokio::try_join!(episodes_future, patterns_future)?;
    Ok(())
}

// ❌ BAD: Blocking in async
pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
    std::thread::sleep(Duration::from_secs(1));  // BLOCKS RUNTIME!
    self.redb.store(episode)?;  // Synchronous blocking call
    Ok(())
}
```

### 4. Memory & Performance

**Criteria**:
- [ ] Minimize allocations (use &str, &[u8], borrowing)
- [ ] Avoid unnecessary clones
- [ ] Use Cow<'a, str> when appropriate
- [ ] Zero-copy deserialization where possible
- [ ] Appropriate collection types (Vec, HashMap, BTreeMap)
- [ ] Streaming large datasets (not loading all in memory)

**Check**:
```bash
# Find clone usage (should be justified)
rg "\.clone\(\)" --glob "*.rs" | wc -l

# Check for inefficient patterns
rg "to_string|to_owned" --glob "*.rs" | wc -l
```

**Best Practices**:
```rust
// ✅ GOOD: Borrowing and zero-copy
pub fn analyze_episode(&self, episode: &Episode) -> Analysis {
    // Borrow, don't clone
    let steps = &episode.steps;
    Analysis::from_steps(steps)
}

pub fn get_description(&self) -> &str {
    &self.description  // Return reference
}

// ❌ BAD: Unnecessary clones
pub fn analyze_episode(&self, episode: &Episode) -> Analysis {
    let steps = episode.steps.clone();  // Unnecessary clone!
    Analysis::from_steps(&steps)
}

pub fn get_description(&self) -> String {
    self.description.clone()  // Always clones!
}
```

### 5. Testing

**Criteria**:
- [ ] Unit tests for each module (#[cfg(test)])
- [ ] Integration tests in tests/ directory
- [ ] Benchmark tests in benches/
- [ ] Test coverage >90% (use cargo-tarpaulin)
- [ ] Test utilities in test-utils crate
- [ ] Property-based tests for complex logic (proptest)
- [ ] Mock implementations for testing

**Check**:
```bash
# Run tests
cargo test --all

# Check test coverage
cargo tarpaulin --out Html --output-dir coverage

# Count tests
rg "#\[test\]|#\[tokio::test\]" --glob "*.rs" | wc -l

# Find integration tests
find tests -name "*.rs" -exec wc -l {} +
```

**Best Practices**:
```rust
// ✅ GOOD: Comprehensive testing
#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::create_test_episode;

    #[tokio::test]
    async fn test_episode_creation() {
        let memory = SelfLearningMemory::new_test().await;
        let context = TaskContext::default();

        let id = memory.start_episode("Test", context, TaskType::CodeGeneration).await;

        assert!(id.is_some());
        let episode = memory.get_episode(id.unwrap()).await.unwrap();
        assert_eq!(episode.task_description, "Test");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let memory = SelfLearningMemory::new_test().await;
        let result = memory.get_episode(Uuid::new_v4()).await;

        assert!(result.is_err());
        match result {
            Err(Error::EpisodeNotFound(_)) => (),
            _ => panic!("Expected EpisodeNotFound error"),
        }
    }
}

// Integration test: tests/learning_cycle.rs
#[tokio::test]
async fn test_full_learning_cycle() {
    // Setup
    let memory = setup_test_memory().await;

    // Execute full cycle
    let id = memory.start_episode(...).await;
    memory.log_step(id, step).await;
    let completed = memory.complete_episode(id, outcome).await;

    // Verify
    assert!(completed.reward.is_some());
    assert!(!completed.patterns.is_empty());
}
```

### 6. Documentation

**Criteria**:
- [ ] Crate-level docs (//! in lib.rs)
- [ ] Module-level docs (//! in mod.rs)
- [ ] Function docs (/// with examples)
- [ ] Public API documented (100% coverage)
- [ ] Code examples that compile (```rust)
- [ ] README.md with quick start
- [ ] CONTRIBUTING.md with guidelines

**Check**:
```bash
# Generate docs
cargo doc --open --no-deps

# Check doc coverage
cargo rustdoc -- -D missing_docs

# Find undocumented public items
rg "^pub (fn|struct|enum|trait)" --glob "*.rs" | head -20
```

**Best Practices**:
```rust
// ✅ GOOD: Comprehensive documentation
/// Start a new learning episode
///
/// Creates a new episode in the memory system and stores it in both
/// Turso (durable) and redb (cache) storage layers.
///
/// # Arguments
///
/// * `task_description` - Human-readable description of the task
/// * `context` - Task context including domain, language, and tags
/// * `task_type` - Type of task (CodeGeneration, Debugging, etc.)
///
/// # Returns
///
/// Returns the UUID of the created episode on success.
///
/// # Errors
///
/// Returns `Error::Storage` if database operations fail.
///
/// # Example
///
/// ```
/// # use memory_core::*;
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// let memory = SelfLearningMemory::new();
/// let context = TaskContext::default();
///
/// let episode_id = memory.start_episode(
///     "Implement authentication".to_string(),
///     context,
///     TaskType::CodeGeneration,
/// ).await?;
///
/// println!("Created episode: {}", episode_id);
/// # Ok(())
/// # }
/// ```
pub async fn start_episode(
    &self,
    task_description: String,
    context: TaskContext,
    task_type: TaskType,
) -> Result<Uuid> {
    // Implementation
}

// ❌ BAD: No documentation
pub async fn start_episode(&self, desc: String, ctx: TaskContext, tt: TaskType) -> Result<Uuid> {
    // No docs, unclear what this does
}
```

### 7. Type Safety & API Design

**Criteria**:
- [ ] Strong typing (avoid String for IDs, use Uuid)
- [ ] Newtype pattern for domain types
- [ ] Builder pattern for complex types
- [ ] Non-exhaustive enums for extensibility
- [ ] Sealed traits where appropriate
- [ ] Default implementations

**Best Practices**:
```rust
// ✅ GOOD: Strong typing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EpisodeId(Uuid);

impl EpisodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

// Builder pattern
pub struct EpisodeBuilder {
    description: String,
    context: TaskContext,
    task_type: TaskType,
}

impl EpisodeBuilder {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            context: TaskContext::default(),
            task_type: TaskType::CodeGeneration,
        }
    }

    pub fn context(mut self, context: TaskContext) -> Self {
        self.context = context;
        self
    }

    pub fn build(self) -> Episode {
        Episode {
            episode_id: Uuid::new_v4(),
            task_description: self.description,
            context: self.context,
            task_type: self.task_type,
            // ...
        }
    }
}

// ❌ BAD: Weak typing
pub async fn get_episode(&self, id: String) -> Result<Episode> {
    // String IDs are error-prone
}
```

### 8. Security & Safety

**Criteria**:
- [ ] No unsafe code (unless absolutely necessary and documented)
- [ ] Input validation for all external inputs
- [ ] SQL parameterization (no string concatenation)
- [ ] Resource limits enforced
- [ ] No sensitive data in logs
- [ ] Secrets from environment variables

**Check**:
```bash
# Find unsafe blocks
rg "unsafe" --glob "*.rs"

# Check for SQL injection risks
rg "format!.*INSERT|format!.*SELECT" --glob "*.rs"

# Check for hardcoded secrets
rg "password|secret|token|key" --glob "*.rs"
```

**Best Practices**:
```rust
// ✅ GOOD: Safe and secure
pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
    // Validate input
    if episode.task_description.len() > 10_000 {
        return Err(Error::InputTooLarge);
    }

    // Parameterized query (safe from injection)
    self.conn.execute(
        "INSERT INTO episodes (id, description) VALUES (?, ?)",
        params![episode.episode_id.to_string(), episode.task_description],
    ).await?;

    Ok(())
}

// Load secrets from environment
let token = std::env::var("TURSO_TOKEN")
    .map_err(|_| Error::MissingConfig("TURSO_TOKEN"))?;

// ❌ BAD: Insecure
pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
    // SQL injection vulnerability!
    let query = format!(
        "INSERT INTO episodes (description) VALUES ('{}')",
        episode.task_description
    );
    self.conn.execute(&query, ()).await?;

    // Hardcoded secret!
    let token = "sk_turso_abc123";
    Ok(())
}
```

## Analysis Workflow

### Step 1: Project Structure Analysis
```bash
# Check workspace
cat Cargo.toml

# Verify crate organization
ls -la memory-*/

# Check file sizes
find . -name "*.rs" -not -path "*/target/*" -exec wc -l {} + | sort -rn
```

### Step 2: Code Pattern Analysis
```bash
# Error handling
rg "Result<|Error::" --glob "*.rs" | wc -l
rg "unwrap\(\)" --glob "!*/tests/*" --glob "*.rs"

# Async patterns
rg "async fn|spawn_blocking|tokio::" --glob "*.rs"

# Performance patterns
rg "clone\(\)|to_string\(\)|Arc<|Rc<" --glob "*.rs"
```

### Step 3: Testing Analysis
```bash
# Run all tests
cargo test --all -- --nocapture

# Coverage
cargo tarpaulin --out Html

# Benchmarks
cargo bench --no-run
```

### Step 4: Documentation Analysis
```bash
# Generate docs
cargo doc --no-deps

# Check for missing docs
cargo rustdoc -- -D missing_docs
```

### Step 5: Linting & Formatting
```bash
# Format check
cargo fmt -- --check

# Clippy (strict mode)
cargo clippy --all-targets --all-features -- -D warnings

# Audit dependencies
cargo audit
```

## Output Format

```markdown
# Rust Code Quality Report
**Generated**: [Date]
**Project**: rust-self-learning-memory

## Executive Summary
- **Overall Score**: X/100
- **Critical Issues**: N
- **Warnings**: M
- **Best Practices**: P/Q met

## Quality Dimensions

### 1. Project Structure: 8/10 ⭐⭐⭐⭐
✅ Good workspace organization
✅ Clear crate separation
⚠️ Some files exceed 500 LOC limit
  - memory-core/src/memory.rs: 623 lines (target: <500)

### 2. Error Handling: 9/10 ⭐⭐⭐⭐⭐
✅ Custom Error enum with thiserror
✅ Consistent Result<T> usage
✅ Minimal unwrap() usage (only in tests)
⚠️ Missing error context in 2 locations
  - memory-storage-turso/src/storage.rs:145

### 3. Async Patterns: 7/10 ⭐⭐⭐⭐
✅ Proper async fn usage
✅ spawn_blocking for redb
❌ Blocking call found in async context
  - memory-core/src/sync.rs:89 - std::fs::read

### 4. Memory & Performance: 8/10 ⭐⭐⭐⭐
✅ Good use of borrowing
✅ Minimal allocations
⚠️ Unnecessary clones in 3 locations
  - memory-core/src/extraction.rs:234

### 5. Testing: 6/10 ⭐⭐⭐
⚠️ Test coverage: 78% (target: >90%)
✅ Good unit test coverage
❌ Missing integration tests for:
  - Full sync cycle
  - Concurrent episode operations
  - Error recovery scenarios

### 6. Documentation: 9/10 ⭐⭐⭐⭐⭐
✅ Crate-level docs complete
✅ Most public APIs documented
⚠️ Missing examples in 2 functions
  - memory-core/src/extraction.rs:extract_patterns

### 7. Type Safety: 9/10 ⭐⭐⭐⭐⭐
✅ Strong typing with Uuid
✅ Good use of newtypes
✅ Builder pattern where appropriate

### 8. Security: 8/10 ⭐⭐⭐⭐
✅ No unsafe code
✅ Parameterized SQL queries
✅ Input validation present
⚠️ Resource limits not enforced in 1 location
  - memory-mcp/src/sandbox.rs:123

## Detailed Findings

### Critical Issues (Must Fix)
1. **Blocking call in async context**
   - File: memory-core/src/sync.rs:89
   - Issue: std::fs::read blocks the Tokio runtime
   - Fix: Use tokio::fs::read
   ```rust
   // BAD
   let data = std::fs::read(path)?;

   // GOOD
   let data = tokio::fs::read(path).await?;
   ```

### Warnings (Should Fix)
1. **File size exceeds limit**
   - File: memory-core/src/memory.rs (623 lines)
   - Target: <500 lines
   - Recommendation: Split into submodules

2. **Test coverage below target**
   - Current: 78%
   - Target: >90%
   - Missing coverage in: pattern extraction, sync logic

### Recommendations (Nice to Have)
1. Add property-based tests with proptest
2. Implement more comprehensive benchmarks
3. Add rustdoc examples for all public APIs

## Action Items

### High Priority
- [ ] Fix blocking call in sync.rs
- [ ] Increase test coverage to 90%
- [ ] Enforce resource limits in sandbox

### Medium Priority
- [ ] Refactor memory.rs (split into submodules)
- [ ] Add missing integration tests
- [ ] Add examples to all public APIs

### Low Priority
- [ ] Reduce unnecessary clones
- [ ] Add property-based tests
- [ ] Improve benchmark coverage
```

## Best Practices Checklist

Use this checklist when reviewing code:

**Project Structure**:
- [ ] Files <500 LOC
- [ ] Clear module hierarchy
- [ ] Consistent naming

**Error Handling**:
- [ ] Custom Error enum
- [ ] Result<T> for fallible ops
- [ ] No unwrap() in production
- [ ] Meaningful error messages

**Async/Await**:
- [ ] async fn for IO operations
- [ ] spawn_blocking for sync/CPU work
- [ ] No blocking calls in async
- [ ] Concurrent operations optimized

**Testing**:
- [ ] Unit tests (>90% coverage)
- [ ] Integration tests
- [ ] Benchmarks
- [ ] Test utilities

**Documentation**:
- [ ] Crate docs
- [ ] Module docs
- [ ] Function docs with examples
- [ ] README and CONTRIBUTING

**Performance**:
- [ ] Minimize allocations
- [ ] Use borrowing
- [ ] Zero-copy where possible

**Security**:
- [ ] No unsafe (unless justified)
- [ ] Input validation
- [ ] Parameterized queries
- [ ] Resource limits

## Example Usage

When invoked, this skill will:
1. Analyze project structure and organization
2. Review error handling patterns
3. Check async/await usage
4. Assess testing quality and coverage
5. Evaluate documentation completeness
6. Identify performance anti-patterns
7. Verify security practices
8. Generate comprehensive quality report with actionable items
