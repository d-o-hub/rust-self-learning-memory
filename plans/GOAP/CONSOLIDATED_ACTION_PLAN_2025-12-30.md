# Consolidated Action Plan: GitHub Issues & PRs

**Date**: 2025-12-30
**Priority**: P0-P3
**Repository**: d-o-hub/rust-self-learning-memory
**Based On**: GOAP Analysis `plans/GOAP/GITHUB_ISSUES_PRS_ANALYSIS_EXECUTION_SUMMARY.md`

---

## Executive Summary

Consolidated, prioritized action items resulting from comprehensive GitHub issues/PRs analysis. All actions are immediately actionable with clear acceptance criteria and estimated effort.

**Total Actions**: 4
**P0 (Critical)**: 1 action (5 minutes)
**P1 (High)**: 2 actions (10 minutes + 2-4 hours)
**P2 (Medium)**: 1 action (1-2 hours)

**Total Estimated Effort**: 3.25-6.25 hours

---

## P0 (Critical - Do Today)

### Action 1: Fix Release Workflow Bug

**Priority**: 🔴 CRITICAL
**Effort**: 5 minutes
**Impact**: Enables successful releases

#### Problem
Release workflow contains invalid parameter `remove_artifacts: true` at line 124, causing all release attempts to fail with "Not Found" errors.

#### Root Cause
Invalid parameter not supported by GitHub Actions `upload-artifact` action.

#### Solution
Remove line 124 from `.github/workflows/release.yml`.

#### Steps

```bash
# 1. Navigate to workflow file
cd /workspaces/feat-phase3

# 2. Backup current version
cp .github/workflows/release.yml .github/workflows/release.yml.backup

# 3. Remove line 124
sed -i '124d' .github/workflows/release.yml

# 4. Verify the fix
git diff .github/workflows/release.yml

# Expected output: Line 124 should be removed
```

#### Verification

```bash
# Check workflow syntax
gh workflow view release.yml

# Dry run (optional - create test tag)
# git tag -a v0.1.10-test -m "Test release"
# git push origin v0.1.10-test
# Monitor workflow: https://github.com/d-o-hub/rust-self-learning-memory/actions
```

#### Acceptance Criteria
- [ ] Line 124 removed from `.github/workflows/release.yml`
- [ ] Workflow syntax is valid (no errors from `gh workflow view`)
- [ ] Test release succeeds (or dry run completes without errors)
- [ ] No "Not Found" errors in release workflow logs

#### Rollback (if needed)
```bash
# Restore backup
cp .github/workflows/release.yml.backup .github/workflows/release.yml
git checkout .github/workflows/release.yml
```

#### References
- Issue: #191
- Plan: `plans/github_actions_issues_and_improvements.md`
- Related: `plans/GOAP/GITHUB_ISSUES_PRS_ANALYSIS_EXECUTION_SUMMARY.md`

---

## P1 (High - Do This Week)

### Action 2: Merge Dependabot PR #183

**Priority**: 🟡 HIGH
**Effort**: 10 minutes
**Impact**: Keeps dependencies up-to-date, includes security fixes

#### Problem
Dependabot PR #183 open to bump sysinfo from 0.30.13 to 0.37.2. Includes important fixes:
- Windows: Correct invalid UTF-8 handling in `Motherboard`
- Linux: Fixed `Process::exe` path handling
- Android: Fixed `System::uptime`
- Security: Various improvements

#### Prerequisites
Verify MSRV (Minimum Supported Rust Version) compatibility:
- sysinfo 0.37.0 updates MSRV to Rust 1.88
- Project currently uses Rust 1.82 (from `rust-toolchain.toml`)

#### Steps

```bash
# 1. Check current MSRV
cat rust-toolchain.toml
# Expected: [toolchain] channel = "1.82"

# 2. Check if MSRV update needed
grep -r "rust-version" Cargo.toml

# 3. Review PR changes
gh pr diff 183 --repo d-o-hub/rust-self-learning-memory

# 4. Verify CI is passing on the PR
gh pr checks 183 --repo d-o-hub/rust-self-learning-memory

# 5. If all checks pass, merge
gh pr comment 183 --repo d-o-hub/rust-self-learning-memory --body "@dependabot merge"
```

#### Alternative: Manual Merge with MSRV Update

If MSRV needs updating to 1.88:

