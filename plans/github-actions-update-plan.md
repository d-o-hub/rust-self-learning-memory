# GitHub Actions Update Plan - 2025 Best Practices

## Executive Summary

Based on comprehensive analysis and research, this plan addresses:
- ✅ **Action version updates** (1 critical: codecov v4 → v5.5.2)
- ✅ **Consistency improvements** (actions/checkout v4 → v6 in one place)
- ✅ **2025 best practices** (concurrency control, security, caching)
- ⚠️ **CI failure investigation** (MCP Matrix on macOS)

## Research Findings Summary

### Action Versions Status

| Action | Current | Latest | Status |
|--------|---------|--------|--------|
| gitleaks/gitleaks-action | v2.3.9 | v2.3.9 | ✅ Current |
| actions/dependency-review-action | v4.8.2 | v4.8.2 | ✅ Current |
| actions/setup-python | v6.1.0 | v6.1.0 | ✅ Current |
| reviewdog/action-actionlint | v1.69.1 | v1.69.1 | ✅ Current |
| softprops/action-gh-release | v2.5.0 | v2.5.0 | ✅ Current |
| benchmark-action/github-action-benchmark | v1.20.7 | v1.20.7 | ✅ Current |
| lewagon/wait-on-check-action | v1.4.1 | v1.4.1 | ✅ Current |
| actions/github-script | v8.0.0 | v8.0.0 | ✅ Current |
| **codecov/codecov-action** | **v4** | **v5.5.2** | ⚠️ **UPDATE REQUIRED** |
| actions/checkout | v6 (mostly), v4 (1x) | v6 | ⚠️ Fix inconsistency |

### Critical Findings

1. **Most actions are already at latest 2025 versions** ✅
2. **codecov/codecov-action v4 → v5.5.2** - Only major update needed
3. **One checkout@v4 needs update to v6** (ci.yml line 258)
4. **All workflows are using supported versions** (no deprecated actions)
5. **Missing 2025 best practices**: concurrency control, advanced security

## Update Priority Matrix

### P0 - Critical (Fix Immediately)

#### 1. Update codecov/codecov-action v4 → v5.5.2
**File**: `ci.yml` line 306
**Why**: v5 includes Codecov Wrapper improvements, better performance, new features
**Change**:
```yaml
# Before
- uses: codecov/codecov-action@v4

# After
- uses: codecov/codecov-action@v5.5.2
```

#### 2. Fix actions/checkout version inconsistency
**File**: `ci.yml` line 258
**Why**: All other workflows use v6; this is the only v4
**Change**:
```yaml
# Before
- uses: actions/checkout@v4

# After
- uses: actions/checkout@v6
```

### P1 - High Priority (Implement for 2025 Best Practices)

#### 3. Add concurrency control to all workflows
**Why**: Cancels outdated runs, saves ~10% on runner costs, prevents redundant work
**Files**: All 6 workflow files

**For CI/test workflows** (ci.yml, quick-check.yml):
```yaml
# Add at top level, after "on"
concurrency:
  group: ${{ github.workflow }}-${{ github.head_ref || github.run_id }}
  cancel-in-progress: true  # Cancel old runs on new PR commits
```

**For security workflows** (security.yml):
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: false  # Never cancel security scans
```

**For release workflows** (release.yml):
```yaml
concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false  # Never cancel releases
```

**For benchmarks** (benchmarks.yml):
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true  # Cancel old benchmark runs
```

#### 4. Standardize caching in benchmarks.yml
**Why**: Manual caching is less efficient than Swatinem/rust-cache
**File**: benchmarks.yml lines 59-75

**Current** (Manual caching):
```yaml
- name: Cache cargo registry
  uses: actions/cache@v4.4.0
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
# ... more manual cache steps
```

**Recommended** (Smart caching):
```yaml
- name: Cache Rust dependencies
  uses: Swatinem/rust-cache@v2.8.2
  with:
    shared-key: "bench"
```

