# GOAP Execution Plan - Decomposed Phase 1 Tasks

**Created**: 2026-02-12
**GOAP Session**: CI/CD Remediation
**Priority**: P0 - CI Stabilization
**Status**: Active Orchestration

---

## Current State Analysis

### CI Status (as of 2026-02-12 19:00 UTC)

| Workflow | Status | Failure Reason |
|----------|--------|----------------|
| CI | ✅ Passing | - |
| Coverage | ❌ Failure | Disk space issue (despite free-disk-space action) |
| YAML Lint | ❌ Failure | 15 trailing spaces + 1 line-length error in benchmarks.yml |
| File Structure | ✅ Passing | - |
| Security | ✅ Passing | - |
| Dependabot PRs | ❌ 5 failing | Clippy warnings blocking |

### Root Cause Analysis

**1. Coverage Disk Space Failure (P0)**
- Already has `jlumbroso/free-disk-space@v1.3.1` at line 36
- But still fails with "No space left on device"
- Hypothesis: Full workspace coverage step (lines 73-84) exhausts disk even after cleanup
- The timeout protection catches it, but generates no coverage report

**2. YAML Lint Failure (P0)**
- Commit `fdbfefdb` (2026-02-12 18:57) introduced formatting issues
- Errors in `.github/workflows/benchmarks.yml`:
  - Lines 43, 80, 84, 86, 91, 93, 136, 140, 151, 160, 165, 171, 183, 370, 385: trailing spaces
  - Line 390: line too long (319 > 120 characters)
- These need immediate fix to unblock CI

**3. Dependabot PR Clippy Failures (P0)**
- 5 open PRs all fail clippy checks
- Need to triage and fix each PR individually

**4. ci-old.yml (P0)**
- File NOT found in `.github/workflows/` directory
- May have already been removed or never existed
- Verify GitHub Actions workflow list for stale entries

---

## Decomposed Task Breakdown

### Phase 1.1: Fix YAML Lint Failures (P0 - CRITICAL)

**Task**: Fix trailing spaces and line-length issues in benchmarks.yml

**Subtasks**:

#### 1.1.A: Fix Trailing Spaces (Parallelizable)
- **File**: `.github/workflows/benchmarks.yml`
- **Lines**: 43, 80, 84, 86, 91, 93, 136, 140, 151, 160, 165, 171, 183, 370, 385
- **Action**: Remove trailing spaces from each line
- **Agent**: `code-quality` (automated fix via `cargo fmt` equivalent for YAML)
- **Effort**: 5 minutes
- **Dependencies**: None
- **Acceptance**: `yamllint -d "{extends: default, rules: {line-length: {max: 120}, indentation: {spaces: 2}}}" .github/` passes

#### 1.1.B: Fix Line Length Issue
- **File**: `.github/workflows/benchmarks.yml`
- **Line**: 390
- **Issue**: 319 characters > 120 max
- **Action**: Break long line into multiple lines using YAML folding
- **Agent**: `github-workflows`
- **Effort**: 10 minutes
- **Dependencies**: None (can run parallel with 1.1.A)
- **Acceptance**: `yamllint` passes for line-length rule

#### 1.1.C: Verify YAML Lint Fix
- **Action**: Run `yamllint` locally to confirm all errors resolved
- **Agent**: `code-quality`
- **Effort**: 2 minutes
- **Dependencies**: 1.1.A AND 1.1.B complete
- **Acceptance**: Zero yamllint errors

**Quality Gate**:
- [ ] `yamllint .github/` passes with zero errors
- [ ] `yamllint` exit code is 0
- [ ] No new YAML syntax errors introduced

---

### Phase 1.2: Fix Coverage Disk Space Issue (P0 - CRITICAL)

**Task**: Resolve "No space left on device" error in coverage workflow

**Subtasks**:

#### 1.2.A: Diagnose Current Coverage Failure
- **Action**: Analyze recent coverage run logs to identify exact failure point
- **Agent**: `debugger` (CI/CD specialist)
- **Effort**: 10 minutes
- **Dependencies**: None
- **Acceptance**: Identify which step fails (library vs full workspace)

