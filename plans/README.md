# Plans Directory

**Workspace**: `v0.1.50` (release preparation) · **Latest published tag**: `v0.1.49`
(`16d92d22`, GitHub Release 2026-11-12T17:23:40Z, 37 assets; all four registered crates
verified at `0.1.49` on crates.io) · **Main baseline**: `4dd1d9aa` · **Last updated**: 2026-11-15
**Open PRs / issues**: 0 / 0 (`gh pr list --state open`, `gh issue list --state open`)
**Policy**: ADR-039 (canonical active set) + ADR-072 (authority / release path)

## Active state

`v0.1.49` shipped and the workspace moved to the `0.1.50` development version in its own
release-closeout PR. Every code-side task in the audited backlogs is closed; the release
and publish chains are green and the released CLI was dogfooded. **There is no open code
task in this tree.** The only remaining work is external:

| Item | State | Where |
|------|-------|-------|
| Dependency advisories (rustls / rustls-webpki / lru / libsql-sqlite3-parser) | ⏸ upstream-blocked; do not open a no-op bump | `ACTIONS.md` ACT-404, `STATUS/GAP_ANALYSIS_LATEST.md` |
| ADR-080 / ADR-081 lifecycle acceptance | ⏸ maintainer-external | `adr/` |
| R-F1…R-F7 product epics | ⏸ `NO-GO` for this cycle | `STATUS/spikes/R-F-product-backlog-2026-11-15.json` |
| Next release | not scheduled; cadence is `severity=clean` (6 commits, 0 feat, 2 fix, 3 days) | `ROADMAPS/ROADMAP_ACTIVE.md` |

## Quick navigation

| Document | Purpose |
|----------|---------|
| [STATUS/CURRENT.md](STATUS/CURRENT.md) | Live project status and metrics |
| [STATUS/VALIDATION_LATEST.md](STATUS/VALIDATION_LATEST.md) | Latest verification evidence |
| [STATUS/GAP_ANALYSIS_LATEST.md](STATUS/GAP_ANALYSIS_LATEST.md) | Live gap register (upstream-blocked advisories) |
| [STATUS/CODEBASE_ANALYSIS_LATEST.md](STATUS/CODEBASE_ANALYSIS_LATEST.md) | Current codebase summary |
| [ROADMAPS/ROADMAP_ACTIVE.md](ROADMAPS/ROADMAP_ACTIVE.md) | Forward-only roadmap |
| [GOALS.md](GOALS.md) | GOAP goal index |
| [ACTIONS.md](ACTIONS.md) | Action backlog and prevention rules |
| [GOAP_STATE.md](GOAP_STATE.md) | GOAP phase snapshot and truth flags |
| [GATE_CONTRACT.md](GATE_CONTRACT.md) | Local/CI quality gate matrix |

## Architecture and decisions

| Document | Purpose |
|----------|---------|
| [ARCHITECTURE/ARCHITECTURE_CORE.md](ARCHITECTURE/ARCHITECTURE_CORE.md) | Core system architecture |
| [ARCHITECTURE/ARCHITECTURE_PATTERNS.md](ARCHITECTURE/ARCHITECTURE_PATTERNS.md) | Design patterns |
| [ARCHITECTURE/ARCHITECTURE_INTEGRATION.md](ARCHITECTURE/ARCHITECTURE_INTEGRATION.md) | Integration architecture |
| [ARCHITECTURE/API_DOCUMENTATION.md](ARCHITECTURE/API_DOCUMENTATION.md) | API surface |
| [adr/README.md](adr/README.md) | ADR index (including number-collision aliases) |
| [spikes/](spikes/) + [STATUS/spikes/](STATUS/spikes/) | Spike configs and decision artifacts |

Current lifecycle proposals: [ADR-080](adr/ADR-080-Automatic-Recommendation-Attribution.md)
and [ADR-081](adr/ADR-081-Attribution-Capability-Truth-And-Feedback-Resolution.md)
(`Proposed`; code evidence merged, acceptance maintainer-external). ADR-079 (fail-closed
`CI / Required`) and ADR-083 (release-gate event filtering) are `Accepted`.

## Archive

Completed plans live under [archive/](archive/) and are not active backlog.
Latest consolidation: [archive/2026-11-consolidation/](archive/2026-11-consolidation/)
(2026-11-15) — completed 2026-07/10 GOAP registers, the 2026-11-01 gap/validation records,
and the prior tracker snapshots. Earlier consolidations: 2026-09, 2026-07, 2026-03.

Archived documents keep their historical links; link rot there is intentional (ADR-039).
