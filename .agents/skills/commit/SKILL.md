---
name: commit
description: Git commit with enforced quality gates, proper message format, and safe push workflow
---

# Git Commit with Quality Gates

Enforce code quality before every commit.

## Workflow (MANDATORY ORDER)

1. **Quality Gates** (BLOCKING - must pass)
   ```bash
   ./scripts/quality-gates.sh
   ```
   - Validates >90% test coverage
   - Runs all tests with strict warnings
   - Checks security vulnerabilities
   - Verifies code quality standards

   **If quality gates FAIL**: STOP. Fix issues before proceeding.

2. **Check Status**
   ```bash
   git status
   git diff --stat
   ```

3. **Stage Changes**
   ```bash
   git add -p  # Interactive staging for atomic commits
   # OR
   git add <specific-files>
   ```

4. **Create Commit** (use message format below)
   ```bash
   git commit
   ```

5. **Sync with Remote**
   ```bash
   git pull --rebase
   ```

6. **Handle Conflicts** (if any)
   - **STOP - DO NOT AUTO-FIX**
   - Notify user for manual resolution
   - Only proceed after user confirms resolution

7. **Push**
   ```bash
   git push
   ```

## Commit Message Format

```
type(scope): Brief description (50 chars max)

- Why this change was necessary from user perspective
- What problem it solves or capability it enables
- Reference issue/ticket numbers if applicable
```

### Commit Types
| Type | Purpose |
|------|---------|
| `feat` | New user-facing feature |
| `fix` | Bug fix resolving user issue |
| `docs` | Documentation changes |
| `refactor` | Code restructure (no user-facing changes) |
| `perf` | Performance improvement |
| `test` | Test additions/changes |
| `chore` | Build/dependency updates |

### Message Rules
- Subject: 50 chars max, capitalized, no period
- Body: Wrap at 72 chars
- Use imperative mood: "Add feature" not "Added feature"
- Explain WHY from user perspective, not WHAT code changed

## Examples

**Good:**
```
feat(search): Add filters to help users find documents faster

Users were spending too much time scrolling through results.
New filters reduce search time by 60% in user testing.

Fixes #123
```

**Bad:**
```
improved stuff
```

## Atomic Commit Principle

- Each commit = ONE logical change
- Commit must compile and pass tests
- Use `git add -p` for partial staging

## Optional Pre-Push Checks

For larger changes:
```bash
./scripts/check-doctests.sh
./scripts/check_performance_regression.sh
```

## References

- [AGENTS.md - Required Checks Before Commit](../../../AGENTS.md)
- [commit command](../../../.opencode/command/commit.md)
