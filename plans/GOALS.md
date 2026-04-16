# GOAP Goals Index

- **Last Updated**: 2026-04-16 (v0.1.31 PLANNING)
- **Source ADR**: ADR-037, ADR-052, ADR-053 (pending)
- **Status**: Active

---

## v0.1.31 Sprint Goals (Planning 🔵)

### Source: Comprehensive Analysis (2026-04-16)

Codebase analysis + academic paper review identified: skill bloat (49 skills, 6,764 LOC), clippy suppression debt (64 lints), version release gap (v0.1.26→v0.1.30), and research-inspired feature opportunities.

### Phase 0: Release & Hygiene

1. **WG-111**: Release v0.1.30
   - Priority: P0
   - Owner: release-guard, commit
   - Target: Tag v0.1.30, publish to crates.io, create GitHub Release
   - Dependencies: None

2. **WG-112**: Version bump to 0.1.31
   - Priority: P0
   - Owner: feature-implement
   - Target: Bump workspace version, update CHANGELOG
   - Dependencies: WG-111

3. **WG-113**: Clippy suppression audit
   - Priority: P1
   - Owner: refactorer
   - Target: Reduce `lib.rs` `#[allow(clippy::*)]` from 64 → ≤20
   - Dependencies: None

### Phase 1: Skills Consolidation

4. **WG-114**: Merge build skills (`build-compile` + `build-rust`)
   - Priority: P1
   - Owner: skill-creator
   - Target: Single `build-rust` skill

5. **WG-115**: Merge research skills (`perplexity-researcher-*` + `web-search-researcher`)
   - Priority: P1
   - Owner: skill-creator
   - Target: Single `web-researcher` skill with tier toggle

6. **WG-116**: Merge code-quality skills (`code-quality` + `rust-code-quality` + `clean-code-developer`)
   - Priority: P1
   - Owner: skill-creator
   - Target: Single `code-quality` skill

7. **WG-117**: Merge context skills (`context-retrieval` + `context-compaction` + `memory-context`)
   - Priority: P1
   - Owner: skill-creator
   - Target: Single `memory-context` skill

8. **WG-118**: Merge test-pattern skills (`quality-unit-testing` + `episodic-memory-testing` + `rust-async-testing`)
   - Priority: P1
   - Owner: skill-creator
   - Target: Single `test-patterns` skill

9. **WG-119**: Compact oversized skills
   - Priority: P2
   - Owner: skill-creator
   - Target: `git-worktree-manager` 549→≤150, `yaml-validator` 486→≤100, `general` 403→≤100

### Phase 2: Code Quality

10. **WG-120**: Split >500 LOC files
    - Priority: P1
    - Owner: refactorer
    - Target: Split retention.rs, affinity.rs, ranking.rs, handlers.rs

11. **WG-121**: Reduce dead_code annotations
    - Priority: P2
    - Owner: refactorer
    - Target: 35 → ≤25 `#[allow(dead_code)]`

12. **WG-122**: Update stale documentation
    - Priority: P2
    - Owner: agents-update
    - Target: STATUS/CURRENT.md, AGENTS.md stale metrics, skill descriptions referencing "2025"

### Phase 3: Research-Inspired Features

13. **WG-123**: Temporal graph edges in episode store
    - Priority: P2
    - Owner: feature-implement
    - Target: Add graph relationships between episodes/patterns with temporal weights
    - Paper: REMem (ICLR 2026, arXiv:2602.13530) — hybrid memory graph, +13.4% reasoning

14. **WG-124**: Procedural memory type
    - Priority: P2
    - Owner: feature-implement
    - Target: New memory type for learned heuristics-as-skills (episodic + semantic + procedural)
    - Paper: ParamAgent (2026) — three-tier memory architecture

15. **WG-125**: Routing-Free MoE evaluation
    - Priority: P3
    - Owner: code-reviewer
    - Target: Evaluate replacing DyMoE centralized routing with self-activating experts
    - Paper: arXiv:2604.00801 (Apr 2026) — eliminates routing drift, better scalability

### Backlog (Future)

16. **WG-126**: Cross-agent memory collaboration (MemCollab)
    - Priority: P3
    - Owner: feature-implement
    - Paper: arXiv:2603.23234 — contrastive trajectory distillation for agent-agnostic memory

17. **WG-127**: Semantic gist extraction + CogniRank (CogitoRAG)
    - Priority: P3
    - Owner: feature-implement
    - Paper: arXiv:2602.15895 — gist-based retrieval outperforms flat RAG

---

## v0.1.30 Sprint Goals (Complete ✅)

### Cross-Repo Impact Analysis Source

