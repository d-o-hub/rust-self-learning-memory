# Agent Coding Guidelines

## Quick Reference
- **Build**: `./scripts/build-rust.sh dev|release|check|clean`
- **Quality**: `./scripts/code-quality.sh fmt|clippy|audit|check`
- **Tests**: `cargo nextest run --all` (doctests: `cargo test --doc`)
- **Quality Gates**: `./scripts/quality-gates.sh`
- **Disk Cleanup**: `./scripts/clean-artifacts.sh [quick|standard|full] [--node-modules]`

## Project Overview
Memory system: Rust/Tokio + Turso + redb + embeddings (OpenAI/Cohere/Ollama/local)

**Crates**: do-memory-core, do-memory-storage-turso, do-memory-storage-redb, do-memory-mcp, do-memory-cli, do-memory-test-utils, benches

## Skill + CLI Pattern (CRITICAL)
Always use Skill + CLI first for high-frequency ops:
| Operation | Skill | CLI |
|-----------|-------|-----|
| Build | `build-rust` | `./scripts/build-rust.sh` |
| Format/Lint | `code-quality` | `./scripts/code-quality.sh` |
| Tests | `test-runner` | `cargo nextest run --all` + `cargo test --doc` |
| Debug | `debug-troubleshoot` | - |

Before task tool: skill? → script? → Skill+CLI? → task tool?

## Change Workflow
1. Identify owner crate + module
2. Read existing patterns
3. Add/update tests
4. `./scripts/code-quality.sh fmt`
5. `./scripts/code-quality.sh clippy --workspace`
6. `cargo nextest run -p <crate>`
7. `cargo nextest run --all`
8. `cargo test --doc`
9. `./scripts/quality-gates.sh` (coverage threshold is `QUALITY_GATE_COVERAGE_THRESHOLD`, default 90)
10. `git status` - verify all changes staged

## Core Invariants (Never Break)
- **Async**: Tokio everywhere. No blocking (use `spawn_blocking`)
- **Storage**: Parameterized SQL only. Short transactions. No locks across `.await`
- **Serialization**: Postcard required (not bincode)
- **Clippy**: Zero warnings (`-D warnings`). Fix, don't suppress
- **Files**: ≤500 LOC per source file
- **Tests**: ≥90% coverage. `#[tokio::test]` for async. AAA pattern
- **Docs**: URLs wrapped in `<...>`. New types re-exported from `lib.rs`

## Documentation Rules
- **Bare URLs**: Wrap all URLs in angle brackets: `<https://example.com>`
- **Re-exports**: Add new public types to `lib.rs` re-exports for doctest imports
- **Check**: Run `cargo doc --no-deps --document-private-items` before commit

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
- Running scripts: `./scripts/*.sh`
- Running workspace tests: `cargo nextest run --all`, `cargo test --doc`

**Before Bash**: Consider if Grep would be more efficient.

## Atomic Change Rules
1. **One change per commit** - message describes exactly what changed
2. **Workflow**: make change → test → quality check → verify → commit
3. **Format**: `feat(module): description`, `fix(module): description`
4. Never batch incomplete work

## Required Checks Before Commit
- [ ] `./scripts/code-quality.sh fmt`
- [ ] `./scripts/code-quality.sh clippy --workspace`
- [ ] `./scripts/build-rust.sh check`
- [ ] `cargo nextest run --all`
- [ ] `cargo test --doc`
- [ ] `cargo doc --no-deps --document-private-items` (catches bare URLs)
- [ ] `./scripts/quality-gates.sh` (coverage must be `>=90%`, unless threshold explicitly raised)
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
| Disk hygiene | `agent_docs/disk_hygiene.md` |
| Token efficiency | `agent_docs/token_efficiency.md` |
| Planning | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| ADRs | `plans/adr/` |

## Disk Space
- Dev profile: `debug = "line-tables-only"`, deps `debug = false`
- Default artifact path: `target/` (or `$CARGO_TARGET_DIR` when set)
- For external disk/offload, set `CARGO_TARGET_DIR` (for example: `CARGO_TARGET_DIR=/mnt/fastssd/rslm-target`)
- Use `./scripts/clean-artifacts.sh standard` for routine cleanup
- Use `./scripts/clean-artifacts.sh standard --node-modules` only when JS dependencies are not needed locally
