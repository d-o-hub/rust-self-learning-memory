# GOAP Execution Plan: v0.1.21 Publishing Infrastructure

- **Created**: 2026-03-15
- **Branch**: release/v0.1.21
- **ADR**: ADR-045 (Publishing Best Practices)
- **Strategy**: Parallel execution with 5 agents

## Goals

| ID | Goal | Priority | Owner |
|----|------|----------|-------|
| WG-031 | Cargo.toml metadata completion | P1 | metadata-agent |
| WG-032 | Supply chain security setup | P0 | security-agent |
| WG-033 | Publishing automation | P0 | publish-agent |
| WG-034 | Documentation updates | P1 | docs-agent |
| WG-035 | Pre-existing issue fixes | P0 | fixer-agent |

## Agent Assignments

### Agent 1: metadata-agent
**Tasks**: ACT-038, ACT-039, ACT-040
- Add Cargo.toml metadata to do-memory-core
- Add Cargo.toml metadata to storage crates
- Add Cargo.toml metadata to do-memory-mcp

### Agent 2: security-agent
**Tasks**: ACT-042, ACT-043
- Configure cargo-deny with deny.toml
- Add supply-chain.yml workflow

### Agent 3: publish-agent
**Tasks**: ACT-044, ACT-045
- Create release.toml for cargo-release
- Add publish-crates.yml workflow

### Agent 4: docs-agent
**Tasks**: Update documentation
- Update GOAP_STATE.md with v0.1.21 sprint
- Verify all docs reference v0.1.21

### Agent 5: fixer-agent
**Tasks**: Fix pre-existing issues
- Reduce dead_code attributes (76 → ≤20)
- Fix any CI issues that arise

## Execution Log

| Time | Agent | Action | Status |
|------|-------|--------|--------|
| 2026-03-15 | team-lead | Created branch, updated plans | ✅ Complete |