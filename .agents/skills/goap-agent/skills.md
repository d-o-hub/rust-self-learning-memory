# GOAP Skills Reference

## Quality & Validation Skills

| Skill | Purpose |
|-------|---------|
| **code-quality** | Rust code review, formatting, linting, clean code principles |
| **architecture-validation** | Validate vs architecture plans |
| **plan-gap-analysis** | Implementation gap analysis |
| **test-runner** | Test execution, management (absorbs test-optimization) |

## Build & Testing Skills

| Skill | Purpose |
|-------|---------|
| **build-rust** | Build management, compilation |
| **test-fix** | Systematic test debugging |
| **test-runner** | Test execution, management |

## Analysis & Decision-Making Skills

| Skill | Purpose |
|-------|---------|
| **analysis-swarm** | Multi-perspective code analysis |
| **codebase-consolidation** | Analyze, consolidate codebases |
| **debug-troubleshoot** | Systematic async debugging |

## Research Skills

| Skill | Purpose |
|-------|---------|
| **web-doc-resolver** | Resolve web/docs sources into compact markdown |
| **memory-context** | Episodic memory retrieval & compaction |

## Memory System Skills

| Skill | Purpose |
|-------|---------|
| **memory-harness** | Record, replay, benchmark agent sessions (absorbs episode lifecycle) |
| **do-memory-mcp** | MCP server operations |
| **do-memory-cli-ops** | CLI operations |
| **storage-sync** | Turso/redb synchronization |

## Workflow & Coordination Skills

| Skill | Purpose |
|-------|---------|
| **task-decomposition** | Break down complex tasks |
| **agent-coordination** | Coordinate Skills/Agents |
| **parallel-execution** | Execute tasks simultaneously |
| **loop-agent** | Iterative refinement |
| **github-workflows** | CI/CD optimization |

## Meta Skills

| Skill | Purpose |
|-------|---------|
| **skill-creator** | Create new skills |
| **feature-implement** | Feature implementation workflow |

## Phase-Specific Recommendations

### Phase 1: Research & Analysis (Parallel)
- `web-doc-resolver` - Best practices and documentation research
- `memory-context` - Past implementations & compaction
- `codebase-consolidation` - Architecture understanding

### Phase 2: Decision-Making (Sequential)
- `task-decomposition` - Break down goals
- `analysis-swarm` - Architectural decisions

### Phase 3: Pre-Implementation (Parallel)
- `code-quality` - Rust best practices
- `architecture-validation` - Design validation
- `plan-gap-analysis` - Requirements coverage

### Phase 4: Implementation (Parallel/Sequential)
- `feature-implement` - Implementation workflow and quality gates
- `code-quality` - Keep implementation aligned with project standards

### Phase 5: Testing & Debugging
- `test-fix` - Test debugging
- `test-patterns` - Test quality and async/memory testing patterns

### Phase 6: Build & CI/CD
- `build-rust` - Build verification
- `github-workflows` - CI validation

### Phase 7: Quality Assurance (Parallel)
- `code-quality` - Final review
- `architecture-validation` - Final validation
- `test-runner` - Final execution validation
