---
name: goap-agent
description: "Invoke for complex multi-step tasks requiring intelligent planning and multi-agent coordination. Use when tasks need decomposition, dependency mapping, parallel/sequential/swarm/iterative execution strategies, or coordination of multiple specialized agents."
---

# GOAP Agent Skill

Goal-Oriented Action Planning for complex multi-step tasks with intelligent planning and multi-agent coordination.

## Quick Reference

- **[Methodology](methodology.md)** - Core GOAP planning cycle and phases
- **[Execution Strategies](execution-strategies.md)** - Parallel, Sequential, Swarm, Hybrid patterns
- **[Skills Reference](skills.md)** - Available skills by category
- **[Agents Reference](agents.md)** - Available task agents and capabilities
- **[Patterns](patterns.md)** - Common GOAP execution patterns
- **[Examples](examples.md)** - Complete GOAP workflow examples
- **[ADR-022](../../../plans/adr/ADR-022-GOAP-Agent-System.md)** - Architecture Decision Record

## When to Use

- Complex multi-step tasks (5+ distinct steps)
- Cross-domain problems (storage, API, testing, documentation)
- Tasks requiring parallel/sequential execution
- Quality-critical work with validation checkpoints
- Large refactors or architectural changes

## CRITICAL: Skills vs Task Agents

**Skills** (via `Skill` tool): Instruction sets that guide Claude directly
**Agents** (via `Task` tool): Autonomous sub-processes that execute tasks

Example:
- WRONG: `Task(subagent_type="code-quality", ...)` → ERROR!
- CORRECT: `Skill(command="code-quality")` → SUCCESS

See **[skills.md](skills.md)** for complete skills list and **[agents.md](agents.md)** for agent capabilities.

## Core Process

1. **ANALYZE** → Understand goals, constraints, resources
   - **Check ADRs**: Read relevant ADRs from `plans/adr/` before planning
   - **Prove premises by measurement**: a step about an external artifact's contents (a bump that
     "clears N alerts") enters the plan only after resolution proves it — scratch crate at the target
     version with the same features, `cargo update && cargo tree -e no-dev`, diff before/after.
     Release notes are claims; an unchanged graph means upstream-blocked, not a PR.
   - **Classify registers**: GHSA, `cargo audit` and `cargo deny` are independent sources; count
     distinct advisories and dedupe across lockfiles (one of them may be read by no local gate).
2. **DECOMPOSE** → Break into atomic tasks with dependencies
   - **Task Decomposition**: Create goal hierarchy with success criteria (see patterns.md)
   - **Dependency Types**: Sequential (A→B), Parallel (A─┐B─┐), Converging (A─┐B─┼─>D)
   - **Ownership before splitting**: check which open PR already touches the target files
     (`gh pr diff <n> --name-only`) and extend that branch rather than opening an overlapping PR —
     same-file splits force manual rebuilds, and every merge re-queues other branches' required runs.
3. **STRATEGIZE** → Choose execution pattern
4. **COORDINATE** → Assign to specialized agents
5. **EXECUTE** → Run with monitoring and quality gates
6. **SYNTHESIZE** → Aggregate results and validate success
   - **Verify readiness, don't assert it**: a plan may not promise a tag or a shippable state until
     `./scripts/verify-release-state.sh --check-unreleased` passes (it is Phase 0 of `release-manager.sh`);
     a merged bump is not shippable — ship only via `release-guard` + `./scripts/release-manager.sh ship --execute`.

## PR Monitoring Guardrail

- During EXECUTE/SYNTHESIZE for PR work, always verify `statusCheckRollup` on the latest head SHA.
- Re-read `gh pr view <n> --json headRefOid,mergeStateStatus` immediately before any readiness claim —
  head SHAs churn under strict required status checks — and batch edits per push (each push restarts the suite).
- Treat an empty required-check rollup as a blocker and document it in `plans/STATUS/VALIDATION_LATEST.md`.
- Avoid adding plans-only follow-up commits until remediation checks are attached.

See **[methodology.md](methodology.md)** for detailed phase-by-phase guidance and **[patterns.md](patterns.md)** for common execution patterns.

## ADR Integration Workflow

**MANDATORY**: Always check ADRs in `plans/adr/` before creating execution plans:

### Step 1: ADR Discovery
```bash
# List all ADRs to identify relevant ones
ls plans/adr/ADR-*.md
```

### Step 2: Read Relevant ADRs
- Search for ADRs related to your task domain
- Note architectural decisions and constraints
- Check ADR status (Accepted/Implemented vs Deprecated)

### Step 3: Incorporate into Planning
- Use ADR constraints when decomposing tasks
- Reference ADRs in execution plans
- Ensure compliance with architectural decisions

### Step 4: Update Progress in plans/
- Create/update execution plan files in `plans/`
- Document progress, blockers, and decisions
- Link to relevant ADRs in plan files
