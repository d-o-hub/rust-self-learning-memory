# Sprint: Hybrid Approach - v0.1.13

**Start Date**: 2026-01-06  
**Duration**: 1 week (5-7 days)  
**Target Release**: v0.1.13  
**Effort**: 20-31 hours

## Sprint Goal

Split top 3 largest files for file size compliance + fix failing tests to restore quality metrics.

## Sprint Backlog

### Phase 1: File Refactoring (12-19 hours)

#### Task 1: Split memory-mcp/src/bin/server.rs (8-10 hours)
**Current**: 2,368 LOC  
**Target**: 5 modules (~450 LOC each)

**Strategy** (Option C - Minimal bin/server.rs):
```
memory-mcp/src/bin/
├── server.rs           # main(), minimal re-exports (~100 LOC)
└── server/
    ├── mod.rs          # pub use re-exports (~50 LOC)
    ├── types.rs        # All type definitions (~270 LOC)
    ├── storage.rs      # Storage initialization (~180 LOC)
    ├── oauth.rs        # OAuth/security functions (~174 LOC)
    ├── jsonrpc.rs      # Server loop + routing (~175 LOC)
    ├── core.rs         # Core handlers (initialize, list_tools, call_tool) (~400 LOC)
    ├── tools.rs        # Tool handlers (query_memory, execute_code, etc) (~400 LOC)
    ├── mcp.rs          # MCP protocol handlers (completion, elicitation, tasks) (~400 LOC)
    └── embedding.rs    # Embedding config handler (~80 LOC)
```

**Rationale**: 
- Follows Rust bin conventions (minimal entry point)
- Clear separation by domain (types, storage, auth, handlers)
- All files under 450 LOC
- Easier to test individual modules

**Steps**:
1. Create `memory-mcp/src/bin/server/` directory
2. Extract type definitions → `server/types.rs` (~270 LOC)
3. Extract storage initialization → `server/storage.rs` (~180 LOC)
4. Extract OAuth functions → `server/oauth.rs` (~174 LOC)
5. Extract JSON-RPC loop + routing → `server/jsonrpc.rs` (~175 LOC)
6. Extract core handlers → `server/core.rs` (initialize, list_tools, call_tool) (~400 LOC)
7. Extract tool handlers → `server/tools.rs` (query_memory, execute_code, etc) (~400 LOC)
8. Extract MCP handlers → `server/mcp.rs` (completion, elicitation, tasks) (~400 LOC)
9. Extract embedding handler → `server/embedding.rs` (~80 LOC)
10. Create `server/mod.rs` with pub use re-exports (~50 LOC)
11. Update `server.rs` to minimal main() + imports (~100 LOC)
12. Run tests: `cargo test -p memory-mcp --lib`
13. Run clippy: `cargo clippy -p memory-mcp -- -D warnings`
14. Verify binary builds: `cargo build --bin memory-mcp-server`

**Success Criteria**:
- ✅ All modules ≤ 500 LOC
- ✅ All tests passing
- ✅ 0 clippy warnings
- ✅ Server functionality unchanged

#### Task 2: Split memory-mcp/src/patterns/statistical.rs (4-5 hours)
**Current**: 1,132 LOC  
**Target**: 3 modules (~370 LOC each)

**Strategy**:
```
memory-mcp/src/patterns/
├── statistical.rs (re-export module)
└── statistical/
    ├── mod.rs       # Core types & public API (~370 LOC)
    ├── analysis.rs  # Statistical analysis logic (~370 LOC)
    └── tests.rs     # Test suite (~370 LOC)
```

**Steps**:
1. Analyze test vs implementation split
2. Create `statistical/` directory
3. Move tests → `tests.rs`
4. Extract analysis functions → `analysis.rs`
5. Keep core types in `mod.rs`
6. Update `statistical.rs` to re-export
7. Run tests: `cargo test -p memory-mcp --test '*statistical*'`
8. Run clippy

**Success Criteria**:
- ✅ All modules ≤ 500 LOC
- ✅ All statistical tests passing
- ✅ 0 clippy warnings

#### Task 3: Split memory-storage-turso/src/lib.rs (3-4 hours)
**Current**: 964 LOC  
**Target**: 2 modules (~480 LOC each)

**Strategy**:
```
memory-storage-turso/src/
├── lib.rs              # Public API & re-exports (~100 LOC)
├── storage.rs          # Core TursoStorage impl (~480 LOC)
└── queries.rs          # SQL query operations (~480 LOC)
```

**Steps**:
1. Analyze query functions vs core impl
2. Extract SQL query functions → `queries.rs`
3. Extract core storage impl → `storage.rs`
4. Keep public API in `lib.rs`
5. Update imports
6. Run tests: `cargo test -p memory-storage-turso`
7. Run clippy

