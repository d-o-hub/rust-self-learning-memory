# GOAP State Snapshot

- **Last Updated**: 2026-04-04 (v0.1.29 sprint — executing)
- **Version**: `0.1.29` (workspace, unreleased)
- **Branch**: `feature/v0.1.29-sprint`
- **Validation**: `plans/STATUS/VALIDATION_LATEST.md`
- **Gap Analysis**: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- **Primary ADRs**: ADR-052 (v0.1.29 comprehensive analysis)

---

## Current Focus: v0.1.29 Sprint (Executing)

| Task | WG | Status | Details |
|------|----|--------|---------|
| Version bump to 0.1.29 | WG-094 | ✅ Complete | Workspace + inter-crate deps updated |
| Archive stale GOAP plans | WG-095 | ✅ Complete | Already archived in v0.1.28 sprint |
| Remove WASM sandbox | WG-096 | ⏳ In progress | 1,899 LOC, 11 files to remove |
| Remove wasmtime/rquickjs deps | WG-097 | ⏳ Blocked by WG-096 | Cargo.toml cleanup |
| Implement vector_top_k search | WG-098 | ⏳ Pending | Native DiskANN queries |
| Embedding format migration | WG-099 | ⏳ Blocked by WG-098 | JSON → F32_BLOB |
| Integration tests | WG-100 | ⏳ Blocked by WG-098/099 | Vector search tests |
| Split >500 LOC files | WG-101 | ⏳ Blocked by WG-096 | LOC counts change after WASM removal |
| Dead code audit | WG-102 | ⏳ Blocked by WG-096 | WASM removal affects count |

## v0.1.28 Sprint (Complete ✅)

| Task | WG | Status |
|------|----|--------|
| DyMoE routing-drift protection | WG-089 | ✅ Affinity gating |
| Dual reward scoring | WG-090 | ✅ Stability + novelty signals |
| Merge AI spam detector PR #406 | WG-091 | ✅ Merged |
| Dependabot alerts | WG-092 | ✅ Tracked (transitive) |
| CodeQL cleartext logging | WG-093 | ✅ Fixed |
| Plans consolidation | ACT-096 | ✅ 87% noise reduction |

## v0.1.27 Sprint (Complete ✅)

| Task | WG | Status |
|------|----|--------|
| Bayesian ranking | WG-073 | ✅ Wilson score from attribution data |
| Diversity retrieval | WG-077 | ✅ MMR reranking |
| Episode GC/TTL | WG-075 | ✅ Retention policy |
| MCP Server Card | WG-078 | ✅ `.well-known/mcp.json` |
| spawn_blocking audit | WG-079 | ✅ CPU-heavy async paths |
| GitHub Pages | WG-084 | ✅ mdBook + cargo doc |
| llms.txt | WG-085 | ✅ LLM context file |

## v0.1.26 Release (Complete ✅)

| Task | Status |
|------|--------|
| Crate renaming (`memory-*` → `do-memory-*`) | ✅ |
| Version bump `0.1.25` → `0.1.26` | ✅ |
| crates.io publish (all 4 crates) | ✅ |
| Binary names (`do-memory-mcp-server`, `do-memory-cli`) | ✅ |
| GitHub Release (tag v0.1.26, multi-platform) | ✅ |

---

## Key Metrics

| Metric | Value | Target |
|--------|-------|--------|
| Workspace version | 0.1.29 | — |
| Total tests | ~2,875 | — |
| Ignored tests | 125 annotations | ceiling ≤125 |
| `allow(dead_code)` (prod) | 31 | ≤40 (target ≤25) |
| Clippy | Clean | 0 warnings |
| Doctests | 0 failures | 0 |
| Cargo audit | 3 unmaintained warnings | transitive |
| Dependabot alerts | 3 open | all transitive, tracked |
| CodeQL alerts | 0 open | ✅ fixed |

---

## Active Issues / Blockers

| Issue | Status | Notes |
|-------|--------|-------|
| CLI_TURSO_SEGFAULT | Known | libsql upstream memory corruption; 71 Turso tests `#[ignore]` (ADR-027) |
| Dependabot alerts (3) | Tracked | All transitive deps, documented in `audit.toml` / `deny.toml` |
| CodeQL alert #60 | ✅ Fixed | PR #420 merged; CodeQL reports `fixed` |
| Issue #401 | Pending | Auto-closes when PR #406 merges |

---

## DyMoE Feature Impact Analysis (Issue #419)

### Architecture Touchpoints

