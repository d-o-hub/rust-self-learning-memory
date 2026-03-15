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
9. `git status` - verify all changes staged

## Core Invariants (Never Break)
- **Async**: Tokio everywhere. No blocking (use `spawn_blocking`)
- **Storage**: Parameterized SQL only. Short transactions. No locks across `.await`
- **Serialization**: Postcard required (not bincode)
- **Clippy**: Zero warnings (`-D warnings`). Fix, don't suppress
- **Files**: ≤500 LOC per source file
- **Tests**: ≥90% coverage. `#[tokio::test]` for async. AAA pattern

## Common Pitfalls (from Session Analysis)

Based on analysis of 34 Claude Code sessions (234 messages, 97 commits):

| Pitfall | Count | Prevention |
|---------|-------|------------|
| wrong_approach | 8 | Read existing patterns before implementing |
| buggy_code | 6 | Run tests after every change |
| excessive_changes | 5 | Use atomic commits |
| tool_errors | 67 | Use correct tool for task |

**Before implementing**:
1. Read at least 3 related source files
2. Identify existing patterns to follow
3. Check ADRs for relevant decisions
4. Consider clarification if uncertain

## Tool Selection Enforcement

Target Bash:Grep ratio of 2:1 (current: 17:1)

**Use Grep for**:
- Finding files: `Grep pattern="*.rs"`
- Searching content: `Grep pattern="fn name"`
- Finding definitions: `Grep pattern="struct Name"`
- Checking usage: `Grep pattern="use crate"`

**Use Bash for**:
- File operations: `cp`, `mv`, `rm`
- Git commands: `git status`, `git diff`
- Running scripts: `./scripts/build-rust.sh`
- Build/test: `cargo build`, `cargo test`

**Before Bash**: Consider if Grep would be more efficient.

## Atomic Change Rules

1. **One logical change per commit**
   - Commit message describes exactly what changed
   - `git diff --stat` shows focused changes
   - No unrelated modifications

2. **Commit workflow**:
   ```
   1. Make focused change
   2. Run tests: cargo nextest run --all
   3. Run quality: ./scripts/code-quality.sh check
   4. Verify: git status && git diff --stat
   5. Commit: git commit -m "scope: description"
   ```

3. **Commit message format**:
   - `feat(module): add feature`
   - `fix(module): fix bug`
   - `docs: update documentation`
   - `refactor(module): improve code`
   - `test(module): add tests`

4. **Never batch incomplete work** - each commit should be a complete, working change

## Required Checks Before Commit
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --tests -- -D warnings`
- [ ] `cargo build --all`
- [ ] `cargo nextest run --all`
- [ ] `git status` - verify all changes staged

## Git Workflow
- **Branch Protection**: Direct pushes to `main` BLOCKED. Always work on a branch.
- **Post-Change**: ALWAYS run `git status` and `git diff --stat`
- See `agent_docs/git_workflow.md` for release workflow and common fixes

## Feature Flags
`openai`, `local-embeddings`, `turso`, `redb`, `embeddings-full`, `full`

## Security
- Use env vars (never hardcode)
- Parameterized SQL
- Validate inputs at API boundaries

## Environment Variables
`TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`, `OPENAI_API_KEY`, `RUST_LOG`

## Performance Targets
- Episode Creation: < 50ms | Step Logging: < 20ms
- Episode Completion: < 500ms | Memory Retrieval: < 100ms

## Cross-References
| Topic | Document |
|-------|----------|
| Build | `agent_docs/building_the_project.md` |
| Tests | `agent_docs/running_tests.md` |
| Code style | `agent_docs/code_conventions.md` |
| Git workflow | `agent_docs/git_workflow.md` |
| CI guidance | `agent_docs/ci_guidance.md` |
| Dependency upgrades | `agent_docs/dependency_upgrades.md` |
| GH Actions patterns | `agent_docs/github_actions_patterns.md` |
| Architecture | `plans/adr/` |
| Active roadmap | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |

## Disk Space
- Dev profile: `debug = "line-tables-only"`, deps `debug = false`
- Linker: Use `mold` on Linux
- Cleanup: `cargo clean` periodically