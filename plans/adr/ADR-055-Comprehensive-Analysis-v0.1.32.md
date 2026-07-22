# ADR-055: Comprehensive Analysis — v0.1.32 Post-Release

**Status**: Accepted
**Date**: 2026-06-06
**Deciders**: Agent codebase analysis (fresh `git fetch` + workspace audit on `main`)
**Supersedes**: N/A (consolidates the never-filed "ADR-055 Missing-Implementation-Remediation" reference)
**Related**: ADR-052 (v0.1.29), ADR-053 (v0.1.31), ADR-054 (CloudEvents / DAG-State), ADR-027 (Ignored tests)

---

## Context

A fresh analysis of `main` (fast-forwarded to `47a8609c`, 13 commits ahead of the
previously checked-out feature branch) revealed that the plan documents have drifted
significantly from the released reality. Verified facts collected on 2026-06-06:

### Release reality vs. plan claims

| Fact | Verified Value | Plan docs claimed |
|------|----------------|-------------------|
| Latest GitHub release | **v0.1.32 (2026-05-24)** via `gh release list` | "v0.1.31 released; v0.1.32 in flight, release pending" |
| Workspace version | `0.1.32` (`Cargo.toml`) | `0.1.32` ✅ |
| `cargo check --workspace --all-features` | ✅ Clean (3m33s) | n/a |
| Missing-impl sprint (WG-150..164) | Effectively complete; CHANGELOG attributes it | "9/15 landed, 6 open, Phase 4 blocked" |

### Broken cross-references (doc-integrity defects)

Four active plan docs reference two files that **were never created**:

- `plans/adr/ADR-055-Missing-Implementation-Remediation-v0.1.32.md` — **missing**
- `plans/GOAP_MISSING_IMPLEMENTATION_2026-05-21.md` — **missing**

Referencing files: `GOAP_STATE.md`, `ROADMAPS/ROADMAP_ACTIVE.md`, `STATUS/CURRENT.md`,
`STATUS/GAP_ANALYSIS_LATEST.md`. This ADR fills the ADR-055 slot with the actual,
verified analysis and the referenced GOAP plan is created as
[`GOAP_COMPREHENSIVE_ANALYSIS_2026-06-06.md`](../archive/2026-07-consolidation/analyses/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-06.md).

### Genuine remaining implementation gaps (verified by `rg`, not by stale docs)

The 2026-05-21/05-22 audit listed 6 "open" WGs. Re-running the audit against the
current tree shows **WG-156 and WG-157 are now resolved** (no placeholder remains;
CHANGELOG `[Unreleased]` confirms `pattern_match_score` and `memory_usage_mb` were
implemented). **Four genuine gaps remain:**

| Gap | Location | Current behavior | Notes |
|-----|----------|------------------|-------|
| Success rate placeholder | `memory-mcp/src/monitoring/types.rs:363` | `episode_success_rate = 99.0` on failure | No real failure counter |
| Turso cache stats stubbed | `memory-storage-turso/src/cache/wrapper.rs:142` | `query_hits: 0, query_misses: 0, evictions: 0, expirations: 0` | Counters never wired |
| Cascade API-cost estimate | `memory-core/src/retrieval/cascade/mod.rs:446` | `estimate_api_call_probability` returns constant `0.5` | `csm`-only; advisory metric |
| Simple embedding fallback | `memory-core/src/memory/retrieval/helpers.rs:59` | `generate_simple_embedding` documented as "placeholder" | **Has a production caller** at `memory/retrieval/context.rs:393` — the prior plan to `#[cfg(test)]`-gate it was incorrect |

### Metrics drift in STATUS/CURRENT.md

| Metric | CURRENT.md claim | Verified value (2026-06-06) |
|--------|------------------|------------------------------|
| `#[allow(dead_code)]` (prod src) | 0 | **15 files contain it** |
| Prod src files >500 LOC | 0 | 0 prod (1 *test* file `episode_relationships/tests/cases.rs` at 629 — acceptable) |
| Latest GitHub release | v0.1.31 | **v0.1.32** |
| Sprint status | in flight | **released** |

---

## Decision

### Phase 0 — Documentation Truth Reconciliation (P0, sequential)

The highest-value, lowest-risk work is making the plan docs match reality. The
codebase is healthy (clean build, released); the docs are not.

1. Mark the v0.1.32 missing-impl sprint **Released** in `GOAP_STATE.md`,
   `ROADMAP_ACTIVE.md`, and `STATUS/CURRENT.md`.
2. Repoint the four dangling `ADR-055-Missing-Implementation-Remediation` /
   `GOAP_MISSING_IMPLEMENTATION_2026-05-21` references to this ADR and the new
   GOAP plan.
3. Correct the `#[allow(dead_code)]` metric (0 → 15) and the release/version facts
   in `STATUS/CURRENT.md`.

### Phase 1 — Close the 4 Genuine Implementation Gaps (P1, parallel-safe)

Each gap is isolated to one module and can be implemented independently.

- **WG-171** (`episode_success_rate`): track real success/failure counts on the
  metrics struct and compute the rate; remove the `99.0` placeholder.
- **WG-172** (Turso cache stats): wire `AtomicU64` counters for `query_hits`,
  `query_misses`, `evictions`, `expirations` into `CacheStats`.
- **WG-173** (`estimate_api_call_probability`): implement a query-class heuristic
  (length / keyword density) **or** remove the method if no caller depends on it.
- **WG-174** (`generate_simple_embedding`): because it has a real production caller,
  either (a) promote it to a documented, intentional lightweight metadata-vector
  fallback (rename/comment, drop "placeholder" wording), or (b) replace the call
  site with the real embedding path. Decision deferred to implementation after
  confirming the call-site contract. **Do not `#[cfg(test)]`-gate it.**

### Phase 2 — Validation & v0.1.33 Release (P2, sequential)

Run the full gate stack (`fmt`, `clippy --workspace`, `nextest run --all`,
`cargo test --doc`, `quality-gates.sh`), confirm the sprint-exit `rg` audit returns
0 genuine placeholders, bump to `0.1.33`, update CHANGELOG, and release via
`release-guard`.

---

## Consequences

**Positive**
- Plan docs become trustworthy again; future audits start from accurate state.
- Four long-standing placeholders eliminated → telemetry/metrics truthfulness.
- Dangling ADR/GOAP references resolved (doc-integrity check clean).

**Negative / risks**
- WG-173/WG-174 may resolve to "remove/document" rather than "implement"; that is an
  acceptable, explicit outcome (consistent with ADR-055-era "resolved-by-typed-error"
  decisions for WG-152/WG-153).
- `csm`-gated WG-173 cannot be validated in the default feature set; requires
  `--features csm` in CI for that WG.

**Follow-ups**
- Re-introduce a periodic "doc drift" guard: every release must update
  `STATUS/CURRENT.md` release/version rows and re-run the metric collection commands
  in this ADR.

---

## Verification Commands (run 2026-06-06)

```bash
git fetch origin && git checkout main && git pull            # ff to 47a8609c
gh release list --limit 5                                    # v0.1.32 = Latest (2026-05-24)
grep -m1 '^version' Cargo.toml                               # 0.1.32
cargo check --workspace --all-features                       # clean (3m33s)
rg -ni 'not yet implemented|// *placeholder' memory-*/src    # 4 genuine gaps
rg -c 'allow\(dead_code\)' memory-*/src                      # 15 files
find memory-*/src -name '*.rs' | wc -l                       # 781 prod files
```