| Component | File | Current Behavior | DyMoE Change |
|-----------|------|------------------|-------------|
| Pattern extraction | `memory/learning.rs` | Extracts & stores all patterns unconditionally | Add affinity gate before `store_pattern` |
| Effectiveness tracker | `patterns/effectiveness/calculator.rs` | Uniform decay by min_effectiveness (0.3) | Add `affinity_clarity` as second gating dimension |
| Reward scoring | `reward/mod.rs` | Single composite `RewardScore` (total/base/efficiency/etc.) | Add `stability_score` + `novelty_score` fields |
| Pattern types | `pattern/types.rs` | `PatternEffectiveness` tracks success_rate + avg_reward_delta | No schema change needed |
| Hybrid extractor | `patterns/extractors/hybrid.rs` | Runs 4 extractors, clusters, deduplicates | Insert affinity classifier before clustering |
| Cosine similarity | `embeddings/similarity.rs` | `cosine_similarity(a, b) -> f32` | Reuse existing — no new dependency |
| Recommendation | `memory/pattern_search/recommendation.rs` | Multi-signal scoring (semantic + recency + success) | Wire `EpisodeAssignmentGuard` into scoring |

### Impact Assessment

| Change | LOC Estimate | Risk | Files Modified | New Files |
|--------|-------------|------|---------------|-----------|
| `EpisodeAssignmentGuard` (WG-089 part 2) | ~50 | Low | `effectiveness/calculator.rs` | 0 |
| `PatternAffinityClassifier` (WG-089 part 1) | ~80 | Medium | `learning.rs`, `hybrid.rs` | `patterns/affinity.rs` |
| `DualRewardScore` (WG-090) | ~60 | Low | `reward/mod.rs` | 0 |
| DB schema migration | ~20 | Low | turso schema | 0 |
| Tests | ~200 | None | — | `tests/dymoe_*.rs` |
| **Total** | **~410** | **Medium** | **4–5** | **1–2** |

### Recommended Execution Order (from issue #419)
1. `EpisodeAssignmentGuard` — smallest diff, highest leverage
2. `PatternAffinityClassifier` — depends on cosine_similarity infra (already exists)
3. `DualRewardScore` — extends existing `RewardScore`, backward-compatible

### Key Finding
The existing `cosine_similarity` function in `embeddings/similarity.rs` can be reused directly for `compute_drel()`. No new embedding infrastructure needed. The `PatternEffectiveness` already tracks `success_rate` which maps to the `success_rate` dimension of the `EpisodeAssignmentGuard`.

---

## Self-Learnings (Consolidated)

Patterns extracted from v0.1.17–v0.1.27 sprint history (34 sessions, 234 msgs, 97 commits):

### API / Dependency Changes
- **redb 3.x**: `begin_read()` is on the `ReadableDatabase` trait — import accordingly
- **rand 0.10**: `thread_rng()` → `rand::rng()`, `gen()` → `random()`, `gen_range()` → `random_range()`

### Development Workflow
- **Turso local dev**: Use `turso dev` — no auth token needed
- **Doctest quality**: New features must have tested doctests before merge
- **File size invariant**: Templates extraction prevents LOC growth (≤500 LOC per file)
- **Integration tests**: Feature implementation without integration tests leaves gaps
- **Documentation sync**: ROADMAP / CURRENT must stay in sync with releases
- **Plan document drift**: Always verify metrics by running actual commands, not trusting stale docs

### CI / GitHub
- **PR supersession**: Track supersession chains (e.g., #388 → #389 → #391) to avoid referencing stale PRs
- **codecov/patch**: Not a required CI check — configure thresholds in `codecov.yml`, don't block merges
- **Jules PRs**: Reconcile external agent PRs before planning more work — they may implement "pending" items

### Quality Gates
- **Ignored test ceiling**: Monitor the 125 ceiling metric; actual count can creep close
- **Tool selection**: Target Bash:Grep ratio of 2:1 (historical 17:1 indicates over-reliance on shell)
- **Atomic commits**: One change per commit — 5 excessive_changes instances in early sprints
- **Wrong approach**: Read 3+ source files before implementing — 8 instances of proceeding without patterns

---

## Cross-References

| Document | Location |
|----------|----------|
| Archived Execution Plans | `plans/archive/2026-03-consolidation/` |
| Active Roadmap | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| Latest Validation | `plans/STATUS/VALIDATION_LATEST.md` |
| Latest Gap Analysis | `plans/STATUS/GAP_ANALYSIS_LATEST.md` |
| ADR Index | `plans/adr/` |
