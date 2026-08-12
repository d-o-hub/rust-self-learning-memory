# ADR-081: Attribution Capability Truth, Feedback Resolution, and Tool-Schema Single Source

- **Status**: Proposed — capability slice (§2) landed 2026-08-10 and the residual acceptance evidence is covered by the closure PR from branch `fix/ci-attribution-truth-closure` (see "Status update" below); lifecycle stays `Proposed` until maintainer acceptance
- **Date**: 2026-08-06
- **Deciders**: Project maintainers
- **Plan**: [`../GOAP_ATTRIBUTION_COMPLETION_AND_CODEBASE_IMPROVEMENTS_2026-08-06.md`](../GOAP_ATTRIBUTION_COMPLETION_AND_CODEBASE_IMPROVEMENTS_2026-08-06.md)
- **Supersedes nothing**; **completes** [ADR-080](ADR-080-Automatic-Recommendation-Attribution.md)
- **Related**: ADR-044 (recommendation attribution), ADR-072 (authority/evidence), ADR-074 (retrieval provenance), ADR-075 (durability truth), ADR-076 (discoverability + empty-result semantics), ADR-079 (fail-closed CI control plane)
- **Code evidence**: `memory-core/src/memory/attribution/tracker/mod.rs`, `memory-core/src/memory/attribution/tracker/integrity.rs`, `memory-core/src/memory/attribution/tracker/stats.rs`, `memory-core/src/memory/attribution/tracker/tests.rs` (split from the former single `tracker.rs`), `memory-core/src/memory/attribution/receipt.rs`, `memory-core/src/memory/attribution/types.rs`, `memory-core/src/memory/persistence.rs`, `memory-core/src/memory/api.rs`, `memory-core/src/memory/pattern_api.rs`, `memory-core/src/memory/retrieval/playbooks.rs`, `memory-core/src/memory/retrieval/playbooks_attributed.rs`, `memory-core/src/storage/backend.rs`, `memory-mcp/src/bin/server_impl/tools/feature_handlers.rs`, `memory-mcp/src/server/tool_definitions.rs`, `memory-mcp/src/server/recommendation_tool_definitions.rs`, `memory-mcp/src/server/tools/registry/builder.rs`, `memory-cli/src/commands/pattern/core/search.rs`, `memory-cli/src/commands/feedback/core.rs`

## Status update (2026-08-11)

- §2 capability advertisement and capability-gated receipts landed 2026-08-10
  (`supports_recommendation_attribution` default `false`; Turso and redb
  advertise `true`; resilient/cached wrappers delegate).
- The closure PR from branch `fix/ci-attribution-truth-closure` now covers the
  residual acceptance evidence: episode-existence validation on both attributed
  entry points, checked manual session/feedback receipt semantics, fallible
  playbook retrieval (no session on generation failure), split tracker modules,
  Turso-only/redb-only cold-restart tests, capability tests for the concrete
  backends, the postcard-safety guard, and the MCP/CLI truth surfaces.
- Ranking adaptation (§8) remains deferred; nothing in the closure PR changes
  recommendation ranking.

---

## Context

ADR-080 was accepted as *Proposed* and a partial implementation now exists in the
working tree on `feat/adr-080-automatic-recommendation-attribution`
(15 modified files, 1 new file, +832/−176). The workspace type-checks cleanly
(`cargo check --workspace --all-targets` → exit 0).

The implemented slice is real and matches ADR-080 for the *happy path*:

| ADR-080 clause | Implementation | Evidence |
|----------------|----------------|----------|
| §1 attributed core ops | `recommend_patterns_attributed`, `retrieve_playbooks_attributed` | `pattern_api.rs:146`, `playbooks.rs:176` |
| §2 session derived in core from returned IDs | Both attributed ops build the session from the exact returned results | `pattern_api.rs:161-175`, `playbooks.rs:203-227` |
| §2 remove hidden nil-episode session | Deleted from `retrieve_playbooks` | `playbooks.rs` (−16 lines at old 151-166) |
| §3 four-state receipt | `PersistenceReceipt::{Persisted, PartiallyPersisted, MemoryOnly, PersistenceFailed}` | `attribution/receipt.rs:26-56` |
| §4 multiple sessions per episode | `episode_sessions: HashMap<Uuid, Vec<Uuid>>` | `tracker.rs:55-56` |
| §4 deterministic latest lookup | timestamp, then `session_id` byte order | `tracker.rs:196-204` |
| §4 applied ⊆ recommended | Rejects non-recommended applied IDs | `tracker.rs:147-160` |
| §4 idempotent replacement feedback | Map overwrite, single feedback row | `tracker.rs:163-166` |
| §1 optional `episode_id` on surfaces | MCP `RecommendPatternsInput`, CLI `--episode-id` ×2 | `mcp/tools/pattern_search.rs:225-227`, `pattern/core/types.rs:92-94`, `playbook/types.rs:39-41` |