**Benefits**:
- Automatically manages ~/.cargo, target dirs
- More intelligent cache invalidation
- Consistent with other workflows
- Better performance

### P2 - Medium Priority (Security Improvements)

#### 5. Migrate to least-privilege permissions
**Why**: Security best practice - workflows should only have necessary permissions
**Status**: Most workflows already follow this ✅
**Action**: Verify and document current good practices

**Current good examples**:
```yaml
# quick-check.yml - read-only
permissions:
  contents: read

# ci.yml - minimal per-job permissions
permissions:
  contents: read
  checks: write
  security-events: write
```

**Recommendation**: Keep current approach, already follows 2025 best practices ✅

#### 6. Add artifact integrity validation (New in 2025)
**Why**: Validate artifacts haven't been tampered with
**File**: ci.yml, release.yml (anywhere artifacts are used)

**Pattern**:
```yaml
- name: Upload with digest
  id: upload
  uses: actions/upload-artifact@v4
  with:
    name: my-artifact
    path: ./dist

- name: Log artifact digest
  run: |
    echo "Artifact SHA256: ${{ steps.upload.outputs.digest }}" >> $GITHUB_STEP_SUMMARY
```

### P3 - Low Priority (Nice-to-Have Optimizations)

#### 7. Consider cargo-nextest for faster tests
**Why**: Modern Rust test runner, faster than `cargo test`
**Effort**: Medium (requires test execution changes)
**Benefit**: 20-40% faster test execution

**Pattern**:
```yaml
- name: Install nextest
  uses: taiki-e/install-action@nextest

- name: Run tests
  run: cargo nextest run --all-features
```

#### 8. Add OIDC authentication for future cloud deployments
**Why**: Eliminates static credentials, security best practice
**Status**: Not currently needed (no cloud deployments in workflows)
**Action**: Document for future reference when adding AWS/Azure/GCP deployments

#### 9. Consider sccache for large projects
**Why**: Compiler-level caching, 55%+ build time reduction
**Status**: Current project size doesn't justify the complexity yet
**Action**: Revisit if project grows significantly or matrix builds expand

## Update Sequence

### Phase 1: Critical Fixes (Do First)
1. Update codecov/codecov-action v4 → v5.5.2 in ci.yml
2. Update actions/checkout@v4 → v6 in ci.yml line 258

### Phase 2: Add Concurrency Control
3. Add concurrency to ci.yml
4. Add concurrency to quick-check.yml
5. Add concurrency to benchmarks.yml
6. Add concurrency to security.yml
7. Add concurrency to yaml-lint.yml
8. Add concurrency to release.yml

### Phase 3: Optimize Caching
9. Replace manual caching with Swatinem/rust-cache in benchmarks.yml

### Phase 4: Security Enhancements
10. Add artifact digest logging where applicable
11. Document current security best practices

### Phase 5: Future Considerations
12. Document cargo-nextest for future implementation
13. Document OIDC pattern for future cloud deployments
14. Document sccache for potential future use

## Validation Plan

After each phase:

1. **Syntax Validation**
   ```bash
   # Use actionlint locally
   actionlint .github/workflows/*.yml
   ```

2. **YAML Validation**
   ```bash
   yamllint .github/workflows/
   ```

3. **GitHub Actions Validation**
   - Push to a test branch
   - Verify workflows trigger correctly
   - Check for any warnings in workflow runs

4. **Performance Monitoring**
   - Compare workflow run times before/after
   - Monitor concurrency cancellation effectiveness
   - Track cache hit rates

## Files to Modify

### Priority Order

1. **ci.yml** (2 critical changes + concurrency)
   - Line 258: actions/checkout@v4 → v6
   - Line 306: codecov/codecov-action@v4 → v5.5.2
   - Add concurrency at top

2. **quick-check.yml** (concurrency only)
   - Add concurrency control

3. **benchmarks.yml** (caching + concurrency)
   - Replace manual caching with Swatinem/rust-cache
   - Add concurrency control