```bash
# 1. Checkout PR branch
gh pr checkout 183

# 2. Update rust-toolchain.toml
echo "[toolchain]
channel = \"1.88\"" > rust-toolchain.toml

# 3. Commit MSRV update
git add rust-toolchain.toml
git commit -m "chore(rust): bump MSRV to 1.88 for sysinfo 0.37.2"

# 4. Push and merge
git push
gh pr merge 183 --merge --delete-branch
```

#### Acceptance Criteria
- [ ] MSRV compatibility verified (or updated to 1.88)
- [ ] All CI checks passing on PR
- [ ] PR merged successfully
- [ ] Tests still passing after merge
- [ ] No regressions introduced

#### References
- Issue: #183
- PR: https://github.com/d-o-hub/rust-self-learning-memory/pull/183
- Dependabot docs: https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates

---

### Action 3: Re-enable Windows Pool Integration Tests

**Priority**: 🟡 HIGH
**Effort**: 2-4 hours
**Impact**: Full platform test coverage

#### Problem
Connection pool integration tests crash on Windows CI with `STATUS_ACCESS_VIOLATION`. Currently disabled with `#[cfg_attr(target_os = "windows", ignore)]`.

#### Root Cause (Hypotheses)
1. **libsql Windows compatibility**: File locking or concurrent access issues
2. **Connection pool implementation**: Race conditions or memory safety issues specific to Windows
3. **TempDir/file cleanup**: Windows file handles not released properly during cleanup

#### Steps

**Phase 1: Investigation (1 hour)**

```bash
# 1. Check current test state
grep -n "cfg_attr.*windows.*ignore" memory-storage-turso/tests/pool_integration_test.rs

# 2. Review connection pool implementation
cat memory-storage-turso/src/pool.rs

# 3. Check libsql Windows issues
# Search: https://github.com/libsql/libsql/issues?q=is%3Aissue+windows

# 4. Review pool test code
cat memory-storage-turso/tests/pool_integration_test.rs
```

**Phase 2: Isolate Issue (1-2 hours)**

```rust
// Add Windows-specific logging
#[cfg(target_os = "windows")]
fn log_windows_info() {
    println!("Windows platform detected");
    println!("libsql version: {:?}", env!("CARGO_PKG_VERSION"));
    // Add more diagnostic logging
}

// Run tests with single thread
// cargo test --package memory-storage-turso --test pool_integration_test -- --test-threads=1
```

**Phase 3: Implement Fix (1-2 hours)**

**Option A: Windows-Specific Configuration**
```rust
#[cfg(target_os = "windows")]
const MAX_CONNECTIONS: usize = 1; // Reduce concurrency on Windows
```

**Option B: Improved File Handle Management**
```rust
// Ensure connections closed before cleanup
impl Drop for ConnectionPool {
    fn drop(&mut self) {
        // Explicitly close all connections
        // Wait for file handles to release
    }
}
```

**Option C: In-Memory SQLite on Windows**
```rust
// Use in-memory database on Windows for testing
#[cfg(target_os = "windows")]
let db_url = ":memory:";
```

#### Verification

```bash
# Run Windows tests (if on Windows)
cargo test --package memory-storage-turso --test pool_integration_test

# Cross-compile for Windows (if on Linux/Mac)
cargo test --target x86_64-pc-windows-gnu --package memory-storage-turso
```

#### Acceptance Criteria
- [ ] Root cause of Windows crash identified
- [ ] Fix implemented and tested
- [ ] All 6 pool integration tests pass on Windows
- [ ] No access violations or crashes
- [ ] Tests complete in reasonable time (<5 minutes)
- [ ] `#[cfg_attr(target_os = "windows", ignore)]` removed

#### Rollback
If fix unstable, keep `#[cfg_attr(target_os = "windows", ignore)]` and create issue for future investigation.

#### References
- Issue: #96 (closed)
- Related: Issue #95 (closed - flaky periodic sync test)
- Plan: `plans/github_actions_issues_and_improvements.md`

---

## P2 (Medium - Do This Month)

### Action 4: Update Index Files

**Priority**: 🟢 MEDIUM
**Effort**: 1-2 hours
**Impact**: Better navigation and discoverability

#### Problem
Index files may be outdated and not reflect current plans folder structure.

#### Files to Update

1. **`plans/README.md`**
   - Update file count and structure
   - Add new plans created in 2025-12
   - Update navigation section

2. **`plans/README_NAVIGATION.md`**
   - Add new plan entries
   - Update links
   - Verify all links are valid

