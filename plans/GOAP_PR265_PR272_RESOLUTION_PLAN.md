# GOAP Execution Plan: PR #265 + PR #272 Conflict Resolution

## Plan Metadata
- **Task**: Multi-PR conflict resolution and feature integration
- **Base Branch**: pr-272 (already merged into integrate-pr265-on-272)
- **Target Branch**: integrate-pr265-on-272
- **Coordination Strategy**: Hybrid (Sequential Groups 1, 4, 5 + Parallel Groups 2, 3)
- **Estimated Duration**: 45-60 minutes

## Executive Summary

PR #265 introduces 30 commands/tools (8 MCP relationship tools + 22 CLI commands) while PR #272 contains critical structural changes (server/→server_impl/, batch module deletion). The conflict resolution requires:

1. **Relocating PR #265's MCP changes** from `server/` to `server_impl/`
2. **Preserving CLI features** (relationship + tag commands)  
3. **Maintaining PR #272's compilation fixes** and structural improvements
4. **Resolving batch module conflict** (PR #272's deletion takes precedence)

## Agent Coordination Matrix

### Group 1: Conflict Analysis (Sequential - 5 min)
**Dependencies**: None  
**Outputs**: Context for all downstream groups

| Agent | Task | Success Criteria |
|-------|------|------------------|
| A1 Conflict Analyzer | Parse phase1_*.md files, extract exact conflict points | Complete conflict map with line numbers |
| A2 Diff Mapper | Generate unified diff between branches, identify overlapping changes | Diff summary showing 6 critical files |

**Handoff Rule**: A1 and A2 output feeds into ALL Group 2, 3, 4 agents

---

### Group 2: Critical Compilation (Parallel - 10 min)
**Dependencies**: Group 1 complete  
**Outputs**: Compilation fix patches

| Agent | Task | Target Files |
|-------|------|--------------|
| A3 Compilation Fix | Extract PR #272 fixes: server_impl rename, batch removal | memory-mcp/src/bin/server_impl/ |
| A4 Test Fix | Extract PR #272 test fixes: duplicate functions, std::sync::Once | tests/, memory-core/src/ |
| A5 Security Fix | Extract PR #272 hardening: JWT validation, JavaScript escaping | memory-mcp/src/security/, oauth.rs |

**Handoff Rule**: All 3 agents complete → Group 4 Integration can begin

---

### Group 3: Feature Preservation (Parallel - 15 min)
**Dependencies**: Group 1 complete  
**Outputs**: Feature implementation files

| Agent | Task | Line Count | Critical Files |
|-------|------|------------|----------------|
| A6 MCP Tools | Preserve 8 relationship tools, relocate from server/ to server_impl/ | ~280 LOC | handlers.rs, tools.rs |
| A7 CLI Relationships | Preserve 7 standalone + 7 episode relationship commands | ~2100 LOC | commands/relationships/, episode/relationships/ |
| A8 CLI Tags | Preserve 8 tag commands | ~1300 LOC | commands/tag/ |
| A9 Documentation | Preserve ProviderConfig migration guide | ~500 LOC | docs/ or README |

**Handoff Rule**: All 4 agents complete → Group 4 Integration can begin

---

### Group 4: Integration (Sequential - 20 min)
**Dependencies**: Groups 2 AND 3 complete  
**Outputs**: Integrated, compilable codebase

| Agent | Task | Success Criteria |
|-------|------|------------------|
| A10 Integration | Merge all outputs, resolve file-level conflicts | Clean git status, no merge markers |
| A11 Quality Gate | Run cargo build, clippy, test | Zero build errors, zero clippy warnings |

**Handoff Rule**: A10 → A11 (sequential). A11 passes → Group 5 can begin

---

### Group 5: Final Verification (Parallel - 10 min)
**Dependencies**: Group 4 complete  
**Outputs**: Final validation report

| Agent | Task | Success Criteria |
|-------|------|------------------|
| A12 Regression Test | Run full test suite, verify no regressions | 717+ tests passing, no new failures |

---

## Critical Path Analysis

```
Group 1 (5 min)
    ↓
Groups 2 + 3 (15 min parallel)
    ↓
Group 4 (20 min sequential)
    ↓
Group 5 (10 min)

Total Critical Path: ~50 minutes
```

**Critical Path**: 1 → (2∥3) → 4 → 5

**Parallelism Opportunities**:
- Groups 2 and 3 run simultaneously (independent)
- Agents within Groups 2, 3, and 5 run in parallel

## Risk Analysis

### High Risk Items
1. **Batch Module Conflict**: PR #265 re-enables batch, PR #272 deletes it
   - **Resolution**: PR #272 wins (deletion), batch was already broken
   
2. **Server Directory Relocation**: 8 MCP tools need path updates
   - **Mitigation**: A6 specializes in this relocation
   
3. **CLI Command Registration**: main.rs and commands/mod.rs may conflict
   - **Mitigation**: A7 and A8 coordinate registration updates

### Medium Risk Items
1. **Test Failures**: Pre-existing embedding test may still fail
   - **Acceptance**: Document as known issue
   
2. **Clippy Warnings**: New code may introduce warnings
   - **Mitigation**: A11 enforces zero-tolerance policy

## Success Criteria Checklist

- [ ] PR #265 features fully preserved (30 commands/tools)
- [ ] PR #272 fixes fully preserved (compilation, tests, security)
- [ ] Zero compilation errors
- [ ] Zero clippy warnings  
- [ ] All tests passing (717+)
- [ ] No regressions
- [ ] Clean git status
- [ ] Documentation updated

## Rollback Plan

If integration fails:
1. `git checkout integrate-pr265-on-272 -- .` (restore to pre-agent state)
2. Preserve work: `git stash` or copy to backup branch
3. Restart with adjusted strategy based on failure analysis

## Agent Assignments Summary

| Agent | Type | Group | Parallelizable |
|-------|------|-------|----------------|
| A1 | Conflict Analyzer | 1 | No (feeds all) |
| A2 | Diff Mapper | 1 | No (feeds all) |
| A3 | Compilation Fix | 2 | Yes |
| A4 | Test Fix | 2 | Yes |
| A5 | Security Fix | 2 | Yes |
| A6 | MCP Tools | 3 | Yes |
| A7 | CLI Relationships | 3 | Yes |
| A8 | CLI Tags | 3 | Yes |
| A9 | Documentation | 3 | Yes |
| A10 | Integration | 4 | No |
| A11 | Quality Gate | 4 | No |
| A12 | Regression Test | 5 | No |

---

*Plan generated by GOAP Orchestrator*  
*Ready for agent coordination*
