# GOAP v0.1.16 Phase B Execution Plan

**Generated**: 2026-02-17
**Episode**: GOAP_V0.1.16_PHASE_B_EPISODE
**Status**: 🔄 READY TO EXECUTE
**Estimated Duration**: 2 weeks (15-23 hours)

---

## 1. Task Registry

### Phase B.1: Baseline Verification (Sequential, 1-2h)

| Task ID | Description | Priority | Effort | Dependencies | Agent |
|---------|-------------|----------|--------|--------------|-------|
| B.1.1 | Count unwrap/expect calls per crate | P0 | 30m | None | code-reviewer |
| B.1.2 | Count #[ignore] tests with categorization | P0 | 30m | None | test-runner |
| B.1.3 | Count #[allow(dead_code)] annotations | P0 | 30m | None | refactorer |
| B.1.4 | Document verified baseline metrics | P0 | 30m | B.1.1, B.1.2, B.1.3 | GOAP |

### Phase B.2: Quick Wins (Parallel, 7-11h)

| Task ID | Description | Priority | Effort | Dependencies | Agent |
|---------|-------------|----------|--------|--------------|-------|
| B.2.1 | Audit all #[ignore] tests | P1 | 1-2h | B.1.2 | test-runner |
| B.2.2 | Categorize ignored tests | P1 | 1h | B.2.1 | test-runner |
| B.2.3 | Fix transient test failures | P1 | 1-2h | B.2.2 | test-runner |
| B.2.4 | Document intentionally ignored tests | P1 | 1h | B.2.3 | test-runner |
| B.2.5 | Audit all #[allow(dead_code)] | P1 | 1-2h | B.1.3 | refactorer |
| B.2.6 | Remove truly dead code | P1 | 1-2h | B.2.5 | refactorer |
| B.2.7 | Replace with #[cfg(feature)] | P1 | 1h | B.2.6 | refactorer |

### Phase B.3: Error Handling Audit (Sequential per crate, 8-12h)

| Task ID | Description | Priority | Effort | Dependencies | Agent |
|---------|-------------|----------|--------|--------------|-------|
| B.3.1-CORE | Audit memory-core unwrap/expect | P1 | 30m | B.1.1 | refactorer |
| B.3.2-CORE | Introduce error types in memory-core | P1 | 1-2h | B.3.1-CORE | feature-implementer |
| B.3.3-CORE | Replace unwraps in memory-core | P1 | 1-2h | B.3.2-CORE | refactorer |
| B.3.4-CORE | Add error path tests for memory-core | P1 | 30m | B.3.3-CORE | test-runner |
| B.3.5-CORE | Validate memory-core | P1 | 30m | B.3.4-CORE | code-reviewer |
| B.3.6-TURSO | Audit memory-storage-turso unwrap/expect | P1 | 30m | B.3.5-CORE | refactorer |
| B.3.7-TURSO | Introduce error types in Turso | P1 | 1-2h | B.3.6-TURSO | feature-implementer |
| B.3.8-TURSO | Replace unwraps in Turso | P1 | 1-2h | B.3.7-TURSO | refactorer |
| B.3.9-TURSO | Add error path tests for Turso | P1 | 30m | B.3.8-TURSO | test-runner |
| B.3.10-TURSO | Validate Turso | P1 | 30m | B.3.9-TURSO | code-reviewer |
| B.3.11-REDB | Audit memory-storage-redb unwrap/expect | P1 | 30m | B.3.10-TURSO | refactorer |
| B.3.12-REDB | Introduce error types in redb | P1 | 1-2h | B.3.11-REDB | feature-implementer |
| B.3.13-REDB | Replace unwraps in redb | P1 | 1-2h | B.3.12-REDB | refactorer |
| B.3.14-REDB | Add error path tests for redb | P1 | 30m | B.3.13-REDB | test-runner |
| B.3.15-REDB | Validate redb | P1 | 30m | B.3.14-REDB | code-reviewer |
| B.3.16-MCP | Audit memory-mcp unwrap/expect | P1 | 30m | B.3.15-REDB | refactorer |
| B.3.17-MCP | Introduce error types in MCP | P1 | 1h | B.3.16-MCP | feature-implementer |
| B.3.18-MCP | Replace unwraps in MCP | P1 | 1h | B.3.17-MCP | refactorer |
| B.3.19-MCP | Add error path tests for MCP | P1 | 30m | B.3.18-MCP | test-runner |
| B.3.20-MCP | Validate MCP | P1 | 30m | B.3.19-MCP | code-reviewer |
| B.3.21-CLI | Audit memory-cli unwrap/expect | P1 | 30m | B.3.20-MCP | refactorer |
| B.3.22-CLI | Introduce error types in CLI | P1 | 1h | B.3.21-CLI | feature-implementer |
| B.3.23-CLI | Replace unwraps in CLI | P1 | 1h | B.3.22-CLI | refactorer |
| B.3.24-CLI | Add error path tests for CLI | P1 | 30m | B.3.23-CLI | test-runner |
| B.3.25-CLI | Validate CLI | P1 | 30m | B.3.24-CLI | code-reviewer |

