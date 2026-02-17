# GOAP Phase B1.3: Fix High-Priority unwrap() Calls
**Date**: 2026-02-16
**Status**: 🔄 READY TO START
**Branch**: `feature/v0.1.16-phase-b1.3-high-priority-unwrap-2026-02-16`
**Phase**: B1.3 - Fix High-Priority unwrap() Calls
**Previous**: B1.2 Complete - 19 critical fixes applied

## Progress from B1.2

**B1.1 Audit**: 1,128 total unwrap() calls identified
**B1.2 Fixes**: 19 critical unwrap() calls eliminated
**Current Status**: 1,109 unwrap() calls remaining
**Target**: ≤280 unwrap() calls (50% reduction from original 561)

**Remaining to Fix**: ~829 unwrap() calls (including test code)

## ADR References

**Relevant ADRs**:
- **ADR-022**: GOAP Agent System methodology
- **ADR-030**: Test Optimization and CI Stability Patterns

**Error Handling Pattern** (from ADR-030):
```rust
// Before (panic risk):
let value = result.unwrap();

// After (safe):
let value = result.map_err(|e| anyhow::anyhow!("Context: {}", e))?;
```

## GOAP Decomposition for B1.3

### Goals (Ordered by Priority)

1. **P0-HIGH**: Fix unwrap() in high-risk production code
2. **P1-MEDIUM**: Fix unwrap() in CLI interface
3. **P2-LOW**: Fix unwrap() in memory-core internal modules
4. **P3-LOW**: Fix unwrap() in MCP internal modules

### Actions (Atomic Tasks - Parallel Execution)

#### Task Group 1: High-Risk Production (2-3 Parallel Agents)

**Agent 1: Core Internal Modules Specialist**
- **Files**: memory-core/src/episode/*.rs, memory-core/src/patterns/*.rs
- **unwrap() Count**: ~40 calls
- **Risk**: MEDIUM (core business logic)
- **Actions**:
  - Fix unwrap() in episode graph algorithms
  - Fix unwrap() in pattern extraction
  - Fix unwrap() in semantic search
  - Add proper error context
  - Test after each module

**Agent 2: CLI Interface Specialist**
- **Files**: memory-cli/src/**/*.rs
- **unwrap() Count**: ~20 calls
- **Risk**: MEDIUM (user-facing)
- **Actions**:
  - Fix unwrap() in command parsing
  - Fix unwrap() in error reporting
  - Improve user-facing error messages
  - Test CLI commands

#### Task Group 2: MCP Internal (1-2 Parallel Agents)

**Agent 3: MCP Tools Specialist**
- **Files**: memory-mcp/src/tools/*.rs, memory-mcp/src/patterns/*.rs
- **unwrap() Count**: ~30 calls
- **Risk**: LOW (internal tools, not external interface)
- **Actions**:
  - Fix unwrap() in tool implementations
  - Fix unwrap() in pattern algorithms
  - Add graceful degradation where appropriate
  - Test MCP server

**Agent 4: Storage Layer Specialist**
- **Files**: memory-storage-redb/src/**/*.rs
- **unwrap() Count**: ~15 calls
- **Risk**: LOW (cache layer, has fallback)
- **Actions**:
  - Fix unwrap() in cache operations
  - Fix unwrap() in storage initialization
  - Add proper error context
  - Test storage operations

## Execution Strategy: Parallel Swarm

### Phase 1: Launch 4 Parallel Agents (All Independent)

```
┌─────────────────────────────────────────────────────────────┐
│ Group 1: High-Risk Production (Priority 1-2)                    │
│ ├─ Agent 1: Core Internal Modules (~40 unwrap)   [MEDIUM]      │
│ └─ Agent 2: CLI Interface (~20 unwrap)           [MEDIUM]      │
│                                                              │
│ Group 2: Internal Components (Priority 3)                        │
│ ├─ Agent 3: MCP Tools (~30 unwrap)               [LOW]         │
│ └─ Agent 4: Storage Layer (~15 unwrap)            [LOW]         │
└─────────────────────────────────────────────────────────────┘
```

### Phase 2: Synthesis & Validation

After all agents complete:
- Run quality gates (fmt, clippy, build, test)
- Create atomic commits per file/group
- Push changes
- Update progress in GOAP plan
- Create summary document

## Quality Gates

### Pre-Commit (Per Agent)
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all -- -D warnings`
- [ ] `cargo test -p <affected-crate>`

### Post-Synthesis (All Agents)
- [ ] `cargo test --all`
- [ ] All CI checks passing
- [ ] unwrap() count reduced by target amount

## Success Criteria

### Must Have (Blocking)
- [ ] High-priority unwrap() fixed (~40 calls in core/cli)
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] Total unwrap() ≤ 1,050 (on track for 50% reduction goal)
- [ ] No regressions in functionality

### Metrics Tracking

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| High-priority unwrap() | ~60 | ≤20 | -67% |
| CLI unwrap() | ~20 | ≤5 | -75% |
| Total unwrap() | 1,109 | ≤1,050 | -59 calls |
| Clippy warnings | 0 | 0 | 0 |

## Atomic Commit Strategy

```
[fix] B1.3.1: Fix unwrap() in core internal modules (episode, patterns, search)
[fix] B1.3.2: Fix unwrap() in CLI interface (commands, errors, main)
[fix] B1.3.3: Fix unwrap() in MCP tools and patterns (internal modules)
[fix] B1.3.4: Fix unwrap() in storage layer (redb cache operations)
[style] Apply rustfmt formatting changes
[docs] Update B1.3 execution summary
```

## Progress Tracking

| Task | Agent | Status | unwrap() Fixed | Notes |
|------|-------|--------|----------------|-------|
| B1.3.1 | Core Internal | 🟡 Pending | 0/~40 | Launching |
| B1.3.2 | CLI Interface | 🟡 Pending | 0/~20 | Launching |
| B1.3.3 | MCP Tools | 🟡 Pending | 0/~30 | Launching |
| B1.3.4 | Storage Layer | 🟡 Pending | 0/~15 | Launching |

## Execution Log

### 2026-02-16 B1.3 Planning Complete
- B1.2 complete: 19 critical fixes applied
- Identified remaining ~105 high-priority unwrap() calls
- Created parallel execution strategy (4 agents)
- ADR-030 patterns for error handling documented
- Branch created: `feature/v0.1.16-phase-b1.3-high-priority-unwrap-2026-02-16`
- Ready to launch specialist agents

## Next Steps

1. ✅ **Launch 4 parallel specialist agents** (Groups 1 & 2)
2. ⏳ **Monitor agent progress** (each agent: 30m-2h)
3. ⏳ **Synthesize results** (quality gates, atomic commits)
4. ⏳ **Validate overall improvements** (metrics, tests)
5. ⏳ **Proceed to B1.4**: Verify and validate
6. ⏳ **Then B2**: Test triage

---

**Status**: 🔄 **READY TO LAUNCH PARALLEL AGENTS**
**Strategy**: 4 specialists working in parallel (Groups 1 & 2)
**Estimated Time**: 30m-2h per agent (total: 2-6h parallel execution)
**Target**: Reduce unwrap() by ~59 calls to achieve ≤1,050 total
