# GOAP: CLI Step Persistence Fix

**Date**: 2026-04-28
**Type**: Bug Fix (CLI-specific)
**Priority**: P1 - Pattern extraction depends on steps
**WG**: WG-145

---

## Problem Statement

CLI validation revealed that episodes show "Steps: 0" despite logging steps, resulting in no pattern extraction.

### Root Cause Analysis

| Layer | Behavior | Issue |
|-------|----------|-------|
| CLI Config | `batch_config: Some(BatchConfig::default())` | Batching enabled |
| Process Model | Each CLI command = separate process | In-memory buffers lost on exit |
| Step Storage | Steps buffered in `step_buffers` HashMap | Never flushed before process ends |
| Episode View | Reads from redb cache | Cache has 0 steps (never persisted) |

**Key Finding**: Tests (`checkpoint_integration_test.rs`, `mcp_tag_chain.rs`) already use `batch_config: None` to disable batching for single-process workflows.

---

## Fix Options Evaluation

### Option A: Disable Batching in CLI Config

**Change**: Set `batch_config: None` in `memory-cli/src/config/storage.rs:231`

| Metric | Rating |
|--------|--------|
| Simplicity | ★★★★★ (1 line change) |
| Safety | ★★★★★ (CLI-only, no library/MCP impact) |
| Test Coverage | ★★★★★ (existing tests use this pattern) |
| Performance | ★★★★☆ (minor: immediate step persist) |

**Pros**:
- Matches existing test patterns
- No architectural changes
- Steps persist immediately on each `log_step` call
- Pattern extraction works correctly

**Cons**:
- More I/O per step (negligible for CLI use case)

**Recommendation**: ✅ **ADOPT**

### Option B: Add Explicit Flush to CLI log_step

**Change**: Call `memory.flush_steps(episode_id)` after `log_step`

| Metric | Rating |
|--------|--------|
| Simplicity | ★★★☆☆ (requires await, error handling) |
| Safety | ★★★★☆ (may race with completion flush) |
| Test Coverage | ★★★☆☆ (new pattern, needs tests) |

**Cons**:
- Double-flush on completion (race condition potential)
- More complex code path

**Recommendation**: ❌ **REJECT** (Option A is cleaner)

### Option C: Documentation Update Only

**Change**: Document that CLI is for single-step workflows, MCP for multi-step