Auditing the same slice against ADR-080's own twelve acceptance criteria shows
the contract is **not yet met**, and one change is a **behavioral regression**
rather than an incomplete feature. The residual gaps are not stylistic: four of
them let the system report durability, attribution, or success that it did not
achieve — the exact failure class ADR-074/075/076/079 were written to eliminate.

### The four truth defects

1. **Feedback resolution is memory-only.** `RecommendationTracker::record_feedback`
   resolves the session from the in-process map only (`tracker.rs:133-142`), and
   `record_recommendation_feedback` does not hydrate from storage before calling it
   (`api.rs:176-185`). Because the same change *also* upgraded "session missing" from
   a `debug!` to a hard `Error::InvalidInput`, feedback against a durably persisted
   session now **fails after restart** where it previously succeeded. ADR-080 §4
   requires resolving "from memory **or storage**". This is a regression introduced by
   implementing half of one clause.

2. **Persistence capability is asserted, not verified.** `persist_session_checked`
   treats every configured `Arc<dyn StorageBackend>` as capable
   (`persistence.rs:203-244`), but `StorageBackend::store_recommendation_session` has a
   **default no-op returning `Ok(())`** (`backend.rs:374-377`). Any backend that does not
   override it is counted as a successful write and yields `Persisted` — a receipt that
   claims restart-safety the system does not have. ADR-080 §3 explicitly forbids this:
   "the checked path never counts the storage trait's successful no-op defaults as a
   write." The same defect class is systemic: `backend.rs` carries **18** such
   `Ok(())`/`Ok(None)`/`Ok(Vec::new())` defaults.

3. **Episode existence is unvalidated and malformed IDs silently degrade.** Both
   attributed operations check only `is_nil()` (`pattern_api.rs:153`, `playbooks.rs:189`);
   a well-formed UUID for a nonexistent episode creates an orphan session. Separately,
   the MCP playbook handler parses with `.and_then(|s| Uuid::parse_str(s).ok())`
   (`feature_handlers.rs:147-150`), so a malformed `episode_id` is **silently dropped**
   and the call quietly returns an unattributed result the caller believes is
   attributed. The CLI does this correctly (`search.rs:154-160` hard-errors); MCP and
   CLI therefore disagree.

4. **Playbook generation failure is indistinguishable from valid emptiness.**
   `retrieve_playbooks` still returns `Vec<RecommendedPlaybook>` and collapses generator
   errors to `vec![]` (`playbooks.rs:161-165`). The attributed wrapper consumes that
   `Vec`, so a *failed* generation produces an empty session — recorded as a real
   abstention. ADR-080 §2 requires the opposite: "A recommendation error must not create
   a session."

### Contract and gate defects

5. **Manual paths remain warning-only.** `record_recommendation_session` →
   `persist_recommendation_session` logs and returns `()` (`api.rs:129-134`,
   `persistence.rs:17-38`); the MCP tool reports `success: true` unconditionally
   (`recommendation_feedback/tool.rs:69-90`), as does CLI `feedback record-session`.
   ADR-080 acceptance requires the checked semantics on these paths too. There is no
   `persist_feedback_checked` counterpart at all.

6. **The new parameter is undiscoverable.** Neither MCP tool schema declares
   `episode_id`: `recommend_patterns` at `server/tool_definitions.rs:171-200` and
   `recommend_playbook` at `server/tools/registry/builder.rs:109-155`. An MCP client
   reading `tools/list` cannot learn the capability exists. This is the ADR-076
   discoverability defect repeated. It is aggravated by **two independent tool
   registries** declaring overlapping tools, with no test asserting they agree.

