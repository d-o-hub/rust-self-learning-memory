# GOAP: Product Truth, CI Trust, and Automatic Recommendation Attribution

- **Status**: Proposed
- **Date**: 2026-07-30
- **Audit checkout**: `main` at `e66defdf`
- **Decisions**: [ADR-078](adr/ADR-078-Automatic-Recommendation-Attribution.md) and [ADR-079](adr/ADR-079-Fail-Closed-CI-Required-Check-Control-Plane.md) — Proposed
- **Relevant constraints**: ADR-039, ADR-044, ADR-072, ADR-075, Tokio-only async, postcard, zero clippy warnings
- **Strategy**: Restore a fail-closed merge control plane first, fix product false-success contracts, then implement attribution integrity from storage/core to MCP/CLI, with converging end-to-end validation

## Analysis summary

The workspace is healthy at version `0.1.38` after release `v0.1.37`, but the
prior “no open code gaps” claim is not supported by current source. The audit
found a critical CI governance gap, two correctness gaps, one
advertised-but-unimplemented command, and one high-value feature that completes
the capture half of ADR-044.

| Priority | Finding | Evidence | Disposition |
|----------|---------|----------|-------------|
| P0 | The active ruleset requires no first-party build/test aggregate | Ruleset `9591004`, `.github/workflows/pr-check-anchor.yml` | Implement ADR-079 and stage `CI / Required` into protection |
| P0 | Five waiters treat cancelled/skipped/missing format/Clippy as acceptable and ignore commit lint | `ci.yml`, `coverage.yml`, `security.yml`, `benchmarks.yml`, `file-structure.yml`; PR #914 | Replace polling with same-run fail-closed aggregation |
| P0 | Cascade retrieval returns successful empty results when `csm` is disabled | `memory-core/src/retrieval/cascade/mod.rs` | ✅ Implemented — typed `CapabilityUnavailable` |
| P0 | CLI storage statistics present fixed estimates/unknowns as measured values | `memory-cli/src/commands/storage/commands.rs`, `types.rs` | ✅ Implemented — `MetricValue` provenance |
| P1 | Dependabot is excluded from most substantive CI and gate contracts overstate parity | workflow actor conditions; `plans/GATE_CONTRACT.md` | Actor parity plus semantic contract validation |
| P1 | Release manual dispatch is broken; publish selection and fuzz evidence have false-success paths | release run `30301797956`, workflow conditions | Truthful trigger/planner/evidence remediation |
| P1 | `eval set-threshold` is advertised but always errors; its help suggests nonexistent `eval show` | `memory-cli/src/commands/eval.rs` | ✅ Implemented — command removed/hidden |
| P1 | Recommendation generation does not automatically create truthful episode-bound sessions | ADR-078 evidence | Implement this plan's RAT packages |
| P2 | Azure/Custom/Cohere embedding adapters are absent | ADR-077, embedding factory | Remain unavailable until a provider-specific ADR and adapter tests exist |
| Non-gap | `execute_agent_code` and batch MCP tools are unavailable | ADR-073 and standing product decisions | Preserve fail-closed/deferred state |

## Goal state

| Fact | Before | Desired |
|------|--------|---------|
| `required_ci_causal` | Ruleset requires only external analysis | Same-run first-party aggregate required |
| `ci_cancellation_fail_closed` | Cancelled/skipped/missing may pass waiters | Applicable non-success always blocks |
| `automation_actor_parity` | Dependabot skips core validation | Same assertions with least privilege |
| `gate_contract_semantic` | Presence/keyword parity only | Commands, DAG, conditions, and ruleset validated |
| `cascade_capability_truthful` | Disabled feature looks like zero matches | ✅ Typed `CapabilityUnavailable` returned |
| `storage_metrics_truthful` | Estimates/unknowns look measured | ✅ Values carry provenance or are omitted |
| `unsupported_threshold_command_hidden` | CLI advertises guaranteed failure | ✅ No advertised non-operation |
| `recommendation_capture_automatic` | Separate manual session step | Optional episode-bound attributed operation |
| `attribution_persistence_truthful` | Warning-only, no-op defaults can look successful | Tagged persistence receipt |
| `feedback_integrity_checked` | Orphan/mismatched feedback accepted | Session and recommended-ID validation |
| `ranking_learns_from_feedback` | Statistics only | Explicitly deferred follow-up |

## Action graph

```text
CIT-A1 required topology ─► CIT-A2 fail-closed + actor parity ─► CIT-A3 contract
            │                           │                            │
            └───────────────────────────┴──► CIT-A4 release/publish  │
                                         └─► CIT-A5 evidence         │
                                                                   v
PTA-A1 cascade truth ───────────────┐                              PTA-A9
PTA-A2 storage metric truth ────────┼──► full validation and plan evidence
PTA-A3 threshold surface cleanup ───┘                 ▲
                                                      │
RAT-A1 contract tests                                 │
     │                                                │
     v                                                │
RAT-A2 capability + checked persistence               │
     │                                                │
     v                                                │
RAT-A3 attributed core APIs + playbook cleanup        │
     │                                                │
     v                                                │
RAT-A4 feedback/session integrity                     │
     │                                                │
     +----------------------+-------------------------+
                            v
                 RAT-A5 MCP + RAT-A6 CLI
                            │
                            v
                 RAT-A7 end-to-end docs/tests
```

