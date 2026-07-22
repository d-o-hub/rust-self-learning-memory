# Validation Latest — 2026-07-22 Missing Tasks (R-E2 + Tracker)

**Goal**: Close remaining implementable plan gaps (medium-risk skill eval depth, tracker truth).  
**Workspace**: `0.1.36` unreleased · **Tag**: `v0.1.35`  
**Release**: ⛔ not performed by this validation (R-A1 remains next after main green)

## Evidence

| Check | Command / observation | Result |
|-------|----------------------|--------|
| Active set files present | `./scripts/validate-plans.sh --active-set` | ✅ |
| Version in CURRENT | workspace `0.1.36` | ✅ |
| Open issues | `gh issue list --state open` | empty |
| Prod LOC >500 (non-test `src`) | production offenders | **0** ✅ |
| Skill eval files | `find .agents/skills -path '*/evals/evals.json'` | 34 |
| Skill evals run | `./scripts/run-evals.sh` | All passed |
| Schema fixtures | `./scripts/run-evals.sh --fixtures` | All passed |
| Skill routes | `./scripts/validate-skill-routes.sh` | 34 unique skills |
| `todo!` / `unimplemented!` in prod `src` | `rg` | 0 |
| MCP provenance | `with_provenance` on `query_memory` | ✅ present |
| Release readiness | `./scripts/verify-release-state.sh` | ready to release v0.1.36 |

## This change set

- **ACT-310 / R-E2**: Expanded medium-risk skill `evals.json` fixtures from presence-only checks to behavioral contract tests (frontmatter, domain keywords, negative ship-path guards).
- **Plans truth**: CURRENT / GOALS / ACTIONS / GOAP_STATE / ROADMAP / GAP / VALIDATION refreshed for post-#877/#880/#881 state.

## Still open after this validation

| Item | Next step |
|------|-----------|
| G-P0-1 / R-A1 | Main green → `./scripts/release-manager.sh ship --execute` |
| R-A2 | Immediate post-tag bump to 0.1.37 |
| R-F* | DEFER until individual spikes under `plans/STATUS/spikes/` return GO |

## Merge / ship note

This change set is **skills + plans**. Do not ship a release from this PR alone unless main is green and release-guard path is followed after merge.
