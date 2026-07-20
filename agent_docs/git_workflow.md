# Git Workflow

## Atomic Change Rules

1. **One change per commit** - message describes exactly what changed
2. **Workflow**: make change → test → quality check → verify → commit
3. **Format**: `feat(module): description`, `fix(module): description`
4. Never batch incomplete work
5. **Commit Frequency**: Commit after each atomic change. If session >2 hours or >3 files modified without commit, pause and commit.

### Commit Message Format

```
feat(module): add feature
fix(module): fix bug
docs: update documentation
refactor(module): improve code
test(module): add tests
```

## Branch Protection

Direct pushes to `main` are BLOCKED. Always work on a branch.

## Release Workflow

**Canonical skill:** `.agents/skills/release-guard/SKILL.md`  
**Canonical CLI:** `./scripts/release-manager.sh ship --execute`  
**GitHub Release creator:** `.github/workflows/release.yml` (tag push only)

**CRITICAL**: Version in `Cargo.toml` MUST match the tag (`vX.Y.Z` ↔ `X.Y.Z`). cargo-dist preflight enforces this. Tags must point at **main** history (R-H5).

### Ship (only path)

```bash
# 1. Version + CHANGELOG + Released Version docs already on main (via PR)
git checkout main && git pull --ff-only origin main

# 2. Preflight
./scripts/release-manager.sh status
./scripts/verify-release-state.sh --check-unreleased

# 3. Tag + push tag only (triggers release.yml)
./scripts/release-manager.sh ship --execute

# 4. Observe
gh run list --workflow=release.yml --limit 3
gh release view "v$(grep -E '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
```

**Forbidden for shipping:**

```bash
gh release create …   # bypasses cargo-dist multi-platform artifacts
```

After tag, bump workspace to the next patch on a follow-up PR so release-drift stays clean.

### Common Release Failure: Version Mismatch

Tag pushed while `Cargo.toml` still at an older version → cargo-dist preflight fails.  
**Prevention**: `grep '^version =' Cargo.toml` must match intended tag (without `v`).

## Post-Change Verification

After making changes, ALWAYS run:
```bash
git status      # Check for unstaged changes
git diff --stat # Review what changed
```

## Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| "push declined due to repository rule violations" | Direct push to main | Create branch, PR, merge |
| `.snap.new` files not cleaned up | insta test run without acceptance | `cargo insta test --accept` then commit |
| Local main ahead of origin | Commits made on main locally | create a branch from current state, then reconcile with the user before any destructive cleanup |
| Uncommitted changes | Forgot to check status | Always run `git status` before/after changes |
| cargo-dist: "nothing to Release" | Tag version != Cargo.toml version | Bump version BEFORE pushing tag |

## Snapshot Tests

When version changes, snapshot tests need updates:
```bash
cargo insta test --accept
git add memory-cli/tests/snapshots/
```
