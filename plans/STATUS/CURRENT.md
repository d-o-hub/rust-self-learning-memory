# Project Status — Self-Learning Memory System

**Last Updated**: 2026-07-22  
**Released Version**: v0.1.36  
**Workspace Version**: 0.1.36  
**Edition**: Rust 2024  
**Active plan**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`  
**Branch**: `main`  

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | *(none on main at tracker refresh; see current branch PR)* |
| Open issues | *(none)* |

## Snapshot

| Area | State |
|------|--------|
| Post-v0.1.35 GOAP campaign (S1/W2/K3/F4/harness) | ✅ Merged (#840–#878) |
| Recommendations implementation | ✅ Merged (#878) |
| Release docs + rust-major deps | ✅ Merged (#880, #877) |
| Plans tracker refresh | ✅ Merged (#881) |
| Medium-risk skill evals (R-E2 second wave) | ✅ This PR |
| Release v0.1.36 | 🟡 **Next** — main green → `./scripts/release-manager.sh ship --execute` |
| Production LOC >500 (non-test `src`) | ✅ Clean |
| Skill evals | 34 skills with behavioral fixtures |
| Skill routes | `.agents/skills/skill-rules.json` complete for catalog |
| Code execution | Fail-closed (S1.1c NO-GO) |
| MCP provenance (`with_provenance`) | ✅ Present on `query_memory` |

## Immediate priorities

| Priority | Item | ID | Status |
|----------|------|-----|--------|
| P0 | Cut v0.1.36 + post-bump 0.1.37 | R-A1 / R-A2 | 🟡 after main green |
| P2 | Research/product spikes (R-F*) | R-F* | ⏸ DEFER until individual spikes GO |
| P2 | Transitive Dependabot advisories | G-P1-9 | Monitor / upstream |

## Recent completed (do not re-open)

| Wave | Result |
|------|--------|
| R-E2 medium-risk skill eval expansion | ✅ This PR |
| Plans tracker #881 | ✅ Merged |
| Release docs #880 / deps #877 | ✅ Merged |
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
