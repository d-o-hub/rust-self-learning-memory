# Agent Harness Map

This document maps feedforward guides (instructions) to feedback sensors (verification scripts) for this repository. It serves as the "Source of Truth" for how agents and maintainers should verify their work after different classes of changes.

## Overview

The harness ensures that every instruction provided in a guide has a corresponding sensor that can detect deviations or regressions.

## Mapping: Guides to Sensors

| Category | Feedforward Guides | Feedback Sensors |
| :--- | :--- | :--- |
| **Core Workflow** | `AGENTS.md`, `agent_docs/git_workflow.md` | `./scripts/harness-check.sh quality` |
| **Rust Code Quality** | `.clippy.toml`, `agent_docs/code_conventions.md` | `./scripts/harness-check.sh clippy` |
| **Build & Dependencies** | `deny.toml`, `agent_docs/building_the_project.md` | `./scripts/harness-check.sh build`, `cargo audit` |
| **Testing & Correctness** | `agent_docs/running_tests.md`, `plans/adr/ADR-027-*.md` | `./scripts/harness-check.sh test`, `./scripts/harness-check.sh ignored-tests` |
| **Documentation** | `llms.txt`, `agent_docs/README.md` | `./scripts/harness-check.sh doc`, `./scripts/harness-check.sh docs-integrity` |
| **Architecture** | `plans/adr/*.md`, `agent_docs/service_architecture.md` | `./scripts/harness-check.sh quality` (file size, ADR checks) |
| **Skills** | `.agents/skills/**/SKILL.md` | `./scripts/run-evals.sh` |
| **Memory System** | `.agents/skills/memory-harness/SKILL.md` | `./scripts/harness-check.sh memory` |
| **Release Management** | `agent_docs/ci_guidance.md` | `./scripts/harness-check.sh release` |

## Sensor Catalog

### Deterministic Sensors (Blocking)

| Sensor | Script | Purpose |
| :--- | :--- | :--- |
| `fmt` | `./scripts/code-quality.sh fmt` | Verify code formatting |
| `clippy` | `./scripts/code-quality.sh clippy --workspace` | Run static analysis (zero warnings) |
| `build` | `./scripts/build-rust.sh check` | Ensure the workspace compiles |
| `test` | `cargo test --workspace` | Run all unit and integration tests |
| `doc` | `./scripts/check-doctests.sh` | Verify doctests and doc build integrity |
| `docs-integrity`| `./scripts/check-docs-integrity.sh` | Ensure cross-links and metadata are valid |
| `quality` | `./scripts/quality-gates.sh` | Enforce coverage and LOC invariants |
| `release` | `./scripts/verify-release-state.sh` | Validate version and changelog state |
| `ignored-tests` | `./scripts/check-ignored-tests.sh` | Enforce the ignored-test ceiling |

### Inferential / Eval Sensors (Informational/Periodic)

| Sensor | Script | Purpose |
| :--- | :--- | :--- |
| `memory` | `.agents/skills/memory-harness/evaluate-learning.sh` | Measure system learning effectiveness |
| `ai-slop` | `.github/workflows/ai-slop-eval.yml` | Evaluate detector accuracy against spam |
| `bench` | `cargo bench` | Detect performance regressions |

## Skill Evaluations

Generic skill evaluations are discovered via `.agents/skills/*/evals/evals.json`. Use `./scripts/run-evals.sh` to execute all discovered evaluations.

## CI Enforcement

- **Blocking Checks**: `fmt`, `clippy`, `build`, `test`, `doc`, `quality`, `release`.
- **Informational**: `memory` (when traces are available), `bench`.
- **Scheduled**: `ai-slop`, `fuzzing`.
