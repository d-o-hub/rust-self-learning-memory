# Git Workflow

## Branch Protection

Direct pushes to `main` are BLOCKED. Always work on a branch.

## Release Workflow

```bash
# 1. Create release branch from main
git checkout main && git pull origin main
git checkout -b release/v0.1.X

# 2. Make changes (version bump, changelog, fixes)
# 3. Verify and commit ALL changes
git status && git diff --stat  # Verify
git add . && git commit -m "chore: release v0.1.X"

# 4. Create tag
git tag -a v0.1.X -m "Release v0.1.X"

# 5. Push branch AND tag
git push origin release/v0.1.X --tags

# 6. Create PR (tag triggers release workflow)
gh pr create --title "chore: release v0.1.X" --body "..."
```

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

## Snapshot Tests

When version changes, snapshot tests need updates:
```bash
cargo insta test --accept
git add memory-cli/tests/snapshots/
```