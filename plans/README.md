# Plans Directory

**Workspace**: `v0.1.51` (post-release development bump) · **Latest published tag**: `v0.1.50`
(`0fe1b89d`, GitHub Release 2026-11-16T08:45:18Z, 37 assets; all four registered crates
verified at `0.1.50` on crates.io) · **Main baseline**: `eb076391` · **Last updated**: 2026-11-24
**Open PRs / issues**: `gh pr list --state open` / `gh issue list --state open` are the
live sources; Dependabot group PRs may be open
**Policy**: ADR-039 (canonical active set) + ADR-072 (authority / release path)

## Active state

`v0.1.50` shipped end-to-end and the workspace moved to the `0.1.51` development version in
its own post-release PR. Every code-side task in the audited backlogs is closed; the release
and publish chains are green and the released CLI was dogfooded. Durable `EpisodeSummary`
persistence and policy-driven episode retention cleanup (`WG-075`) are implemented end-to-end
across `do-memory-core`, `TursoStorage` (plus `ResilientStorage` and `CachedTursoStorage`),
`RedbStorage`, `StorageSynchronizer`, `do-memory-cli` (`episode view` + `episode cleanup` +
`storage sync`), and `do-memory-mcp` (`get_episode` + `cleanup_episodes`). R-F2
(OpenTelemetry) was promoted from the gated product backlog after a measured spike and landed
as ADR-084; the remaining work is external:

| Item | State | Where |
|------|-------|-------|
| Dependency advisories (rustls / rustls-webpki / lru / libsql-sqlite3-parser) | ⏸ upstream-blocked; do not open a no-op bump | `ACTIONS.md` ACT-404, `STATUS/GAP_ANALYSIS_LATEST.md` |
| ADR-080 / ADR-081 / ADR-082 lifecycle acceptance | ✅ accepted 2026-11-16 | `adr/` |
| Durable `EpisodeSummary` & retention cleanup (`WG-075`) across storage, sync, CLI, MCP | ✅ shipped 2026-11-22..24 (#1289 + CLI/MCP/wrapper parity) | `memory-core`, `memory-storage-*`, `memory-cli`, `memory-mcp` |
| R-F1, R-F3…R-F7 product epics (episode versioning remains gated) | ⏸ `NO-GO` for this cycle | `STATUS/spikes/R-F-product-backlog-2026-11-15.json` |
| R-F2 OTel / OTLP tracing | ✅ shipped (ADR-084) | `adr/ADR-084-Opt-In-OpenTelemetry-OTLP-Tracing.md` |
| External-consumer adaptive-discovery example (#1281 + candidate window) | ✅ shipped 2026-11-17..24; recall 0.33 → 0.87 on the fixture | `examples/adaptive_source_discovery.rs`, `docs/ADAPTIVE_SOURCE_DISCOVERY.md` |
| Next release | not scheduled; the `0.1.51` development bump keeps drift `severity=clean` | `ROADMAPS/ROADMAP_ACTIVE.md` |

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

Current lifecycle status: [ADR-080](adr/ADR-080-Automatic-Recommendation-Attribution.md),
[ADR-081](adr/ADR-081-Attribution-Capability-Truth-And-Feedback-Resolution.md) and
[ADR-082](adr/ADR-082-Feedback-Ranking-Adaptation.md) are `Accepted` with
recorded test evidence. ADR-079 (fail-closed `CI / Required`), ADR-083 (release-gate event
filtering) and ADR-084 (opt-in OTLP tracing) are `Accepted`.

## Archive

Completed plans live under [archive/](archive/) and are not active backlog.
Latest consolidation: [archive/2026-11-consolidation/](archive/2026-11-consolidation/)
(2026-11-15) — completed 2026-07/10 GOAP registers, the 2026-11-01 gap/validation records,
and the prior tracker snapshots. Earlier consolidations: 2026-09, 2026-07, 2026-03.

Archived documents keep their historical links; link rot there is intentional (ADR-039).