#### 1.2.B: Optimize Free Disk Space Action
- **File**: `.github/workflows/coverage.yml`
- **Current**: Uses `jlumbroso/free-disk-space@v1.3.1` with standard options
- **Potential Improvements**:
  1. Add more aggressive cleanup options
  2. Move cleanup step earlier (before checkout)
  3. Add explicit cache clearing
  4. Increase tool removal
- **Agent**: `github-workflows`
- **Effort**: 15 minutes
- **Dependencies**: 1.2.A complete
- **Acceptance**: Free disk space action shows >20GB available

#### 1.2.C: Alternative: Split Coverage into Jobs
- **Action**: If 1.2.B insufficient, split coverage into separate jobs:
  - Job 1: Library coverage (fast, always runs)
  - Job 2: Workspace coverage (optional, main only)
- **Agent**: `github-workflows`
- **Effort**: 20 minutes
- **Dependencies**: 1.2.B complete AND still failing
- **Acceptance**: Both jobs complete successfully

#### 1.2.D: Verify Coverage Fix
- **Action**: Run coverage workflow locally or trigger manual run
- **Agent**: `test-runner`
- **Effort**: 30 minutes (runtime)
- **Dependencies**: 1.2.B OR 1.2.C complete
- **Acceptance**: Coverage report generated, Codecov upload succeeds

**Quality Gate**:
- [ ] Coverage workflow completes without disk space errors
- [ ] Codecov badge restored
- [ ] Coverage percentage ≥90% maintained

---

### Phase 1.3: Fix Dependabot PR Clippy Failures (P0)

**Task**: Unblock 5 Dependabot PRs by resolving clippy warnings

**Subtasks**:

#### 1.3.A: Triage Dependabot PRs
- **PRs**: #266, #267, #268, #269, #270, #271
- **Action**: For each PR:
  1. Checkout PR branch
  2. Run `cargo clippy --all -- -D warnings`
  3. Document clippy warnings
  4. Categorize by severity
- **Agent**: `code-reviewer` (with `test-runner` for execution)
- **Effort**: 20 minutes
- **Dependencies**: None (can run parallel with 1.1 and 1.2)
- **Acceptance**: All PRs documented with clippy error list

#### 1.3.B: Fix Simple PRs (Patch/Minor Bumps)
- **Target PRs**: #266, #267, #268 (GitHub Actions version bumps)
- **Action**: These are typically non-breaking, fix any actual code issues
- **Agent**: `rust-specialist`
- **Effort**: 15 minutes per PR
- **Dependencies**: 1.3.A complete
- **Acceptance**: `cargo clippy` passes, PRs ready to merge

#### 1.3.C: Evaluate Criterion Bump (Major Version)
- **PR**: #271 (criterion 0.5.1 → 0.8.2)
- **Action**: Assess breaking changes, decide:
  - Fix clippy issues if minor
  - Defer to separate migration PR if major
- **Agent**: `rust-specialist` + `code-reviewer`
- **Effort**: 30 minutes
- **Dependencies**: 1.3.A complete
- **Acceptance**: Decision documented (merge or defer)

#### 1.3.D: Merge or Close PRs
- **Action**: For each PR:
  - If fixed: merge via squash
  - If deferred: close with comment explaining why
  - If major version: create tracking issue
- **Agent**: `github-workflows` (or manual git operation)
- **Effort**: 10 minutes
- **Dependencies**: 1.3.B AND 1.3.C complete
- **Acceptance**: All PRs either merged or closed with rationale

**Quality Gate**:
- [ ] All Dependabot PRs resolved (merged or closed)
- [ ] Zero open Dependabot PRs with failing CI
- [ ] Dependencies updated if compatible

---

### Phase 1.4: Remove Stale ci-old.yml (P0)

**Task**: Verify and remove any stale workflow files

**Subtasks**:

#### 1.4.A: Verify Stale Workflow Existence
- **Action**:
  1. Check `.github/workflows/` for `ci-old.yml` or similar
  2. Query GitHub Actions API for active workflows
  3. Identify any deprecated workflows