CIT-A1…A3 are the first implementation wave because later code cannot rely on
merge gates until they are causal and fail closed. CIT-A4/A5 may follow in
parallel. PTA-A1…A3 are independent. RAT packages are sequential through the
core contract; MCP and CLI can proceed in parallel after RAT-A4.

## Work packages

### CIT-A1: Build and stage one required aggregate

Implement ADR-079's same-workflow PR orchestrator and stable `CI / Required`
context. Keep path classification in the workflow, make the aggregate run with
`always()`, and fail on every applicable non-success. First deploy without a
ruleset change; fault-inject normal, Dependabot, fork, docs-only, code, failure,
cancellation, timeout, and missing-result cases. With maintainer approval, add
the observed context to ruleset `9591004`, prove a failure blocks merge, then
remove the unused echo anchor and cross-workflow waiters.

**Exit**: The live ruleset requires a first-party aggregate causally derived
from substantive checks, with no merge bypass used during verification.

### CIT-A2: Make fast gates and actor behavior fail closed

Aggregate commit lint with formatting, Clippy, doctests/docs, frontmatter, and
ignored-test ceiling. Remove `cancelled`, unclassified `skipped`, and missing
checks from accepted outcomes. Run the same code-quality/test assertions for
Dependabot and forks with read-only permissions, no secrets, and no cache writes.
Secret-dependent jobs declare explicit applicability rather than skipping actors.

**Exit**: Commit-lint failure and cancelled/missing fast checks block; actor type
changes privileges, not the asserted code contract.

### CIT-A3: Reconcile and enforce the gate contract

Choose and document the required Linux test surface, using the full workspace or
explicitly owned exclusions. Replace copied Clippy flags and partial quality
bundles with canonical scripts/shared commands. Extend gate-contract validation
to inspect exact command scope, aggregate dependencies/results, actor conditions,
and the expected ruleset context. Add negative fixtures for each current defect.

**Exit**: Local documentation, workflow commands, aggregate semantics, and live
protection agree; presence-only validation cannot pass semantic drift.

### CIT-A4: Repair release and publish orchestration

Remove Release `workflow_dispatch` because ADR-072 permits tag-only creation.
Give publish a package plan and dependency closure, exact-version preflight,
`--locked`, bounded sparse-index polling, and an independent dry-run mode. A
selected crate must run, be proven already published, or fail explicitly.

**Exit**: No advertised trigger is guaranteed to fail or silently skip requested
publish work; tag release remains the sole release authority.

### CIT-A5: Make informational CI produce durable evidence

Ensure fuzz crashes/startup failures/timeouts are visible and upload artifacts
with `always()`. Preserve mutation reports even when the job is informational.
Record baselines before proposing blocking mutation/fuzz thresholds. After trust
is restored, remove duplicate builds/tests through reusable workflows and
artifact handoff, measuring queue-to-result time and compute before/after.

**Exit**: Informational checks cannot greenwash or discard failures, and cost
optimization does not weaken the aggregate contract.

### PTA-A1: Make disabled cascade retrieval truthful

**Status: ✅ Implemented (2026-08-01).** `CascadeRetriever::retrieve` now returns
`Result<CascadeResult, CascadeError>`; non-`csm` builds return
`Err(CascadeError::CapabilityUnavailable)` instead of a successful empty result.

Change the non-`csm` contract so callers cannot confuse unavailable capability
with a valid zero-match result. Prefer compile-time exclusion where public API
compatibility permits; otherwise return a typed `CapabilityUnavailable` error.
Update feature-matrix tests and docs. Do not silently fall back to unrelated
retrieval under the `CascadeRetriever` name.

**Exit**: A non-`csm` build cannot produce a successful empty cascade result.

### PTA-A2: Make CLI storage metrics truthful

**Status: ✅ Implemented (2026-08-01).** Storage `stats`/`connections` now carry
`MetricValue` provenance (`measured`/`estimated`/`unavailable`); the fabricated
"last 24h", zero cache-hit rate, and fixed connection-pool values were removed.

Define measured/estimated/unavailable provenance in machine output, or remove
fields that cannot be measured through current backend interfaces. Human output
must label estimates. Do not report completed episode count as “last 24h,” zero
as an observed cache-hit rate, or fixed connection-pool values. Adding full
backend telemetry is a follow-up unless existing interfaces expose it.

**Exit**: Every displayed metric is measured, explicitly estimated, or unavailable.

### PTA-A3: Remove the advertised threshold non-operation

**Status: ✅ Implemented (2026-08-01).** `eval set-threshold` removed from Clap,
dispatch, help snapshot, and user docs; the dangling `eval show` reference is gone.

