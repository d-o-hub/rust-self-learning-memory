# Plans Consolidation — 2026-11-15

Completed plans and tracker snapshots archived while `v0.1.49` is released and the
workspace is on the `0.1.50` development version. The active set is defined by
ADR-039 / ADR-072 and checked by `./scripts/validate-plans.sh --active-set`; the dated
files were removed from `plans/` root so the root carries only the canonical set.

## Layout

| Path | Contents |
|------|----------|
| `GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md` | 22-issue audit-backlog wave; every verdict closed |
| `GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md` | Post-v0.1.36 recommendations register; all tracks closed |
| `GOAP_RELEASE_CLOSEOUT_2026-10-16.md` | Release closeout ACT-399…ACT-412; all landed |
| `GOAP_RELEASE_PIPELINE_C5_C7_2026-10-04.md` | Release-pipeline C5–C7; merged in #1125 |
| `trackers-2026-11-13/` | Pre-compaction CURRENT / GOALS / ACTIONS / GOAP_STATE snapshots |

The full 2026-11-01 gap and validation records are preserved next to the other status
archives:

- `plans/STATUS/archive/2026/GAP_ANALYSIS_2026-11-01.md`
- `plans/STATUS/archive/2026/VALIDATION_2026-11-01.md`

## Active set after consolidation

`plans/README.md`, `GOALS.md`, `ACTIONS.md`, `GOAP_STATE.md`, `GATE_CONTRACT.md`,
`invariants.json`, `ROADMAPS/`, `STATUS/`, `adr/`, `spikes/`, and
[`STATUS/spikes/R-F-product-backlog-2026-11-15.json`](../../STATUS/spikes/R-F-product-backlog-2026-11-15.json)
as the authoritative product gate.

Archived documents are historical evidence, not backlog. Links inside them are not
maintained (ADR-039). Do not re-open a completed campaign without new regression
evidence.
