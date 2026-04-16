# ADR-053: Comprehensive Analysis — v0.1.31 Sprint

**Status**: Accepted
**Date**: 2026-04-16
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

Backlog papers (WG-126, WG-127):
- **MemCollab** (arXiv:2603.23234): Cross-agent memory via contrastive trajectory distillation
- **CogitoRAG** (arXiv:2602.15895): Semantic gist extraction + CogniRank reranking

## Consequences

### Positive
- Closes 4-version release gap
- 30% reduction in skill count (49→35) and 40% LOC reduction
- Research-backed features strengthen episodic memory architecture
- Reduced cognitive load for agents loading skills

### Negative
- Skills consolidation requires updating all skill references
- Procedural memory type adds schema complexity
- Routing-Free MoE evaluation may lead to significant DyMoE refactor

### Risks
- Skills merge could break existing skill-rules.json mappings
- Temporal graph edges increase Turso query complexity

## Execution

- **Strategy**: Hybrid (Phase 0 sequential → Phase 1-2 parallel → Phase 3 sequential)
- **Work Items**: WG-111 through WG-127 (15 tasks + 2 backlog)
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
