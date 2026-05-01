# Active Development Roadmap

**Last Updated**: 2026-04-30 (v0.1.31 release verified)
**Released Version**: v0.1.31 (crates.io + GitHub Release)
**Branch**: `main` (all PRs merged)
**Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373) — ALL ISSUES CLOSED

---

## Current State

`v0.1.31` was released on 2026-04-30. Workspace and publishable crate versions are `0.1.31`.

Verified publishable workspace packages at `0.1.31`: `do-memory-core`, `do-memory-storage-redb`, `do-memory-storage-turso`, `do-memory-mcp`, `do-memory-cli`, `do-memory-examples`.

The 2026-04-21 comprehensive analysis added a CSM integration phase (BM25+HDC+ConceptGraph cascading retrieval) targeting 50-70% API call elimination, plus 6 new research papers and housekeeping WGs.

See [STATUS/CURRENT.md](../STATUS/CURRENT.md) for detailed metrics.

---

## Current Sprint — v0.1.31 (Released ✅)

**Source**: Release/package verification (2026-04-30)
**ADR**: ADR-053 (Accepted)

### Phase 0: Release & Package Truth

| Task | Description | Status | WG |
|------|-------------|--------|-----|
| WG-111 | Verify `v0.1.30` GitHub release and publishable crate parity | ✅ Complete | 111 |
| WG-112 | Bump workspace + publishable crates to `0.1.31`, update CHANGELOG | ✅ Complete | 112 |
| WG-113 | Refresh stale roadmap/status/GOAP truth sources after release verification | ✅ Complete | 113 |

### Phase 1: CPU Efficiency

| Task | Description | Status | WG |
|------|-------------|--------|-----|
| WG-114 | Reduce QueryCache contention and lock overhead (`parking_lot::RwLock` + benchmarks) | ✅ Complete | 114 |
| WG-115 | Replace placeholder Turso cached query paths with real storage-backed retrieval | ✅ Complete | 115 |
| WG-116 | Tune compression and zero-copy cache thresholds to avoid wasted CPU | ✅ Complete | 116 |

### Phase 1.5: CSM Integration (CPU-Local Retrieval) ✅ Complete

**Implementation**: CSM added as crate dependency (`chaotic_semantic_memory = "0.3.2"`), not source code copy.
Types re-exported under `csm` feature flag in `memory-core/src/retrieval/mod.rs`.

| Task | Description | Status | WG |
|------|-------------|--------|-----|
| WG-128 | Add BM25 keyword index from `chaotic_semantic_memory` as first retrieval tier | ✅ Complete | 128 |
| WG-129 | Wire HDC text encoder as CPU-local embedding fallback (via crate) | ✅ Complete | 129 |
| WG-130 | Add ConceptGraph ontology expansion for synonym retrieval without LLM | ✅ Complete | 130 |
| WG-131 | Implement cascading retrieval pipeline: BM25 → HDC → ConceptGraph → API | ✅ Complete (732 LOC, 20+ tests) | 131 |

### Phase 2: Token Efficiency

| Task | Description | Status | WG |
|------|-------------|--------|-----|
| WG-117 | Implement `BundleAccumulator` sliding window for bounded context assembly | ✅ Complete | 117 |
| WG-118 | Add hierarchical/gist reranking to return fewer, denser context items | ✅ Complete | 118 |
| WG-119 | Compact high-frequency skills/docs to reduce prompt token load | ✅ Complete | 119 |

### Phase 3: Research-Inspired Retrieval Upgrades

| Task | Description | Status | WG |
|------|-------------|--------|-----|
| WG-120 | Add reconstructive retrieval windows around top-k hits (E-mem-inspired) | ✅ Complete | 120 |
| WG-121 | Add execution-signature retrieval for traces and failures (APEX-EM-inspired) | ✅ Complete | 121 |
| WG-122 | Add scope-before-search shard routing to cut query cost (ShardMemo-inspired) | ✅ Complete | 122 |