Impact analysis of `d-o-hub/github-template-ai-agents` and `d-o-hub/chaotic_semantic_memory` identified unadopted runtime patterns and skill gaps. All P1/P2 goals achieved.

### P1: Runtime Patterns (All Complete)

1. **WG-103**: `MemoryEvent` broadcast channel ✅
   - Priority: P1
   - Owner: feature-implement
   - Target: Add `tokio::broadcast`-based event channel for episode lifecycle
   - Result: `types/event.rs` + `subscribe()` method on SelfLearningMemory

2. **WG-104**: `select_nth_unstable_by` for top-k retrieval ✅
   - Priority: P1
   - Owner: feature-implement
   - Target: Replace O(n log n) sort with O(n) partial sort
   - Result: `search/top_k.rs` module with `select_top_k()` utilities

3. **WG-105**: Idempotent cargo publish ✅
   - Priority: P1
   - Owner: ci-fix
   - Target: Add crates.io version check before `cargo publish`
   - Result: Already exists in `publish-crates.yml` (version check step)

### P2: Agent Harness Skills (All Complete)

4. **WG-106**: Add `memory-context` skill ✅
   - Priority: P2
   - Owner: skill-creator
   - Target: Skill for episode retrieval via do-memory-cli
   - Result: `.agents/skills/memory-context/SKILL.md`

5. **WG-107**: Add `learn` skill (dual-write learning) ✅
   - Priority: P2
   - Owner: skill-creator
   - Target: Post-task learning pattern
   - Result: `.agents/skills/learn/SKILL.md`

### P3: Future Backlog

6. **WG-108**: Version-retained persistence
   - Priority: P3
   - Owner: feature-implement
   - Target: Track concept drift across episode versions
   - Status: 🔵 Backlog

7. **WG-109**: `BundleAccumulator` sliding window
   - Priority: P3
   - Owner: feature-implement
   - Target: Recency-weighted context for pattern retrieval
   - Status: 🔵 Backlog

8. **WG-110**: SIMD-accelerated similarity
   - Priority: P3
   - Owner: feature-implement
   - Target: SIMD cosine similarity — defer until benchmarks justify
   - Status: 🔵 Backlog

---

## Completed Sprint Summary

| Sprint | WGs | Status | Key Deliverables |
|--------|-----|--------|------------------|
| v0.1.30 | WG-103-107 | ✅ All Complete | MemoryEvent broadcast, top-k optimization, memory-context skill, learn skill |
| v0.1.29 | WG-094-102 | ✅ All Complete | WASM removal (-6,982 LOC), Turso native vector search, file splitting, dead code audit |
| v0.1.28 | WG-089-093 | ✅ All Complete | DyMoE routing-drift, dual reward scoring, AI spam detector, CodeQL fix |
| v0.1.27 | WG-073,075,077-079,084-085 | ✅ All Complete | Bayesian ranking, Episode GC, MMR diversity, MCP Server Card, spawn_blocking audit, GH Pages, llms.txt |
| v0.1.26 | WG-086-088 | ✅ All Complete | Crate renaming do-memory-*, crates.io publish, GitHub Release |
| v0.1.24 | WG-059-067,080-083 | ✅ All Complete | Test stability, dependency updates, CHANGELOG backfill, tag+release |
| v0.1.23 | WG-051-058 | ✅ All Complete | Durable attribution/checkpoints, MCP contract, docs refresh, CI coverage, disk hygiene |
| v0.1.22 | WG-040-050 | ✅ All Complete | Doctests, file splits, dead_code, snapshots, property tests, MCP parity, git-cliff |
| v0.1.20 | WG-022-024,026-027,030 | ✅ All Complete | redb compilation, ignored test fixes, coverage improvement, codecov config |
| v0.1.19 | WG-012-021 | ✅ All Complete | Nightly filter, changelog workflow, dead_code audit, stale TODOs |
| v0.1.18 | WG-008-011 | ✅ All Complete | Ignored test triage, batch MCP tools, error handling, dep dedup |
| v0.1.17 | WG-001-007 | ✅ All Complete | Docs integrity, release wrapper, GOAP index, Dependabot merges |

---

## Partially Complete / Backlog

1. **WG-025**: Un-ignore fixable tests
   - Status: 🟡 Partial — 119→118 (pattern CLI e2e un-ignored); 6 sandbox/WASM tests still pending

2. **WG-028**: Property test expansion
   - Status: 🟡 Partial — ACT-030 (serialization) and ACT-031 (calculator) complete; ACT-032 (fuzz) pending

3. **WG-029**: Integration coverage
   - Status: 🟠 Pending — ACT-033, ACT-034, ACT-035 not started