### Phase B.4: Final Validation (Sequential, 1-2h)

| Task ID | Description | Priority | Effort | Dependencies | Agent |
|---------|-------------|----------|--------|--------------|-------|
| B.4.1 | Run full quality gate suite | P0 | 30m | All B.3 tasks | test-runner |
| B.4.2 | Verify all metrics meet targets | P0 | 30m | B.4.1 | code-reviewer |
| B.4.3 | Create atomic git commits | P0 | 30m | B.4.2 | GOAP |
| B.4.4 | Generate completion report | P0 | 30m | B.4.3 | GOAP |

---

## 2. Dependency Graph

```
B.1.1 ──┐
B.1.2 ──┼──→ B.1.4 (Baseline Verified)
B.1.3 ──┘

B.1.4 ──→ B.2.1 ──→ B.2.2 ──→ B.2.3 ──→ B.2.4 (B2 Complete)
     │
     └──→ B.2.5 ──→ B.2.6 ──→ B.2.7 (B3 Complete)
     │
     └──→ B.3.1-CORE ──→ B.3.2-CORE ──→ B.3.3-CORE ──→ B.3.4-CORE ──→ B.3.5-CORE
                                      │
                                      ↓ (sequential per crate)
          B.3.6-TURSO → ... → B.3.10-TURSO
                                      │
                                      ↓
          B.3.11-REDB → ... → B.3.15-REDB
                                      │
                                      ↓
          B.3.16-MCP → ... → B.3.20-MCP
                                      │
                                      ↓
          B.3.21-CLI → ... → B.3.25-CLI
                                      │
                                      ↓
                                  B.4.1 ──→ B.4.2 ──→ B.4.3 ──→ B.4.4
```

---

## 3. Execution Strategy

### Week 1: Baseline + Quick Wins (Day 1-3)

**Day 1 (Morning)**: Phase B.1 - Baseline Verification
```
GOAP: Launch B.1.1, B.1.2, B.1.3 in parallel
      → code-reviewer (unwrap count)
      → test-runner (ignored tests)
      → refactorer (dead_code)

GOAP: Wait for completion, aggregate results
      → B.1.4: Document verified baseline
```

**Day 1 (Afternoon) - Day 2**: Phase B.2 - Quick Wins (Parallel)
```
GOAP: Launch B.2.1-B.2.4 (test-runner) and B.2.5-B.2.7 (refactorer) in parallel
      → test-runner: Fix ignored tests
      → refactorer: Remove dead_code

GOAP: Monitor progress, collect results
      → B.2 complete: ≤10 ignored tests, ≤40 dead_code annotations
```

**Day 3**: Phase B.3 Start - memory-core
```
GOAP: Execute B.3.1-CORE through B.3.5-CORE sequentially
      → refactorer: Audit unwraps
      → feature-implementer: Add error types
      → refactorer: Replace unwraps
      → test-runner: Add error path tests
      → code-reviewer: Validate

GOAP: Create atomic commit for memory-core
      → git commit -m "refactor(core): improve error handling"
```

### Week 2: Complete B1 + Validation (Day 4-7)

**Day 4-5**: Phase B.3 - Storage Crates (Turso + redb)
```
GOAP: Execute B.3.6-TURSO through B.3.10-TURSO sequentially
      → [Same pattern as memory-core]

GOAP: Create atomic commit for Turso
      → git commit -m "refactor(turso): improve error handling"

GOAP: Execute B.3.11-REDB through B.3.15-REDB sequentially
      → [Same pattern as memory-core]

GOAP: Create atomic commit for redb
      → git commit -m "refactor(redb): improve error handling"
```

**Day 6**: Phase B.3 - MCP + CLI
```
GOAP: Execute B.3.16-MCP through B.3.20-MCP sequentially
      → [Same pattern as memory-core]

GOAP: Create atomic commit for MCP
      → git commit -m "refactor(mcp): improve error handling"

GOAP: Execute B.3.21-CLI through B.3.25-CLI sequentially
      → [Same pattern as memory-core]

GOAP: Create atomic commit for CLI
      → git commit -m "refactor(cli): improve error handling"
```

**Day 7**: Phase B.4 - Final Validation
```
GOAP: Execute B.4.1 through B.4.4 sequentially
      → test-runner: Full quality gate suite
      → code-reviewer: Verify metrics
      → GOAP: Atomic commits
      → GOAP: Completion report

GOAP: Episode complete
      → Calculate episode score
      → Extract patterns for learning
      → Update ROADMAP_ACTIVE.md
```

