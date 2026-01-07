# Next Development Priorities

**Date**: 2026-01-06  
**Current Version**: v0.1.12 (released 2026-01-05)  
**Status**: Active Development

## Immediate Priorities for Next Release (v0.1.13)

Based on analysis, here are the actionable priorities:

### 🔥 Priority 1: File Size Compliance (P0 - CRITICAL)

**Goal**: Split remaining large files to meet 500 LOC guideline

**Current Status**:
- ✅ Pre-storage extractor: DONE (split in recent commits)
- ✅ Spatiotemporal retriever: DONE (split in recent commits)
- ❌ Several large files remain

**Top Files to Split (>500 LOC)**:

1. **memory-mcp/src/bin/server.rs** (2,368 LOC) 🔴 CRITICAL
   - Target: 5 modules (~450 LOC each)
   - Modules: server/mod.rs, server/handlers.rs, server/tools.rs, server/lifecycle.rs, server/tests.rs
   - Effort: 8-10 hours
   - Impact: Main MCP server, high visibility

2. **memory-mcp/src/patterns/statistical.rs** (1,132 LOC) 🔴
   - Target: 3 modules (~370 LOC each)
   - Modules: statistical/mod.rs, statistical/analysis.rs, statistical/tests.rs
   - Effort: 4-5 hours
   - Impact: Pattern analysis features

3. **memory-storage-turso/src/lib.rs** (964 LOC) 🟡
   - Target: 2 modules (~480 LOC each)
   - Modules: lib/core.rs, lib/queries.rs
   - Effort: 3-4 hours
   - Impact: Primary storage backend

4. **memory-core/src/memory/retrieval.rs** (891 LOC) 🟡
   - Target: 2 modules (~445 LOC each)
   - Modules: retrieval/mod.rs, retrieval/hierarchical.rs
   - Effort: 3-4 hours
   - Impact: Core memory retrieval

5. **memory-core/src/patterns/optimized_validator.rs** (889 LOC) 🟡
   - Target: 2 modules (~445 LOC each)
   - Modules: validator/mod.rs, validator/optimization.rs
   - Effort: 3-4 hours
   - Impact: Pattern validation

6. **memory-core/src/retrieval/cache.rs** (861 LOC) 🟡
   - Target: 2 modules (~430 LOC each)
   - Modules: cache/mod.rs, cache/lru.rs
   - Effort: 3-4 hours
   - Impact: Query cache performance

**Total Effort**: 24-35 hours for top 6 files

### 🚀 Priority 2: Feature Enhancements (P1 - HIGH VALUE)

Based on roadmap, embeddings are already complete (100%). Focus on:

#### A. Test Infrastructure Improvements (HIGH)
**Problem**: Test pass rate dropped from 99.3% to 76.7%
**Goal**: Restore >95% pass rate

Tasks:
- Investigate failing tests (7 file size compliance tests)
- Fix test infrastructure issues
- Update test assertions for refactored code
- Effort: 8-12 hours

#### B. Performance Benchmarking (MEDIUM)
**Goal**: Validate and document current performance

Tasks:
- Run full benchmark suite
- Document results
- Compare against v0.1.12 baseline
- Update performance metrics in docs
- Effort: 4-6 hours

#### C. Documentation Updates (MEDIUM)
**Goal**: Keep documentation current with recent changes

Tasks:
- Update MCP protocol compliance docs
- Document new refactored modules
- Add migration guides for API changes
- Update architecture diagrams
- Effort: 4-6 hours

### 🔧 Priority 3: Technical Debt (P2 - QUALITY)

#### A. Security - Unmaintained Dependencies (LOW RISK)
**Status**: 5 unmaintained transitive dependencies identified

Action items:
1. Monitor upstream updates (wasmtime, augurs, libsql)
2. Create tracking issues for each dependency
3. Plan migration if needed
4. Effort: 2-3 hours (monitoring setup)

#### B. Code Quality Improvements
From gap analysis:
- 356 unwrap/expect calls (target: <50 in production code)
- 298 clone operations (target: <200)

Tasks:
- Audit and convert unwraps to proper error handling
- Reduce unnecessary clones
- Effort: 20-30 hours (ongoing)

## Recommended Next Sprint

### Sprint: File Size Compliance + Test Fixes
**Duration**: 1-2 weeks  
**Effort**: 32-47 hours  
**Target Release**: v0.1.13

**Sprint Goal**: Achieve full file size compliance and restore test pass rate

**Backlog**:
1. Split top 6 large files (24-35 hours)
2. Fix failing tests (8-12 hours)

**Success Criteria**:
- ✅ All files ≤ 500 LOC
- ✅ Test pass rate >95%
- ✅ 0 clippy warnings
- ✅ All benchmarks passing

**Deliverables**:
- 6+ new modular file structures
- Updated test suite
- Updated documentation

## Alternative: Quick Wins Sprint

If you want faster iteration, focus on smaller tasks:

### Sprint: Quick Improvements
**Duration**: 3-5 days  
**Effort**: 12-18 hours

**Backlog**:
1. Fix failing tests (8-12 hours) ✅ HIGH VALUE
2. Run and document benchmarks (4-6 hours) ✅ VISIBILITY
3. Update documentation (4-6 hours) ✅ USER VALUE

This gets v0.1.13 out faster with lower risk.

## Decision Points

**Choose your path:**

1. **File Compliance Path** (1-2 weeks)
   - Pro: Achieves critical P0 goals
   - Pro: Clean codebase for future work
   - Con: Longer timeline
   - Con: More refactoring risk

2. **Quick Wins Path** (3-5 days)
   - Pro: Fast release cycle
   - Pro: High user value
   - Pro: Low risk
   - Con: Technical debt remains

3. **Hybrid Path** (1 week)
   - Split top 3 files only (12-19 hours)
   - Fix tests (8-12 hours)
   - Quick release, partial compliance

## Recommended Action

**Start with Quick Wins Sprint** 🎯

Rationale:
- v0.1.12 just released (yesterday)
- Quick wins provide immediate value
- Can tackle file compliance in v0.1.14
- Maintains momentum
- Lowers risk

After Quick Wins:
- Release v0.1.13 (tests fixed, benchmarks documented)
- Then start File Compliance Sprint for v0.1.14

---

**What would you like to work on first?**

A. File size compliance (split large files)
B. Fix failing tests
C. Run and document benchmarks
D. Something else - specify
