# Follow-up Issue: Integrate Reward and Reflection into Learning Module

## Problem
The `memory-core/src/learning/` module is currently scoped only to async pattern extraction queue coordination. However, `reward` and `reflection` are conceptually part of the learning cycle.

## Goal
Integrate `reward` and `reflection` modules into a unified learning orchestration layer within the `learning` module.

## Proposed Changes
1. Introduce a `LearningOrchestrator` struct in `learning/mod.rs` (or `learning/orchestrator.rs`).
2. Wire `PatternExtractionQueue`, `RewardCalculator`, and `ReflectionGenerator` into the orchestrator.
3. Provide a unified entry point (e.g., `on_episode_complete`) that triggers the full learning cycle:
   - Pattern extraction
   - Reward computation
   - Reflection generation
4. Update `SelfLearningMemory::complete_episode` to use this new orchestrator.

## References
- ADR-028: Feature Enhancement Roadmap
- Issue: 🟡 learning/mod.rs: Document scope or wire up as orchestration layer for reward-reflection-pattern cycle
