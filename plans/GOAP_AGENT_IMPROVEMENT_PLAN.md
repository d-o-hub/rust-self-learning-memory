# GOAP: Agent Improvement Plan

**Date**: 2026-05-16
**Type**: Quality Improvement Plan
**Priority**: P2 - Developer experience
**WG**: WG-147

---

## Executive Summary

**Goal**: Improve agent coding efficiency, reduce friction, and standardize agent workflows across the codebase.

**Current State**: Agents rely on AGENTS.md for guidance but lack structured improvement cycles.
**Target**: Systematic agent refinement with measurable quality improvements.

---

## Phase 1: ANALYZE

### Current Friction Points

| Issue | Impact | Frequency |
|-------|--------|-----------|
| Circular agent loops on non-trivial tasks | ~5-10 min wasted per session | High |
| Stale context after long sessions | ~3-5 min context rebuilding | Medium |
| Inconsistent agent response quality | ~2-5 min clarification rounds | Medium |
| Missing skill definitions for common tasks | ~3-8 min ad-hoc instruction writing | High |

### Root Causes

1. **No structured handoff protocol** between specialist agents
2. **Context pruning triggers too late**, losing relevant state
3. **Skill discovery is ad-hoc** — agents don't know what skills are available
4. **No agent quality metrics** — can't measure improvement

---

## Phase 2: DECOMPOSE

### WG Tasks

| WG | Task | Priority | Dependencies |
|----|------|----------|--------------|
| WG-147.1 | Define agent handoff protocol in AGENTS.md | HIGH | None |
| WG-147.2 | Add context-pruning thresholds to config | MEDIUM | WG-147.1 |
| WG-147.3 | Create agent quality metrics checklist | MEDIUM | None |
| WG-147.4 | Audit existing skills for gaps | HIGH | None |
| WG-147.5 | Document common error patterns in LESSONS.md | MEDIUM | WG-147.4 |

---

## Phase 3: EXECUTE

### Sprint 1: Foundation

```text
Week 1: WG-147.1 (handoff protocol) + WG-147.4 (skill audit)
Week 2: WG-147.2 (context thresholds) + WG-147.3 (quality metrics)
Week 3: WG-147.5 (lessons documentation)
```

### Sprint 2: Measurement

```bash
# Track agent effectiveness
# Pre/post metrics:
# - Time to first quality commit
# - Number of clarification rounds per task
# - Context pruning frequency
```

---

## Quality Gates

| Milestone | Check | Target |
|-----------|-------|--------|
| Handoff protocol defined | AGENTS.md updated | Pass/Fail |
| Skills audit complete | Gap report in plans/ | ≥90% coverage |
| Quality metrics collected | 2-week baseline | Perceptible improvement |

---

## Cross-References

- AGENTS.md: Current agent guidelines
- LESSONS.md: Non-obvious learnings
- GOAP_STATE.md: Current GOAP state tracking
- ROADMAPS/ROADMAP_ACTIVE.md: Active roadmap
