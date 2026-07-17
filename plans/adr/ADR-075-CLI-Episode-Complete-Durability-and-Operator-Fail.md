# ADR-075: CLI Episode Complete Durability and Operator Fail Path

- **Status**: Accepted
- **Date**: 2026-07-17
- **Deciders**: Project maintainers
- **Issue**: [#847](https://github.com/d-o-hub/rust-self-learning-memory/issues/847)
- **Related**: ADR-056 (local storage), ADR-071 (abstention checkpoint), GOAP `plans/GOAP_OPEN_ISSUES_ANALYSIS_2026-07-17.md`
- **Code**: `memory-core/src/memory/completion.rs`, `memory-cli/src/commands/episode/core/complete.rs`

## Context

Users report that `do-memory-cli episode complete <id> failure` exits 0 and prints success while `episode list` still shows `Status: in_progress` for bot-generated episodes stuck with zero steps.

Code review shows:

1. CLI quality threshold is `0.0`, so empty-step episodes are allowed to complete.
2. There is no “must have steps” lifecycle guard in `SelfLearningMemory::complete_episode`.
3. Backend `store_episode` failures after in-memory completion are **`warn!`-only**; the method still returns `Ok(())`.
4. CLI prints “Episode completed / Status: completed” whenever core returns `Ok`, with **no re-read verification**.
5. There is no operator-oriented `episode fail` (or force-finalize) command for abandoned rows.

Silent success is worse than a hard error: automation and humans treat the episode as finalized while durable storage remains incomplete.

## Decision

### 1. Durable complete is all-or-nothing for configured backends

When `complete_episode` runs:

- Apply in-memory completion (end_time + outcome) as today.
- Attempt to persist to every **configured** backend (`cache_storage`, `turso_storage`).
- If **any** configured backend fails to store the completed episode, return a **hard error** (`Error::Storage` or dedicated variant). Do not return `Ok(())` after a failed durable write.
- In-memory map may still hold the completed episode for the process lifetime, but callers must treat the operation as failed unless backends succeed.

Rationale: CLI and MCP are multi-process; the durable store is the source of truth across invocations.

### 2. Verify-after-write for CLI (and recommended for MCP)

After a successful `complete_episode` call, CLI must:

1. Re-fetch the episode via `get_episode` (lazy path: memory → redb → Turso).
2. Assert `episode.is_complete()` and that the outcome kind matches the request (at least Failure/Success/Partial).
3. Only then print human/JSON success.
4. On verification failure, exit non-zero with a message that names the likely cause (backend write, wrong `--db-path`, corrupted entry).

### 3. Empty steps remain completable; patterns may be empty

Zero-step completion stays allowed for CLI operator cleanup. Pattern extractors may yield zero patterns; that is **not** a complete failure. Document that tool-sequence patterns require ≥1 step (see ADR-076).

### 4. Operator force-fail path

Add one of (prefer first if clap surface stays small):

- **`episode fail <EPISODE_ID>`** — completes with `TaskOutcome::Failure` and the same durability rules; or
- **`episode complete <id> failure --force`** — identical semantics with explicit force flag for already-partial state.

No separate “skip quality” flag is required while CLI threshold remains `0.0`. If thresholds become configurable later, force-fail must bypass quality rejection for operator cleanup.

### 5. Tests (mandatory)

| Case | Expectation |
|------|-------------|
| Zero-step create → complete failure → new process list/view | `is_complete`, outcome Failure |
| Complete when redb store mocked to fail | Non-zero CLI exit; no success banner |
| Complete unknown id | Non-zero exit |
| Optional force-fail on stuck row | Completes and persists |

## Consequences

### Positive

- Eliminates false-green complete for durable workflows.
- Gives operators a supported way to finalize abandoned `in_progress` rows.
- Aligns CLI success messaging with storage reality.

### Negative / tradeoffs

- Previously “soft” store failures become visible (may break scripts that ignored corrupt storage).
- Slightly more I/O (re-read after complete).

### Non-goals

- Changing default quality threshold for library `MemoryConfig` (remains 0.7).
- Requiring steps for completion.
- Making `storage sync` perform completion.

## Implementation sketch

1. `completion.rs`: accumulate backend store errors; `return Err` if any configured backend fails.
2. `complete.rs` (CLI): post-verify + optional `Fail` subcommand / `--force`.
3. E2E under `tests/e2e/` or CLI integration: cross-process complete of zero-step episode.
4. Update CHANGELOG and CLI help.

## References

- Issue #847
- Related path bugs: #830 (db-path), #831 (pattern durability)
- GOAP plan: `plans/GOAP_OPEN_ISSUES_ANALYSIS_2026-07-17.md`