Remove/hide `eval set-threshold` from Clap and active documentation because no
override model is persisted or consumed by reward calculation. Correct references
to the existing `eval stats DOMAIN` command. A future override feature requires a
separate decision covering storage, precedence, runtime loading, and deletion.

**Exit**: CLI help contains no command guaranteed to fail by design.

### RAT-A1: Freeze attributed recommendation contracts

Add failing tests for legacy compatibility, episode validation, exact returned-ID
capture, empty recommendations, playbook generation errors, multiple sessions,
and all persistence receipt states. Define stable serialized receipt names and
ensure no raw backend errors cross MCP boundaries.

**Exit**: Tests distinguish recommendation failure, valid empty output, memory-only,
partial persistence, and total persistence failure.

### RAT-A2: Add capability-aware checked persistence

Add an explicit recommendation-attribution capability or typed unsupported result
so successful storage trait defaults cannot count as writes. Implement checked
session and feedback recording across Turso/redb/cache configurations. Keep old
warning-only public methods as compatibility delegates. Return backend-neutral,
tagged receipts and preserve detailed errors only in logs.

**Exit**: Receipt state is derived from configured capable backends and verified
loads, never from a successful no-op default.

### RAT-A3: Add attributed core APIs and remove hidden playbook recording

Add attributed pattern/playbook methods requiring a valid episode ID. Derive the
session from exact returned IDs through the shared checked recorder. Retain old
unattributed APIs unchanged. Remove `Uuid::nil()` playbook sessions and make
playbook generation return enough status to distinguish error from valid empty.

**Exit**: Core tests prove exact capture and no hidden attribution side effects.

### RAT-A4: Enforce session and feedback integrity

Resolve sessions by ID from memory or persistence before feedback. Reject unknown
sessions and applied IDs outside the recommended set. Define multiple sessions per
episode as valid and latest lookup as deterministic. Treat replacement feedback
as idempotent replacement in statistics; do not count it as another exposure or
application.

**Exit**: Orphan/mismatched feedback fails and repeated replacement does not inflate
adoption or success statistics.

### RAT-A5: Expose optional attribution through MCP

Add optional `episode_id` to pattern and playbook recommendation tool schemas.
Absent means the exact legacy response. Present means an attribution envelope is
returned with results. Route manual session/feedback tools through checked core
semantics and retain lazy registry/schema consistency.

**Exit**: MCP protocol tests cover both shapes and all stable receipt states.

### RAT-A6: Expose optional attribution through CLI

Add `--episode-id` to pattern and playbook recommendation commands. Preserve
legacy JSON/YAML exactly when absent; include the envelope when present. Human
output prints the session ID and persistence state, warning clearly for
process-only or failed durability. Keep manual feedback commands compatible.

**Exit**: CLI end-to-end tests can recommend for an episode, capture the returned
session ID, and submit validated feedback.

### RAT-A7: Document capture limits and define ranking follow-up

Update API/CLI/MCP docs to explain attributed versus legacy calls, receipt states,
restart implications, multiple sessions, and feedback validation. Record a
follow-up requirement for idempotent feedback-to-effectiveness ranking updates;
do not claim the learning loop is closed until that work has its own ADR/tests.

**Exit**: User docs call this attribution capture and make degraded persistence actionable.

### PTA-A9: Validate and update authority documents

Run focused crate tests after each package, then workspace gates. Update ADR-078
and ADR-079 to Accepted/Implemented only after code, live ruleset state, and
evidence exist. Record commit, feature set, UTC timestamp, and validation
artifacts per ADR-072.

## Quality gates

```bash
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy --workspace
./scripts/build-rust.sh check
cargo nextest run -p do-memory-core
cargo nextest run -p do-memory-storage-turso
cargo nextest run -p do-memory-storage-redb
cargo nextest run -p do-memory-mcp
cargo nextest run -p do-memory-cli
cargo nextest run --all
cargo test --doc
cargo doc --no-deps --document-private-items
./scripts/quality-gates.sh
./scripts/validate-gate-contract.sh --ci-parity
./scripts/validate-plans.sh --active-set --version-state --adrs --identifiers --links
```

Cascade checks must compile and test both default and `csm` feature sets.
Attribution storage tests must cover memory-only, redb-only, Turso-only, dual
backend, partial failure, total failure, and restart retrieval.

## Promotion gates

1. Accept ADR-078 before RAT-A2 changes public persistence semantics.
2. Accept ADR-079 before replacing required-check topology; obtain explicit
   approval immediately before mutating the repository ruleset.
3. Review the receipt/capability model before MCP/CLI wire changes.
4. Prove deterministic multiple-session behavior and idempotent feedback before
   calling attribution statistics trustworthy.
5. Do not promote the ranking follow-up without a separate durability and
   idempotency decision.

## Definition of done

- CIT-A1…A5, PTA-A1…A3, and RAT-A1…A7 exits are met.
- All quality gates pass with evidence recorded.
- ADR-078, ADR-079, and canonical trackers reflect actual, not intended, implementation.
- No plan or user documentation claims feedback changes ranking until verified.
