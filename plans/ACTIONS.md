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

## Active Actions (v0.1.30 Sprint — Planning)

### Cross-Repo Impact Analysis (2026-04-09)

Source: `d-o-hub/github-template-ai-agents` + `d-o-hub/chaotic_semantic_memory`

- **ACT-097**: Implement `MemoryEvent` broadcast channel
   - Goal: WG-103
   - Action: Add `tokio::broadcast::Sender<MemoryEvent>` to episode service; emit on create/complete/gc events
   - Status: 🔵 Planned

- **ACT-098**: Replace sorted retrieval with `select_nth_unstable_by`
   - Goal: WG-104
   - Action: Identify retrieval hot paths using full sort; replace with O(n) partial sort for top-k
   - Status: 🔵 Planned

- **ACT-099**: Add idempotent cargo publish guard
   - Goal: WG-105
   - Action: Add crates.io version check to `publish-crates.yml` — `curl -s https://crates.io/api/v1/crates/$CRATE/$VERSION` → skip if 200
   - Status: 🔵 Planned

- **ACT-100**: Create `memory-context` skill
   - Goal: WG-106
   - Action: Port `memory-context` skill from `github-template-ai-agents`; adapt for local conventions; requires `cargo install chaotic_semantic_memory --features cli`
   - Status: 🔵 Planned

- **ACT-101**: Create `learn` skill
   - Goal: WG-107
   - Action: Port `learn` skill from `github-template-ai-agents`; dual-write pattern: distill → nearest `AGENTS.md`, verbose → `agents-docs/LESSONS.md`
   - Status: 🔵 Planned

## Completed Actions (v0.1.28–v0.1.29)

- **ACT-094**: Merge PR #406 (ai-slop detector) — ✅ Merged (WG-091)
- **ACT-095**: Fix CodeQL cleartext logging — ✅ Complete, commit fc9c302c (WG-093)
- **ACT-096**: Archive completed plans/ noise — ✅ 87% noise reduction
