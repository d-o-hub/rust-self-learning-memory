# Agent Documentation Index

Quick reference for AI coding agents working on this project. Start with `AGENTS.md` in the project root for the primary guidance.

## Documents (Reading Order)

| # | Document | Purpose |
|---|----------|---------|
| 1 | `building_the_project.md` | Build commands, feature flags, prerequisites, troubleshooting |
| 2 | `code_conventions.md` | Rust idioms, formatting, naming, error handling, serialization |
| 3 | `running_tests.md` | Test categories, coverage, benchmarks, debugging tests |
| 4 | `service_architecture.md` | System design, crate responsibilities, module breakdown |
| 5 | `database_schema.md` | Turso + redb schemas, tables, indexes, relationships |
| 6 | `service_communication_patterns.md` | Inter-service communication, MCP protocol, async patterns |
| 7 | `common_friction_points.md` | Friction patterns from session analysis, prevention strategies |
| 8 | `disk_hygiene.md` | Disk cleanup workflow, `CARGO_TARGET_DIR`, and coverage artifact hygiene |
| 9 | `token_efficiency.md` | Prompt/token budget guidance for tool and context selection |
| 10 | `ci_guidance.md` | CI triage, required checks, and local parity expectations |
| 11 | `external_signals.md` | External-signal integration and current implementation caveats |
| 12 | `dependency_upgrades.md` | Upgrade workflow and dependency-risk handling |
| 13 | `session_state_preservation.md` | Preserve context across long multi-step sessions |
| 14 | `LESSONS.md` | Verbose workflow learnings paired with distilled AGENTS notes |

## Quick Links

- **Primary guidance**: `../AGENTS.md`
- **Architecture decisions**: `../plans/adr/`
- **Active roadmap**: `../plans/ROADMAPS/ROADMAP_ACTIVE.md`
- **GOAP state**: `../plans/GOAP_STATE.md`
- **Skills reference**: `../.agents/skills/`
- **GOAP skill**: `../.agents/skills/goap-agent/SKILL.md`
- **Agent coordination skill**: `../.agents/skills/agent-coordination/SKILL.md`
- **Scripts**: `../scripts/` (build-rust.sh, code-quality.sh, quality-gates.sh, clean-artifacts.sh, check-docs-integrity.sh, release-manager.sh)
- **Machine-readable context**: `../docs/architecture/context.yaml`

## Version Policy

Do not hardcode version numbers in documentation. Reference `Cargo.toml` workspace version instead.

## CI/PR Operational Note

- After PR remediation pushes, always verify required checks are attached to the latest head SHA via GH CLI.
- If `statusCheckRollup` is empty on a required-check PR, treat it as a blocking condition and record evidence in `../plans/STATUS/VALIDATION_LATEST.md`.

## Workflow Impact Files

- Update root `../AGENTS.md` for distilled coding-workflow changes.
- Update `../agent_docs/LESSONS.md` for verbose non-obvious learnings.
- Update `../plans/ROADMAPS/ROADMAP_ACTIVE.md`, `../plans/GOALS.md`, `../plans/ACTIONS.md`, and `../plans/GOAP_STATE.md` together when sprint goals change.
- Check `../.agents/skills/` when workflow changes affect skill references, invocation order, or compactness.
