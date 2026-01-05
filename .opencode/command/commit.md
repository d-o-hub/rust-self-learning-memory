---
description: git commit and push
subtask: true
---

commit and push

## Commit Message Structure

Use this format for commits:
```
type(scope): Brief description (50 chars max)

- Why this change was necessary from user perspective
- What problem it solves or capability it enables
- Reference issue/ticket numbers if applicable
```

## Commit Types
- **feat**: New user-facing feature or capability
- **fix**: Bug fix that resolves user issue
- **docs**: Documentation changes
- **refactor**: Code restructure (no user-facing changes)
- **perf**: Performance improvement users will notice
- **test**: Test additions/changes
- **chore**: Build/dependency updates

## Key Principles

**1. Atomic Commits**
- Each commit = ONE logical change
- Commit must compile and pass tests
- Use `git add -p` for partial staging if needed

**2. Clear User-Focused Messages**
- Explain WHY from end user perspective, not WHAT code changed
- Bad: "improved agent experience"
- Good: "feat(agent): Enable retry on timeout so users don't lose work"
- Use imperative mood: "Add feature" not "Added feature"

**3. Message Format**
- Subject: 50 chars max, capitalized, no period
- Body: Wrap at 72 chars, explain context and reasoning
- Reference related issues/tickets

## Before Pushing

1. **Run quality gates**: `./scripts/quality-gates.sh`
   - Validates >90% test coverage
   - Runs all tests with strict warnings
   - Checks security vulnerabilities
   - Verifies code quality standards

2. **Optional additional checks** (recommended for larger changes):
   - `./scripts/check-doctests.sh` - Validate documentation tests
   - `./scripts/check_performance_regression.sh` - Ensure no performance regressions

3. **Sync with remote**: `git pull --rebase`
4. **Handle conflicts**: If conflicts occur: **STOP - DO NOT FIX**
   - Notify immediately for manual resolution
5. **Push**: Only after successful rebase with no conflicts

## Examples

Good:
```
feat(search): Add filters to help users find documents faster

Users were spending too much time scrolling through results.
New filters reduce search time by 60% in user testing.

Fixes #123
```

Bad:
```
improved stuff
```