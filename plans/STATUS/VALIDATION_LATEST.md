# Validation Latest — 2026-07-21 Open Tracker + Gap Refresh

**Goal**: Reconcile plans status with live GitHub open PRs/issues; close completed gap rows; keep single active recommendations plan.  
**Workspace**: `0.1.36` unreleased · **Tag**: `v0.1.35` · **HEAD**: `1ebab995`  
**Release**: ⛔ not performed by this validation (R-A1 remains next after #880)

## Evidence

| Check | Command / observation | Result |
|-------|----------------------|--------|
| Active set files present | `./scripts/validate-plans.sh --active-set` | Run on this branch |
| Version in CURRENT | workspace `0.1.36` | ✅ |
| Open PRs | `gh pr list --state open` | #880, #877 |
| Open issues | `gh issue list --state open` | #879 release drift |
| Commits since tag | `git rev-list --count v0.1.35..origin/main` | **26** |
| Prod LOC >500 (non-test `src`) | `provider_config.rs` | **237** ✅ |
| Skill eval files | `find .agents/skills -path '*/evals/evals.json'` | 34 |
| `todo!` / `unimplemented!` in prod `src` | `rg` | 0 |
| ADR 025/054 | filenames dual; `plans/adr/README.md` aliases | ✅ documented |
| #880 commitlint | body-max-line-length; squashed rewrite | ✅ pushed |
| #877 compile breakers | sha2 0.11 hex; lz4_flex `std`; cargo_metadata `PackageName` | ✅ pushed |

## Prior waves (historical)

- Recommendations implementation: PR **#878** merged  
- F4 remainder / missing-tasks / harness: #873/#874/#870 (2026-07-18)  
- Archive: `plans/archive/2026-07-consolidation/`

## Still open after this validation

| Item | Next step |
|------|-----------|
| G-P0-1 / R-A1 | Merge #880 → main green → `./scripts/release-manager.sh ship --execute` |
| R-A2 | Immediate post-tag bump to 0.1.37 |
| #877 | Wait CI green; merge when ready (has `release-preparation`) |
| #879 | Auto-resolves / updates after release |
| R-E2 / R-F* | Backlog |

## Merge / ship note

This change set is **plans/docs tracker refresh only**. Do not ship a release from this PR alone — use release-guard path after #880 lands on main.
