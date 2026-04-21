# ADR-053: Comprehensive Analysis — v0.1.31 Sprint

**Status**: Accepted
**Date**: 2026-04-21 (refreshed with CSM integration + new papers)
**Deciders**: Agent analysis + academic paper review
**Supersedes**: N/A
**Related**: ADR-052 (v0.1.29), ADR-037 (CSM adoption), ADR-046 (agent config)

---

## Context

Post-v0.1.30 analysis identified four areas requiring attention:

1. **Version release gap**: v0.1.26 is the last published release; v0.1.27–v0.1.30 represent 40+ commits without tags/releases
2. **Skills bloat**: 49 skills totaling 6,764 LOC with significant overlap (5 merge groups, 3 oversized skills)
3. **Tech debt**: 64 clippy suppressions in `lib.rs`, 35 `#[allow(dead_code)]`, 4 files >500 LOC
4. **Research opportunities**: 6 papers from Feb–Apr 2026 directly applicable to the episodic memory architecture
5. **CSM integration opportunity**: Analysis of `d-o-hub/chaotic_semantic_memory` v0.3.2 identified CPU-local retrieval tiers (BM25, HDC, ConceptGraph) that can eliminate 50-70% of embedding API calls
6. **New research papers**: 6 additional papers from Feb-Apr 2026 discovered: LottaLoRA (arXiv:2604.08749), Anatomy of Agentic Memory (arXiv:2602.19320), Keyword Search RAG (arXiv:2602.23368), DAG State Management (arXiv:2602.22398), Federated HDC (arXiv:2603.20037), HD Cross-Modal Alignment (arXiv:2602.23588)
7. **Implementation gaps**: 9 placeholder/stub locations in production code, `performance` skill referenced 6× but missing, STATUS/CURRENT.md has contradictory metrics

## Decision

### Phase 0: Release & Hygiene (P0)

Release v0.1.30 to close the version gap. Audit the 64 clippy suppressions in `memory-core/src/lib.rs` — most are blanket `clippy::restriction` suppressing pedantic lints that provide no value.

### Phase 1: Skills Consolidation (P1)

Merge 14 overlapping skills into 6, reducing from 49 → ~35 skills and ~6,764 → ~4,000 LOC:

| Merge Group | From | To | LOC Savings |
|-------------|------|----|-------------|
| Build | `build-compile` (178) + `build-rust` (102) | `build-rust` | ~150 |
| Research | `perplexity-researcher-pro` (428) + `perplexity-researcher-reasoning-pro` (467) + `web-search-researcher` (47) | `web-researcher` | ~700 |
| Code quality | `code-quality` (96) + `rust-code-quality` (81) + `clean-code-developer` (395) | `code-quality` | ~400 |
| Context | `context-retrieval` (108) + `context-compaction` (41) + `memory-context` (86) | `memory-context` | ~150 |
| Testing | `quality-unit-testing` (95) + `episodic-memory-testing` (93) + `rust-async-testing` (97) | `test-patterns` | ~180 |

Compact oversized skills: `git-worktree-manager` (549→150), `yaml-validator` (486→100), `general` (403→100).

### Phase 1.5: CSM Cascading Retrieval (P0)

Integrate `chaotic_semantic_memory` v0.3.2 as optional dependency behind `csm` feature flag. Implement cascading retrieval:

| Tier | Method | CPU Cost | API Cost | Quality |
|------|--------|----------|----------|---------|
| 1 | BM25 keyword search | Low | **Zero** | Exact/lexical matches |
| 2 | HDC 10,240-bit encoding | Medium | **Zero** | Token-overlap similarity |
| 3 | ConceptGraph expansion | Negligible | **Zero** | Curated synonym recall |
| 4 | API embedding (fallback) | None | 1 call | Full semantic similarity |

Validated by arXiv:2602.23368 ("Keyword Search Is All You Need"): BM25 achieves >90% of RAG performance without vector databases.

Work items: WG-128 (BM25), WG-129 (HDC fallback), WG-130 (ConceptGraph), WG-131 (cascade pipeline)

### Phase 2: Code Quality (P1)

- Split 4 remaining >500 LOC files (retention.rs, affinity.rs, ranking.rs, handlers.rs)
- Reduce `#[allow(dead_code)]` from 35 → ≤25
- Update stale docs (STATUS/CURRENT.md, AGENTS.md frozen metrics)

