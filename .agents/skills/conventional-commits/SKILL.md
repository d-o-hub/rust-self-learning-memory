---
name: conventional-commits
description: Create Conventional Commits formatted git commits. Use when committing code to ensure consistent commit messages following the Conventional Commits specification.
---

# Conventional Commits

Create properly formatted commit messages following [Conventional Commits](https://www.conventionalcommits.org/) specification.

## Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

## Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `style` | Code style (formatting, semicolons) |
| `refactor` | Code refactoring |
| `test` | Adding or updating tests |
| `chore` | Maintenance, dependencies, tooling |
| `perf` | Performance improvements |
| `ci` | CI/CD changes |
| `build` | Build system or dependencies |

## Examples

```
feat(memory): add episodic memory retrieval
fix(storage): resolve Turso connection timeout
docs(api): update function signatures
chore(deps): update tokio version
refactor(core): simplify episode lifecycle
test(mcp): add integration tests for tool registry
perf(retrieval): optimize embedding similarity search
build(cargo): update workspace dependencies
```

## Workflow

1. Check staged changes: `git status`
2. Review diff: `git diff --staged`
3. Check recent commits for style: `git log --oneline -10`
4. Stage files: `git add <files>`
5. Commit with conventional format: `git commit -m "type(scope): description"`

## Best Practices

- Use imperative mood: "add feature" not "added feature"
- Keep subject line under 72 characters
- Scope is optional but recommended (e.g., memory, storage, mcp)
- Body explains "what" and "why", not "how"
- Footer for breaking changes: `BREAKING CHANGE: description`
