# Agent Coding Guidelines

## Quick Reference
- **Build**: `./scripts/build-rust.sh dev|release|check|clean`
- **Quality**: `./scripts/code-quality.sh fmt|clippy|audit|check`
- **Tests**: `cargo nextest run --all` (doctests: `cargo test --doc`)
- **Quality Gates**: `./scripts/quality-gates.sh`

## Project Overview
Memory system: Rust/Tokio + Turso + redb + embeddings (OpenAI/Cohere/Ollama/local)

**Crates**: memory-core, memory-storage-turso, memory-storage-redb, memory-mcp, memory-cli, test-utils, benches

## Skill + CLI Pattern (CRITICAL)
Always use Skill + CLI first for high-frequency ops:
| Operation | Skill | CLI |
|-----------|-------|-----|
| Build | `build-rust` | `./scripts/build-rust.sh` |
| Format/Lint | `code-quality` | `./scripts/code-quality.sh` |
| Tests | `test-runner` | `cargo nextest run --all` |
| Debug | `debug-troubleshoot` | - |

Before task tool: skill? → script? → Skill+CLI? → task tool?

## Change Workflow
1. Identify owner crate + module
2. Read existing patterns
3. Add/update tests
4. `./scripts/code-quality.sh fmt`
5. `cargo clippy --all -- -D warnings`
6. `cargo nextest run -p <crate>`
7. `cargo nextest run --all`
8. `./scripts/quality-gates.sh`

## Required Checks Before Commit
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --tests -- -D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used`
- [ ] `cargo build --all`
- [ ] `cargo nextest run --all`
- [ ] `./scripts/quality-gates.sh`
- [ ] `git status` - verify all changes are staged

## Git Workflow (CRITICAL)

**Branch Protection**: Direct pushes to `main` are BLOCKED. Always work on a branch.

### Release Workflow
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

### Post-Change Verification
After making changes, ALWAYS run:
```bash
git status      # Check for unstaged changes
git diff --stat # Review what changed
```

### Common Fixes
- **Local main ahead of origin**: `git reset --hard origin/main` after creating branch
- **Uncommitted changes**: Check `git status` before switching branches

## Core Invariants (Never Break)
- **Async**: Tokio everywhere. No blocking (use `spawn_blocking`)
- **Storage**: Parameterized SQL only. Short transactions. No locks across `.await`
- **Serialization**: Postcard required (not bincode)
- **Clippy**: Zero warnings (`-D warnings`). Fix, don't suppress
- **Files**: ≤500 LOC per source file
- **Tests**: ≥90% coverage. `#[tokio::test]` for async. AAA pattern

## Feature Flags
- `openai`, `local-embeddings`, `turso`, `redb`, `embeddings-full`, `full`

## Security
- Use env vars (never hardcode)
- Parameterized SQL
- Validate inputs at API boundaries

## Environment Variables
`TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`, `OPENAI_API_KEY`, `RUST_LOG`
See `.env.example`. Never commit secrets.

## Performance Targets
- Episode Creation: < 50ms | Step Logging: < 20ms
- Episode Completion: < 500ms | Memory Retrieval: < 100ms

## Cross-References
| Topic | Document |
|-------|----------|
| Build | `agent_docs/building_the_project.md` |
| Tests | `agent_docs/running_tests.md` |
| Code style | `agent_docs/code_conventions.md` |
| Token efficiency | `agent_docs/token_efficiency.md` |
| CI guidance | `agent_docs/ci_guidance.md` |
| Dependency upgrades | `agent_docs/dependency_upgrades.md` |
| GH Actions patterns | `agent_docs/github_actions_patterns.md` |
| Session state | `agent_docs/session_state_preservation.md` |
| Context compaction | Use `context-compaction` skill |
| Architecture | `plans/adr/` |
| Active roadmap | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |

## Disk Space
- Dev profile: `debug = "line-tables-only"`, deps `debug = false`
- Linker: Use `mold` on Linux
- Cleanup: `cargo clean` periodically
- Monitor: `cargo tree -d | grep -cE "^[a-z]"` for duplicate deps