### Phase 3: Research-Inspired Features (P2)

Three papers with direct implementation potential:

| Paper | Key Idea | Application |
|-------|----------|-------------|
| **REMem** (ICLR 2026, arXiv:2602.13530) | Hybrid memory graph with time-aware gists + agentic retriever | Add temporal graph edges between episodes/patterns in Turso; +13.4% on reasoning tasks |
| **ParamAgent** (2026) | Three-tier memory: parametric + episodic + procedural | Add procedural memory type for learned heuristics-as-skills |
| **Routing-Free MoE** (arXiv:2604.00801, Apr 2026) | Self-activating experts, no centralized router | Evaluate as DyMoE replacement; eliminates routing-drift problem |

| **LottaLoRA** (arXiv:2604.08749, Apr 2026) | Random frozen backbone + LoRA adapters recover 96-100% of trained perf; validates HDC + ESN approach | CPU-only episode type classifier via reservoir computing |
| **Anatomy of Agentic Memory** (arXiv:2602.19320) | Taxonomy of 4 memory structures; benchmarks saturated, system costs overlooked | Align memory types to canonical taxonomy |
| **DAG State Management** (arXiv:2602.22398) | DAG-based conversation state: 86% token reduction, reference Claude Code impl | Adapt for episode context assembly |
| **Federated HDC** (arXiv:2603.20037) | HDC prototype exchange for federated learning, minimal bandwidth | Multi-agent memory sharing via HDC prototypes |
| **Keyword Search RAG** (arXiv:2602.23368) | BM25 achieves >90% of RAG without vector DB | Validates BM25-first cascade (WG-128) |

Backlog papers (WG-126, WG-127):
- **MemCollab** (arXiv:2603.23234): Cross-agent memory via contrastive trajectory distillation
- **CogitoRAG** (arXiv:2602.15895): Semantic gist extraction + CogniRank reranking

## Consequences

### Positive
- Closes 4-version release gap
- 30% reduction in skill count (49→35) and 40% LOC reduction
- Research-backed features strengthen episodic memory architecture
- Reduced cognitive load for agents loading skills
- CSM cascade eliminates 50-70% of embedding API calls (CPU-local BM25+HDC tiers)
- Local embedding fallback enables fully offline operation
- 6 additional research papers strengthen theoretical foundation

### Negative
- Skills consolidation requires updating all skill references
- Procedural memory type adds schema complexity
- Routing-Free MoE evaluation may lead to significant DyMoE refactor
- CSM adds optional dependency (~3K LOC crate)
- HDC is lexical-only (not semantic) — "cat" ≠ "kitten" limitation
- ConceptGraph requires manual ontology curation

### Risks
- Skills merge could break existing skill-rules.json mappings
- Temporal graph edges increase Turso query complexity

## Execution

- **Strategy**: Hybrid (Phase 0 sequential → Phase 1-2 parallel → Phase 3 sequential)
- **Work Items**: WG-111 through WG-139 (29 tasks + backlog)
- **Quality Gates**: 3 checkpoints per GOAP methodology
- **Estimated Effort**: 2–3 sprints

## References

- REMem: <https://arxiv.org/abs/2602.13530>
- Routing-Free MoE: <https://arxiv.org/abs/2604.00801>
- MemCollab: <https://arxiv.org/abs/2603.23234>
- CogitoRAG: <https://arxiv.org/abs/2602.15895>
- Memory in the Age of AI Agents: <https://arxiv.org/abs/2512.13564>
- ParamAgent: Three-tier parametric memory (2026)
- Memento-Skills: Skills as structured markdown via memory-based RL (2026)
- LottaLoRA: <https://arxiv.org/abs/2604.08749>
- Anatomy of Agentic Memory: <https://arxiv.org/abs/2602.19320>
- Keyword Search RAG: <https://arxiv.org/abs/2602.23368>
- DAG State Management: <https://arxiv.org/abs/2602.22398>
- Federated HDC: <https://arxiv.org/abs/2603.20037>
- Comprehensive Analysis: `plans/STATUS/COMPREHENSIVE_ANALYSIS_2026-04-21.md`
- CSM Repo: <https://github.com/d-o-hub/chaotic_semantic_memory>
