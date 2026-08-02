# ADR-080: Automatic, Episode-Bound Recommendation Attribution

- **Status**: Proposed
- **Date**: 2026-07-30
- **Deciders**: Project maintainers
- **Plan**: [`../GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`](../GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md)
- **Related**: ADR-039 (plans governance), ADR-044 (recommendation attribution), ADR-072 (authority/evidence), ADR-075 (durability truth)
- **Code evidence**: `memory-core/src/memory/pattern_api.rs`, `memory-core/src/memory/retrieval/playbooks.rs`, `memory-core/src/memory/api.rs`, `memory-core/src/memory/persistence.rs`, `memory-mcp/src/mcp/tools/pattern_search.rs`, `memory-cli/src/commands/pattern/core/search.rs`

## Context

ADR-044 introduced recommendation sessions, feedback, aggregate effectiveness
statistics, Turso/redb persistence, and MCP/CLI commands. The pieces exist, but
normal recommendation operations do not use them consistently:

- `recommend_patterns_for_task` returns recommendations without a session;
- MCP and CLI require a separate manual `record-session` operation, so most
  recommendation exposures are not attributable;
- `retrieve_playbooks` records a hidden in-memory session against `Uuid::nil()`
  and bypasses the public persistence path;
- the public recorder returns `()` and logs persistence failures, while storage
  trait defaults can return `Ok(())` without writing anything;
- feedback can be accepted without proving that its session exists or that its
  applied pattern IDs were among those recommended; and
- each episode can legitimately have multiple recommendation exposures, while
  the episode lookup contract is only implicitly “latest session.”

This is an attribution-capture and contract-truth gap. It does **not** by itself
close the ranking-learning loop: current feedback feeds statistics, but no
production path idempotently applies attributed outcomes to later pattern scores.

## Decision

### 1. Add explicit attributed core operations

Keep existing unattributed pattern and playbook APIs behavior-compatible. Add
attributed variants that require a non-nil, existing episode ID and return the
recommendations together with an attribution receipt.

MCP and CLI accept an optional `episode_id`. When absent, they call the existing
unattributed operation and preserve the current wire/output shape. When present,
they call the attributed operation and add an `attribution` envelope. Ordinary
search, analogous discovery, and explanation operations remain unattributed.

### 2. Derive sessions from returned recommendations in core

One shared core path creates the session from the exact pattern and playbook IDs
returned to the caller. MCP and CLI do not reconstruct ID lists independently.
The hidden `Uuid::nil()` side effect in playbook retrieval is removed.

A valid recommendation operation that returns no matches may still create an
empty session so abstention/coverage can be measured. A recommendation error
must not create a session. The playbook API must therefore distinguish a valid
empty result from generation failure instead of collapsing both to `Vec::new()`.

### 3. Return a truthful persistence receipt

The attributed operation remains successful when recommendation generation
succeeds even if attribution persistence is degraded. Its receipt is a tagged,
machine-stable state:

| State | Meaning |
|-------|---------|
| `persisted` | At least one persistence backend is configured and every configured capable backend wrote the session |
| `partially_persisted` | At least one configured capable backend wrote it and at least one failed |
| `memory_only` | No persistence backend is configured; the session exists only in this process |
| `persistence_failed` | Configured capable backends all failed; the session remains process-local |

Every state includes `session_id` and `episode_id`; degraded states include
stable backend identifiers, not raw backend errors or credentials. Detailed
errors remain in logs. A failed persistence receipt never implies restart-safe
feedback.

The current warning-only recorder remains for source compatibility and delegates
to a new checked recorder. Recommendation-attribution capability must be explicit:
the checked path never counts the storage trait's successful no-op defaults as a
write. Turso and redb advertise support; unsupported implementations are skipped
or return a typed unsupported result.

### 4. Make session and feedback integrity explicit

- Reject malformed, nil, and nonexistent episode IDs before attribution.
- Multiple exposures for one episode create distinct session IDs; do not upsert
  or deduplicate by episode ID.
- Preserve lookup by session ID. Define episode lookup as the latest session and
  make tie-breaking deterministic beyond second-resolution timestamps.
- Resolve a session from memory or storage before accepting feedback.
- Reject applied pattern IDs that were not recommended in that session.
- Treat replacement feedback for one session as idempotent replacement, not a
  second application event.
- Return the same persistence receipt semantics for manual session creation and
  feedback writes so MCP/CLI success is truthful.

Caller-provided retry/idempotency keys and list-all-sessions-by-episode are
deferred until at-least-once retry behavior demonstrates a need.

### 5. Defer ranking adaptation to a separate decision

ADR-080 captures trustworthy data but does not mutate pattern effectiveness or
recommendation ranking. A follow-up must define idempotent durable updates,
replacement-feedback semantics, rollback behavior, and how attributed evidence
changes ranking weights before the project claims the learning loop is closed.

## Consequences

### Positive

- Normal recommendation use can produce feedback-ready session IDs automatically.
- Pattern and playbook attribution share one core contract across library, MCP,
  CLI, Turso, and redb.
- Callers can distinguish durable, partial, process-only, and failed persistence.
- Existing unattributed consumers retain their current output contract.
- Orphan feedback and impossible applied-pattern claims are rejected.

### Negative and trade-offs

- Persistence reporting needs a capability-aware core seam across both backends.
- Attributed calls validate episode existence and perform extra writes.
- The separate manual session command remains for compatibility even though it
  is unnecessary after an attributed recommendation.
- Capture quality improves before ranking quality; this ADR intentionally avoids
  overstating immediate self-learning behavior.

## Alternatives considered

1. **Always create sessions**: rejected because it breaks library and wire
   compatibility and cannot bind recommendations to an episode reliably.
2. **Create sessions only in MCP/CLI**: rejected because surfaces could diverge
   and reconstruct IDs differently from the recommendations actually returned.
3. **Fail the whole recommendation on persistence error**: rejected because
   attribution telemetry must not suppress otherwise valid recommendations.
4. **Keep warning-only persistence**: rejected because returning a session ID
   without durability state misleads callers after restart.
5. **Apply feedback to ranking in the same change**: rejected until idempotent,
   durable update and replacement semantics are designed and tested.

## Acceptance criteria

- Legacy calls without `episode_id` preserve their existing response/output
  shape and create no session.
- Attributed calls reject malformed, nil, and nonexistent episode IDs.
- Pattern and playbook sessions contain exactly the IDs returned to the caller.
- No playbook path creates a nil-episode session or bypasses the checked recorder.
- No configured persistence backend yields `memory_only`, never `persisted`.
- All configured capable stores succeed → `persisted`; some fail →
  `partially_persisted`; all fail → `persistence_failed` without losing results.
- Persisted sessions remain retrievable by session ID after restart.
- Two attributed calls for one episode create and retain two distinct sessions;
  latest lookup is deterministic.
- Unknown-session feedback and non-recommended applied IDs are rejected.
- Manual MCP/CLI session and feedback commands use the checked semantics.
- Documentation describes this as attribution capture, not ranking adaptation.
