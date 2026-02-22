# Plans Index

**Last Updated**: 2026-02-22 (Token Optimization Complete)

## Current Status (2026-02-22)
- **v0.1.16**: Token optimization complete
- **MCP Token Optimization**: ✅ COMPLETE - 82% reduction measured (1,237 → 227 tokens)
- **Opencode Integration**: ✅ memory-agent created with token optimization
- **Benchmark Script**: `./scripts/benchmark-mcp-tokens.sh`

## New Resources (2026-02-22)

### Token Optimization
- `.agents/skills/memory-mcp/token-optimization.md` - Token optimization guide
- `.opencode/agent/memory-agent.md` - Memory agent with token optimization
- `scripts/benchmark-mcp-tokens.sh` - Benchmark script
- `plans/adr/ADR-024-MCP-Lazy-Tool-Loading.md` - Updated with measured results

### Opencode Configuration
- `opencode.json` - Updated with default_agent and memory-mcp per-agent config

## Phase 1 & v0.1.16 Planning (2026-02-16)

### Phase 1 Completion - CI/CD Remediation ✅

- `GOAP_PHASE1_COMPLETION_SUMMARY.md` - Phase 1 completion report (CI fixes, 31 tests)
- `CLI_TEST_FIX_SUMMARY_2026-02-16.md` - CLI test fix report (2 passing, 4 code-fixed, 2 #[ignore])
- `GOAP_NIGHTLY_CI_FIXES_2026-02-16.md` - Nightly CI optimization documentation

### v0.1.16 Planning - Code Quality + Pattern Algorithms 🔄

- `V0.1.16_ROADMAP_SUMMARY.md` - Complete v0.1.16 roadmap (4 phases, 44-70 hours)
- `V0.1.16_SPRINT_CHECKLIST.md` - Sprint task checklist with success criteria
- `V0.1.16_PRIORITIZATION_MATRIX.md` - Task prioritization by impact vs. effort
- `GOAP_V0.1.16_EXECUTION_PLAN_2026-02-16.md` - Detailed execution plan
- `GOAP_V0.1.16_TASK_BREAKDOWN_2026-02-16.md` - Task breakdown by phase
- `GOAP_V0.1.16_EXECUTION_STRATEGY_2026-02-16.md` - Execution strategy and dependencies

**Status**: Phase A (CI) ✅ COMPLETE | Phase B (Code Quality) 🔄 READY TO START

## Active Architecture Decision Records (ADR)

| ADR | Title | Status |
|-----|-------|--------|
| ADR-024 | MCP Lazy Tool Loading | Active |
| ADR-025 | Non-Deterministic BOCPD Tests | Active |
| ADR-026 | Performance Benchmark CI Failures | Active |
| ADR-027 | Ignored Tests Strategy | Active |
| ADR-028 | Feature Enhancement Roadmap | Active (v0.1.16 planning complete) |
| ADR-029 | GitHub Actions Modernization | ✅ Complete |
| ADR-030 | Test Optimization and CI Stability | ✅ Complete |

**Location**: `plans/adr/`

## Recent Achievements (2026-02-16)

### Phase 1 Complete: CI/CD Remediation ✅
- **Nightly CI Fixed**: Disk space management, memory leak test optimization, test isolation improvements
- **Tests Fixed**: 31 integration tests now passing (12 of 39 flaky tests fixed)
- **PR #296 Merged**: Comprehensive CI fixes across 12 files in 6 crates
- **PR #297 Created**: CLI workflow improvements with JSON parsing fixes
- **Test Suite**: 33 of 39 tests passing (84.6%)
- **ADR-030 Created**: Test optimization and CI stability patterns documented

### v0.1.16 Planning Complete 🔄
- **Comprehensive Roadmap**: 4 phases, 44-70 hours effort, clear dependencies
- **Phase B Ready**: Code quality remediation unblocked and ready to start
- **Task Breakdown**: Detailed task lists, prioritization matrix, execution strategy
- **8 Documents Created**: Complete planning package for v0.1.16 sprint

### Key Metrics
- **CI Workflows**: All passing (6/6)
- **Test Coverage**: 92.5% maintained
- **Clippy Warnings**: 0 (zero-tolerance policy)
- **Nightly CI**: Stable, disk space <90%, memory leak test passing

## Architecture

- `ARCHITECTURE/ARCHITECTURE_CORE.md` - Core system architecture
- `ARCHITECTURE/ARCHITECTURE_PATTERNS.md` - Design patterns
- `ARCHITECTURE/ARCHITECTURE_INTEGRATION.md` - Integration patterns
- `ARCHITECTURE/API_DOCUMENTATION.md` - API documentation

## Configuration

- `CONFIGURATION/CONFIG_*.md` - Multi-phase configuration guides

## Features

- `EPISODE_RELATIONSHIPS_GUIDE.md` - Episode relationships documentation
- `EPISODE_RELATIONSHIPS_ROADMAP.md` - Relationship roadmap
- `EPISODE_RELATIONSHIPS_IMPLEMENTATION_STATUS.md` - Current status
- `EPISODE_RELATIONSHIPS_TESTING_STRATEGY.md` - Testing approach
- `EPISODE_TAGGING_FEATURE_SPEC.md` - Tagging feature spec
- `RELATIONSHIP_MODULE.md` - Module documentation
- `MCP_RELATIONSHIP_TOOLS_IMPLEMENTATION_PLAN.md` - MCP tools plan

## Operations

- `CIRCUIT_BREAKER_CONFIGURATION_GUIDE.md` - Circuit breaker setup
- `CIRCUIT_BREAKER_INCIDENT_RUNBOOK.md` - Incident response
- `AUDIT_LOGGING.md` - Audit logging documentation
- `PRODUCTION_ENABLEMENT_GUIDE.md` - Production deployment
- `STAGING_DEPLOYMENT_PLAN.md` - Staging deployment
- `CI_TIMEOUT_FIX.md` - CI timeout solutions

## Releases

- `RELEASE_NOTES_v0.1.13.md` - Release notes

## Research

- `research/` - Research documents and findings

## Benchmark Results

- `benchmark_results/AGGREGATED_RESULTS.md` - Performance benchmarks

## Archived Documents

Historical documents are in `archive/` subdirectories.

## GitHub Actions

- `.github/workflows/` - CI/CD workflows
- `.github/PULL_REQUEST_TEMPLATE.md` - PR template
- `.github/ISSUE_TEMPLATE/` - Bug report and feature request templates

## Status Reports

- `STATUS/IMPLEMENTATION_STATUS.md` - Current implementation status
- `GOAP_GITHUB_ACTIONS_2026-02-14.md` - GitHub Actions modernization status

---

**Note**: Many legacy status reports have been removed. See `archive/` for historical documents.
