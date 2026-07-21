# Project Status — Self-Learning Memory System

**Last Updated**: 2026-07-21  
**Released Version**: v0.1.35  
**Workspace Version**: 0.1.36 (unreleased; 26 commits since tag)  
**Edition**: Rust 2024  
**Active plan**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`  
**Branch**: `main` @ `1ebab995`  

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | **#880** release docs v0.1.36 · **#877** rust-major deps (sha2/lz4_flex/cargo_metadata) |
| Open issues | **#879** release drift (`commit_warning`, 26 commits / 2 days) |

## Snapshot

| Area | State |
|------|--------|
| Post-v0.1.35 GOAP campaign (S1/W2/K3/F4/harness) | ✅ Merged (#840–#878) |
| Recommendations backlog implementation | ✅ Merged (#878) |
| Release v0.1.36 | 🟡 **Next** — merge #880 → main green → `release-manager.sh ship --execute` |
| Production LOC >500 (non-test `src`) | ✅ Clean (`provider_config.rs` 237 after R-B1 split) |
| Skill evals | 34 eval files under `.agents/skills/*/evals/` |
| Skill routes | `.agents/skills/skill-rules.json` complete for catalog |
| Code execution | Fail-closed (S1.1c NO-GO) |
| Plans hygiene | ✅ Refreshed 2026-07-21 (open PR/issue truth) |

## Immediate priorities

| Priority | Item | ID | Status |
|----------|------|-----|--------|
| P0 | Merge #880 (release docs) when CI CLEAN | R-A1 prep | 🟡 CI |
| P0 | Cut v0.1.36 + post-bump 0.1.37 | R-A1 / R-A2 | 🟡 after #880 + main green |
| P0 | Land #877 dependabot major bumps (compat fixes pushed) | deps | 🟡 CI |
| P1 | Close #879 after ship | release-drift | 🟡 auto after release |
| P2 | MCP provenance fields; research spikes | R-C4 / R-F* | Backlog |

## Recent completed (do not re-open)

| Wave | Result |
|------|--------|
| Recommendations #878 (skills, LOC, journal CLI, plans) | ✅ Merged |
| CLI UX v0.1.35 (#828–#832 family) | ✅ Shipped in v0.1.35 |
| ADR-075 durable complete + `episode fail` | ✅ |
| ADR-076 pattern empty-result UX | ✅ |
| S1.2–S1.7 / W2.1–W2.5 / K3.1–K3.2 / F4 pilots | ✅ |
| Release-cadence-manager + release-guard | ✅ |
| Harness engineering #861–#869 | ✅ |

## Metrics notes

- Prefer command-backed numbers in `VALIDATION_LATEST.md` over prose estimates.
- Coverage floor / target: see `plans/GATE_CONTRACT.md` (do not invent % in status).
- Ignored-test ceiling: enforced by `./scripts/check-ignored-tests.sh`.

## Canonical companions

- Roadmap: `plans/ROADMAPS/ROADMAP_ACTIVE.md`
- Goals / actions / GOAP: `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`
- Gaps: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- Archive: `plans/archive/2026-07-consolidation/`
