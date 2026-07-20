# Skills Index

> Generated from skill frontmatter / catalog at 2026-07-20T10:02:33Z.
> Canonical path: `.agents/skills/<skill>/SKILL.md`
> Catalog: `.agents/skills/skill-catalog.generated.json`
> Routing: `.agents/skills/skill-rules.json`

**Skill count**: 34  
**Routed skills**: 34 / 34  

## Inventory

| Skill | Evals | Routed | Description |
|-------|:-----:|:------:|-------------|
| [`agent-coordination`](skills/agent-coordination/SKILL.md) | yes | yes | Coordinate multiple specialized Skills and Task Agents through parallel, sequential, swarm, hybrid, or iterative exec... |
| [`agents-update`](skills/agents-update/SKILL.md) | yes | yes | Update AGENTS.md and agent_docs/ following best practices. Use when modifying agent guidelines, adding new documentat... |
| [`analysis-swarm`](skills/analysis-swarm/SKILL.md) | yes | yes | Multi-perspective code analysis using three AI personas (RYAN, FLASH, SOCRATES) for comprehensive decision-making. Us... |
| [`architecture-validation`](skills/architecture-validation/SKILL.md) | yes | yes | Dynamically validate that the implemented codebase matches architectural decisions documented in plan files. Use when... |
| [`build-rust`](skills/build-rust/SKILL.md) | yes | yes | Build Rust code with proper error handling, optimization, and workspace support for development, testing, and production |
| [`ci-fix`](skills/ci-fix/SKILL.md) | yes | yes | Diagnose and fix GitHub Actions CI failures for Rust projects. Use when CI fails, tests timeout, or linting issues oc... |
| [`ci-poll`](skills/ci-poll/SKILL.md) | yes | yes | Poll GitHub CI status with exponential backoff until all checks complete. Use after pushing to a PR branch to monitor... |
| [`code-quality`](skills/code-quality/SKILL.md) | yes | yes | Maintain high code quality through formatting, linting, static analysis, and clean code principles. Use for rustfmt, ... |
| [`codebase-analyzer`](skills/codebase-analyzer/SKILL.md) | yes | yes | Analyze implementation details, trace data flow, explain technical workings, locate files, and consolidate codebases.... |
| [`commit`](skills/commit/SKILL.md) | yes | yes | Git commit with enforced quality gates, proper message format, and safe push workflow |
| [`debug-troubleshoot`](skills/debug-troubleshoot/SKILL.md) | yes | yes | Systematic debugging approach for Rust async code with Tokio, Turso, and redb. Use when diagnosing runtime issues, pe... |
| [`do-memory-cli-ops`](skills/do-memory-cli-ops/SKILL.md) | yes | yes | Execute and troubleshoot do-memory-cli commands for episode management, pattern analysis, and storage operations. Use... |
| [`do-memory-mcp`](skills/do-memory-mcp/SKILL.md) | yes | yes | Use and troubleshoot the Memory MCP server for episodic memory retrieval and pattern analysis. Use when working with ... |
| [`external-signal-provider`](skills/external-signal-provider/SKILL.md) | yes | yes | Integrate external signal providers (AgentFS, audit trails, toolcall logs) into the reward system. Use when adding ex... |
| [`feature-implement`](skills/feature-implement/SKILL.md) | yes | yes | Systematic approach to implementing new features in the Rust memory system following project conventions. Use when ad... |
| [`git-worktree-manager`](skills/git-worktree-manager/SKILL.md) | yes | yes | Manage git worktrees for efficient multi-branch development. Use when creating worktrees for feature branches, organi... |
| [`github-release-best-practices`](skills/github-release-best-practices/SKILL.md) | yes | yes | Background reference for release prep (SemVer, changelog categories). For actual releases, ALWAYS use the release-gua... |
| [`github-workflows`](skills/github-workflows/SKILL.md) | yes | yes | Diagnose, fix, and optimize GitHub Actions workflows for Rust projects. Use when setting up CI/CD, troubleshooting wo... |
| [`goap-agent`](skills/goap-agent/SKILL.md) | yes | yes | Invoke for complex multi-step tasks requiring intelligent planning and multi-agent coordination. Use when tasks need ... |
| [`learn`](skills/learn/SKILL.md) | yes | yes | Extract non-obvious session learnings into scoped AGENTS.md files |
| [`loop-agent`](skills/loop-agent/SKILL.md) | yes | yes | Execute workflow agents iteratively for refinement and progressive improvement until quality criteria are met. Use wh... |
| [`memory-context`](skills/memory-context/SKILL.md) | yes | yes | Retrieve relevant context from memory and preserve essential state. Use for episode retrieval, semantic search, or co... |
| [`memory-harness`](skills/memory-harness/SKILL.md) | yes | yes | Universal agent memory harness — record, replay, and benchmark real agent sessions. Use when testing memory system le... |
| [`performance`](skills/performance/SKILL.md) | yes | yes | Benchmarking and performance optimization for Rust. Use when profiling CPU/memory bottlenecks, running Criterion benc... |
| [`plan-gap-analysis`](skills/plan-gap-analysis/SKILL.md) | yes | yes | Analyze gaps between implementation plans and actual codebase implementation for the Rust self-learning memory project |
| [`pr-readiness`](skills/pr-readiness/SKILL.md) | yes | yes | Comprehensive PR health check: merge state, CI status, conflicts, cancelled checks, AND all PR comments/reviews (huma... |
| [`release-cadence-manager`](skills/release-cadence-manager/SKILL.md) | yes | yes | Monitor release cadence, detect drift, and coordinate resolution using GOAP orchestrator with swarm agents. Use when ... |
| [`release-guard`](skills/release-guard/SKILL.md) | yes | yes | Canonical release workflow for this repo. One path every time: main green → release-manager ship → tag vX.Y.Z → relea... |
| [`skill-creator`](skills/skill-creator/SKILL.md) | yes | yes | Create new Claude Code skills with proper structure, YAML frontmatter, and best practices. Use when creating reusable... |
| [`storage-sync`](skills/storage-sync/SKILL.md) | yes | yes | Synchronize memories between Turso (durable) and redb (cache) storage layers. Use when cache appears stale, after fai... |
| [`test-fix`](skills/test-fix/SKILL.md) | yes | yes | Systematic approach to diagnosing and fixing failing tests in Rust projects. Use when tests fail and you need to diag... |
| [`test-patterns`](skills/test-patterns/SKILL.md) | yes | yes | Unified testing patterns for Rust: unit testing quality, episodic memory operations, and async/tokio code. Use when w... |
| [`test-runner`](skills/test-runner/SKILL.md) | yes | yes | Execute Rust tests (unit, integration, doc). Use cargo nextest for fast parallel execution. |
| [`web-doc-resolver`](skills/web-doc-resolver/SKILL.md) | yes | yes | Resolve queries or URLs into compact, LLM-ready markdown using a low-cost cascade. Prioritizes llms.txt for structure... |

## High-frequency ops

| Operation | Skill | CLI |
|-----------|-------|-----|
| Build | `build-rust` | `./scripts/build-rust.sh` |
| Format/Lint | `code-quality` | `./scripts/code-quality.sh` |
| Tests | `test-runner` | `cargo nextest run --all` |
| PR readiness | `pr-readiness` | `./scripts/check-pr-readiness.sh` |
| Release | `release-guard` | `./scripts/release-manager.sh ship --execute` |
| CI poll | `ci-poll` | — |

See AGENTS.md for the full Skill + CLI pattern.

