# GOAP Execution Plan: Plans Folder Cleanup & P1 Issue Fix

**Plan ID**: GOAP-2025-12-19-001  
**Created**: 2025-12-19T12:00:00Z  
**Priority**: HIGH (P1 Issue + Critical Path Cleanup)  
**Estimated Duration**: 4-6 hours  
**Strategy**: Hybrid Parallel + Sequential Execution  

---

## Executive Summary

Based on analysis-swarm findings, this plan addresses:
1. **P1 Critical Issue**: Episode retrieval lazy loading in memory-core
2. **Plans Folder Cleanup**: Archive 59 active files to ~27 files
3. **Documentation Updates**: Reflect v0.1.6 release status
4. **Debt Reduction**: Remove outdated loop-agent and completed plan files

**Success Criteria**:
- ✅ P1 lazy loading issue resolved with proper fallback chain
- ✅ Plans folder consolidated from 59 to ~27 active files
- ✅ PROJECT_STATUS.md updated with P1 issue details
- ✅ ROADMAP.md reflects v0.1.6 completion
- ✅ All outdated files properly archived

---

## Goal-Oriented Action Planning (GOAP) Analysis

### Goal Decomposition

**Primary Goal**: Clean up plans folder and fix P1 issue
**Sub-goals**:
1. Fix P1 lazy loading in episode retrieval
2. Update project documentation
3. Archive completed plan files
4. Delete outdated files
5. Validate cleanup results

### Dependency Graph

```
Phase 1 (Parallel):
├─ Task A: Fix P1 lazy loading (no deps)
├─ Task B: Update PROJECT_STATUS.md (no deps)
└─ Task C: Update ROADMAP.md (no deps)

Phase 2 (Sequential - deps: Phase 1):
└─ Task D: Archive completed files (deps: A,B,C)

Phase 3 (Sequential - deps: Phase 2):
└─ Task E: Delete outdated files (deps: D)

Phase 4 (Validation - deps: Phase 3):
└─ Task F: Validate cleanup (deps: E)
```

### Agent Coordination Strategy

**Phase 1**: Parallel execution (3 agents)
- `feature-implementer`: Fix P1 lazy loading issue
- `code-reviewer`: Update PROJECT_STATUS.md  
- `refactorer`: Update ROADMAP.md

**Phase 2**: Sequential execution
- `goap-agent`: Archive completed files

**Phase 3**: Sequential execution  
- `goap-agent`: Delete outdated files

**Phase 4**: Validation
- `code-reviewer`: Validate cleanup results

---

## Detailed Execution Plan

### Phase 1: Critical Fixes & Updates (Parallel Execution)
**Estimated Time**: 2-3 hours  
**Agents**: 3 specialized agents working in parallel

#### Task 1A: Fix P1 Lazy Loading Issue
**Agent**: `feature-implementer`  
**File**: `memory-core/src/memory/episode.rs:356-362`  
**Effort**: 2 hours  
**Priority**: P1 (Critical)

**Current Issue**:
```rust
// Only checks in-memory HashMap
pub async fn get_episode(&self, episode_id: Uuid) -> Result<Episode> {
    if let Some(ep) = {
        let episodes = self.episodes_fallback.read().await;
        episodes.get(&episode_id).cloned()
    } {
        return Ok(ep);
    }
    // Missing: fallback to redb → Turso
}
```

**Required Fix**:
```rust
pub async fn get_episode(&self, episode_id: Uuid) -> Result<Episode> {
    // 1) Try in-memory cache first
    if let Some(ep) = {
        let episodes = self.episodes_fallback.read().await;
        episodes.get(&episode_id).cloned()
    } {
        return Ok(ep);
    }

    // 2) Try redb cache
    if let Ok(Some(ep)) = self.redb_storage.get_episode(episode_id).await {
        // Cache in memory for future requests
        let mut episodes = self.episodes_fallback.write().await;
        episodes.insert(episode_id, ep.clone());
        return Ok(ep);
    }

    // 3) Try Turso (source of truth)
    match self.turso_storage.get_episode(episode_id).await {
        Ok(Some(ep)) => {
            // Cache in both redb and memory
            let _ = self.redb_storage.store_episode(&ep).await;
            let mut episodes = self.episodes_fallback.write().await;
            episodes.insert(episode_id, ep.clone());
            Ok(ep)
        }
        Ok(None) => Err(Error::NotFound(episode_id.to_string())),
        Err(e) => Err(e)
    }
}
```

