# GOAP Execution Plan — v0.1.25 Analysis & Research-Driven Sprint

- **Date**: 2026-03-31
- **Branch**: `main`
- **Scope**: Noise reduction, Bayesian ranking, diversity retrieval, MCP server card, spawn_blocking audit, episode GC
- **Strategy**: Sequential Phase 0→1, then Parallel Phases 2–6
- **Primary ADR**: ADR-049
- **Research Sources**: arXiv:2602.02007, OpenSpace/HKUDS, MCP Roadmap Mar 2026, Turso Mar 2026, TokioConf/Carl Lerche Feb 2026

## Goals

1. Reduce plans/ noise — archive completed WGs/ACTs, trim GOAP_STATE.md
2. Fix CLI test bottleneck (670s → <30s)
3. Implement Bayesian/Wilson pattern ranking using existing attribution data
4. Add diversity-aware retrieval (research-driven, xMemory paper)
5. Add MCP Server Card for discoverability + token-savings metric
6. Audit CPU-heavy async paths for `spawn_blocking` compliance
7. Add episode garbage collection / TTL retention policy

## Quality Gates

- `./scripts/code-quality.sh fmt`
- `./scripts/code-quality.sh clippy --workspace`
- `cargo nextest run --all` → 0 failures, 0 timeouts
- `cargo test --doc` → 0 failures
- `./scripts/quality-gates.sh`

## Phase Plan

### Phase 0 — Noise Reduction (Sequential, P0, Day 1)

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-068 | Archive old GOAP plans | Move `GOAP_EXECUTION_PLAN_v0.1.20` through `v0.1.23` to `plans/archive/` | docs |
| WG-069 | Trim GOAP_STATE.md | Keep only v0.1.24+ sprint data; archive history to `plans/archive/GOAP_STATE_HISTORY.md` | docs |
| WG-070 | Archive completed GOALS/ACTIONS | Move completed WGs (1-67) and ACTs (1-88) to `plans/archive/`; keep only active/pending items | docs |

**Validation**: `ls plans/GOAP_EXECUTION_PLAN*.md | wc -l` = 2 (v0.1.24 + v0.1.25).

### Phase 1 — CLI Test Performance (Sequential, P0, Day 1-2)

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-072 | Fix cli_update_test 670s bottleneck | Options: (a) build CLI binary once with `lazy_static`/`once_cell`, (b) convert to unit tests on command parsing, (c) add `#[ignore]` with CI-only profile. Preferred: (b) — test the parse/dispatch layer, not the binary rebuild. | test-fix |

**Validation**: `cargo nextest run -p memory-cli --test cli_update_test` < 30s.

### Phase 2 — Bayesian Pattern Ranking (Parallel, P0, Days 2-4)

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-073 | Implement Wilson/Bayesian scoring | Add `memory-core/src/memory/pattern_search/bayesian_ranking.rs`. Wilson score lower-bound replaces fixed `effectiveness_weight`. Consumes `RecommendationSession` + `RecommendationFeedback` data. Falls back to current weights when data < 10 observations. | feature-implement |

**Validation**: Unit tests for Wilson score. Property tests for scoring invariants. Integration test showing ranking adapts with feedback.

### Phase 3 — Diversity-Aware Retrieval (Parallel, P0, Days 2-4)

**Research basis**: arXiv:2602.02007 (xMemory, Feb 2026) — standard top-k retrieval returns redundant spans from correlated episodic memory.

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-077 | Add coverage-diversity reranking | In `memory-core/src/memory/retrieval/context.rs`, after top-k cosine retrieval, apply MMR (Maximal Marginal Relevance) reranking using existing pattern categories as diversity signal. Simple formula: `score = λ·relevance - (1-λ)·max_similarity_to_selected`. Default λ=0.7. | feature-implement |

**Validation**: Unit test showing diverse results vs. pure cosine. Property test: no two results from same pattern category unless insufficient diversity.

### Phase 4 — Episode GC/TTL (Parallel, P1, Days 3-5)

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-075 | Episode garbage collection | Add `memory-core/src/memory/retention.rs`: configurable TTL (default 90 days), soft-delete + hard-delete, storage trait extension `delete_episodes_before(cutoff)`. MCP tool: `gc_episodes`. CLI: `episode gc --older-than 90d`. | feature-implement |

**Validation**: Unit + integration tests. GC'd episodes no longer returned by retrieval.

### Phase 5 — MCP Server Card + Token Savings Metric (Parallel, P1, Day 3)

**Research basis**: MCP Roadmap Mar 2026 (Server Cards), OpenSpace/HKUDS (46% token reduction tracking)

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-078 | MCP Server Card + token metric | (a) Add `.well-known/mcp.json` server card with tool inventory and capabilities metadata. (b) Add `tokens_saved` field to `RecommendationSession` to track ROI from pattern reuse. | feature-implement |

**Validation**: Server card is valid JSON, served at well-known path. `tokens_saved` populated on playbook recommendations.

### Phase 6 — spawn_blocking Audit (Parallel, P1, Day 3)

**Research basis**: Carl Lerche/TokioConf Feb 2026 — CPU-heavy work without `.await` stalls the runtime.

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-079 | Audit CPU-heavy async paths | Review `memory-core/src/extraction/`, `memory-core/src/memory/pattern_search/`, and DBSCAN clustering for CPU-heavy operations not wrapped in `spawn_blocking`. Fix any violations of the async invariant. | code-quality |

**Validation**: `cargo clippy` clean. No CPU-heavy loops (>1ms) outside `spawn_blocking` in production code.

## Dependencies & Sequencing

1. Phase 0 completes first (clean baseline for all subsequent work)
2. Phase 1 can run in parallel with Phase 0 (independent)
3. Phases 2, 3, 4, 5, 6 are independent — can run in parallel after Phase 0
4. No phase depends on another phase's output

## Contingencies

- If Bayesian ranking underperforms: keep as opt-in behind feature flag, default to current weights
- If CLI test fix requires binary caching infrastructure: use `#[ignore]` as interim
- If MCP Server Card spec changes: track upstream SEP and adapt

## Exit Criteria

- `cargo nextest run --all` total time < 60s (currently 136s due to CLI tests)
- Plans/ folder active files < 20 (currently ~50 non-archived)
- Pattern ranking uses real attribution data when available
- Retrieval returns diverse results (not redundant top-k)
- Episode GC exists and is configurable
- CPU-heavy async paths wrapped in `spawn_blocking`
