# Create PR and Fix GitHub Actions

This command creates a PR and fixes all failing GitHub Actions using the GOAP agent orchestrator.

## Usage

```
/pr-fix-actions [options]
```

## Options

- `--base <branch>` - Base branch for PR (default: main)
- `--title <title>` - PR title (optional, auto-generated if not provided)
- `--body <body>` - PR body description (optional)
- `--draft` - Create as draft PR
- `--skip-lint` - Skip linting checks (not recommended)
- `--max-iterations <n>` - Maximum fix iterations (default: 10)

## Examples

```bash
# Basic usage - creates PR from current branch to main
/pr-fix-actions

# Custom base branch
/pr-fix-actions --base develop

# Full PR with custom title/body
/pr-fix-actions --title "feat: Add new feature" --body "This PR adds..."

# Draft PR
/pr-fix-actions --draft
```

## What It Does

1. **Analyzes** current branch changes and GitHub Actions status
2. **Researches** 2025 GitHub best practices if needed
3. **Orchestrates** 2-6 specialized agents:
   - Git agent: commits and pushes changes
   - Lint agent: fixes linting issues (never skips)
   - Test agent: runs and fixes failing tests
   - CI agent: diagnoses and fixes GitHub Actions
   - PR agent: creates/updates PR
4. **Iterates** until all CI checks pass
5. **Updates** progress in `plans/GITHUB_ACTIONS_FIX_PROGRESS.md`

## Best Practices Applied

- Atomic commits with descriptive messages
- Lint fixes only (no feature changes)
- Progressive CI fixing with retries
- Full CI verification before PR merge

## Output

Progress is saved to: `plans/GITHUB_ACTIONS_FIX_PROGRESS.md`