**Similar fixes needed for**:
- `list_episodes()` method
- `retrieve_relevant_context()` method

#### Task 1B: Update PROJECT_STATUS.md
**Agent**: `code-reviewer`  
**Effort**: 30 minutes  
**Priority**: P1 (Documentation)

**Updates Required**:
1. Update "Last Updated" to 2025-12-19
2. Add P1 lazy loading issue to "Known Issues" section
3. Update current status to reflect P1 issue
4. Add P1 fix to "Next Steps" section

#### Task 1C: Update ROADMAP.md  
**Agent**: `refactorer`  
**Effort**: 30 minutes  
**Priority**: P2 (Documentation)

**Updates Required**:
1. Update v0.1.6 status to include P1 fix
2. Add P1 resolution to release notes
3. Update "Last Updated" field
4. Clean up any outdated references

### Phase 2: Archive Completed Files (Sequential)
**Estimated Time**: 1 hour  
**Agent**: `goap-agent` (coordination)

#### Files to Archive (Move to `plans/archive/goap-plans/`):

**GitHub Actions Related (COMPLETED)**:
- `github-actions-fix-2025.md`
- `github-actions-issues-analysis.md` 
- `github-actions-update-plan.md`
- `goap-github-actions-2025-update-report.md`
- `goap-github-actions-optimization-plan.md`

**Javy Integration (COMPLETED)**:
- `goap-phase2c-javy-plan.md`
- `phase2c-javy-completion-final.md`
- `phase2c-javy-completion-status.md`
- `phase2c-javy-verification-report.md`
- `javy-research-findings.md`

**Loop Agent (OUTDATED)**:
- `loop-agent-final-summary.md`
- `loop-agent-github-actions-monitor.md`
- `loop-iteration-1-results.md`

**Execution Plans (COMPLETED)**:
- `goap-execution-schedule.md`

**Archive Strategy**:
1. Create timestamped archive folder: `plans/archive/2025-12-19-goap-cleanup/`
2. Move files with preservation of git history
3. Create `ARCHIVE_INDEX.md` listing moved files and reasons

### Phase 3: Delete Outdated Files (Sequential)
**Estimated Time**: 30 minutes  
**Agent**: `goap-agent` (coordination)

#### Files to Delete (Completely Outdated):

**Loop Agent Files** (if not archived):
- Any remaining loop-agent related temporary files
- Debug or monitoring files that are no longer relevant

**Temporary Files**:
- Any `.tmp` or draft files
- Outdated test reports that have been superseded

**Deletion Protocol**:
1. Verify files are truly outdated (check git log)
2. Ensure no references exist in active documentation
3. Use `git rm` to preserve deletion history
4. Commit deletion with descriptive message

### Phase 4: Validation & Quality Check (Parallel)
**Estimated Time**: 1 hour  
**Agents**: 2 specialized agents

#### Task 4A: Code Quality Validation
**Agent**: `code-reviewer`  
**Effort**: 30 minutes

**Validation Checklist**:
- ✅ P1 fix compiles without errors
- ✅ All tests pass with new implementation
- ✅ Code follows project style guidelines
- ✅ No clippy warnings introduced
- ✅ Documentation is accurate

#### Task 4B: Documentation Validation  
**Agent**: `code-reviewer`  
**Effort**: 30 minutes

**Validation Checklist**:
- ✅ PROJECT_STATUS.md reflects current state
- ✅ ROADMAP.md is accurate and up-to-date
- ✅ No broken references in documentation
- ✅ Archive index is complete
- ✅ All active files are necessary

