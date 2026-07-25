# Validation Latest — 2026-07-25

**Goal**: Reconcile plans trackers with live post-v0.1.36 + post-#893 state.  
**Workspace**: `0.1.37` · **Tag**: `v0.1.36` · **HEAD**: `5b4b9776`  

## Evidence

| Check | Observation | Result |
|-------|-------------|--------|
| Released tag | `v0.1.36` published 2026-07-22 | ✅ |
| Workspace version | `Cargo.toml` `0.1.37` | ✅ |
| Open issues | `gh issue list --state open` | empty |
| Open PRs | #894 skills+plans (CI green) | live |
| R-A1 / R-A2 | ship + post-bump | ✅ closed |
| R-E2 / docs integrity | #883 / #885 | ✅ merged |
| R-F8 / R-F9 | #893 | ✅ merged |
| Skills | 40/40 routed | ✅ |
| ADR-071 | auto-checkpoint on Abstained | ✅ Implemented |
| ADR-072 | authority + governance | ✅ Implemented |
| ADR-073 | S1.1c NO-GO, fail-closed | ✅ Implemented |
| Prod `todo!` / unimplemented | live search | 0 |

## Closed validation goals

| Goal | Result |
|------|--------|
| Ship v0.1.36 | ✅ release-manager + release.yml |
| Post-bump 0.1.37 | ✅ #886 |
| Skill eval depth (R-E2) | ✅ #883 |
| Docs integrity ship gate | ✅ #885 |
| R-F8 relationship polish | ✅ #893 |
| R-F9 HNSW persistence | ✅ #893 |
| 6 new domain skills (40 total) | ✅ #894 |
| ADR-071/072/073 status updated | ✅ #894 |

## Still open after this validation

| Item | Next step |
|------|-----------|
| #894 | Merge when CI CLEAN |
| R-F* remaining | DEFER until individual GO spikes |
| v0.1.37 release | Trigger when unreleased commits accumulate |