- **Agent**: `github-workflows`
- **Effort**: 5 minutes
- **Dependencies**: None
- **Acceptance**: List of stale workflow files

#### 1.4.B: Remove Stale Workflows
- **Action**:
  1. Delete identified stale files
  2. Verify no references in other workflows
  3. Commit removal
- **Agent**: `github-workflows`
- **Effort**: 5 minutes
- **Dependencies**: 1.4.A finds stale files
- **Acceptance**: No stale workflow files remain

#### 1.4.C: Update Workflow Documentation
- **Action**: Update `plans/CI_GITHUB_ACTIONS_STATUS_2026-02-12.md`
- **Agent**: `documentation` (or manual)
- **Effort**: 5 minutes
- **Dependencies**: 1.4.B complete
- **Acceptance**: Documentation reflects current workflow state

**Quality Gate**:
- [ ] No stale workflow files in repository
- [ ] CI status shows only active workflows
- [ ] Documentation updated

---

## Execution Strategy

### Parallel Execution Opportunities

```
┌─────────────────────────────────────────────────────────────────┐
│ Phase 1: P0 CI Stabilization (Parallel Launch)                  │
└─────────────────────────────────────────────────────────────────┘
         │
         ├─[Group A: Immediate Fixes (Parallel)]
         │   ├─ 1.1.A: Fix trailing spaces (code-quality)
         │   ├─ 1.1.B: Fix line length (github-workflows)
         │   ├─ 1.2.A: Diagnose coverage (debugger)
         │   ├─ 1.3.A: Triage Dependabot (code-reviewer + test-runner)
         │   └─ 1.4.A: Verify stale workflows (github-workflows)
         │
         ├─[Group B: Sequential Dependencies]
         │   ├─ 1.1.C: Verify YAML lint (depends: 1.1.A, 1.1.B)
         │   ├─ 1.2.B: Optimize disk space (depends: 1.2.A)
         │   │   └─ 1.2.C: Split coverage jobs (if needed)
         │   ├─ 1.3.B: Fix simple PRs (depends: 1.3.A)
         │   │   ├─ 1.3.C: Evaluate criterion (parallel with 1.3.B)
         │   │   └─ 1.3.D: Merge/close PRs (depends: 1.3.B, 1.3.C)
         │   └─ 1.4.B: Remove stale files (depends: 1.4.A)
         │       └─ 1.4.C: Update docs (depends: 1.4.B)
         │
         └─[Group C: Final Verification (Sequential)]
             ├─ 1.2.D: Verify coverage fix (depends: 1.2.B or 1.2.C)
             └─ Final CI validation (all groups complete)
```

### Agent Assignment Matrix

| Subtask | Primary Agent | Secondary Agent | Parallel? | Est. Time |
|---------|---------------|-----------------|-----------|-----------|
| 1.1.A Fix trailing spaces | code-quality | - | Yes | 5m |
| 1.1.B Fix line length | github-workflows | - | Yes | 10m |
| 1.1.C Verify YAML lint | code-quality | - | No | 2m |
| 1.2.A Diagnose coverage | debugger | - | Yes | 10m |
| 1.2.B Optimize disk space | github-workflows | - | No | 15m |
| 1.2.C Split coverage jobs | github-workflows | - | No | 20m |
| 1.2.D Verify coverage | test-runner | - | No | 30m |
| 1.3.A Triage PRs | code-reviewer | test-runner | Yes | 20m |
| 1.3.B Fix simple PRs | rust-specialist | - | No | 45m |
| 1.3.C Evaluate criterion | rust-specialist | code-reviewer | Yes | 30m |
| 1.3.D Merge/close PRs | github-workflows | - | No | 10m |
| 1.4.A Verify stale workflows | github-workflows | - | Yes | 5m |
| 1.4.B Remove stale files | github-workflows | - | No | 5m |
| 1.4.C Update docs | documentation | - | No | 5m |

### Estimated Timeline

**Group A (Parallel)**: 20 minutes maximum
**Group B (Sequential)**: 60-90 minutes (varies by dependencies)
**Group C (Verification)**: 30-60 minutes

