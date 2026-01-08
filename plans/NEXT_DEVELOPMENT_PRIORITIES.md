# Next Development Priorities

**Date**: 2026-01-08
**Current Version**: v0.1.12
**Status**: Active Development - File Compliance 62.5% Complete

## Immediate Priorities for Next Release (v0.1.13)

Based on analysis, here are the actionable priorities:

### 🔥 Priority 1: File Size Compliance (P0 - CRITICAL)

**Goal**: Split remaining large files to meet 500 LOC guideline

**Current Status**:
- ✅ 10 of 16 files COMPLETED (62.5% progress)
- Completed splits: sandbox, wasmtime_sandbox, reward, embeddings/mod, spatiotemporal/embeddings, semantic/summary
- ❌ 6 large files remain

**Remaining Files to Split (>500 LOC)**:

1. **memory-storage-turso/src/storage.rs** (2,502 LOC) 🔴 CRITICAL
   - Target: 5 modules (~500 LOC each)
   - Modules: storage/mod.rs, storage/episodes.rs, storage/patterns.rs, storage/embeddings.rs, storage/monitoring.rs
   - Effort: 10-12 hours
   - Impact: Primary database storage

2. **memory-mcp/src/patterns/predictive.rs** (2,435 LOC) 🔴 CRITICAL
   - Target: 5 modules (~485 LOC each)
   - Modules: predictive/mod.rs, predictive/forecasting.rs, predictive/anomaly.rs, predictive/analysis.rs, predictive/tests.rs
   - Effort: 10-12 hours
   - Impact: Pattern prediction features

3. **memory-core/src/memory/mod.rs** (1,530 LOC) 🟡
   - Target: 3 modules (~510 LOC each)
   - Effort: 6-8 hours
   - Impact: Core memory operations

4. **memory-storage-redb/src/storage.rs** (1,514 LOC) 🟡
   - Target: 3 modules (~505 LOC each)
   - Effort: 6-8 hours
   - Impact: Cache storage backend

5. **memory-mcp/src/server.rs** (1,414 LOC) 🟡
   - Target: 3 modules (~471 LOC each)
   - Effort: 6-8 hours
   - Impact: MCP server core

6. **memory-cli/src/commands/episode.rs** (1,201 LOC) 🟡
   - Target: 2-3 modules (~400 LOC each)
   - Effort: 4-6 hours
   - Impact: CLI episode commands

**Total Effort**: 42-54 hours for remaining 6 files

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
- ~340 unwrap/expect calls (target: <50 in production code)
- ~280 clone operations (target: <200)

Tasks:
- Audit and convert unwraps to proper error handling
- Reduce unnecessary clones
- Effort: 20-30 hours (ongoing)

## Recommended Next Sprint

### Sprint: File Size Compliance + Test Fixes
**Duration**: 1.5-2 weeks
**Effort**: 50-66 hours
**Target Release**: v0.1.13

**Sprint Goal**: Achieve full file size compliance and restore test pass rate

**Backlog**:
1. Split remaining 6 large files (42-54 hours)
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

**Recommended Action**

**Start with File Compliance Sprint** 🎯

Rationale:
- 62.5% file compliance already achieved (10/16 files)
- Critical P0 priority per AGENTS.md guidelines
- Clear path forward with well-defined modules
- Remaining files are well-understood
- Can release v0.1.11 (File Compliance) then v0.1.12 (Quick Wins)

Recommended Order:
1. Split remaining 6 files (v0.1.11)
2. Fix tests and run benchmarks (v0.1.12)
3. Release with full compliance

---
*Updated: 2026-01-08 - File compliance progress: 10 of 16 files completed*

---

**What would you like to work on first?**

A. File size compliance (split remaining 6 large files)
B. Fix failing tests
C. Run and document benchmarks
D. Something else - specify