---

## Execution Timeline

```
Phase 1 (Parallel):     2-3 hours
├─ Task 1A: P1 Fix (2h)
├─ Task 1B: Status Update (0.5h)
└─ Task 1C: Roadmap Update (0.5h)

Phase 2 (Archive):      1 hour
└─ Move 12+ files to archive

Phase 3 (Cleanup):     0.5 hours  
└─ Delete outdated files

Phase 4 (Validation):   1 hour
├─ Code validation (0.5h)
└─ Documentation validation (0.5h)

Total Estimated Time: 4.5-6.5 hours
```

---

## Risk Assessment & Mitigation

### High-Risk Items

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| P1 fix breaks existing functionality | Critical | Low | Comprehensive testing, backward compatibility |
| Archive breaks documentation references | High | Medium | Check references before moving, update links |
| Git history loss | Medium | Low | Use `git mv` for moves, `git rm` for deletions |

### Medium-Risk Items

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Incomplete P1 fix resolution | High | Low | Test all episode retrieval methods |
| Documentation inconsistencies | Medium | Medium | Thorough validation phase |

---

## Success Metrics

### Quantitative Metrics
- **P1 Issue**: 0 remaining lazy loading problems
- **File Count**: Reduce from 59 to ~27 active files (54% reduction)
- **Documentation**: 100% accurate status reporting
- **Test Coverage**: Maintain >90% coverage after changes

### Qualitative Metrics
- **Code Quality**: No regressions, clean implementation
- **Documentation Clarity**: Accurate, up-to-date project status
- **Maintainability**: Cleaner plans folder structure
- **Developer Experience**: Easier navigation of plans folder

---

## Prerequisites & Blockers

### Prerequisites
- ✅ Analysis-swarm findings completed
- ✅ Current codebase state understood
- ✅ Archive structure prepared
- ✅ Git workspace clean

### Potential Blockers
- ❌ Unexpected test failures with P1 fix
- ❌ Complex dependencies in episode retrieval
- ❌ Missing documentation references

### Contingency Plans
- If P1 fix is complex: Create separate focused issue
- If archive breaks references: Keep files and add deprecation notices
- If validation fails: Roll back changes and reassess

---

## Next Steps & Execution

### Immediate Actions (Ready to Execute)
1. **Start Phase 1**: Launch 3 parallel agents for P1 fix and documentation updates
2. **Monitor Progress**: Track agent completion status
3. **Quality Gates**: Validate each phase before proceeding

### Execution Commands
```bash
# Phase 1: Parallel execution
task feature-implementer "Fix P1 lazy loading in episode.rs"
task code-reviewer "Update PROJECT_STATUS.md with P1 issue"  
task refactorer "Update ROADMAP.md for v0.1.6"

# Phase 2: Archive files (after Phase 1 complete)
task goap-agent "Archive completed plan files"

# Phase 3: Delete outdated files (after Phase 2 complete)
task goap-agent "Delete outdated loop-agent files"

# Phase 4: Validation
task code-reviewer "Validate P1 fix implementation"
task code-reviewer "Validate documentation accuracy"
```

---

## Completion Criteria

### Must-Have (Release Criteria)
- [ ] P1 lazy loading issue completely resolved
- [ ] All episode retrieval methods work correctly
- [ ] Plans folder consolidated to target file count
- [ ] Documentation accurately reflects project state
- [ ] All tests pass with no regressions
- [ ] No broken references in documentation

### Nice-to-Have (Quality Criteria)
- [ ] Archive index includes helpful descriptions
- [ ] Git commit messages are descriptive
- [ ] Code follows all style guidelines
- [ ] Documentation includes examples for P1 fix

---

**Status**: READY FOR EXECUTION ✅  
**Confidence**: HIGH - All prerequisites met, risks mitigated  
**Next Action**: Execute Phase 1 (Parallel Agent Coordination)