**Total Estimated Time**: 2-3 hours (including agent execution time)

---

## Quality Gates & Success Criteria

### Per-Phase Quality Gates

**Phase 1.1 (YAML Lint)**:
- [ ] `yamllint .github/` passes with zero errors
- [ ] No trailing spaces
- [ ] No line-length violations
- [ ] No new YAML syntax errors

**Phase 1.2 (Coverage)**:
- [ ] Coverage workflow completes successfully
- [ ] Codecov upload succeeds
- [ ] Coverage badge displays correctly
- [ ] Coverage ≥90% maintained

**Phase 1.3 (Dependabot)**:
- [ ] Zero open Dependabot PRs with failing CI
- [ ] All resolved PRs have documented rationale
- [ ] Dependencies updated (if compatible)

**Phase 1.4 (Stale Workflows)**:
- [ ] No stale workflow files
- [ ] Documentation updated
- [ ] GitHub Actions UI shows clean workflow list

### Final Quality Gates (Phase 1 Complete)

- [ ] All GitHub Actions workflows passing on `main` branch
- [ ] Zero YAML lint errors
- [ ] Zero Dependabot PRs with failing CI
- [ ] Coverage workflow passing
- [ ] CI/CD status green across all workflows
- [ ] No stale workflow files
- [ ] Updated GOAP execution plan with results

---

## Execution Commands

### Launch Group A (Parallel)

```bash
# Subtask 1.1.A: Fix trailing spaces
# Agent: code-quality
cargo fmt --version || echo "fmt check not applicable for YAML"
# Manual: sed -i 's/[[:space:]]*$//' .github/workflows/benchmarks.yml

# Subtask 1.1.B: Fix line length
# Agent: github-workflows
# Manual: Edit line 390 to break into multiple lines

# Subtask 1.2.A: Diagnose coverage
# Agent: debugger
gh run list --workflow=coverage.yml --limit 3 --json conclusion,name --jq '.'

# Subtask 1.3.A: Triage Dependabot
# Agent: code-reviewer + test-runner
gh pr list --author app/dependabot --json number,title,state,headRefName

# Subtask 1.4.A: Verify stale workflows
# Agent: github-workflows
ls -la .github/workflows/ && gh workflow list
```

### Commit Strategy

After each task group completion:

```bash
# After 1.1 (YAML Lint)
git add .github/workflows/benchmarks.yml
git commit -m "fix(ci): resolve yamllint errors in benchmarks.yml

- Fix 15 trailing space errors
- Fix line-length violation on line 390
- All yamllint checks now passing"

# After 1.2 (Coverage)
git add .github/workflows/coverage.yml
git commit -m "fix(ci): resolve coverage workflow disk space issue

- Optimize free-disk-space action configuration
- Split coverage into library/workspace jobs
- Coverage workflow now completes successfully"

# After 1.3 (Dependabot)
# Individual commits per PR fix
git commit -m "fix(deps): resolve clippy warnings for dependabot PRs

- Fix clippy warnings introduced by dependency updates
- Unblock Dependabot PRs #266, #267, #268, #269, #270
- Criterion bump #271 deferred to separate migration"

# After 1.4 (Stale Workflows)
git add .github/workflows/ plans/
git commit -m "chore(ci): remove stale workflow files and update docs

- Remove deprecated ci-old.yml (if found)
- Update CI status documentation
- Clean workflow list"
```

---

## Monitoring & Progress Tracking

### Progress Indicators

