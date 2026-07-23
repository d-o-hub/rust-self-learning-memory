# Validation Latest — 2026-07-22 Ship + Post-bump

**Goal**: Close remaining plan P0 (ship v0.1.36 + post-bump 0.1.37).  
**Workspace**: `0.1.37` (this PR) · **Tag**: `v0.1.36`  

## Evidence

| Check | Result |
|-------|--------|
| `./scripts/release-manager.sh ship --execute` | ✅ tag `v0.1.36` pushed |
| Main CI at ship HEAD | All green (incl. Performance Benchmarks) |
| `./scripts/check-docs-integrity.sh` | ✅ (after #885) |
| Skill evals | ✅ (after #883) |
| Post-bump workspace `0.1.37` | 🟡 This PR |

## Still open

| Item | Next |
|------|------|
| R-A2 | Merge this post-bump PR |
| R-F* | DEFER until spike GO |

## Note

Do not cut another release from the post-bump PR alone. Next release when 0.1.37 content warrants it.
