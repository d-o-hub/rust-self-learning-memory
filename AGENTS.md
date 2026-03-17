# Agent Coding Guidelines (Simplified)

## Quick Reference
- **Build**: `./scripts/build-rust.sh dev|release|check|clean`
- **Quality**: `./scripts/code-quality.sh fmt|clippy|audit|check`
- **Tests**: `cargo nextest run --all` (doctests: `cargo test --doc`)
- **Quality Gates**: `./scripts/quality-gates.sh`

## Core Invariants
- **Async**: Tokio everywhere. No blocking across `.await`.
- **Storage**: Parameterized SQL only. Postcard for serialization.
- **Compliance**: ≤500 LOC per source file. Zero clippy warnings.
- **Tests**: ≥90% coverage. AAA pattern.

## Workflow Rules
- **Understand**: Read existing patterns before implementation.
- **Verify**: Run tests after *every* change.
- **Atomic**: One logical change per commit.
- **Tools**: Prefer Grep for search, Bash for operations (target 2:1 ratio).

## Required Checks
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --tests -- -D warnings`
- [ ] `cargo build --all`
- [ ] `cargo nextest run --all`
- [ ] `git status` - verify all changes staged

## Cross-References
| Topic | Document |
|-------|----------|
| **Primary Reference** | `agent_docs/README.md` |
| Build & Install | `agent_docs/building_the_project.md` |
| Testing Strategy | `agent_docs/running_tests.md` |
| Code Conventions | `agent_docs/code_conventions.md` |
| Git & Releases | `agent_docs/git_workflow.md` |
| Architecture | `agent_docs/service_architecture.md` |
| Database Schema | `agent_docs/database_schema.md` |
| Security Patterns | `agent_docs/security_patterns.md` |
| Common Pitfalls | `agent_docs/common_friction_points.md` |
| Token Efficiency | `agent_docs/token_efficiency.md` |
| ADRs | `plans/adr/` |
| Roadmap | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |

## Recent Updates (v0.1.22)
- Added Recommendation Feedback & Checkpoints (ADR-044).
- Added `memory-cli storage check` for consistency verification.
- Enforced strict 500 LOC limit across all production crates.