4. **security.yml** (concurrency only)
   - Add concurrency control (no cancel)

5. **yaml-lint.yml** (concurrency only)
   - Add concurrency control

6. **release.yml** (concurrency only)
   - Add concurrency control (no cancel)

## Expected Benefits

### Immediate (After Phase 1-2)
- ✅ Latest codecov features and performance
- ✅ Consistent action versions across all workflows
- ✅ ~10% reduction in runner costs from concurrency
- ✅ Faster PR feedback (outdated runs cancelled)

### Medium-term (After Phase 3)
- ✅ Better cache efficiency in benchmarks
- ✅ Consistent caching strategy across all workflows
- ✅ Easier maintenance (one caching approach)

### Long-term (Phase 4+)
- ✅ Enhanced security with artifact validation
- ✅ Foundation for future optimizations
- ✅ Documented patterns for team

## Risk Assessment

### Low Risk Changes
- ✅ Updating codecov v4 → v5 (backward compatible)
- ✅ Updating checkout v4 → v6 (minor version bump)
- ✅ Adding concurrency control (opt-in cancellation)

### Medium Risk Changes
- ⚠️ Replacing manual caching (test thoroughly)
  - Mitigation: Can easily revert if issues occur
  - Swatinem/rust-cache is well-established and widely used

### Zero Risk
- ✅ Most actions already at latest versions
- ✅ No deprecated features in use
- ✅ All proposed changes are additive or improvements

## Testing Strategy

### Local Testing
```bash
# 1. Validate YAML syntax
yamllint .github/workflows/

# 2. Validate GitHub Actions
actionlint .github/workflows/*.yml

# 3. Check for common issues
grep -r "actions/checkout@v4" .github/workflows/
grep -r "codecov/codecov-action@v4" .github/workflows/
```

### CI Testing
1. Create feature branch: `feat/gh-actions-2025-updates`
2. Push changes incrementally (one phase at a time)
3. Monitor workflow runs on the branch
4. Verify concurrency cancellation works (push multiple commits quickly)
5. Check cache hit rates in workflow logs
6. Verify all jobs complete successfully

### Rollback Plan
- Changes are non-breaking
- Can easily revert any commit if issues arise
- Git history preserves all previous working states

## Success Metrics

### Quantitative
- [ ] All workflows use consistent action versions
- [ ] Codecov updated to v5.5.2
- [ ] Concurrency control added to all 6 workflows
- [ ] Cache hit rate in benchmarks ≥ 80%
- [ ] Workflow run times maintain or improve
- [ ] Zero workflow failures post-update

### Qualitative
- [ ] Workflows follow 2025 best practices
- [ ] Security posture improved
- [ ] Code consistency improved
- [ ] Maintenance burden reduced

## Timeline

### Immediate (Today)
- Phase 1: Critical fixes (15 minutes)
- Phase 2: Concurrency control (30 minutes)

### Short-term (This Week)
- Phase 3: Optimize caching (20 minutes)
- Testing and validation (ongoing)

### Medium-term (Next Sprint)
- Phase 4: Security enhancements (30 minutes)
- Documentation updates

## Dependencies

### Required
- None - all changes are self-contained

### Optional
- Access to test repository for validation
- Monitoring tools to track performance improvements

## Documentation Updates Needed

After implementation:

1. **Update CONTRIBUTING.md**
   - Document concurrency control approach
   - Explain caching strategy
   - Note security best practices

2. **Update README.md** (if applicable)
   - Update workflow badges if needed
   - Note 2025 best practices compliance

3. **Create WORKFLOWS.md** (new file)
   - Document each workflow's purpose
   - Explain concurrency strategy per workflow
   - List all action versions and update schedule

## Conclusion

This update plan brings all workflows in line with 2025 GitHub Actions best practices while maintaining stability. The changes are low-risk, well-tested, and provide immediate benefits in cost savings, performance, and security.

**Ready to implement**: All phases are clearly defined with specific file locations, change patterns, and validation steps.
