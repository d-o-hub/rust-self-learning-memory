# GOAP Actions Backlog

- **Last Updated**: 2026-04-02 (v0.1.28 sprint)
- **Archived Plans**: `plans/archive/2026-03-consolidation/`

## Completed Actions Summary

All actions from v0.1.17 through v0.1.27 sprints are complete. See archived execution plans in `plans/archive/2026-03-consolidation/` for full details.

| Sprint | Actions | Count | Status |
|--------|---------|-------|--------|
| v0.1.27 | Bayesian, GC, Pages, llms.txt, semver fix | 7 | ✅ All Complete |
| v0.1.24 | ACT-089 through ACT-093 | 5 | ✅ All Complete |
| v0.1.23 | ACT-080 through ACT-088 | 9 | ✅ All Complete |
| v0.1.22 | ACT-053 through ACT-075 | 23 | ✅ All Complete |
| v0.1.21 | ACT-038 through ACT-052 | 15 | ✅ All Complete |
| v0.1.20 | ACT-020 through ACT-037 | 18 | ✅ All Complete |
| v0.1.17-19 | ACT-001 through ACT-019 | 19 | ✅ All Complete |

## Learning Delta (2026-03)

### redb 3.x Breaking Changes
- `begin_read()` moved to `ReadableDatabase` trait (must import it)
- `begin_write()` remains on `Database` struct (no change)

### rand 0.10 Breaking Changes
- `thread_rng()` → `rand::rng()` (function rename)
- `Rng::gen()` → `RngExt::random()` (method rename)
- `Rng::gen_range()` → `RngExt::random_range()` (method rename)
- Import `RngExt` for user-level RNG methods
- Keep `rand` and `rand_chacha` versions aligned

## Active Actions (v0.1.28 Sprint)

- **ACT-094**: Merge PR #406 (ai-slop detector)
   - Goal: WG-091
   - Action: Auto-merge with rebase enabled; verify CI + close #401
   - Status: ⏳ Auto-merge armed

- **ACT-095**: Fix CodeQL cleartext logging alert #60
   - Goal: WG-093
   - Action: Remove session_id/episode_id from dry-run println in feedback CLI
   - Status: ✅ Complete — commit fc9c302c

- **ACT-096**: Archive completed plans/ noise
   - Goal: Reduce plans/ from ~5,000 lines to ~500 lines
   - Action: Move completed GOAP plans + STATUS files to archive; compact GOAP_STATE, GOALS, ACTIONS
   - Status: ⏳ In Progress
