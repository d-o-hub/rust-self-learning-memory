# Git Workflow

## Atomic Change Rules

1. **One change per commit** - message describes exactly what changed
2. **Workflow**: make change → test → quality check → verify → commit
3. **Format**: `feat(module): description`, `fix(module): description`
4. Never batch incomplete work

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

### Recommended: Use `cargo release` (ADR-034)
```bash
# Handles version bump + commit + tag atomically
cargo release patch  # or minor/major
```

### Manual Release Process
```bash
# 1. Create release branch from main
git checkout main && git pull origin main
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

# 6. Create tag (only after version bump committed)
git tag -a v0.1.X -m "Release v0.1.X"

# 7. Push branch AND tag
git push origin release/v0.1.X --tags

# 8. Create PR (CI will run, tag triggers release workflow after merge)
gh pr create --title "chore: release v0.1.X" --body "..."
```

### Common Release Failure: Version Mismatch

**v0.1.22 Incident**: Tag was pushed with `Cargo.toml` still at `0.1.21`. cargo-dist failed:
```
× This workspace doesn't have anything for dist to Release!
help: --tag=v0.1.21 will Announce: memory-mcp, memory-cli
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
| Local main ahead of origin | Commits made on main locally | `git reset --hard origin/main` after creating branch |
| Uncommitted changes | Forgot to check status | Always run `git status` before/after changes |
| cargo-dist: "nothing to Release" | Tag version != Cargo.toml version | Bump version BEFORE pushing tag |

## Snapshot Tests

When version changes, snapshot tests need updates:
```bash
cargo insta test --accept
git add memory-cli/tests/snapshots/
```