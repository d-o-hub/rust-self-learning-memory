# ADR-031: Episode Step Logging CLI Alias

**Status**: Proposed
**Date**: 2026-02-17
**Deciders**: Project maintainers

## Context

The `Nightly Full Tests` scheduled workflow on `main` is failing (example: run `22085572056` at `2026-02-17T04:07:26Z`) with E2E CLI tests reporting:

- `error: unrecognized subcommand 'step'`

The repository currently contains both:

- An E2E expectation that step logging is invoked as `memory-cli episode step ...` (see `tests/e2e/cli_workflows.rs`)
- A CLI definition for step logging as `memory-cli episode log-step ...` (see `memory-cli/src/commands/episode/core/types.rs`, `EpisodeCommands::LogStep`)

This mismatch breaks CI and creates an unstable CLI contract for users and tests.

## Decision

1. **Canonical command**: `memory-cli episode log-step ...` remains the canonical spelling for logging an episode step.
2. **Backwards compatibility alias**: `memory-cli episode step ...` is supported as an alias for `episode log-step`.
3. **Test policy**: E2E tests and documentation should use the canonical `episode log-step` spelling to avoid relying on aliases.
4. **Deprecation policy**: The alias can be deprecated only after:
   - At least one minor release cycle with the alias present, and
   - All internal call sites (tests/scripts/docs) use the canonical spelling.

## Alternatives Considered

1. **Update tests only (no alias)**
   - Pros: smaller CLI surface area
   - Cons: breaks user scripts; guarantees future churn when commands are reorganized
   - Rejected: avoidable backwards-compat break

2. **Rename command back to `episode step`**
   - Pros: aligns with existing E2E test expectations
   - Cons: loses descriptive naming (`log-step` makes the intent explicit)
   - Rejected: name clarity is useful, and alias provides compatibility

3. **Add alias + update tests (chosen)**
   - Pros: restores CI, preserves compatibility, keeps clear canonical naming
   - Cons: slightly larger CLI surface area

## Consequences

- Nightly E2E failures caused by the missing `step` subcommand are eliminated once the alias is implemented and tests are aligned.
- CLI contract becomes stable: renames can preserve an alias to avoid breaking user scripts.
- Minimal maintenance burden: alias remains a thin clap-level mapping.

## Implementation Notes

- Add clap alias `step` to the `EpisodeCommands::LogStep` subcommand.
- Update `tests/e2e/cli_workflows.rs` to use `episode log-step`.
- Add a unit parsing test ensuring `episode step` maps to the same command as `episode log-step`.

## References

- `Nightly Full Tests` failure: https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/22085572056
- E2E CLI workflow tests: `tests/e2e/cli_workflows.rs`
- Episode CLI commands: `memory-cli/src/commands/episode/core/types.rs`
- Related stability guidance: `plans/adr/ADR-030-Test-Optimization-and-CI-Stability-Patterns.md`

