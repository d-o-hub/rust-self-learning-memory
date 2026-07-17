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

**CRITICAL**: Version in `Cargo.toml` MUST match the tag before pushing. cargo-dist requires this.

### Recommended: Use the release manager
```bash
# Cargo.toml must already contain the intended release version. Finalize the
# changelog and released-version documents before running this command.
./scripts/release-manager.sh full --execute
```

The manager validates metadata and quality before creating the matching tag.
It does not increment the version during release preparation. If the release
cadence gate is already blocking ordinary PRs, a maintainer may apply the
`release-preparation` label to the release PR; the tag must be pushed
immediately after that PR merges.

### Manual Release Process
```bash
# 1. Create release branch from main
git checkout main
git checkout -b release/v0.1.X

# 2. BUMP VERSION FIRST in Cargo.toml
#    workspace.package.version = "0.1.X"

# 3. Update snapshot tests if needed
cargo insta test --accept
git add memory-cli/tests/snapshots/

# 4. Verify version matches intended tag
grep '^version =' Cargo.toml  # Must show 0.1.X for tag v0.1.X

# 5. Commit changes
git add . && git commit -m "chore: release v0.1.X"

# 6. Push only the release branch and create the PR
git push origin release/v0.1.X
gh pr create --title "chore: release v0.1.X" --body "..."

# 7. Merge the PR and wait for every required check on main to pass

# 8. Update local main, then create and push only the intended tag
git checkout main
git pull --ff-only origin main
./scripts/release-manager.sh full --execute
```

### Common Release Failure: Version Mismatch

**v0.1.22 Incident**: Tag was pushed with `Cargo.toml` still at `0.1.21`. cargo-dist failed:
```
× This workspace doesn't have anything for dist to Release!
help: --tag=v0.1.21 will Announce: do-memory-mcp, do-memory-cli
```

**Prevention**: Always verify `grep '^version =' Cargo.toml` matches tag (without 'v' prefix) before pushing tag.

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