3. **`plans/archive/ARCHIVE_INDEX.md`**
   - Add recently archived files
   - Update file count
   - Update structure documentation

#### Steps

```bash
# 1. Get current file counts
echo "Active plans: $(find plans -type f -name '*.md' ! -path '*/archive/*' | wc -l)"
echo "Archived plans: $(find plans -type f -name '*.md' -path '*/archive/*' | wc -l)"

# 2. List recent plans (last 7 days)
find plans -type f -name '*.md' -mtime -7 ! -path '*/archive/*'

# 3. Check for broken links
grep -r "](.*\.md)" plans/README*.md | grep -v "Binary" | cut -d: -f1 | sort -u

# 4. Update each file
# Edit plans/README.md
# Edit plans/README_NAVIGATION.md
# Edit plans/archive/ARCHIVE_INDEX.md
```

#### Acceptance Criteria
- [ ] File counts updated and accurate
- [ ] New plans from 2025-12 added
- [ ] All links valid (no 404s)
- [ ] Navigation structure clear and intuitive
- [ ] Archive index up-to-date

#### References
- Related: `plans/GOAP/PHASE1_ANALYSIS_GITHUB_ISSUES_PRS.md`
- Analysis shows: 127 active plans, 154 archived plans

---

## Execution Timeline

| Day | Action | Priority | Effort |
|------|--------|----------|--------|
| **Today** | Fix Release Workflow Bug | P0 | 5 min |
| **Today/Tomorrow** | Merge Dependabot PR #183 | P1 | 10 min |
| **This Week** | Re-enable Windows Pool Tests | P1 | 2-4 hours |
| **This Month** | Update Index Files | P2 | 1-2 hours |

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Critical bugs fixed | 1 | Release workflow bug resolved |
| Dependencies updated | 1 | sysinfo merged |
| Platform coverage | 100% | Windows tests passing |
| Documentation current | 100% | Index files accurate |
| Total actions completed | 4 | P0-P3 all done |

---

## Risk Management

### Action 1 (Release Workflow)
- **Risk**: Release still fails after fix
- **Mitigation**: Test with dry run, monitor workflow logs
- **Fallback**: Restore backup, investigate further

### Action 2 (Dependabot Merge)
- **Risk**: MSRV incompatibility
- **Mitigation**: Verify MSRV before merge, update if needed
- **Fallback**: Manual merge with MSRV update

### Action 3 (Windows Tests)
- **Risk**: Cannot root cause or fix crash
- **Mitigation**: Start with isolation, try multiple approaches
- **Fallback**: Keep tests disabled, create issue for future work

### Action 4 (Index Updates)
- **Risk**: Miss files or broken links
- **Mitigation**: Systematic review, link checker
- **Fallback**: Incremental updates over multiple iterations

---

## Reporting

### After Each Action
Document completion in:
- This action plan (mark items as done)
- Related GitHub issue (#191, #183, #96)
- `plans/STATUS/PROJECT_STATUS_UNIFIED.md`

### After All Actions Complete
Create summary report:
- Actions completed
- Issues resolved
- Remaining work (if any)
- Lessons learned

---

## Appendix: Quick Reference

### Commands

```bash
# Fix Release Workflow
sed -i '124d' .github/workflows/release.yml
gh workflow view release.yml

# Merge Dependabot PR
gh pr checks 183
gh pr comment 183 --body "@dependabot merge"

# Windows Tests
cargo test --package memory-storage-turso --test pool_integration_test

# Update Indexes
find plans -type f -name '*.md' ! -path '*/archive/*' | wc -l
```

### Checklists

**Release Workflow Fix**:
- [ ] Backup file
- [ ] Remove line 124
- [ ] Verify syntax
- [ ] Test release
- [ ] Update status

**Dependabot Merge**:
- [ ] Check MSRV
- [ ] Review changes
- [ ] Verify CI passing
- [ ] Merge PR
- [ ] Run tests

**Windows Tests**:
- [ ] Investigate crash
- [ ] Identify root cause
- [ ] Implement fix
- [ ] Verify tests pass
- [ ] Remove ignore attribute

**Index Updates**:
- [ ] Count files
- [ ] Update README.md
- [ ] Update README_NAVIGATION.md
- [ ] Update ARCHIVE_INDEX.md
- [ ] Verify links

---

**Action Plan Version**: 1.0
**Created**: 2025-12-30
**Status**: Ready for Execution
**Next Action**: P0 - Fix Release Workflow Bug (5 minutes)
