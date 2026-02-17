# B1.1: Error Handling Baseline - Complete Audit

**Date**: 2026-02-17
**Status**: ✅ COMPLETE

## Verified Baseline (Production Code Only)

### By Crate

| Crate | unwrap() | .expect() | Total | Priority |
|-------|----------|-----------|-------|----------|
| memory-core | 215 | 47 | 262 | HIGH (public API) |
| memory-storage-turso | 180 | 35 | 215 | HIGH (storage layer) |
| memory-mcp | 108 | 3 | 111 | Medium |
| memory-cli | 48 | 16 | 64 | Low |
| memory-storage-redb | 26 | 21 | 47 | Low |
| **TOTAL** | **577** | **122** | **699** | - |

### Comparison to Documentation

| Metric | Documented | Actual | Variance |
|--------|-----------|--------|----------|
| Total calls | 651 | 699 | +48 (+7.4%) |
| Target | ≤280 | ≤280 | - |
| Calls to remove | 371 | 419 | +48 (+12.9%) |

**Conclusion**: Baseline is **higher than expected** by ~7%. This validates SOCRATES' emphasis on verifying baselines before execution.

### Impact on v0.1.16 Plan

**Original Estimate**:
- B1 (Error Handling): 8-12h
- Target: 50% reduction (651 → 280)

**Revised Estimate**:
- B1: **10-15h** (+2-3h due to higher baseline)
- Target: 60% reduction (699 → 280) to meet original goal

**Contingency**:
- If B1 exceeds 15h: Focus on public API boundaries only (memory-core, storage-turso)
- Defer internal cleanup to v0.1.17

## Methodology

```bash
# Count production code only (exclude tests)
for crate in memory-core memory-storage-turso memory-storage-redb memory-mcp memory-cli; do
  unwrap=$(find $crate -name "*.rs" -type f | grep -v test | xargs grep "unwrap()" | wc -l)
  expect=$(find $crate -name "*.rs" -type f | grep -v test | xargs grep ".expect(" | wc -l)
  echo "$crate: $unwrap unwrap + $expect expect"
done
```

## Next Steps (B1.2-B1.9)

### Priority Order (by impact)
1. **memory-core** (262 calls) - Public API, highest priority
2. **memory-storage-turso** (215 calls) - Storage layer, critical path
3. **memory-mcp** (111 calls) - Server errors, user-facing
4. **memory-cli** (64 calls) - CLI errors, low priority
5. **memory-storage-redb** (47 calls) - Cache layer, lowest priority

### B1.2: Design Error Enum (1.5h)
- Create base `Error` enum in `memory-core/src/error.rs`
- Use `thiserror` for automatic implementations
- Make extensible (add variants without breaking changes)

### B1.3-B1.7: Implement by Crate (6-8h)
- Start with memory-core (validate design)
- Parallel: storage-turso + storage-redb
- Parallel: memory-mcp + memory-cli
- Total: 3 sequential rounds, 2 crates in parallel each round

### B1.8: Add Error Path Tests (1h)
- Sample 10% of error paths (70 tests)
- Validate error messages are actionable
- Ensure ≥90% error context preservation

### B1.9: Validate Clippy (0.5h)
- `cargo clippy --all -- -D warnings`
- All crates must pass
- Fix any remaining issues

## Risk Mitigation

**Risk**: Higher baseline may exceed 15h budget
**Mitigation**:
1. Focus on public API first (80% of user impact)
2. Use `ok_or_else()` pattern (fastest conversion)
3. Document remaining unwraps with `expect("invariant: ...")`
4. Defer internal logic to v0.1.17 if needed

---

**Part of**: GOAP v0.1.16 Phase B execution
**Next**: B1.2 (Design error enum)
