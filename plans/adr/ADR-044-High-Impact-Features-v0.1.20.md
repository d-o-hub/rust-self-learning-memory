# ADR-044: High-Impact Features for v0.1.22 Sprint

- **Status**: Implemented
- **Date**: 2026-03-15
- **Implemented**: 2026-03-16
- **Deciders**: Project maintainers
- **Related**: ADR-028 (Feature Roadmap), ADR-043 (Codebase Analysis)

## Context

Codebase analysis (ADR-043) shows the core learning pipeline — Episodes → Pattern Extraction → Heuristic Learning → Context Retrieval → Recommendation — is fully functional. However, three critical gaps limit real-world agent adoption:

1. **Usability gap**: The system stores knowledge well but returns raw episodes/patterns — agents must infer what to do
2. **Feedback gap**: Recommendations aren't tracked — the system can't learn which recommendations actually helped
3. **Handoff gap**: Learning happens only at episode completion — multi-agent workflows can't share mid-task progress

## Decision

Implement three high-impact features, prioritized by: **impact on core learning loop × feasibility (1-2 weeks) × user-facing value**.

---

### Feature 1: Actionable Recommendation Playbooks (P0)

**Problem**: `retrieve_relevant_context()` returns full `Episode` arcs. `recommend_patterns_for_task()` returns scored `PatternSearchResult`. Neither tells the agent *what to do next*. The semantic summary generated in `completion.rs` is computed but not persisted.

**Impact**: Closes the biggest product gap — memory exists but agents can't consume it efficiently.

**Design**:

```rust
pub struct RecommendedPlaybook {
    pub playbook_id: Uuid,
    pub task_match_score: f32,
    pub why_relevant: String,
    pub when_to_apply: Vec<String>,
    pub when_not_to_apply: Vec<String>,
    pub ordered_steps: Vec<PlaybookStep>,
    pub pitfalls: Vec<String>,
    pub expected_outcome: String,
    pub confidence: f32,
    pub supporting_pattern_ids: Vec<String>,
    pub supporting_episode_ids: Vec<Uuid>,
}

pub struct PlaybookStep {
    pub order: u32,
    pub action: String,
    pub tool_hint: Option<String>,
    pub expected_result: Option<String>,
}
```

**Implementation**:

| File | Change |
|------|--------|
| `memory-core/src/memory/completion.rs` | Persist semantic summary (currently dropped) |
| `memory-core/src/memory/playbook/` | New module: `PlaybookGenerator` synthesizes playbooks from patterns + reflections + summaries |
| `memory-core/src/memory/retrieval/context.rs` | Add `retrieve_playbooks()` returning playbooks instead of raw episodes |
| MCP: `handlers.rs` | Add `recommend_playbook`, `explain_pattern` tools |
| CLI: `commands/playbook/` | Add `playbook recommend`, `playbook explain` commands |

**Key constraint**: Template-driven synthesis using existing pattern types and summaries — no LLM on the hot path.

**Effort**: 3-5 days

---

### Feature 2: Recommendation Attribution & Online Effectiveness (P0)

**Problem**: The engine recommends patterns but doesn't know what was shown, what the agent used, or whether it helped. This makes `effectiveness_weight` (25% of ranking) much weaker than it should be — it's based on historical success rate, not actual recommendation-to-outcome attribution.

**Impact**: Makes the system *actually* self-learning — closes the feedback loop from recommendation → usage → outcome → improved ranking.

**Design**:

```rust
pub struct RecommendationSession {
    pub session_id: Uuid,
    pub episode_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub recommended_pattern_ids: Vec<String>,
    pub recommended_playbook_ids: Vec<Uuid>,
}

pub struct RecommendationFeedback {
    pub session_id: Uuid,
    pub applied_pattern_ids: Vec<String>,
    pub consulted_episode_ids: Vec<Uuid>,
    pub outcome: TaskOutcome,
    pub agent_rating: Option<f32>,  // 0.0-1.0
}
```

**Implementation**:

| File | Change |
|------|--------|
| `memory-core/src/memory/attribution/` | New module: `RecommendationTracker` records sessions + feedback |
| `memory-core/src/memory/learning.rs` | Wire completion to update pattern effectiveness for *applied* patterns |
| `memory-core/src/memory/completion.rs` | Consume recommendation feedback metadata during completion |
| `memory-core/src/reward/mod.rs` | Add adoption-rate bonus: patterns that are recommended AND applied AND succeed get boosted |
| MCP: `handlers.rs` | Add `record_recommendation_feedback` tool |
| CLI: `commands/feedback/` | Add `feedback record` command |

