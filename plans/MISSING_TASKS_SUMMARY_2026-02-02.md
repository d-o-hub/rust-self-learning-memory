# Missing Tasks Summary - February 2, 2026

**Status**: Consolidated from plans folder analysis  
**Total Tasks Identified**: 47 missing implementations  
**Current Priority**: Focus on quick wins and P0 blockers  

---

## Executive Summary

Based on comprehensive analysis of the plans folder, the following critical gaps exist:

### Immediate Blockers (Can Complete Today)

#### 1. Example File Updates (15 minutes)
- **Status**: REMAINING_WORK_STATUS.md indicates ModelConfig → ProviderConfig migration incomplete
- **Files**: `memory-core/examples/embedding_optimization_demo.rs`
- **Action**: Update any remaining ModelConfig references to ProviderConfig
- **Priority**: P2 (Low impact, quick fix)

#### 2. Memory-MCP Compilation Issues (15 minutes)
- **Status**: REMAINING_WORK_STATUS.md mentions compilation errors
- **Location**: `memory-mcp/src/mcp/tools/embeddings/tool/execute.rs`
- **Action**: Fix provider_config field access and json_value_len helper
- **Priority**: P1 (Blocks MCP compilation)

#### 3. Batch Module TODOs (30 minutes)
- **Location**: `memory-mcp/src/server/tools/mod.rs:7`
- **Issue**: "TODO: Fix batch module - uses non-existent jsonrpsee and ServerState"
- **Action**: Remove or fix broken batch module reference
- **Priority**: P1 (Compilation warning)

---

## Critical Missing Features (P0 - Weeks 1-2)

### Episode Relationships - MCP Tools (41-56 hours)
**Status**: Documented in EPISODE_RELATIONSHIPS_PHASE4_5_PLAN.md but NOT IMPLEMENTED

All 8 tools missing:
1. `add_episode_relationship` (6-8h)
2. `remove_episode_relationship` (4-6h)
3. `get_episode_relationships` (4-6h)
4. `find_related_episodes` (6-8h)
5. `check_relationship_exists` (3-4h)
6. `get_dependency_graph` (8-10h)
7. `validate_no_cycles` (4-6h)
8. `get_topological_order` (6-8h)

**Impact**: Episode relationships unusable via MCP interface  
**Location**: Should be in `memory-mcp/src/mcp/tools/episode_relationships/`  
**Note**: Directory exists but tools not implemented

---

### Episode Relationships - CLI Commands (20-30 hours)
**Status**: Missing CLI interface for relationship management

Missing commands:
1. `relationship add` - Add relationship
2. `relationship remove` - Remove relationship
3. `relationship list` - List relationships
4. `relationship graph` - Visualize dependency graph
5. `relationship validate` - Check for cycles
6. `relationship find` - Find related episodes
7. `relationship info` - Get relationship details

**Impact**: No user-facing interface for relationships  
**Location**: Should be in `memory-cli/src/commands/`

---

### Episode Tags - CLI Commands (15-20 hours)
**Status**: Backend implemented, CLI missing

Missing commands:
1. `tag add` - Add tag to episode
2. `tag remove` - Remove tag
3. `tag list` - List tags
4. `tag search` - Search by tags
5. `tag rename` - Rename tag
6. `tag stats` - Tag usage statistics

**Impact**: Tags only accessible via MCP, not CLI  
**Location**: Should be in `memory-cli/src/commands/tags.rs`

---

### Security - Rate Limiting (2-3 days)
**Status**: NOT IMPLEMENTED - DoS vulnerability

**Risk**: Unprotected API endpoints can be overwhelmed  
**Location**: `memory-mcp/src/server/rate_limiter.rs` exists but not integrated  
**Action**: Wire up rate limiting to all MCP tool endpoints

---

### Security - Audit Logging (2-3 days)
**Status**: PARTIAL - audit module exists but incomplete

**Location**: `memory-mcp/src/server/audit/` - 9 files exist  
**Issue**: Not fully integrated with all operations  
**Action**: Complete audit logging for all security-sensitive operations

---

### Testing - 79 Ignored Tests (Variable time)
**Status**: CRITICAL - Significant coverage disabled

Breakdown:
- 35 "slow integration test" - Need CI optimization
- 8 "Flaky in CI" - Sandbox timing issues
- 10 "Slow test - complete_episode with pattern extraction"
- 6 WASI/WASM implementation gaps
- 4 Changepoint detection non-determinism
- 4 Test isolation issues with env vars
- 2 Temporarily disabled (PerformanceMetrics visibility)

**Action**: Systematically review and fix or remove each ignored test  
**Priority**: P0 for test quality

---

## High-Value Features (P1 - Weeks 3-5)

