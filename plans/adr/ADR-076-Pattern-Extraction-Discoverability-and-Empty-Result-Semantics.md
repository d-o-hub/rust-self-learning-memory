# ADR-076: Pattern Extraction Discoverability and Empty-Result Semantics

- **Status**: Accepted
- **Date**: 2026-07-17
- **Deciders**: Project maintainers
- **Issues**: [#845](https://github.com/d-o-hub/rust-self-learning-memory/issues/845) (residual), closed [#831](https://github.com/d-o-hub/rust-self-learning-memory/issues/831)
- **Related**: ADR-075 (complete durability), postcard Pattern fix in v0.1.35, GOAP `plans/GOAP_OPEN_ISSUES_ANALYSIS_2026-07-17.md`
- **Code**: `memory-core/src/extraction/`, `memory-core/src/memory/learning.rs`, `memory-cli/src/commands/pattern/`, `memory-cli/src/commands/storage/`

## Context

Issue #845 reports that after bulk `episode create` → `episode complete` ingest on **v0.1.34**, `pattern list` / `pattern search` return empty and `storage sync` appears to no-op.

Two separate classes of problem exist:

### A. Correctness bug (fixed on main, unreleased)

v0.1.34 could store patterns but fail to list them across processes:

- `Pattern` used serde internal tagging incompatible with postcard.
- `get_all_patterns` did not reliably hydrate from redb.

Fixed in commit `58bed23f` (issue #831): postcard-compatible `Pattern`, redb `get_all_patterns`, e2e cross-process list. Users still on v0.1.34 need the **v0.1.35** release.

### B. Semantics / discoverability (still open)

Even on fixed code:

1. **Tool-sequence extraction requires ≥1 step** (`extract_tool_sequence` returns `None` when `episode.steps.is_empty()`). Create→complete with no `log-step` yields zero patterns **by design**.
2. **`storage sync` is not pattern extraction** — it synchronizes Turso ↔ redb. Local-only redb setups should not imply that sync “derives patterns.”
3. Empty list/search responses do not explain *why* (no durable patterns vs wrong db path vs zero extractable steps).
4. There is no first-class **re-extract** command for operators who completed episodes before a bugfix or without steps.

Issue #845 conflates A and B. This ADR freezes B so we do not re-break A while improving UX.

## Decision

### 1. Do not re-open the postcard / hydration design

Pattern persistence remains:

- Externally tagged / postcard-safe `Pattern` encoding.
- Lazy `get_all_patterns`: memory → redb → Turso.
- Cross-process e2e coverage is a regression gate.

Any change to Pattern serde must add postcard roundtrip tests.

### 2. Extraction contract (document and surface)

| Input | Tool-sequence patterns | Context patterns | Complete allowed |
|-------|------------------------|------------------|------------------|
| Completed + ≥1 successful step path | May extract | May extract | Yes |
| Completed + zero steps | None from tool-sequence | Context extractor may still run | Yes (CLI threshold 0.0) |
| In progress | Not extracted | Not extracted | No |

`complete_episode` remains the primary extraction trigger (sync or async queue). Extraction producing zero patterns is **success of complete**, not failure.

### 3. Empty-result CLI semantics

When `pattern list` or `pattern search` returns zero results in human mode, print a short diagnostic footer (not only “No patterns found”), covering:

- Patterns are created on **episode complete** (not on `storage sync`).
- Tool sequences need **at least one step** before complete.
- Confirm same `--db-path` / config as the process that completed episodes.
- Point to `config show` and troubleshooting skill docs.

JSON/YAML modes remain machine-stable (empty arrays) unless a structured `diagnostics` field is added later without breaking consumers.

### 4. `storage sync` messaging

- Require both Turso and redb (existing).
- If only local redb is configured, fail with an explicit message: sync is for dual-backend reconciliation, not pattern extraction; use complete + pattern list.
- Never advertise sync as “trigger extraction.”

### 5. Optional re-extract (P2)

A future `pattern extract [--episode-id <uuid>]` may re-run extractors for completed episodes and store results. This is additive; it must use the same extractors as complete and respect durability rules from ADR-075 for any episode rewrites.

### 6. Release coupling

Close #845 for users only after:

1. v0.1.35 (or later) is released with #831, **and**
2. Repro is re-checked, **and**
3. Empty-result messaging (this ADR §3) is shipped or waived with a follow-up issue.

## Consequences

### Positive

- Separates “upgrade to fixed binary” from “document learning contract.”
- Prevents misuse of `storage sync` as an extractor.
- Keeps postcard durability as a hard invariant.

### Negative / tradeoffs

- Create→complete blog ingest without steps still yields empty patterns (by design); users must log steps or accept episode-only grounding.
- Slightly more verbose human CLI output.

### Non-goals

- Lowering success thresholds to invent patterns from empty episodes.
- Making `storage sync` run extractors.
- Changing MCP tool names for pattern search.

## Implementation sketch

1. Docs: README / CLI guide / CHANGELOG note for #831 + step requirement.
2. CLI human empty-list footer in `pattern/core/list.rs` and search path.
3. `storage sync` error/help text update.
4. Optional later: `pattern extract` command + tests.

## References

- Issues #845, #831
- `plans/GOAP_CLI_UX_PATCH_0.1.35_2026-07-15.md`
- `plans/GOAP_OPEN_ISSUES_ANALYSIS_2026-07-17.md`
- ADR-075
