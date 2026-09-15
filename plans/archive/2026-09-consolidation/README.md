# Plans Consolidation — 2026-09-15

Completed execution plans archived during the v0.1.41 cycle. Canonical active set
is defined by ADR-039 / ADR-072 and `./scripts/validate-plans.sh --active-set`;
this archive keeps the execution history out of the active tree because
`plans/README.md` root had grown past the ADR-039 dated-file budget
(`validate-plans.sh --active-set` warned: 10 dated files > 5).

## Layout

| Directory | Contents |
|-----------|----------|
| `superseded-goap/` | Finished GOAP waves (CIT-A1/A4, PR review CI fix, ADR-081 capability truth, codebase truth, runtime embedding activation, fuzz/nightly LTO, perf-PR guardrails) |
| `analyses/` | Dated `gh`/`gh skill` best-practices analysis |
| `operations-reference/` | do-harness integration plan (sensors, hooks, AGENTS.md diff — all landed) |
| `stale-status/` | Superseded status snapshots (`STATUS/CI_ANALYSIS_2026-07-27.md`) |

## Active set after consolidation

`plans/README.md`, `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`,
`plans/GATE_CONTRACT.md`, `plans/invariants.json`, `plans/ROADMAPS/`, `plans/STATUS/`,
`plans/adr/`, `plans/spikes/`, plus the recommendations register
`plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`.

Links inside archived documents are intentionally not maintained (ADR-039).
