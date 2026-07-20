# Project Status — Self-Learning Memory System

**Last Updated**: 2026-07-20  
**Released Version**: v0.1.35  
**Workspace Version**: 0.1.36 (unreleased; 18 commits since tag)  
**Edition**: Rust 2024  
**Active plan**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`  
**Branch**: `main` @ `2e0a2b89`  
**Open PRs / issues**: none  

## Snapshot

| Area | State |
|------|--------|
| Post-v0.1.35 GOAP campaign (S1/W2/K3/F4/harness) | ✅ Merged (#840–#875) |
| Release v0.1.36 | 🟡 Recommended next (R-A1) |
| Production LOC >500 | 1 file (`provider_config.rs` 511) |
| Skill evals | 33/34 skills (`ci-poll` missing) |
| Skill routes | 16/34 in `skill-rules.json` |
| Code execution | Fail-closed (S1.1c NO-GO) |
| Plans hygiene | ✅ Consolidated 2026-07-20 |

## Immediate priorities (from recommendations)

| Priority | Item | ID | Status |
|----------|------|-----|--------|
| P0 | Cut v0.1.36 + post-bump 0.1.37 | R-A1 / R-A2 | 🟡 after PR merge |
| P0 | Split `provider_config.rs` ≤500 LOC | R-B1 | ✅ |
| P1 | Complete skill routes + `ci-poll` evals + SKILLS.md | R-C1–C3 | ✅ |
| P1 | F4 operator journal CLI | R-B2 / R-C5 | ✅ `storage journal` |
| P1 | ADR 025/054 alias registry | R-B5 | ✅ |
| P1 | Docs (AGENTS, TECH_DEBT, vision) | R-D* | ✅ partial |
| P2 | MCP provenance fields; research spikes | R-C4 / R-F* | Backlog |

## Recent completed (do not re-open)

| Wave | Result |
|------|--------|
| CLI UX v0.1.35 (#828–#832 family) | ✅ Shipped in v0.1.35 |
| ADR-075 durable complete + `episode fail` | ✅ |
| ADR-076 pattern empty-result UX | ✅ |
| S1.2–S1.7 correctness | ✅ |
| W2.1–W2.5 gate honesty | ✅ |
| K3.1/K3.2 skill evals | ✅ |
| F4.1–F4.4 pilots + spikes GO | ✅ |
| S1.1c Wasmtime spike | ✅ NO-GO |
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