**Metrics enabled**:
- Pattern adoption rate (recommended vs. applied)
- Success-after-adoption rate
- Recommendation precision (did useful patterns get recommended?)

**Effort**: 3-4 days

---

### Feature 3: Episode Checkpoints & Handoff Packs (P1)

**Problem**: Learning happens only at `complete_episode()`. Multi-agent systems (planner → executor → verifier) need to share partial progress mid-task. Currently impossible without completing the episode.

**Impact**: Unlocks multi-agent adoption without requiring CRDT/distributed sync infrastructure.

**Design**:

```rust
pub struct HandoffPack {
    pub checkpoint_id: Uuid,
    pub episode_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub current_goal: String,
    pub steps_completed: Vec<ExecutionStep>,
    pub what_worked: Vec<String>,
    pub what_failed: Vec<String>,
    pub salient_facts: Vec<String>,
    pub suggested_next_steps: Vec<String>,
    pub relevant_patterns: Vec<PatternSearchResult>,
    pub relevant_heuristics: Vec<Heuristic>,
}
```

**Implementation**:

| File | Change |
|------|--------|
| `memory-core/src/memory/checkpoint/` | New module: `checkpoint_episode()`, `get_handoff_pack()`, `resume_from_handoff()` |
| `memory-core/src/extraction/extractor.rs` | Reuse salient extraction on partial episodes |
| `memory-core/src/episode/structs.rs` | Add `checkpoints: Vec<CheckpointMeta>` to Episode |
| MCP: `handlers.rs` | Add `checkpoint_episode`, `get_handoff_pack`, `resume_from_handoff` tools |
| CLI: `commands/episode/` | Add `episode checkpoint`, `episode handoff` subcommands |

**Key constraint**: Explicit/manual checkpoints only (no auto-checkpointing to avoid write amplification).

**Effort**: 4-6 days

---

## Execution Order

```
Week 1:
  Feature 2 (Attribution) ──► Feature 1 (Playbooks)
  ↑ Attribution first so playbooks can record sessions from day one

Week 2:
  Feature 3 (Checkpoints/Handoff)
  ↑ Independent of 1+2, can parallel
```

## GOAP World State Transition

| Fact | Before | After |
|------|--------|-------|
| `recommendations_actionable` | ❌ raw patterns/episodes | ✅ structured playbooks |
| `feedback_loop_closed` | ❌ no attribution | ✅ recommendation → outcome tracked |
| `multi_agent_handoff` | ❌ completion-only | ✅ mid-task checkpoints |
| `effectiveness_data_quality` | 🟡 inferred | ✅ attributed |
| `pattern_ranking_accuracy` | 🟡 historical-only | ✅ adoption-weighted |

## Consequences

### Positive
- **Playbooks** make the memory system immediately useful to agents — not just a storage system
- **Attribution** creates a genuine self-improving loop — recommendations get better with usage
- **Handoff** unlocks the multi-agent market without heavy infrastructure
- All three features build on existing code (extractors, summaries, recommendation engine)
- Sets foundation for future A/B testing (attribution data) and real-time learning (checkpoints)

### Negative
- ~2 weeks engineering effort
- New storage schema additions (recommendation sessions, checkpoints)
- Playbook quality depends on pattern/summary quality

### Risks
- **Mitigated**: Don't put LLM synthesis on hot path — use template-driven playbooks
- **Mitigated**: Require explicit agent feedback, don't infer "used" from "recommended"
- **Mitigated**: Manual checkpoints only, no write amplification

## Future Unlocks

These 3 features directly enable:
- **A/B testing** (ADR-028 #14): Attribution data provides the metrics layer
- **Real-time pattern learning** (ADR-028 #12): Checkpoints provide partial-episode signal
- **Bayesian/Wilson ranking**: Replace hand-tuned weights with data-driven scoring once attribution data accumulates

## Cross-References

- [ADR-028: Feature Enhancement Roadmap](ADR-028-Feature-Enhancement-Roadmap.md)
- [ADR-043: Codebase Analysis v0.1.20](ADR-043-Comprehensive-Codebase-Analysis-v0.1.20.md)
- [completion.rs](../../memory-core/src/memory/completion.rs) — episode completion & summary generation
- [recommendation.rs](../../memory-core/src/memory/pattern_search/recommendation.rs) — pattern recommendation engine
- [learning.rs](../../memory-core/src/memory/learning.rs) — heuristic update loop
- [context.rs](../../memory-core/src/memory/retrieval/context.rs) — context retrieval