### 1. Episode Relationships Phase 3 - Memory Layer (2 days)
**Status**: 838-line implementation plan exists but NOT EXECUTED  
**Plan**: `plans/EPISODE_RELATIONSHIPS_PHASE3_PLAN.md`  
**Impact**: Relationships not integrated with memory retrieval

### 2. Connection-Aware Prepared Statement Cache (2-3 days)
**Status**: DISABLED due to connection issues  
**Location**: `memory-storage-turso/src/lib_impls/helpers.rs:77`  
**Impact**: -35% query performance  
**Fix**: Make prepared statements connection-aware

### 3. Batch Operations for Patterns/Heuristics (4 days)
**Status**: Partial for episodes, missing for patterns and heuristics  
**Impact**: -80% throughput for bulk operations

### 4. Property-Based Testing (3-5 days)
**Status**: Only CLI has generators, core library missing  
**Impact**: Reduced confidence in edge case handling

---

## Medium Priority (P2 - Weeks 6-8)

### 1. Keep-Alive Connection Pool (Already Implemented!)
**Status**: ✅ COMPLETE but behind feature flag  
**Location**: `memory-storage-turso/src/pool/keepalive_pool.rs`  
**Action**: Enable by default and document usage  
**Effort**: 4-6 hours (testing + docs)

### 2. Adaptive Pool Sizing (Already Implemented!)
**Status**: ✅ COMPLETE but connection exposure issue  
**Location**: `memory-storage-turso/src/pool/adaptive.rs:356`  
**Issue**: Cannot access pooled connection from public API  
**Effort**: 1 day

### 3. Compression Integration (Already Implemented!)
**Status**: ✅ Code exists but not integrated  
**Location**: `memory-storage-turso/src/compression/`  
**Action**: Wire up compression for large payloads  
**Effort**: 2-3 days

### 4. Native Vector Search (Fallback mode)
**Status**: Using brute-force fallback instead of native  
**Impact**: O(n) instead of O(log n) for vector search  
**Effort**: 3-5 days

---

## Quick Wins (Can Complete in 1-2 Hours)

1. ✅ **Update example files** - ModelConfig → ProviderConfig (15 min)
2. ✅ **Fix memory-mcp compilation** - Provider config access (15 min)
3. ✅ **Fix batch module TODO** - Clean up reference (30 min)
4. **Enable keep-alive pool** - Remove feature flag (2 hours)
5. **Document ProviderConfig migration** - API docs (2 hours)

---

## Recommended Action Plan

### Today (February 2, 2026) - Quick Wins
- [ ] Fix example files (15 min)
- [ ] Fix memory-mcp compilation (15 min)
- [ ] Fix batch module TODO (30 min)
- [ ] Run workspace validation (30 min)
- [ ] Create ProviderConfig migration guide (2 hours)

**Total**: 4 hours of high-value cleanup

### Week 1 - P0 Blockers
- [ ] Implement 8 MCP relationship tools (3 days)
- [ ] Implement 7 CLI relationship commands (2 days)

### Week 2 - P0 Continued
- [ ] Implement 6 CLI tag commands (2 days)
- [ ] Start fixing ignored tests (ongoing)
- [ ] Implement rate limiting (2 days)

### Weeks 3-5 - P1 Features
- [ ] Episode Relationships Phase 3 integration
- [ ] Connection-aware prepared statement cache
- [ ] Batch operations completion
- [ ] Audit logging completion

---

## Files That Need Attention

### Immediate (Today)
1. `memory-core/examples/embedding_optimization_demo.rs` - Check for ModelConfig
2. `memory-mcp/src/mcp/tools/embeddings/tool/execute.rs` - Fix compilation
3. `memory-mcp/src/server/tools/mod.rs` - Fix batch module TODO

### This Week
1. `memory-mcp/src/mcp/tools/episode_relationships/tool.rs` - Add 8 tools
2. `memory-cli/src/commands/relationships.rs` - Create CLI commands
3. `memory-cli/src/commands/tags.rs` - Create CLI commands
4. `memory-mcp/src/server/rate_limiter.rs` - Enable rate limiting

### Next 2-4 Weeks
1. `memory-core/src/episodic/memory_layer.rs` - Phase 3 integration
2. `memory-storage-turso/src/lib_impls/helpers.rs` - Fix prepared cache
3. `memory-storage-turso/src/storage/batch_operations.rs` - Complete batch ops

---

## Verification Checklist

After completing immediate tasks:

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes (or note ignored tests)
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] All examples compile and run
- [ ] Documentation updated
- [ ] Plans folder updated with completion status

---

**Next Steps**: Start with the 4-hour quick wins, then move to P0 blockers.

**Last Updated**: 2026-02-02  
**Maintainer**: Rovo Dev Agent
