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

## Common Pitfalls
Based on 34 sessions (234 msgs, 97 commits):

| Pitfall | Prevention |
|---------|------------|
| wrong_approach | Read patterns first |
| buggy_code | Run tests after change |
| excessive_changes | Atomic commits |
| tool_errors | Use correct tool |

Before implementing: Read 3+ source files, check ADRs

## Planning & Decisions
- **Use `goap-agent` skill** for complex tasks - decomposes into atomic goals
- **Check `plans/adr/`** for Architecture Decision Records before changes
- **Update `plans/ROADMAPS/ROADMAP_ACTIVE.md`** with progress

See `plans/` folder for ADRs and roadmap.

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
1. **One change per commit** - message describes exactly what changed
2. **Workflow**: make change → test → quality check → verify → commit
3. **Format**: `feat(module): description`, `fix(module): description`
4. Never batch incomplete work

## Required Checks Before Commit
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --tests -- -D warnings`
- [ ] `cargo build --all`
- [ ] `cargo nextest run --all`
- [ ] `git status` - verify all changes staged

## Git Workflow
- **Branch Protection**: Direct pushes to `main` BLOCKED. Always work on a branch.
- See `agent_docs/git_workflow.md` for details.

## Feature Flags
`openai`, `local-embeddings`, `turso`, `redb`, `embeddings-full`, `full`

## Security
- Use env vars (never hardcode)
- Parameterized SQL

## Environment Variables
`TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`, `OPENAI_API_KEY`, `RUST_LOG`

**Local Development**: Set `TURSO_DATABASE_URL="http://127.0.0.1:8080"` and leave `TURSO_AUTH_TOKEN` empty when using `turso dev`.

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
| Dependencies | `agent_docs/dependency_upgrades.md` |
| GH Actions | `agent_docs/github_actions_patterns.md` |
| Architecture | `agent_docs/service_architecture.md` |
| Database | `agent_docs/database_schema.md` |
| Patterns | `agent_docs/service_communication_patterns.md` |
| Friction points | `agent_docs/common_friction_points.md` |
| Token efficiency | `agent_docs/token_efficiency.md` |
| Planning | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| ADRs | `plans/adr/` |

## Disk Space
- Dev profile: `debug = "line-tables-only"`, deps `debug = false`
- Linker: Use `mold` on Linux