| Phase | Subtask | Status | Agent | Time | Notes |
|-------|---------|--------|-------|------|-------|
| 1.1.A | Fix trailing spaces | ⏳ Pending | code-quality | 5m | Ready |
| 1.1.B | Fix line length | ⏳ Pending | github-workflows | 10m | Ready |
| 1.1.C | Verify YAML lint | ⏳ Blocked | code-quality | 2m | Dep: 1.1.A, 1.1.B |
| 1.2.A | Diagnose coverage | ⏳ Pending | debugger | 10m | Ready |
| 1.2.B | Optimize disk space | ⏳ Blocked | github-workflows | 15m | Dep: 1.2.A |
| 1.2.C | Split coverage jobs | ⏳ Blocked | github-workflows | 20m | Dep: 1.2.B |
| 1.2.D | Verify coverage | ⏳ Blocked | test-runner | 30m | Dep: 1.2.B/C |
| 1.3.A | Triage Dependabot | ⏳ Pending | code-reviewer | 20m | Ready |
| 1.3.B | Fix simple PRs | ⏳ Blocked | rust-specialist | 45m | Dep: 1.3.A |
| 1.3.C | Evaluate criterion | ⏳ Blocked | rust-specialist | 30m | Dep: 1.3.A |
| 1.3.D | Merge/close PRs | ⏳ Blocked | github-workflows | 10m | Dep: 1.3.B, 1.3.C |
| 1.4.A | Verify stale workflows | ⏳ Pending | github-workflows | 5m | Ready |
| 1.4.B | Remove stale files | ⏳ Blocked | github-workflows | 5m | Dep: 1.4.A |
| 1.4.C | Update docs | ⏳ Blocked | documentation | 5m | Dep: 1.4.B |

### Next Actions (Immediate)

1. **Start Group A** (Parallel launch)
   - Spawn 5 agents simultaneously
   - Wait for all Group A tasks to complete
   - Collect results and hand off to Group B

2. **Execute Group B** (Sequential with dependencies)
   - Process unblocked tasks as dependencies resolve
   - Monitor agent progress
   - Handle any failures or blockages

3. **Verify Group C** (Final validation)
   - Run all CI workflows to confirm fixes
   - Document outcomes
   - Update GOAP plan

---

## Success Metrics

### Quantitative Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| YAML Lint Errors | 16 | 0 | `yamllint .github/` |
| Coverage Workflow | ❌ Failure | ✅ Success | GitHub Actions status |
| Dependabot PRs Failing | 5 | 0 | `gh pr list --author app/dependabot` |
| Active Workflows | 13 (1 stale) | 10-12 (clean) | `gh workflow list` |
| CI Pass Rate | 4/5 (80%) | 5/5 (100%) | CI dashboard |

### Qualitative Outcomes

- **CI Reliability**: All workflows pass consistently
- **Developer Experience**: Clear, actionable CI feedback
- **Dependency Hygiene**: Dependabot PRs processed promptly
- **Documentation**: CI status reflects actual state

---

## Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| YAML fix introduces new errors | Low | Medium | Run yamllint after each change |
| Coverage fix doesn't resolve disk issue | Medium | High | Have fallback: split jobs ready |
| Dependabot PRs have breaking changes | Medium | High | Triage first, defer major versions |
| Removing stale workflow breaks something | Low | Low | Verify no references, test locally |
| Agent coordination issues | Low | Medium | Clear handoff protocols, status tracking |

---

## Handoff Protocol

### Agent → Agent Communication

**From Group A to Group B**:
- Provide: YAML lint pass confirmation, coverage diagnosis results, PR triage report, stale workflow list
- Format: Structured report in `plans/GOAP_HANDOFF_1A_TO_1B.md`

**From Group B to Group C**:
- Provide: Completed fixes, PR resolution status, workflow cleanup confirmation
- Format: Updated progress table in this document

**Final Report**:
- Update: `plans/GOAP_EXECUTION_PLAN_2026-02-12.md` with completion status
- Create: `plans/GOAP_COMPLETION_REPORT_PHASE1_2026-02-12.md`

---

## References

- **Original Plan**: `plans/GOAP_EXECUTION_PLAN_2026-02-12.md`
- **CI Status**: `plans/CI_GITHUB_ACTIONS_STATUS_2026-02-12.md`
- **ADR-023**: `plans/adr/ADR-023-CI-CD-GitHub-Actions-Remediation.md`
- **AGENTS.md**: `/home/do/rust-self-learning-memory/AGENTS.md`

---

*Generated by GOAP Agent - Task Decomposition & Orchestration Plan*
*Next Update: After Group A completion*
