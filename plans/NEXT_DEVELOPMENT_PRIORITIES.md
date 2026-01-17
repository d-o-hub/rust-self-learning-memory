# Next Development Priorities

**Date**: 2026-01-09
**Current Version**: v0.1.13 (In Development)
**Status**: Active Development - File Compliance In Progress

## Immediate Priorities for Next Release (v0.1.13)

Based on current analysis, here are the actionable priorities:

### 🔥 Priority 1: File Size Compliance (P0 - CRITICAL)

**Goal**: Split remaining large files to meet 500 LOC guideline

**Current Status**:
- ✅ 10+ files successfully split in early January 2026
- ❌ 20+ large files remain exceeding 500 LOC

**Remaining Files to Split (>500 LOC)**:

#### P0 - Critical (Start Immediately)

**memory-mcp (Server files)**:
1. `src/wasm_sandbox.rs` (683 LOC) - Split into runtime/instance/modules
2. `src/javy_compiler.rs` (679 LOC) - Extract compiler phases
3. `src/unified_sandbox.rs` (533 LOC) - Split handler/implementation

**memory-storage (Storage files)**:
4. `memory-storage-redb/src/cache.rs` (654 LOC) - Split cache operations
5. `memory-storage-turso/src/pool.rs` (589 LOC) - Extract pool management

#### P1 - High (Next Sprint)

**memory-core (Core files)**:
6. `src/patterns/clustering.rs` (673 LOC)
7. `src/memory/learning.rs` (673 LOC)
8. `src/embeddings/openai.rs` (672 LOC)
9. `src/pre_storage/quality.rs` (666 LOC)
10. `src/learning/queue.rs` (662 LOC)
11. `src/embeddings/config.rs` (660 LOC)
12. `src/episode.rs` (649 LOC)

#### P2 - Medium (Future Sprints)

**memory-core (Continued)**:
13. `src/embeddings/real_model.rs` (634 LOC)
14. `src/patterns/effectiveness.rs` (631 LOC)
15. `src/patterns/validation.rs` (623 LOC)
16. `src/episodic/capacity.rs` (613 LOC)
17. `src/monitoring/storage.rs` (598 LOC)

**memory-cli (CLI files)**:
18. `src/config/validator.rs` (636 LOC)
19. `src/config/loader.rs` (623 LOC)
20. `src/config/progressive.rs` (564 LOC)

#### P3 - Low (Cleanup)

21. `src/sync.rs` (511 LOC) - Just over limit
22. `src/reward/adaptive.rs` (510 LOC) - Just over limit

**Total Effort**: 80-120 hours for full compliance

### 🚀 Priority 2: Code Quality (P1 - HIGH VALUE)

Based on roadmap, embeddings are already complete (100%). Focus on:

#### A. Error Handling Audit (HIGH)
**Problem**: ~340 unwrap/expect calls in production code
**Goal**: Reduce to <50 with proper error handling

Tasks:
- [ ] Audit all unwrap/expect locations
- [ ] Convert hot path unwraps to proper error handling
- [ ] Add context to error messages
- [ ] Ensure all error paths are tested
- Effort: 20-30 hours

#### B. Clone Reduction (MEDIUM)
**Problem**: ~280 clone operations, some unnecessary
**Goal**: Reduce to <200 with Arc/Cow/references

Tasks:
- [ ] Profile clone hotspots
- [ ] Convert episode/pattern clones to Arc
- [ ] Use Cow for conditional cloning
- Effort: 15-25 hours

#### C. Test Infrastructure (MEDIUM)
**Problem**: Test pass rate dropped from 99.3% to ~85%
**Goal**: Restore >95% pass rate

Tasks:
- [ ] Identify and fix failing tests
- [ ] Update test assertions for refactored code
- [ ] Add integration tests for new modules
- Effort: 10-15 hours

### 🔧 Priority 3: Technical Debt (P2 - QUALITY)

#### A. Dependency Cleanup (LOW RISK)
**Status**: 5+ duplicate dependencies identified

Action items:
- [ ] Analyze dependency tree
- [ ] Consolidate duplicate versions
- [ ] Remove unused dependencies
- [ ] Optimize feature flags
- Effort: 10-15 hours

#### B. Documentation Updates (MEDIUM)
**Goal**: Keep documentation current with recent changes

Tasks:
- [ ] Update MCP protocol compliance docs
- [ ] Document new refactored modules
- [ ] Update architecture diagrams
- Effort: 5-10 hours

## Recommended Next Sprint

### Sprint: P0 File Compliance
**Duration**: 1-2 weeks
**Effort**: 25-35 hours (P0 files only)
**Target Release**: v0.1.13

**Sprint Goal**: Achieve 80% file size compliance

**Backlog**:
1. Split memory-mcp sandbox files (15-20 hours)
2. Split memory-storage files (10-15 hours)

**Success Criteria**:
- ✅ All P0 files ≤ 500 LOC
- ✅ Test pass rate >90%
- ✅ 0 clippy warnings
- ✅ All tests passing

**Deliverables**:
- 5+ new modular file structures
- Updated test suite
- Updated documentation

## Alternative: Quick Wins Sprint

If you want faster iteration, focus on smaller tasks:

### Sprint: Quick Improvements
**Duration**: 3-5 days
**Effort**: 15-20 hours

**Backlog**:
1. Fix failing tests (8-12 hours) ✅ HIGH VALUE
2. Update documentation (4-6 hours) ✅ USER VALUE
3. Run and document benchmarks (3-5 hours) ✅ VISIBILITY

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
   - Split P0 files only (25-35 hours)
   - Fix tests (10-15 hours)
   - Quick release, partial compliance

**Recommended Action**

**Start with P0 File Compliance Sprint** 🎯

Rationale:
- 10+ files already successfully split (proven pattern)
- Critical P0 priority per AGENTS.md guidelines
- Clear path forward with well-defined modules
- Remaining files are well-understood
- Can release v0.1.13 with significant progress

Recommended Order:
1. Split P0 memory-mcp files (wasm_sandbox, javy_compiler, unified_sandbox)
2. Split P0 storage files (cache, pool)
3. Release v0.1.13
4. Continue with P1 files in v0.1.14

---

## Files Status Summary

| Category | Count | Total LOC | Status |
|----------|-------|-----------|--------|
| Compliant (≤500 LOC) | ~180 | ~40,000 | ✅ Good |
| P0 - Critical | 5 | ~3,138 | 🔴 Needs work |
| P1 - High | 7 | ~4,550 | 🟡 Next sprint |
| P2 - Medium | 8 | ~4,780 | 🟢 Future |
| P3 - Low | 2 | ~1,021 | ⚪ Cleanup |

**Total LOC**: ~53,489 across ~200+ files
**Compliance Rate**: ~90% (files) | ~75% (LOC)

---

**What would you like to work on first?**

A. File size compliance (P0 files - memory-mcp sandbox + storage)
B. Error handling audit (unwrap → proper errors)
C. Clone reduction (Arc/Cow optimization)
D. Fix failing tests (restore pass rate)
E. Something else - specify

---

*Updated: 2026-01-09 - File compliance in progress, 20+ files remain*
