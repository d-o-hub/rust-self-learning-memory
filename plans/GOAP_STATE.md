# GOAP State Snapshot

- **Last Updated**: 2026-04-02 (v0.1.28 sprint — progress update)
- **Version**: `0.1.26` (released 2026-04-01, published to crates.io)
- **Branch**: `main`
- **Validation**: `plans/STATUS/VALIDATION_LATEST.md`
- **Gap Analysis**: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- **Primary ADRs**: ADR-048 (v0.1.24 stability), ADR-049 (v0.1.25 analysis)

---

## Current Focus: v0.1.28 Sprint (Executing)

| Task | WG | Status | Details |
|------|----|--------|---------|
| Merge PR #406 (ai-slop) | WG-091 | ⏳ Auto-merge armed (rebase) | CI re-running after branch update |
| Fix CodeQL alert #60 | WG-093 | ✅ Merged (PR #420) | CodeQL now reports `fixed` |
| Plans consolidation | ACT-096 | ✅ Merged (PR #420) | 87% noise reduction (5,000→650 lines) |
| Resolve Dependabot alerts | WG-092 | ✅ Tracked | All 3 transitive, in audit.toml/deny.toml |
| DyMoE routing-drift protection | WG-089 | 🔵 Impact analysis complete | See below |
| Dual reward scoring | WG-090 | 🔵 Impact analysis complete | See below |
| Close issue #401 | — | ⏳ Pending | Auto-closed when PR #406 merges |

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
| Workspace version | 0.1.26 | — |
| Total tests | ~2,849 | — |
| Ignored tests | 341 annotations / 82 files | ceiling ≤125 |
| `allow(dead_code)` (prod) | 37 | ≤40 |
| Clippy | Clean | 0 warnings |
| Doctests | 0 failures | 0 |
| Cargo audit | 3 unmaintained warnings | transitive |
| Dependabot alerts | 3 open | all transitive, tracked |
| CodeQL alerts | 0 open | ✅ fixed (PR #420) |

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
