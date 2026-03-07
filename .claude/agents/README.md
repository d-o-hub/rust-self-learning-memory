# Claude Code Agents Reference

Quick reference for selecting the right agent based on task needs.

## Token-Efficient Agents (New)

| Agent | Use Case | Tools |
|-------|----------|-------|
| `quick-check` | Fast CI-parity validation (fmt + clippy) | Bash, Read |
| `cargo-check` | Minimal compilation check | Bash |
| `rust-lsp` | Code navigation via LSP | LSP, Read |
| `file-search` | Find files by name/content | Glob, Grep |
| `deps-check` | Dependency health | Bash |
| `git-status` | Repository state | Bash, Read |
| `code-reader` | Targeted file reading | Read |
| `test-quick` | Targeted test runs | Bash, Read |
| `plan-mode` | Read-only planning | Read, Glob, Grep, LSP |
| `team-coordinator` | Multi-agent orchestration | Task, Team* |

## Comprehensive Agents (Existing)

| Agent | Use Case | Tools |
|-------|----------|-------|
| `test-runner` | Full test suite, failure diagnosis | Bash, Read, Grep, Edit |
| `build-compile` | Full builds, release optimization | Bash, Read, Grep, Edit |
| `code-quality` | Full quality checks + fixes | Bash, Read, Grep, Edit |
| `code-reviewer` | Comprehensive code review | Read, Glob, Grep, Bash |
| `debugger` | Runtime issue diagnosis | Read, Bash, Grep, Edit |
| `refactorer` | Code restructuring | Read, Edit, Bash, Grep, Glob |
| `feature-implementer` | Full feature implementation | Read, Write, Edit, Bash, Glob, Grep |

## Agent Selection Guide

### By Task Type

```
Quick validation    → quick-check
Find files          → file-search
Navigate code       → rust-lsp
Read specific code  → code-reader
Check compilation   → cargo-check
Run specific tests  → test-quick
Check dependencies  → deps-check
Check git state     → git-status
Plan implementation → plan-mode
Coordinate teams    → team-coordinator

Full test suite     → test-runner
Full build          → build-compile
Quality + fixes     → code-quality
Code review         → code-reviewer
Debug issues        → debugger
Refactor code       → refactorer
New feature         → feature-implementer
```

### By Tool Access

| Tools | Agents |
|-------|--------|
| Bash only | `cargo-check`, `deps-check` |
| Glob + Grep | `file-search` |
| Read only | `code-reader` |
| LSP + Read | `rust-lsp` |
| Read-only (multiple) | `quick-check`, `git-status`, `plan-mode` |
| Full edit access | `test-runner`, `debugger`, `refactorer`, `feature-implementer` |

## Token Efficiency Tips

1. **Use Glob/Grep tools directly** before spawning agents
2. **Prefer `quick-check`** over `code-quality` for validation-only
3. **Use `cargo-check`** during development, `build-compile` for releases
4. **Use `rust-lsp`** for navigation instead of reading entire files
5. **Use `file-search`** to find locations, then read only what's needed
6. **Use `plan-mode`** to design before implementing

## Agent Teams

For complex multi-agent tasks:

1. Use `team-coordinator` to orchestrate
2. Create team with `TeamCreate`
3. Add tasks with `TaskCreate`
4. Spawn teammates with `Task` (set `team_name`)
5. Monitor with `TaskList`
6. Clean up with `TeamDelete`

## Files

- Agent definitions: `.claude/agents/*.md`
- Project guidelines: `AGENTS.md`
- Skills: `.claude/skills/*.md`