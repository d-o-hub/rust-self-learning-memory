# GOAP Agent Codebase Verification

- **Last Updated**: 2026-03-06
- **Status**: Active
- **Purpose**: Verification checklist for codebase state before/during GOAP execution

## Pre-Execution Verification

### 1. Repository State

```bash
# Check working directory is clean
git status --porcelain

# Verify current branch
git branch --show-current

# Check for unpushed commits
git log origin/$(git branch --show-current)..HEAD --oneline
```

- [ ] Working directory clean (or intentional changes)
- [ ] On correct branch
- [ ] No unexpected unpushed commits

### 2. Build Verification

```bash
# Quick build check
./scripts/build-rust.sh check

# Full build
./scripts/build-rust.sh dev
```

- [ ] `cargo check --all` passes
- [ ] No compilation warnings
- [ ] All features compile

### 3. Test Baseline

```bash
# Quick test suite
cargo nextest run --all

# Doctests
cargo test --doc
```

- [ ] All tests pass
- [ ] No flaky tests observed
- [ ] Coverage baseline established

### 4. Quality Gates

```bash
# Run quality gates
./scripts/quality-gates.sh
```

- [ ] Format check passes
- [ ] Clippy passes with no warnings
- [ ] File size gate passes
- [ ] GOAP checks pass (non-blocking)

## During Execution Verification

### After Each Commit

```bash
# Verify commit
git log -1 --stat

# Quick validation
./scripts/build-rust.sh check
```

- [ ] Commit message follows convention
- [ ] Build still passes
- [ ] No unintended changes

### Before Push

```bash
# Full validation
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy
cargo nextest run --all
./scripts/quality-gates.sh
```

- [ ] All quality checks pass
- [ ] Tests pass locally
- [ ] Documentation updated

### After Push

```bash
# Monitor CI
gh pr view --json statusCheckRollup,mergeStateStatus

# Check specific workflow
gh run list --limit 5
```

- [ ] CI workflows triggered
- [ ] Required checks attached
- [ ] No unexpected failures

## Post-Execution Verification

### 1. CI Validation

```bash
# Wait for CI and check status
gh pr view [PR_NUMBER] --json statusCheckRollup,mergeStateStatus

# Verify all checks
gh pr checks [PR_NUMBER]
```

- [ ] All required checks pass
- [ ] No blocking failures
- [ ] Coverage maintained or improved

### 2. State File Updates

- [ ] `GOAP_STATE.md` updated
- [ ] `GOALS.md` reflects completion
- [ ] `ACTIONS.md` updated
- [ ] Learning delta captured

### 3. Documentation Sync

```bash
# Check docs integrity
./scripts/check-docs-integrity.sh
```

- [ ] No broken links
- [ ] Version references current
- [ ] ADR cross-references valid

## Verification Commands Quick Reference

| Check | Command | Expected |
|-------|---------|----------|
| Clean state | `git status --porcelain` | Empty output |
| Build | `./scripts/build-rust.sh check` | Success |
| Tests | `cargo nextest run --all` | All pass |
| Format | `cargo fmt --all -- --check` | No output |
| Clippy | `cargo clippy --all -- -D warnings` | No warnings |
| Quality | `./scripts/quality-gates.sh` | All pass |
| CI Status | `gh pr view --json statusCheckRollup` | All green |

## Common Issues & Resolution

### Issue: CI checks not attaching

**Symptoms**: Empty `statusCheckRollup` after push

**Resolution**:
1. Verify workflow triggers match push event
2. Check workflow file syntax
3. Wait 30-60 seconds for check creation
4. Use `gh run list` to verify workflow started

### Issue: Tests pass locally but fail in CI

**Symptoms**: Local green, CI red

**Resolution**:
1. Check for environment-specific code
2. Verify all test dependencies installed
3. Look for timing-dependent tests
4. Check CI environment variables

### Issue: Quality gates fail unexpectedly

**Symptoms**: Gates fail after seemingly unrelated changes

**Resolution**:
1. Run `./scripts/code-quality.sh fmt` first
2. Check for new clippy warnings
3. Verify file size limits
4. Review GOAP check warnings

## Verification Checklist Template

```markdown
## Pre-Execution
- [ ] Repository clean
- [ ] Build passes
- [ ] Tests pass
- [ ] Quality gates pass

## During Execution
- [ ] Commits follow convention
- [ ] Build passes after each commit
- [ ] Tests pass before push

## Post-Execution
- [ ] CI checks pass
- [ ] State files updated
- [ ] Documentation synced
- [ ] Learning captured
```