---

## 4. Quality Gates

### Per-Crate Quality Gates (after B.3.x-CRATE)
```bash
# Format check
cargo fmt --all -- --check

# Linter check (zero warnings)
cargo clippy --all -- -D warnings

# Crate-specific tests
cargo test -p <crate>

# Verify no new warnings introduced
cargo clippy -p <crate> -- -D warnings
```

### Phase B Complete Quality Gates
```bash
# Full quality gate suite
./scripts/quality-gates.sh

# All tests passing
cargo test --all

# Zero clippy warnings
cargo clippy --all -- -D warnings

# Verify metrics
cargo clippy --all 2>&1 | grep -c "unwrap()"    # Should be ≤280
cargo test --all --list 2>&1 | grep "ignored"   # Should be ≤10
cargo clippy --all 2>&1 | grep "dead_code"      # Should be ≤40
```

---

## 5. Atomic Commit Strategy

### Commit Granularity
One commit per completed task group:
- B.2.1-B.2.4: Single commit for test triage
- B.2.5-B.2.7: Single commit for dead_code cleanup
- B.3.1-B.3.5-CORE: Single commit for memory-core
- B.3.6-B.3.10-TURSO: Single commit for Turso
- B.3.11-B.3.15-REDB: Single commit for redb
- B.3.16-B.3.20-MCP: Single commit for MCP
- B.3.21-B.3.25-CLI: Single commit for CLI

### Commit Message Format
```
[module] description

- Change 1
- Change 2
- Change 3

Refs: #episode GOAP_V0.1.16_PHASE_B
```

### Example Commits
```
[test] triage and fix ignored tests

- Categorized 63 #[ignore] tests
- Fixed 37 transient test failures
- Documented 26 intentionally ignored tests
- Reduced ignored test count to ≤10

Refs: #episode GOAP_V0.1.16_PHASE_B

[refactor(core)] improve error handling

- Introduced MemoryCoreError enum with thiserror
- Replaced 87 unwrap() calls with proper error propagation
- Added 23 error path unit tests
- Reduced unwrap count from 561 to 474

Refs: #episode GOAP_V0.1.16_PHASE_B
```

---

## 6. Success Criteria

### Phase B Complete When:
- [ ] B.1: Baseline metrics verified and documented
- [ ] B.2: ≤10 #[ignore] tests (down from 3-63)
- [ ] B.3: ≤40 #[allow(dead_code)] annotations (down from 168-951)
- [ ] B.4: ≤280 unwrap() calls (down from 561, 50% reduction)
- [ ] All quality gates passing
- [ ] Atomic commits created for each task group
- [ ] ROADMAP_ACTIVE.md updated with progress
- [ ] Episode score calculated and patterns extracted

### Episode Scoring (100pts total)
- **Goal Achievement** (30pts): All targets met
- **Efficiency** (20pts): Parallel execution utilized, optimal agent usage
- **Quality** (30pts): Zero warnings, all tests passing, clean git history
- **Adaptability** (20pts): Plan adjusted effectively, patterns extracted

---

## 7. Risk Mitigation

### If B2 Takes Longer Than Expected
- Focus on fixable tests only (document rest)
- Target: ≤20 ignored tests (relaxed from ≤10)
- Defer complex test fixes to v0.1.17

### If B3 dead_code Is More Complex
- Remove only obvious dead code
- Keep feature-gated code with #[cfg(feature)]
- Target: ≤60 annotations (relaxed from ≤40)

### If B1 unwrap() Reduction Is Slower
- Focus on public API boundaries first (highest impact)
- Keep CLI unwrap() calls (more acceptable)
- Target: ≤350 calls (relaxed from ≤280)
- Document remaining unwraps with TODOs

### If CI Fails During Execution
- Pause feature work
- Debug and fix CI issues first
- Ensure CI green before continuing

---

## 8. Execution Checklist

### Pre-Execution
- [ ] Episode context loaded
- [ ] Baseline metrics verified
- [ ] Agent assignments confirmed
- [ ] Quality gates defined
- [ ] Commit strategy documented

### During Execution
- [ ] Log each task execution
- [ ] Track agent progress
- [ ] Monitor quality gates
- [ ] Create atomic commits
- [ ] Adapt plan as needed

### Post-Execution
- [ ] All quality gates passing
- [ ] Metrics verified
- [ ] Episode score calculated
- [ ] Patterns extracted
- [ ] Documentation updated
- [ ] Retrospective completed

---

**Status**: 🔄 READY TO EXECUTE
**Next Action**: Execute Phase B.1 (Baseline Verification)
**Owner**: GOAP Agent
**Review Date**: 2026-02-23 (end of Week 1)
