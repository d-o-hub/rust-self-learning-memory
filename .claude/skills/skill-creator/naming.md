# Skill Naming Rules

## Requirements

| Rule | Description |
|------|-------------|
| Lowercase | Letters only (a-z) |
| Numbers | Allowed (0-9) |
| Hyphens | Word separation only |
| No underscores | Not allowed |
| No spaces | Not allowed |
| Max length | 64 characters |
| Descriptive | Clear purpose |

## Valid Examples

- `episode-management`
- `test-debugging`
- `api-integration`
- `rust-async-testing`
- `github-workflows-advanced`

## Invalid Examples

| Name | Problem |
|------|---------|
| `Episode_Management` | Uppercase + underscore |
| `test debugging` | Spaces |
| `TEST-DEBUGGING` | Uppercase |
| `test_debugging` | Underscore |

## Project Naming Conventions

| Category | Pattern | Examples |
|----------|---------|----------|
| Episode | `episode-[operation]` | `episode-start`, `episode-complete` |
| Storage | `storage-[operation]` | `storage-sync` |
| Pattern | `pattern-[operation]` | `pattern-extraction` |
| Memory | `memory-[operation]` | `memory-retrieval` |
| Testing | `[type]-testing` | `rust-async-testing`, `episodic-memory-testing` |
| Quality | `quality-[area]` | `quality-unit-testing` |
| Build | `build-[target]` | `build-compile` |
| Debug | `debug-[area]` | `debug-troubleshoot` |
| Code | `code-[action]` | `code-quality`, `code-review` |

## Validation Command

```bash
# Check skill name format
SKILL_NAME="my-skill"

if [[ "$SKILL_NAME" =~ ^[a-z0-9-]+$ ]]; then
    echo "✓ Valid: $SKILL_NAME"
else
    echo "✗ Invalid: $SKILL_NAME"
    echo "  Use lowercase letters, numbers, and hyphens only"
fi
```