### P3: Backlog (Future Sprints)

| Task | Description | Status | WG |
|------|-------------|--------|-----|
| WG-123 | Temporal graph edges in episode store (REMem-inspired, arXiv:2602.13530) | 🔵 Backlog | 123 |
| WG-124 | Procedural memory type: learned heuristics-as-skills (ParamAgent-inspired) | 🔵 Backlog | 124 |
| WG-125 | Evaluate Routing-Free MoE for DyMoE replacement (arXiv:2604.00801) | 🔵 Backlog | 125 |
| WG-108 | Version-retained persistence (concept drift tracking) | 🔵 Backlog | 108 |
| WG-109 | `BundleAccumulator` sliding window (recency-weighted context) | ✅ Complete (WG-117) | 109 |
| WG-110 | SIMD-accelerated similarity (defer until benchmarks justify) | 🔵 Backlog | 110 |
| WG-126 | Cross-agent memory collaboration via contrastive trajectory distillation (MemCollab, arXiv:2603.23234) | 🔵 Backlog | 126 |
| WG-127 | Semantic gist extraction + CogniRank reranking (CogitoRAG, arXiv:2602.15895) | 🔵 Backlog | 127 |
| WG-132 | LottaLoRA-inspired local episode classifier (arXiv:2604.08749) | 🔵 Backlog | 132 |
| WG-133 | Align memory types with Anatomy of Agentic Memory taxonomy (arXiv:2602.19320) | 🔵 Backlog | 133 |
| WG-134 | DAG-based state management for episode context — 86% token reduction (arXiv:2602.22398) | 🔵 Backlog | 134 |
| WG-135 | Federated HDC for multi-agent memory sharing (arXiv:2603.20037) | 🔵 Backlog | 135 |
| WG-136 | Create `performance` skill (referenced but missing) | ✅ Complete | 136 |
| WG-137 | Prune skills from 40 → ≤35 (merge/remove 5 overlapping skills) | ✅ Complete | 137 |
| WG-138 | Fix STATUS/CURRENT.md metric contradictions (dead_code 35 vs 41) | ✅ Complete | 138 |
| WG-139 | Refresh CODEBASE_ANALYSIS_LATEST.md (stale since 2026-03-09) | ✅ Complete | 139 |

---

## Completed Sprint — v0.1.30 ✅

**Source**: Cross-repo impact analysis of `d-o-hub/github-template-ai-agents` and `d-o-hub/chaotic_semantic_memory` (2026-04-09)
**ADR**: ADR-037 (Selective Workflow Automation Adoption)

### P1: Runtime Pattern Adoption from CSM

| Task | Description | Status | WG |
|------|-------------|--------|-----|
| WG-103 | `MemoryEvent` broadcast channel for episode lifecycle | ✅ Complete | 103 |
| WG-104 | `select_nth_unstable_by` O(n) top-k in retrieval hot paths | ✅ Complete | 104 |
| WG-105 | Idempotent cargo publish (crates.io version check) | ✅ Already exists | 105 |

### P2: Agent Harness Skill Adoption from Template

| Task | Description | Status | WG |
|------|-------------|--------|-----|
| WG-106 | `memory-context` skill — CSM CLI for HDC retrieval over lessons | ✅ Complete | 106 |
| WG-107 | `learn` skill — dual-write post-task learning pattern | ✅ Complete | 107 |

---

## Completed Sprint — v0.1.29 ✅

