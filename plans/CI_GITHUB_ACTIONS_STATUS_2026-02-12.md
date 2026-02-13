# CI/CD GitHub Actions Status Report

**Date**: 2026-02-12  
**Branch**: main  
**Triggered by**: GOAP orchestration audit  

---

## Workflow Summary

| # | Workflow | Status (main) | Open PRs | Action Needed |
|---|---------|--------------|----------|---------------|
| 1 | CI | ✅ Pass | ❌ All Fail | Fix clippy for deps |
| 2 | Coverage | ❌ Disk Full | ❌ Fail | Free disk space |
| 3 | CodeQL | ✅ Pass | ❌ Fail (some) | Depends on CI fix |
| 4 | File Structure Validation | ✅ Pass | ✅ Pass | None |
| 5 | YAML Lint | ✅ Pass | ✅ Pass | None |
| 6 | Performance Benchmarks | ⏳ Running | Skipped | Monitor |
| 7 | Security | ✅ Pass | ✅ Pass | None |
| 8 | Quick Check | ✅ Pass | ❌ Fail | Fix clippy |
| 9 | Release | — | — | None |
| 10 | Nightly Full Tests | — | — | Monitor |
| 11 | Dependabot Updates | Active | — | None |
| 12 | pages-build-deployment | Active | — | None |
| 13 | ci-old.yml | ⚠️ STALE | — | Remove |

**Total Workflows**: 13 (all active)

---

## Critical Issues

### Issue 1: Coverage Workflow Disk Space Failure (P0)

- **Workflow**: [`coverage.yml`](../.github/workflows/coverage.yml)
- **Run**: https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955770671
- **Error**: `error: couldn't create a temp dir: No space left on device (os error 28)`
- **Root Cause**: Full workspace coverage (`cargo llvm-cov --workspace`) on main branch exhausts GitHub Actions runner disk (~14GB)
- **Failing Step**: "Generate Coverage Report (Full Workspace - Optional)" (coverage.yml lines 68-80)
- **Impact**: Coverage badge not generated, Codecov upload fails
- **Fix Options**:
  1. Add `jlumbroso/free-disk-space` action before coverage step
  2. Split coverage into separate jobs per crate
  3. Use `--exclude` flags to skip bench/example crates from full coverage
  4. Remove `/opt/hostedtoolcache` more aggressively

### Issue 2: Performance Benchmarks Stuck/Long-Running (P1)

- **Workflow**: [`benchmarks.yml`](../.github/workflows/benchmarks.yml)
- **Runs in progress**:
  - https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955770592 (main)
  - https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955540371 (branch)
- **Previous run**: Cancelled (https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955052421)
- **Concern**: Two runs in_progress simultaneously, previous runs cancelled — indicates timeout/resource issues
- **Impact**: Benchmark results not stored, regression detection blocked

### Issue 3: 5 Open Dependabot PRs Failing CI (P1)

All PRs fail with identical patterns — clippy errors and MCP build failures:

| PR | Title | Clippy | MCP Build | Tests |
|----|-------|--------|-----------|-------|
| [#271](https://github.com/d-o-hub/rust-self-learning-memory/pull/271) | bump criterion 0.5.1 → 0.8.2 | ❌ | ❌ | Cancelled |
| [#270](https://github.com/d-o-hub/rust-self-learning-memory/pull/270) | bump sysinfo 0.38.0 → 0.38.1 | ❌ | ❌ | Cancelled |
| [#269](https://github.com/d-o-hub/rust-self-learning-memory/pull/269) | bump reqwest 0.13.1 → 0.13.2 | ❌ | ❌ | Cancelled |
| [#268](https://github.com/d-o-hub/rust-self-learning-memory/pull/268) | bump actions/download-artifact 4 → 7 | ❌ | ❌ | Cancelled |
| [#267](https://github.com/d-o-hub/rust-self-learning-memory/pull/267) | bump github/codeql-action 3 → 4 | ❌ | ❌ | Cancelled |
| [#266](https://github.com/d-o-hub/rust-self-learning-memory/pull/266) | bump actions/upload-artifact 4 → 6 | ❌ | ❌ | Cancelled |

**Common failure pattern**:
- Essential Checks (clippy) → FAILURE
- MCP Build (default) → FAILURE
- MCP Build (wasm-rquickjs) → FAILURE
- Multi-Platform Test (ubuntu/macos) → FAILURE
- Tests → CANCELLED (due to earlier failures)
- Quality Gates → SKIPPED

### Issue 4: Stale Workflow File (P2)

- **File**: `.github/workflows/ci-old.yml`
- **Status**: Listed as active (workflow ID 225276009) but appears to be a legacy/duplicate CI workflow
- **Action**: Remove or disable

### Issue 5: No GitHub Issues Tracking (P2)

- **Finding**: Zero open GitHub Issues (`gh issue list` returns empty)
- **Impact**: CI failures and technical debt not tracked
- **Action**: Create issues for each CI problem with appropriate labels

---

## Latest Run Details (main branch, 2026-02-12T16:46)

| Workflow | Run ID | Conclusion |
|----------|--------|------------|
| CI | [21955770667](https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955770667) | ✅ success |
| Coverage | [21955770671](https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955770671) | ❌ failure |
| CodeQL | [21955772854](https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955772854) | ✅ success |
| File Structure | [21955770630](https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955770630) | ✅ success |
| YAML Lint | [21955770614](https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955770614) | ✅ success |
| Benchmarks | [21955770592](https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955770592) | ⏳ in_progress |
| Security | [21955770550](https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21955770550) | ✅ success |

---

## Recommendations (Priority Order)

1. **P0**: Fix coverage disk space — add `jlumbroso/free-disk-space` or exclude bench/example crates from `--workspace` coverage
2. **P1**: Triage Dependabot PRs — fix clippy issues for minor/patch bumps, close criterion major bump (#271) for dedicated migration
3. **P1**: Monitor benchmark runs for completion; add explicit timeout if stuck
4. **P2**: Remove `ci-old.yml` stale workflow
5. **P2**: Create GitHub Issues for tracking each CI problem
6. **P3**: Add disk space monitoring step to long-running workflows

---

## Related Documents

- **ADR**: [`plans/adr/ADR-023-CI-CD-GitHub-Actions-Remediation.md`](adr/ADR-023-CI-CD-GitHub-Actions-Remediation.md)
- **Previous Report**: [`plans/CI_STATUS_REPORT_2026-02-03.md`](CI_STATUS_REPORT_2026-02-03.md)
- **Critical Items**: [`plans/CRITICAL_ACTION_ITEMS_2026-02-12.md`](CRITICAL_ACTION_ITEMS_2026-02-12.md)