**Success Criteria**:
- ✅ All modules ≤ 500 LOC
- ✅ All storage tests passing
- ✅ 0 clippy warnings
- ✅ Storage functionality unchanged

### Phase 2: Test Fixes (8-12 hours)

#### Task 4: Investigate and Fix Failing Tests

**Current Status**:
- Test pass rate: 76.7%
- Known issues: 7 file size compliance tests failing

**Steps**:
1. Run full test suite to identify failures:
   ```bash
   cargo test --workspace --no-fail-fast 2>&1 | tee test_results.txt
   ```

2. Categorize failures:
   - File size compliance tests
   - Tests broken by recent refactoring
   - Integration test issues
   - Other failures

3. Fix file size compliance tests:
   - Update assertions for newly split files
   - Verify all files now pass 500 LOC limit
   
4. Fix refactoring-related test failures:
   - Update imports in test files
   - Update function call sites
   - Fix any API changes

5. Run tests iteratively until pass rate >95%

6. Document any remaining known issues

**Success Criteria**:
- ✅ Test pass rate >95%
- ✅ All file size compliance tests passing
- ✅ All integration tests passing
- ✅ Document remaining issues (if any)

### Phase 3: Validation & Documentation (2-4 hours)

#### Task 5: Comprehensive Validation

**Steps**:
1. Run full test suite:
   ```bash
   cargo test --workspace --all-features
   ```

2. Run clippy on all crates:
   ```bash
   cargo clippy --workspace --all-features -- -D warnings
   ```

3. Check formatting:
   ```bash
   cargo fmt --check
   ```

4. Build release binaries:
   ```bash
   cargo build --release --workspace
   ```

5. Run quick smoke tests on binaries

**Success Criteria**:
- ✅ All tests passing
- ✅ 0 clippy warnings
- ✅ 100% formatted
- ✅ Release builds successful

#### Task 6: Update Documentation

**Steps**:
1. Update CHANGELOG.md with refactoring details
2. Update file structure documentation
3. Add migration notes if APIs changed
4. Update architecture diagrams if needed
5. Document new module organization

**Files to Update**:
- `CHANGELOG.md`
- `memory-mcp/README.md`
- `memory-storage-turso/README.md`
- `agent_docs/service_architecture.md` (if needed)

**Success Criteria**:
- ✅ CHANGELOG.md updated
- ✅ Module documentation current
- ✅ Architecture docs updated

## Sprint Metrics

### File Size Compliance Progress
- **Before Sprint**: 6 files >500 LOC
- **After Sprint**: 3 files >500 LOC
- **Progress**: 50% reduction

### Test Quality
- **Target**: >95% pass rate
- **Current**: 76.7%
- **Improvement**: +18.3 percentage points

### Code Quality
- **Clippy Warnings**: 0 (maintain)
- **Formatting**: 100% (maintain)
- **Documentation**: Updated

## Success Criteria

**Must Have** (Release Blockers):
- ✅ Top 3 files split and under 500 LOC
- ✅ Test pass rate >95%
- ✅ All tests passing
- ✅ 0 clippy warnings
- ✅ CHANGELOG.md updated

**Nice to Have** (Can defer):
- Documentation improvements
- Performance benchmarks
- Additional file splits

## Risk Assessment

**Low Risk**:
- File splitting (proven pattern from recent refactors)
- Test fixes (straightforward)

**Medium Risk**:
- Integration test failures (may reveal deeper issues)
- Unexpected API breakage

**Mitigation**:
- Test frequently during refactoring
- Keep commits small and focused
- Document any breaking changes
- Revert if critical issues found

## Release Checklist

When sprint completes:
- [ ] All success criteria met
- [ ] CHANGELOG.md updated with changes
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Security audit clean (cargo audit)
- [ ] Ready to merge to main
- [ ] Tag v0.1.13
- [ ] Push tag to trigger release

## Timeline

**Day 1-2**: MCP server split (8-10 hours)
**Day 3**: Statistical patterns split (4-5 hours)
**Day 4**: Turso storage split (3-4 hours)
**Day 5-6**: Test fixes (8-12 hours)
**Day 7**: Validation & documentation (2-4 hours)

**Total**: 25-35 hours over 7 days

## Next Steps After Sprint

After v0.1.13 release:
1. **v0.1.14**: Complete file compliance (remaining 3 files)
2. **v0.1.15**: Code quality improvements (unwraps, clones)
3. **v0.2.0**: Major features or breaking changes (if needed)

---

**Ready to start?** Let's begin with Task 1: Split the MCP server file!