**ADR**: [ADR-052](../adr/ADR-052-Comprehensive-Analysis-v0.1.29.md)
**PR**: [#425](https://github.com/d-o-hub/rust-self-learning-memory/pull/425)

### Phase 0: Version & Hygiene

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-094 | Bump workspace version to 0.1.29, update CHANGELOG | ✅ Complete | 94 |
| WG-095 | Archive stale GOAP plans, trim GOALS/ACTIONS | ✅ Complete | 95 |

### Phase 1: WASM Removal — Swarm Decision

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-096 | Remove WASM sandbox (1,899 LOC, 127 refs, 11 files) | ✅ Complete | 96 |
| WG-097 | Remove wasmtime + rquickjs from workspace deps | ✅ Complete | 97 |

### Phase 2: Turso Native Vector Search

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-098 | Replace brute-force with `vector_top_k()` DiskANN queries | ✅ Complete | 98 |
| WG-099 | Add embedding migration (JSON text → F32_BLOB binary) | ✅ Complete | 99 |
| WG-100 | Integration tests for native vector search | ✅ Complete | 100 |

### Phase 3: Quality

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-101 | Split remaining >500 LOC files (6 files split) | ✅ Complete | 101 |
| WG-102 | Dead code audit (31 → target ≤25 `#[allow(dead_code)]`) | ✅ Complete | 102 |

---

## Completed Sprint — v0.1.28 ✅

### P1: Feature Enhancements

| Task | Description | Status | WG | Source |
|------|-------------|--------|----|--------|
| WG-089 | DyMoE routing-drift protection + affinity gating | ✅ Complete | 89 | [#419](https://github.com/d-o-hub/rust-self-learning-memory/issues/419) |
| WG-090 | Dual reward scoring (stability + novelty signals) | ✅ Complete | 90 | [#419](https://github.com/d-o-hub/rust-self-learning-memory/issues/419) |

### P1: Security

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-092 | Resolve open Dependabot alerts | ✅ Analyzed (transitive) | 92 |
| WG-093 | Fix CodeQL cleartext logging alert in feedback CLI | ✅ Fixed | 93 |

### P2: CI/Infra

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-091 | Merge AI spam detector PR #406 | ✅ Merged | 91 |

---

## Previous Sprint — v0.1.27 Feature Sprint (Complete ✅)

### P1: Feature Enhancements

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-073 | Bayesian ranking with Wilson score from attribution data | ✅ Complete | 73 |
| WG-077 | MMR diversity retrieval for search results | ✅ Complete | 77 |
| WG-075 | Episode GC/TTL with retention policy | ✅ Complete | 75 |
| WG-078 | MCP Server Card at `.well-known/mcp.json` | ✅ Complete | 78 |

### P2: Code Quality

| Task | Description | Status |
|------|-------------|--------|
| WG-079 | Audit spawn_blocking usage for CPU-heavy async paths | ✅ Complete |

### P2: Infrastructure

| Task | Description | Status |
|------|-------------|--------|
| WG-084 | Restore GitHub Pages with mdBook | ✅ Complete |
| WG-085 | Add llms.txt for LLM context | ✅ Complete |

### P3: CI Optimization

| Task | Description | Status |
|------|-------------|--------|
| Semver timeout fix | Increase timeout + add baseline caching | ✅ Complete (PR #416) |

---

## Upcoming Sprint — v0.1.23 Remediation (Complete)

The 2026-03-24 audit reopened several items. The new sprint focuses on truth-source reset, ADR-044 durability, CI/test parity, and disk/DX hygiene. Full execution plan: [GOAP_EXECUTION_PLAN_v0.1.23.md](../GOAP_EXECUTION_PLAN_v0.1.23.md).

### P0: Feature Integrity (ADR-044)

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-051 | Persist recommendation sessions + feedback via storage traits and schema updates | ✅ Complete (2026-03-24) | 51 |
| WG-052 | Persist checkpoints/handoffs across Turso/redb read paths | ✅ Complete (2026-03-24) | 52 |
| WG-053 | Decide/implement batch MCP tool strategy + align status/docs | ✅ Complete (2026-03-24) | 53 |

### P1: Documentation & Contract Truth Sources

| Task | Description | Status |
|------|-------------|--------|
| WG-054 | Regenerate API reference, README, playbook/checkpoint docs, CLI command tables | ✅ Complete (2026-03-24) |
| WG-058 | Align AGENTS.md, agent_docs/, `.agents/skills/` with script-first workflow + disk/cov guidance | ✅ Complete (2026-03-24) |

### P1: Validation & Coverage Parity

| Task | Description | Status |
|------|-------------|--------|
| WG-055 | Expand PR-required CI workflows to cover integration + CLI/MCP suites, benchmark coverage | ✅ Complete (2026-03-24) |
| WG-056 | Enforce ≥90% coverage threshold in scripts/tests (update `scripts/check-coverage.sh`, `quality_gates.rs`) | ✅ Complete (2026-03-24) |

### P2: Disk & Developer Experience

| Task | Description | Status |
|------|-------------|--------|
| WG-057 | Reduce local disk footprint (target/node_modules) + automate cleanup | ✅ Complete (2026-03-24) |

---

## Sprint v0.1.22 — COMPLETE ✅

### P0: Critical Fixes — ALL COMPLETE ✅

| Task | Description | Status | Issue |
|------|-------------|--------|-------|
| WG-040 | Fix 2 failing doctests (attribution, playbook) | ✅ Complete | #374 — CLOSED |
| WG-041 | Fix test timeout (quality_gate_no_clippy_warnings) | ✅ Complete | #375 — CLOSED |
| WG-042 | Split 3 production files >500 LOC | ✅ Complete | #376 — CLOSED |

### P1: Quality Polish — ALL COMPLETE ✅

| Task | Description | Final | Target | Status | Issue |
|------|-------------|-------|--------|--------|-------|
| WG-043 | Reduce `#[allow(dead_code)]` | 31 | ≤40 | ✅ Target met | #377 — CLOSED |
| WG-044 | Fix broken markdown links | 0 active | ≤80 | ✅ 101 archived-only (acceptable) | #378 — CLOSED |
| WG-045 | Add snapshot tests for new features | 80 | ≥80 | ✅ Complete | #379 — CLOSED |
| WG-046 | Add property tests for new features | 16 | ≥13 | ✅ Exceeds target | #380 — CLOSED |

### P2: Feature Enhancements — ALL COMPLETE ✅

| Task | Description | Status | Issue |
|------|-------------|--------|-------|
| WG-047 | MCP tool contract parity for new tools | ✅ Complete | #381 — CLOSED |
| WG-048 | Integration tests for attribution + checkpoint flows | ✅ Complete | #382 — CLOSED |
| WG-049 | Changelog automation (git-cliff) | ✅ Complete | #383 — CLOSED |
| WG-050 | Documentation for new features | ✅ Complete | #384 — CLOSED |

### P3: Infrastructure — ALL COMPLETE ✅

| Task | Description | Since | Status | Issue |
|------|-------------|-------|--------|-------|
| WG-051 | Nightly trend tracking artifact | v0.1.20 | ✅ Complete | #385 — CLOSED |
| WG-052 | libsql upstream version monitor | v0.1.20 | ✅ Complete | #386 — CLOSED |
| WG-053 | Structured tech-debt registry | v0.1.17 | ✅ Complete | #387 — CLOSED |

### Shipped in v0.1.21

- ✅ ADR-045: Publishing infrastructure (supply chain, OIDC, metadata)
- ✅ ADR-046: Claude Code configuration improvements (session analysis, tool enforcement)

### Shipped in v0.1.22 (Features — Pre-Tag)

- ✅ ADR-044: High-Impact Features (100% code complete)
  - ✅ Feature 1: Actionable Playbooks (26 tests)
  - ✅ Feature 2: Recommendation Attribution (8 tests)
  - ✅ Feature 3: Episode Checkpoints/Handoff (6 tests)
  - ✅ Feature 4: Recommendation Feedback (3 tests)

---

## Backlog

### Code Quality (from #373 epic — ALL RESOLVED)

| Item | Current | Target | Status | Notes |
|------|---------|--------|--------|-------|
| `#[allow(dead_code)]` annotations | 31 | ≤40 | ✅ Target met | Down from 70 |
| Broken markdown links | 0 active | ≤80 | ✅ 101 archived-only (acceptable) | |

### Testing

| Item | Current | Target | Status | Notes |
|------|---------|--------|--------|-------|
| Ignored tests | 124 | ≤125 ceiling | ✅ | 70 upstream libsql bug, rest by design |
| Property test expansion | 16 files | ≥13 | ✅ Exceeds target | |
| Snapshot test growth | 80 snaps | ≥80 | ✅ Target met | |

### Infrastructure

| Item | Status | Notes |
|------|--------|-------|
| Changelog automation (git-cliff) | ✅ Complete | `.github/workflows/changelog.yml` |
| Structured tech-debt registry | ✅ Complete | `docs/TECH_DEBT.md` |
| libsql version monitor | ✅ Complete | `scripts/check-libsql-version.sh` |
| Nightly trend tracking | ✅ Complete | Artifact added |
| CLI workflow parity generator | Not started | Opportunity O6 |

---

## Release History

| Version | Date | Highlights |
|---------|------|------------|
| v0.1.31 | 2026-04 | CSM integration (BM25+HDC+ConceptGraph via crate dependency), BundleAccumulator, hierarchical reranking, skills consolidation (31 skills, ≤35 target met), release verification |
| v0.1.30 | 2026-04 | MemoryEvent broadcast, top-k optimization, memory-context skill, learn skill, zero-copy retrieval caching, CSM pattern adoption (WG-103/104) |
| v0.1.29 | 2026-04 | WASM sandbox removal (-6,982 LOC), Turso native vector search (vector_top_k/DiskANN), file splitting (6 files), release workflow improvements |
| v0.1.27 | 2026-04 | Wilson score ranking, Episode GC/TTL, spawn_blocking audit, MCP Server Card, GitHub Pages, llms.txt, semver timeout fix |
| v0.1.24 | 2026-03 | Test stability (DBSCAN budget, quality gate timeout), dependency updates |
| v0.1.22 | 2026-03 | ADR-044 High-Impact Features (Playbooks, Attribution, Checkpoints, Feedback) |
| v0.1.21 | 2026-03 | Publishing infrastructure (ADR-045), supply chain security |
| v0.1.20 | 2026-03 | Test coverage improvements, sprint fixes, coverage script |
| v0.1.19 | 2026-03 | MCP enhancements, gitleaks fixes |
| v0.1.18 | 2026-03 | AdaptiveCache, CLI filters, transport compression docs |
| v0.1.17 | 2026-03 | MCP contract parity, dead code removal, doc fixes, G2/G9 |
| v0.1.16 | 2026-02 | Edition 2024, CI stabilization, quick wins |
| v0.1.15 | 2026-02 | MCP token optimization, GitHub Actions modernization |
| v0.1.14 | 2026-02 | Episode tagging, relationships, file compliance |
| v0.1.13 | 2026-01 | Semantic pattern search, recommendation engine |
| v0.1.12 | 2026-01 | Tasks utility, embedding config, contrastive learning |

---

## Cross-References

- **Current status**: [STATUS/CURRENT.md](../STATUS/CURRENT.md)
- **Gap analysis**: [STATUS/GAP_ANALYSIS_LATEST.md](../STATUS/GAP_ANALYSIS_LATEST.md)
- **Execution plans**: [GOAP_EXECUTION_PLAN_v0.1.22.md](../archive/2026-03-consolidation/GOAP_EXECUTION_PLAN_v0.1.22.md), [GOAP_EXECUTION_PLAN_v0.1.23.md](../archive/2026-03-consolidation/GOAP_EXECUTION_PLAN_v0.1.23.md)
- **Long-term vision**: [ROADMAP_V030_VISION.md](ROADMAP_V030_VISION.md)
- **ADRs**: [adr/](../adr/)
