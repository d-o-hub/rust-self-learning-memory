# GOAP Phase B1.2: Fix Critical unwrap() Calls
**Date**: 2026-02-16
**Status**: 🔄 IN PROGRESS
**Branch**: `feature/v0.1.16-phase-b-code-quality-2026-02-16`
**Phase**: B1.2 - Fix Critical unwrap() Calls
**Previous**: B1.1 Audit Complete (5 critical fixes applied)

## Progress from B1.1

**Audit Results**:
- Total unwrap() calls: 1,128
- Critical fixes already applied: 5 (tracing module)
- Remaining critical unwrap(): ~50 calls across 4 priority files

## Top Priority Files (From B1.1 Audit)

| Priority | File | unwrap() Count | Risk Level | Est. Time |
|----------|------|-----------------|------------|-----------|
| 1 | memory-core/src/embeddings/local.rs | 25 | CRITICAL | 1-2h |
| 2 | memory-core/src/search/regex.rs | 12 | HIGH | 30-60m |
| 3 | memory-storage-turso/src/lib.rs | ~50 | CRITICAL | 2-3h |
| 4 | memory-mcp/src/bin/server.rs | ~30 | HIGH | 1-2h |

## ADR References

**Relevant ADRs**:
- **ADR-022**: GOAP Agent System methodology
- **ADR-030**: Test optimization patterns

**Error Handling Pattern** (from ADR-030):
```rust
// Before (panic risk):
let value = result.unwrap();

// After (safe):
let value = result.map_err(|e| anyhow::anyhow!("Context: {}", e))?;
```

## GOAP Decomposition for B1.2

### Goals (Ordered by Priority)

1. **P0-CRITICAL**: Fix unwrap() in embeddings/local.rs (25 calls)
2. **P0-CRITICAL**: Fix unwrap() in storage-turso/lib.rs (~50 calls)
3. **P1-HIGH**: Fix unwrap() in search/regex.rs (12 calls)
4. **P1-HIGH**: Fix unwrap() in mcp/bin/server.rs (~30 calls)

### Actions (Atomic Tasks - Parallel Execution)

#### Task Group 1: Core Infrastructure (2 Parallel Agents)

**Agent 1: Local Embeddings Specialist** (Priority 1)
- File: `memory-core/src/embeddings/local.rs`
- unwrap() count: 25
- Risk: CRITICAL (file I/O, external input)
- Actions:
  - Replace unwrap() on file operations
  - Add proper error context
  - Handle missing embedding files gracefully
  - Test with various embedding providers

**Agent 2: Database Operations Specialist** (Priority 2)
- File: `memory-storage-turso/src/lib.rs`
- unwrap() count: ~50
- Risk: CRITICAL (database operations)
- Actions:
  - Replace unwrap() on DB connections
  - Fix query execution error handling
  - Handle transaction errors properly
  - Test with various DB states

#### Task Group 2: User Interface (2 Parallel Agents)

**Agent 3: Regex Search Specialist** (Priority 3)
- File: `memory-core/src/search/regex.rs`
- unwrap() count: 12
- Risk: HIGH (user input parsing)
- Actions:
  - Fix regex compilation error handling
  - Handle invalid user input gracefully
  - Add clear error messages
  - Test with edge cases

**Agent 4: MCP Server Specialist** (Priority 4)
- File: `memory-mcp/src/bin/server.rs`
- unwrap() count: ~30
- Risk: HIGH (external interface)
- Actions:
  - Fix MCP protocol error handling
  - Handle client disconnects gracefully
  - Improve server initialization
  - Test error recovery

## Execution Strategy: Parallel Swarm

### Phase 1: Launch 4 Parallel Agents (All Independent)

```
┌─────────────────────────────────────────────────────────────┐
│ Group 1: Core Infrastructure (Priority 1-2)                │
│ ├─ Agent 1: Local Embeddings (25 unwrap)  [CRITICAL]      │
│ └─ Agent 2: Database Operations (50 unwrap) [CRITICAL]     │
│                                                              │
│ Group 2: User Interface (Priority 3-4)                      │
│ ├─ Agent 3: Regex Search (12 unwrap)     [HIGH]           │
│ └─ Agent 4: MCP Server (30 unwrap)       [HIGH]           │
└─────────────────────────────────────────────────────────────┘
```

### Phase 2: Synthesis & Validation

After all agents complete:
- Run quality gates (fmt, clippy, build, test)
- Create atomic commits per file
- Push changes
- Update progress in GOAP plan

## Quality Gates

### Pre-Commit (Per Agent)
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all -- -D warnings`
- [ ] `cargo test -p <crate>`

### Post-Synthesis (All Agents)
- [ ] `cargo test --all`
- [ ] All CI checks passing
- [ ] Zero unwrap() in critical paths

## Success Criteria

### Must Have (Blocking)
- [ ] Priority 1-4 files fixed (all critical unwrap() replaced)
- [ ] ≤75 unwrap() calls eliminated from production code
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] No performance regression

### Metrics Tracking

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| Critical unwrap() | ~55 | ≤10 | -80% |
| High unwrap() | ~42 | ≤20 | -50% |
| Total unwrap() | 1,128 | ≤1,050 | -75 calls |

## Atomic Commit Strategy

```
[fix] B1.2.1: Fix unwrap() in local embeddings (25 calls)
[fix] B1.2.2: Fix unwrap() in database operations (50 calls)
[fix] B1.2.3: Fix unwrap() in regex search (12 calls)
[fix] B1.2.4: Fix unwrap() in MCP server (30 calls)
[style] Apply rustfmt formatting changes
[docs] Update B1.2 execution summary
```

## Progress Tracking

| Task | Agent | Status | unwrap() Fixed | Notes |
|------|-------|--------|----------------|-------|
| B1.2.1 | Local Embeddings | 🟡 Pending | 0/25 | Launching |
| B1.2.2 | Database Ops | 🟡 Pending | 0/50 | Launching |
| B1.2.3 | Regex Search | 🟡 Pending | 0/12 | Launching |
| B1.2.4 | MCP Server | 🟡 Pending | 0/30 | Launching |

## Execution Log

### 2026-02-16 B1.2 Planning Complete
- B1.1 audit complete (5 fixes applied)
- Prioritized 4 files with ~117 unwrap() calls
- Created parallel execution strategy (4 agents)
- ADR-030 patterns for error handling documented
- Ready to launch specialist agents

## Next Steps

1. ✅ **Launch 4 parallel specialist agents** (Group 1 & 2)
2. ⏳ **Monitor agent progress** (each agent: 30m-3h)
3. ⏳ **Synthesize results** (quality gates, atomic commits)
4. ⏳ **Validate overall improvements** (metrics, tests)
5. ⏳ **Proceed to B1.3**: Fix remaining high-priority calls

---

**Status**: 🔄 **READY TO LAUNCH PARALLEL AGENTS**
**Strategy**: 4 specialists working in parallel (Groups 1 & 2)
**Estimated Time**: 30m-3h per agent (total: 2-8h parallel execution)