7. **The 500-LOC gate is breached.** `memory-core/src/memory/attribution/tracker.rs`
   is **557 LOC**.

8. **Latent postcard hazard.** `PersistenceReceipt` uses `#[serde(tag = "state")]`
   (`receipt.rs:22`). Sessions are postcard-serialized in redb
   (`memory-storage-redb/src/recommendations.rs:22`), and the repo's standing prevention
   rule is "Never `#[serde(tag=)]` on postcard types". Receipts are JSON-only today so
   nothing breaks now, but the type is re-exported from `lib.rs` and will fail at
   runtime the first time anyone embeds it in a persisted struct.

---

## Decision

### 1. Feedback resolves through the memory→storage chain, and rejection is fail-closed only after that chain is exhausted

Session resolution for feedback becomes a `SelfLearningMemory` responsibility, not a
`RecommendationTracker` responsibility. The tracker keeps pure in-memory integrity
checks; the memory layer resolves the session (tracker → Turso → redb), hydrates the
tracker on a storage hit, and only then applies the integrity rules.

Rejection of unknown sessions is retained — it is correct and required by ADR-080 §4 —
but it must fire only when no configured backend can produce the session. A durable
session must accept feedback after restart. This is a hard acceptance criterion, not a
best-effort behavior.

### 2. Attribution persistence capability is advertised, not assumed

Add an explicit capability query to `StorageBackend` that defaults to **false**, and
have Turso and redb advertise **true**. `persist_session_checked` and the new
`persist_feedback_checked` count only advertised-capable configured backends.

Consequences that follow directly:

- Zero capable backends configured → `MemoryOnly`, regardless of how many storage
  backends are attached. A non-advertising backend can never contribute a `Persisted`.
- The default trait method stops being a silent success path for attribution.

Generalizing all 18 no-op defaults in `backend.rs` to a full capability matrix is
**out of scope here** and deferred to a follow-up; this ADR adds capability truth only
for the recommendation-attribution methods, which is what ADR-080 §3 requires.

### 3. Attributed operations validate the episode, and every surface rejects malformed IDs identically

- Attributed core operations verify the episode **exists** before creating a session,
  in addition to the existing nil check. A nonexistent episode is `InvalidInput`, not an
  orphan session.
- A malformed `episode_id` is an **error on every surface**. MCP stops using
  `.ok()`-and-continue; parse failure returns a tool error matching the CLI's behavior.
  Absent `episode_id` remains the unattributed legacy path — silence is the only
  legitimate way to opt out.

### 4. Playbook retrieval distinguishes failure from emptiness

Introduce a fallible playbook retrieval path so the attributed wrapper can tell a valid
empty result from generation failure. The existing `Vec`-returning `retrieve_playbooks`
is retained for source compatibility and delegates to the fallible path, mapping errors
to `vec![]` exactly as today. Only the attributed path observes the error.

Generation failure → error returned, **no session created**. Valid empty result →
empty session created, so abstention and coverage remain measurable per ADR-080 §2.

### 5. Manual session and feedback writes return the same receipt

`record_recommendation_session` and `record_recommendation_feedback` gain checked
variants returning `PersistenceReceipt`. MCP and CLI manual commands report the receipt
state instead of unconditional `success: true`. The existing `()`-returning functions
remain for source compatibility and delegate to the checked variants, discarding the
receipt — the deprecation path, not a second implementation.

### 6. MCP tool schemas are single-source and contract-tested

Both registries declare `episode_id` for `recommend_patterns` and `recommend_playbook`,
with the receipt/attribution envelope documented in the tool description. A test asserts
that any tool declared in both registries has identical name, description, and input
schema, so the dual-registry drift cannot silently return.

Collapsing the two registries into one is **deferred**; this ADR requires only that
they cannot disagree.

### 7. Receipts stay JSON-only until a postcard-safe representation exists

`PersistenceReceipt` keeps its internally-tagged JSON shape — it is the wire contract
for MCP/CLI and is already implemented. It is documented as **JSON-only** and must not be
embedded in any postcard-serialized type. A compile-time or test-time guard asserts no
persisted type transitively contains it. If a receipt ever needs persisting, an
externally-tagged or struct-with-discriminant representation is required first.