| Metric | Rating |
|--------|--------|
| Simplicity | ★★★★★ |
| Safety | ★★★★★ |
| Effectiveness | ★☆☆☆☆ (doesn't fix the bug) |

**Cons**:
- Doesn't fix the actual issue
- CLI should work for episode workflows

**Recommendation**: ❌ **REJECT** (apply Option A, update docs as follow-up)

---

## Execution Plan

### Phase 1: Implement Fix (Sequential)

| Task | WG | Status | Owner |
|------|----|--------|-------|
| Disable batching in CLI config | WG-145.1 | 🔵 Planned | direct edit |
| Verify fix with CLI workflow | WG-145.2 | 🔵 Planned | CLI test |

### Phase 2: Documentation Update (Sequential)

| Task | WG | Status | Owner |
|------|----|--------|-------|
| Update troubleshooting.md | WG-145.3 | ✅ Complete | done in session |
| Update commands.md | WG-145.4 | ✅ Complete | done in session |
| Add LESSON-007 for CLI batching | WG-145.5 | 🔵 Planned | learn skill |

### Phase 3: Validation (Parallel)

| Task | WG | Status | Owner |
|------|----|--------|-------|
| Run CLI episode workflow test | WG-145.6 | 🔵 Planned | test-runner |
| Verify pattern extraction works | WG-145.7 | 🔵 Planned | CLI ops |

---

## Quality Gates

- **Gate 1**: CLI `episode view` shows Steps > 0 after `log_step`
- **Gate 2**: Pattern extraction produces patterns after episode completion
- **Gate 3**: No regression in MCP/library batching behavior

---

## Critical Finding: Postcard Deserialization Limitation

### Root Cause Update (2026-04-28)

**Deeper issue discovered**: Even with `batch_config: None`, episodes with ExecutionSteps cannot be retrieved from redb storage.

**Test Evidence** (`memory-core/examples/test_postcard_issue.rs`):
```
✓ Episode without steps deserialized successfully
✗ Episode with 1 step deserialization FAILED: This is a feature that PostCard will never implement
✗ ExecutionStep deserialization FAILED: This is a feature that PostCard will never implement
✗ serde_json::Value deserialization FAILED: This is a feature that PostCard will never implement
```

**Root Cause**: `ExecutionStep.parameters` field uses `serde_json::Value`:
- Postcard CAN serialize serde_json::Value (writes bytes without schema)
- Postcard CANNOT deserialize serde_json::Value (needs type schema)

**Location**: `memory-core/src/episode/structs.rs:58`
```rust
pub struct ExecutionStep {
    pub parameters: serde_json::Value,  // <-- ROOT CAUSE
    // ...
}
```

**Impact**: Episodes are written to redb but cannot be read back, appearing to "disappear".

### Fix Options for Postcard Limitation

| Option | Approach | Pros | Cons | Recommendation |
|--------|----------|------|------|----------------|
| **A** | Replace postcard with serde_json for redb | Full serde_json::Value support | Violates ADR postcard requirement | ❌ REJECT |
| **B** | Change parameters to `HashMap<String, PostcardValue>` enum | Fully postcard-compatible | Breaking API change | ⏳ Long-term |
| **C** | Store parameters as JSON string (`parameters_json: String`) | Non-breaking, preserves semantics | Double-serialization overhead | ✅ **ADOPT** |
| **D** | Separate parameters storage | Clean separation | Schema migration required | ❌ REJECT |

**Recommended Fix (Option C)**: Immediate CLI fix using JSON string field.

### Phase 0: Postcard Fix (Priority) - ✅ COMPLETE

| Task | WG | Status | Owner |
|------|----|--------|-------|
| Rename `ExecutionStep.parameters` → `parameters_json` | WG-145.0.1 | ✅ Complete | direct edit |
| Add `parameters()` helper method | WG-145.0.2 | ✅ Complete | direct edit |
| Add `set_parameters()` helper method | WG-145.0.3 | ✅ Complete | direct edit |
| Add `with_parameters()` builder method | WG-145.0.4 | ✅ Complete | direct edit |
| Update all usages across workspace | WG-145.0.5 | ✅ Complete | refactorer agent |
| Verify postcard test passes | WG-145.0.6 | ✅ Complete | test runner |
| Verify CLI workflow test passes | WG-145.0.7 | ✅ Complete | test runner |

**Verification Results**:
```
✓ Episode without steps deserialized successfully
✓ Episode with 1 step deserialized successfully  ← FIXED!
✓ ExecutionStep deserialized successfully       ← FIXED!
```

**All 2942 tests pass**. CLI workflow test confirms episodes with steps persist correctly.

---

## Implementation Details

### Postcard Fix (Phase 0) - Option C Implementation

**Target**: `memory-core/src/episode/structs.rs`

```rust
// Current:
pub struct ExecutionStep {
    pub parameters: serde_json::Value,
    // ...
}

// Fixed:
pub struct ExecutionStep {
    pub parameters_json: String,  // JSON string, postcard-compatible
    // ...
}

impl ExecutionStep {
    /// Get parameters as serde_json::Value
    pub fn parameters(&self) -> serde_json::Value {
        serde_json::from_str(&self.parameters_json).unwrap_or(serde_json::Value::Null)
    }

    /// Set parameters from serde_json::Value
    pub fn set_parameters(&mut self, value: serde_json::Value) {
        self.parameters_json = serde_json::to_string(&value).unwrap_or_default();
    }

    /// Create new step with parameters
    pub fn new(step_number: usize, tool: String, action: String) -> Self {
        Self {
            step_number,
            timestamp: chrono::Utc::now(),
            tool,
            action,
            parameters_json: "{}".to_string(),  // Empty JSON object
            result: None,
            latency_ms: 0,
            tokens_used: None,
            metadata: HashMap::new(),
        }
    }

    /// Create step with parameters value
    pub fn with_parameters(mut self, params: serde_json::Value) -> Self {
        self.set_parameters(params);
        self
    }
}
```

**Affected files** (need updates):
- `memory-core/src/episode/structs.rs` - ExecutionStep definition
- `memory-core/src/memory/episode.rs` - log_step uses parameters
- `memory-core/src/extraction/` - pattern extraction uses parameters
- `memory-storage-redb/src/episodes.rs` - serialize/deserialize
- `memory-cli/src/commands/step.rs` - CLI log_step command

### Why This Works

The `log_step` implementation in `memory-core/src/memory/episode.rs:239-265` has two paths:

1. **Batching enabled** (`batch_config: Some(...)`) → Steps buffered in HashMap, flushed on threshold/interval
2. **Batching disabled** (`batch_config: None`) → Steps immediately persisted to storage

CLI commands are separate processes that exit before flush thresholds are met. Disabling batching forces immediate persistence.

---

## Related Files

- `memory-cli/src/config/storage.rs:231` - CLI config
- `memory-core/src/memory/episode.rs:239-265` - log_step implementation
- `memory-core/src/memory/step_buffer/config.rs` - BatchConfig definition
- `.claude/skills/do-memory-cli-ops/troubleshooting.md` - Already updated

---

## Cross-References

- Tests using `batch_config: None`: `tests/checkpoint_integration_test.rs:18`, `tests/e2e/mcp_tag_chain.rs:26`
- DEPLOYMENT.md batching docs: Lines 105-114, 417-423