---
name: git-worktree-manager
description: Manage git worktrees for efficient multi-branch development. Use when creating worktrees for feature branches, organizing worktree directories, cleaning up unused worktrees, or implementing worktree-based workflows.
---

# Git Worktree Manager

Manage git worktrees to enable parallel multi-branch development while keeping the main working directory clean.

## Recommended Directory Structure

```
project/
├── main-repo/          # Main working directory
└── worktrees/          # All worktrees here
    ├── feature/name/
    ├── bugfix/name/
    └── experiment/name/
```

## Core Commands

### Create Worktree
```bash
# From existing branch
git worktree add ../worktrees/feature-name feature-branch

# Create new branch
git worktree add -b feature/new ../worktrees/new-feature

# Detached HEAD (specific commit)
git worktree add ../worktrees/temp HEAD~5
```

### List Worktrees
```bash
git worktree list
git worktree list --porcelain  # Machine-readable
```

### Remove Worktree
```bash
git worktree remove ../worktrees/name        # Safe (clean only)
git worktree remove --force ../worktrees/name  # Force removal
git worktree prune                           # Clean administrative data
```

### Lock/Unlock
```bash
git worktree lock ../worktrees/name    # Prevent operations
git worktree unlock ../worktrees/name
```

## Workflow Patterns

### Feature Development
```bash
git worktree add -b feature/new ../worktrees/feature/new
cd ../worktrees/feature/new
# ... develop ...
cargo test  # Test in isolation
git push
cd /path/to/main && git merge feature/new
git worktree remove ../worktrees/feature/new
```

### Parallel Testing
```bash
for branch in feature/a feature/b; do
  git worktree add ../worktrees/$branch $branch
  cd ../worktrees/$branch && cargo test &
done
wait
```

### Code Review
```bash
git worktree add ../worktrees/review/pr-123 origin/pr-123
cd ../worktrees/review/pr-123
cargo test
git worktree remove ../worktrees/review/pr-123
```

## Find Orphaned Worktrees
```bash
git worktree list | while read path commit branch; do
  if ! git show-ref --verify --quiet "refs/heads/${branch#*/}"; then
    echo "Orphaned: $path"
  fi
done
```

## Best Practices

### DO
- Organize in `../worktrees/` directory
- Use descriptive branch-based names
- Clean up after merging
- Test before merging to main
- Prune regularly to remove orphaned data

### DON'T
- Create in random locations
- Leave with uncommitted changes
- Forget to prune administrative data
- Create conflicting branches
- Mix worktree/non-worktree workflows