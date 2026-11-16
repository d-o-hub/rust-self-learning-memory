# Project Status — Self-Learning Memory System

**Last Updated**: 2026-11-16
**Released Version**: v0.1.50 (`0fe1b89d`, GitHub Release 2026-11-16T08:45:18Z with 37
assets; all four registered crates verified at `0.1.50` on crates.io)
**Workspace Version**: 0.1.51 (post-release development bump)
**Edition**: Rust 2024 (toolchain pinned `1.99.1`)
**Active plan**: post-`v0.1.50` closeout. The release, publish chain and released-CLI
dogfood are green; the workspace is `0.1.51`. Remaining: upstream-blocked advisories and
maintainer-external ADR lifecycle acceptance.

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | `gh pr list --state open` is the live source; Dependabot group PRs may be open |
| Open issues | `gh issue list --state open` is the live source; open issues are not restated here |
| Dependency audit | Remaining Dependabot rows are upstream-blocked (ACT-404); consult the security workflow's latest run |
| Feature backlog | R-F1…R-F7 are `NO-GO` for this cycle (`STATUS/spikes/R-F-product-backlog-2026-11-15.json`) |

## Active plan — post-`v0.1.50` closeout

| Area | State | Evidence |
|------|-------|----------|
| Release | ✅ `v0.1.50` tagged on `0fe1b89d`; release workflow green; 37 assets | release workflow + `gh release view` |
| crates.io | ✅ all four registered crates at `0.1.50`; `do-memory-cli` / `do-memory-examples` remain unregistered by policy | crates.io API per crate |
| Released-CLI dogfood | ✅ asset digest `ea2da8ed…d4087d` matched, `--version` = 0.1.50, 3 steps, 1 episode, 2 patterns, `storage stats` 2 == `pattern list` 2 | this record |
| Development bump | ✅ workspace `0.1.51`; drift `severity=clean` | `check-release-drift.sh` |
| Harness contract | ✅ pin 0.4.6 measured green; doctrine updated to the 0.4.6 exit tiers | `check-do-harness-compat.sh`, `check-do-harness-doctrine.sh` |
| Plans tree | ✅ compacted; completed registers and prior snapshots archived | `plans/archive/2026-11-consolidation/` |
| Agent contract | ✅ `.jules/` and the `GEMINI.md` shim removed; `AGENTS.md` is the single contract | root listing, file-structure workflow |

## Recent closes (pointer)

| Window | Result |
|--------|--------|
| 2026-11-16 | `v0.1.50` release, publish chain green, released-CLI dogfood, `0.1.51` bump |
| 2026-11-15 | plans compaction (#1274), harness 0.4.6 pin (#1273), release-prep CHANGELOG/docs (#1275) |
| 2026-11-13 | `0.1.50` bump (#1267), Dependabot libsql pin policy (#1268), harness inputs for all sensors (#1269), dependency/action group bumps (#1270…#1272) |
| 2026-11-12 | `v0.1.49` release, publish chain green, released-CLI dogfood |
| 2026-11-04 | `v0.1.48` release and dogfood |
| 2026-10-28 | `v0.1.47` release, ADR-079 stage-4 proof, toolchain pin 1.99.1 |
| 2026-10-16 | Release closeout ACT-399…ACT-412 (publish guard, readiness ordering, schema sensor) |
| 2026-10-09 | Audit-backlog wave W1–W6 and Wave A (all 22 findings closed) |

Full evidence for those windows: `plans/STATUS/archive/2026/` and
`plans/archive/2026-11-consolidation/`.

## Snapshot (current)

| Area | State |
|------|-------|
| First-party merge gate | ✅ ruleset `9591004` requires `Codacy Static Code Analysis` + `CI / Required`; same-run, fail-closed aggregate |
| Serialization | Postcard required (no internally-tagged enums on stored types) |
| Code execution | Fail-closed; Wasmtime/WASI `NO-GO` |
| Storage | Turso durable + redb cache; parameterized SQL, short transactions |
| Nightly | Turso native suite isolated and fail-visible; last recorded scheduled run green (20/20, 0 skipped) |
| Skill evals / routes | ✅ full route validation and schema fixtures pass |
| Harness | ✅ `v0.4.6` pinned; feedback set, compat/doctrine probes and the fast verification sensors are green |

## Immediate priorities

| Priority | Item | ID | State |
|----------|------|----|-------|
| P1 | Upstream dependency advisories (named pinning chains, no no-op bump) | ACT-404 | ⏸ upstream |
| P2 | ADR-080 / ADR-081 lifecycle acceptance | maintainer | ⏸ external |
| P2 | Promote an R-F epic only after a measured spike passes its thresholds | R-F* | ⏸ `NO-GO` |
| P3 | Prepare the next release when cadence or a feature justifies it; use `release-guard` + `release-manager.sh ship --execute` | release | not scheduled |

## Canonical companions

- Roadmap: `plans/ROADMAPS/ROADMAP_ACTIVE.md`
- Goals / actions / GOAP: `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`
- Gaps: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- Validation: `plans/STATUS/VALIDATION_LATEST.md`
- Archive: `plans/archive/2026-11-consolidation/`