### 8. Ranking adaptation remains deferred

Unchanged from ADR-080 §5. This ADR closes capture correctness only. The project must
not describe the learning loop as closed until a further ADR defines idempotent durable
ranking updates.

---

## Consequences

### Positive

- Feedback works after restart; the regression is removed rather than documented.
- `Persisted` becomes a claim the code can actually substantiate.
- Orphan sessions from nonexistent episodes and silently-unattributed MCP calls become
  impossible.
- Failed playbook generation stops polluting abstention statistics.
- MCP clients can discover attribution from `tools/list`.
- The two tool registries are prevented from drifting.

### Negative and trade-offs

- Feedback acquires up to two storage reads before acceptance; attributed calls acquire
  an episode-existence read. Both are on attribution paths only, not on the hot
  retrieval path, and neither affects the < 100 ms retrieval target.
- `StorageBackend` grows a capability method; every backend must consider it, though the
  `false` default keeps it non-breaking.
- The compatibility shims (`retrieve_playbooks`, `record_recommendation_session`,
  `persist_recommendation_session`) persist as thin delegating wrappers, which is
  duplication the project accepts in exchange for not breaking library consumers.
- Capability truth is added narrowly for attribution; the other 17 no-op defaults keep
  their current behavior until the follow-up.

## Alternatives considered

1. **Relax the unknown-session rejection back to a warning.** Rejected: it restores
   ADR-044's silent orphan-feedback acceptance, which ADR-080 §4 exists to end. The
   defect is missing storage resolution, not the rejection.
2. **Require all backends to override the attribution methods (remove the defaults).**
   Rejected: a breaking change to a public trait for a problem an advertised capability
   solves without breaking anyone.
3. **Probe capability by attempting a write and inspecting the result.** Rejected:
   indistinguishable from a successful no-op — the exact defect being fixed.
4. **Change `retrieve_playbooks` to return `Result` directly.** Rejected: breaks every
   existing caller for a benefit only the attributed path needs.
5. **Treat a malformed MCP `episode_id` as absent.** Rejected: the caller asked for
   attribution and would receive an unattributed result reported as success — a silent
   truth defect, and it makes MCP disagree with the CLI.
6. **Merge the two MCP tool registries now.** Deferred: worthwhile, but a refactor of
   that size does not belong in the change that closes an attribution contract.

---

## Acceptance criteria

1. Feedback for a session persisted before restart is **accepted** after restart with a
   cold in-memory tracker, in both Turso-only and redb-only configurations.
2. Feedback for a session that exists in no backend and no tracker is rejected with
   `InvalidInput`.
3. A configured backend that does not advertise attribution capability never produces
   `Persisted`; with no capable backend the receipt is `MemoryOnly`.
4. All capable backends succeed → `Persisted`; some fail → `PartiallyPersisted` with
   stable backend identifiers; all fail → `PersistenceFailed`; recommendations are
   returned intact in every case.
5. Attributed calls reject nil, malformed, and nonexistent episode IDs — identically on
   core, MCP, and CLI.
6. A malformed MCP `episode_id` returns a tool error; it never degrades to an
   unattributed success.
7. Playbook generation failure returns an error and creates **no** session; a valid
   empty result creates an empty session.
8. Manual MCP/CLI session and feedback commands return receipt state; `success: true` is
   never reported for `PersistenceFailed`.
9. `episode_id` appears in the `recommend_patterns` and `recommend_playbook` input
   schemas in **both** registries, and a test asserts the registries agree.
10. Every source file touched is ≤ 500 LOC, including
    `memory-core/src/memory/attribution/tracker.rs`.
11. No new `#[expect(clippy::too_many_arguments)]`; the attributed playbook operation
    takes a request struct.
12. A test asserts no postcard-serialized type transitively contains `PersistenceReceipt`.
13. Coverage across the changed crates is ≥ 90%, with restart-safety and receipt-state
    matrix tests present rather than only unit-level assertions.
14. `docs/API_REFERENCE.md` documents `episode_id`, the four receipt states, and states
    plainly that this is attribution **capture**, not ranking adaptation.
