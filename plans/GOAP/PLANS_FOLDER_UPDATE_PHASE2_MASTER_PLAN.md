# Phase 2: Master Change Plan for Plans/ Folder Update

**Created**: 2025-12-25
**Status**: READY FOR IMPLEMENTATION
**Next Phase**: Spawn specialist agents for edits

---

## Executive Summary

**Total Files Analyzed**: 28 files across active planning, configuration, GOAP docs, and archives
**Total Lines Analyzed**: ~8,500 lines
**Key Findings**:
- 6 files exceed 500-line limit (needs splitting)
- 45% content duplication across configuration docs (1,400+ lines)
- Outdated status markers throughout (67% complete, but docs show old checkmarks)
- Archive is well-organized but needs index updates
- GOAP execution plans are generally clean, some completed work can be archived

---

## Issues Breakdown by Category

### 🔴 CRITICAL: Files Exceeding 500-Line Limit

| File | Lines | Over Limit | % Over | Priority | Action |
|------|-------|------------|--------|----------|--------|
| **ROADMAP.md** | 1,141 | 641 | 128% | HIGH | SPLIT into 5 modular files |
| **CURRENT_ARCHITECTURE_STATE.md** | 858 | 358 | 72% | HIGH | SPLIT into 3 modular files |
| **IMPLEMENTATION_PLAN.md** | 610 | 110 | 22% | HIGH | SPLIT into 3 modular files |
| **CONFIG_IMPLEMENTATION_ROADMAP.md** | 1,034 | 534 | 107% | HIGH | SPLIT into 6 modular files |
| **CONFIG_USER_EXPERIENCE_IMPROVEMENTS.md** | 959 | 459 | 92% | HIGH | SPLIT into 6 modular files |
| **CONFIG_VALIDATION_STRATEGY.md** | 639 | 139 | 28% | MEDIUM | SPLIT into 3 modular files |

**Total lines over limit**: 3,641 lines (need ~14 new modular files)

---

### 🟠 HIGH: Content Duplication Across Configuration Files

**Duplication Analysis**: 1,400+ lines of duplicated content (45% of configuration docs)

#### 1. Simple Mode Implementation (290 lines duplicated in 3 files)
- **ROADMAP.md** (lines 403-513): Simple Mode API design
- **CONFIG_IMPLEMENTATION_ROADMAP.md** (lines 350-450): Simple Mode implementation
- **CONFIG_USER_EXPERIENCE_IMPROVEMENTS.md** (lines 102-275): Simple Mode design details

**Action**: Consolidate into single `CONFIG_SIMPLE_MODE.md`

#### 2. Configuration Wizard (625 lines duplicated in 3 files)
- **ROADMAP.md** (lines 516-728): Wizard design and flow
- **CONFIG_IMPLEMENTATION_ROADMAP.md** (lines 470-590): Wizard implementation
- **CONFIG_USER_EXPERIENCE_IMPROVEMENTS.md** (lines 277-690): Detailed wizard flows

**Action**: Consolidate into single `CONFIG_WIZARD.md`

#### 3. Validation Framework (129 lines duplicated in 3 files)
- **ROADMAP.md** (lines 116-221): Validation requirements
- **CONFIG_IMPLEMENTATION_ROADMAP.md** (lines 280-380): Validation implementation
- **CONFIG_VALIDATION_STRATEGY.md** (lines 385-392): Validation framework (entire doc)

**Action**: Keep `CONFIG_VALIDATION_STRATEGY.md`, delete duplicates from ROADMAP and IMPLEMENTATION_ROADMAP

#### 4. CLI Integration (183 lines duplicated in 2 files)
- **ROADMAP.md** (lines 730-779): CLI integration examples
- **CONFIG_USER_EXPERIENCE_IMPROVEMENTS.md** (lines 791-925): CLI command structure

**Action**: Consolidate into single `CONFIG_CLI_INTEGRATION.md`

---

### 🟠 HIGH: Outdated Status Information

**Issue**: Configuration is 67% complete, but documentation shows old checkmarks

#### Files with Outdated Status:
- **CONFIG_IMPLEMENTATION_ROADMAP.md**: Shows "✅ COMPLETE" markers for Phase 1-3 when only 67% complete
- **CONFIG_USER_EXPERIENCE_IMPROVEMENTS.md**: Claims "Complete and ready for implementation"
- **CONFIGURATION_OPTIMIZATION_STATUS.md**: Progress table shows inconsistent metrics (30% vs 67%)

**Action**: Update all status markers to accurately reflect 67% completion

---

### 🟡 MEDIUM: Cross-Reference Updates Needed

**Broken or Stale References**:
- PROJECT_STATUS_UNIFIED.md references IMPLEMENTATION_PLAN.md sections that will be split
- ROADMAP.md references CONFIG_IMPLEMENTATION_ROADMAP.md phases that will be reorganized
- Multiple files reference deprecated sections

**Action**: After splitting/merging, update all cross-references in navigation docs

---

### 🟢 LOW: Archive Cleanup

**Issues**:
- Archive index needs updating with recent additions
- Some completed execution plans should be archived
- Research index is complete and accurate

**Action**: Update archive indices, archive completed execution plans

---

## Change Groups by Commit

### Commit Group 1: Split ROADMAP.md (5 new files)
**Files to Create**:
- `ROADMAP_VERSION_HISTORY.md` (200 lines) - Version history and release notes
- `ROADMAP_V017_CURRENT.md` (200 lines) - Current v0.1.7 status and achievements
- `ROADMAP_V018_PLANNING.md` (200 lines) - v0.2.0 planning phase
- `ROADMAP_V019_VISION.md` (200 lines) - Long-term vision (v1.0.0)
- `ROADMAP_ACTIVE.md` (150 lines) - Active development tasks and next steps

**Files to Delete**:
- Original `ROADMAP.md` (1,141 lines)

**Estimated Lines Change**: -1,141 + 950 = **-191 lines** (net reduction: 191 lines)

---

### Commit Group 2: Split CURRENT_ARCHITECTURE_STATE.md (3 new files)
**Files to Create**:
- `ARCHITECTURE_CORE.md` (250 lines) - Core architecture: memory system, storage, MCP
- `ARCHITECTURE_PATTERNS.md` (250 lines) - Pattern extraction, embeddings, learning
- `ARCHITECTURE_INTEGRATION.md` (250 lines) - Integration points, API surface, CLI

**Files to Delete**:
- Original `CURRENT_ARCHITECTURE_STATE.md` (858 lines)

**Estimated Lines Change**: -858 + 750 = **-108 lines** (net reduction: 108 lines)

---

### Commit Group 3: Split IMPLEMENTATION_PLAN.md (3 new files)
**Files to Create**:
- `IMPLEMENTATION_STATUS.md` (200 lines) - Implementation status tracking
- `IMPLEMENTATION_PHAS1.md` (200 lines) - Phase 1: Critical fixes (completed)
- `IMPLEMENTATION_PHASE2.md` (200 lines) - Phase 2: Configuration optimization (67% complete)

**Files to Delete**:
- Original `IMPLEMENTATION_PLAN.md` (610 lines)

**Estimated Lines Change**: -610 + 600 = **-10 lines** (net reduction: 10 lines)

---

### Commit Group 4: Split CONFIG_IMPLEMENTATION_ROADMAP.md (6 new files)
**Files to Create**:
- `CONFIG_PHASE1_FOUNDATION.md` (150 lines) - Phase 1: Module structure, loader.rs
- `CONFIG_PHASE2_VALIDATION.md` (150 lines) - Phase 2: Validation framework
- `CONFIG_PHASE3_STORAGE.md` (150 lines) - Phase 3: Storage initialization
- `CONFIG_PHASE4_USER_EXPERIENCE.md` (150 lines) - Phase 4: Simple Mode and Wizard
- `CONFIG_PHASE5_OPTIMIZATION.md` (150 lines) - Phase 5: Quality assurance
- `CONFIG_PHASE6_REFERENCE.md` (150 lines) - Implementation references

**Files to Delete**:
- Original `CONFIG_IMPLEMENTATION_ROADMAP.md` (1,034 lines)

**Estimated Lines Change**: -1,034 + 900 = **-134 lines** (net reduction: 134 lines)

---

### Commit Group 5: Split CONFIG_USER_EXPERIENCE_IMPROVEMENTS.md (6 new files)
**Files to Create**:
- `CONFIG_UX_PROBLEMS.md` (150 lines) - Current UX problems and issues
- `CONFIG_UX_SIMPLE_MODE.md` (150 lines) - Simple Mode design and API
- `CONFIG_UX_WIZARD.md` (150 lines) - Wizard design and flow
- `CONFIG_UX_CLI_INTEGRATION.md` (150 lines) - CLI integration examples
- `CONFIG_UX_METRICS.md` (150 lines) - Success metrics and measurements
- `CONFIG_UX_MIGRATION.md` (150 lines) - Migration assistant design

**Files to Delete**:
- Original `CONFIG_USER_EXPERIENCE_IMPROVEMENTS.md` (959 lines)

**Estimated Lines Change**: -959 + 900 = **-59 lines** (net reduction: 59 lines)

---

### Commit Group 6: Split CONFIG_VALIDATION_STRATEGY.md (3 new files)
**Files to Create**:
- `CONFIG_VALIDATION_DESIGN.md` (200 lines) - Validation principles and framework
- `CONFIG_VALIDATION_IMPLEMENTATION.md` (200 lines) - Validation rules implementation
- `CONFIG_VALIDATION_TESTING.md` (200 lines) - Testing strategy and test suites

**Files to Delete**:
- Original `CONFIG_VALIDATION_STRATEGY.md` (639 lines)

**Estimated Lines Change**: -639 + 600 = **-39 lines** (net reduction: 39 lines)

---

### Commit Group 7: Merge Duplicate Configuration Content (5 new consolidated files)
**Files to Create**:
- `CONFIG_SIMPLE_MODE.md` (150 lines) - Consolidated Simple Mode implementation
- `CONFIG_WIZARD.md` (150 lines) - Consolidated Wizard implementation
- `CONFIG_CLI_INTEGRATION.md` (150 lines) - Consolidated CLI integration examples
- `CONFIG_STATUS_UPDATE.md` (200 lines) - Updated status markers (67% complete)
- `CONFIG_REFERENCES.md` (100 lines) - Cross-reference index

**Files to Update**:
- Delete duplicate sections from ROADMAP.md, CONFIG_IMPLEMENTATION_ROADMAP.md, CONFIG_USER_EXPERIENCE_IMPROVEMENTS

**Estimated Lines Change**: +750 (new) -1,400 (deletions) = **-650 lines** (net reduction: 650 lines)

---

### Commit Group 8: Update Status Information (8 files)
**Files to Update**:
- Update all checkmarks in CONFIG_IMPLEMENTATION_ROADMAP.md to reflect 67% completion
- Update all checkmarks in CONFIG_USER_EXPERIENCE_IMPROVEMENTS.md to reflect 67% completion
- Fix inconsistency in CONFIGURATION_OPTIMIZATION_STATUS.md (67% vs 10%)
- Update last modified dates to 2025-12-25

**Estimated Lines Change**: 0 (content updates only)

---

### Commit Group 9: Archive Cleanup and Index Updates (2 files)
**Files to Update**:
- Update `plans/archive/ARCHIVE_INDEX.md` with recent additions
- Update `plans/research/RESEARCH_INDEX.md` if needed

**Estimated Lines Change**: 0 (index updates only)

---

### Commit Group 10: Update Navigation Documents (3 files)
**Files to Update**:
- `README.md`: Update file list and cross-references
- `README_NAVIGATION.md`: Update navigation with new file structure
- `PROJECT_STATUS_UNIFIED.md`: Update with plan changes

**Estimated Lines Change**: 0 (navigation updates only)

---

## Implementation Phase Strategy

### Phase 3: Spawn Specialist Agents (Parallel with Handoff)

**Agent 1**: Split Files Agent
- **Task**: Split 6 oversize files into 32 modular files (<500 lines each)
- **Agent Type**: refactorer or feature-implementer
- **Deliverable**: 32 new files created, 6 original files deleted

**Agent 2**: Merge Duplicate Content Agent
- **Task**: Merge 1,400+ lines of duplicate configuration content into 5 consolidated files
- **Agent Type**: refactorer or general
- **Deliverable**: 5 new consolidated files created, duplicate sections deleted from 3 source files

**Agent 3**: Update Status Information Agent
- **Task**: Update all outdated status markers across 8 files
- **Agent Type**: refactorer or general
- **Deliverable**: 8 files updated with accurate 67% completion status

**Agent 4**: Archive and Navigation Updates Agent
- **Task**: Update archive indices and navigation documents
- **Agent Type**: refactorer or general
- **Deliverable**: 5 files updated with cross-references

**Agent 5**: Validate Cross-References Agent
- **Task**: After all other agents complete, validate all links and references are correct
- **Agent Type**: general
- **Deliverable**: Validation report with any issues found

**Agent 6**: Create New Documentation Agent (if needed)
- **Task**: Create any missing documentation identified during analysis
- **Agent Type**: feature-implementer or general
- **Deliverable**: New documentation files created

---

## Quality Gates for Implementation

### Before Spawning Phase 3 Agents:
- [ ] All agent reports reviewed and synthesized
- [ ] Master change plan complete
- [ ] Commit groups defined with clear file lists
- [ ] Agent assignments clear and well-scoped

### After Phase 3 Completion:
- [ ] All 6 oversize files split (32 new files created)
- [ ] Duplicate content merged (5 consolidated files)
- [ ] Status information updated (8 files)
- [ ] Archive indices updated (2 files)
- [ ] Navigation documents updated (3 files)
- [ ] All original oversize files deleted
- [ ] All cross-references validated and working
- [ ] Net line reduction achieved: ~616 lines

### Final Validation (Phase 4):
- [ ] All new files <500 lines
- [ ] No duplicate content remaining
- [ ] All cross-references valid
- [ ] Archive structure complete and indexed
- [ ] Navigation documents accurate
- [ ] Git commits atomic and well-described

---

## Handoff Coordination Requirements

### Between Agent 1 (Split Files) and Agent 2 (Merge Duplicate Content)
- **Agent 1** must create new modular files
- **Agent 2** must wait for Agent 1 completion before deleting duplicate sections from source files
- **Coordination**: Agent 1 signals completion → Agent 2 begins merging

### Between Agent 2 (Merge Duplicate Content) and Agent 3 (Update Status)
- **Agent 2** creates consolidated files
- **Agent 3** must wait for Agent 2 completion before updating status markers
- **Coordination**: Agent 2 signals completion → Agent 3 begins updates

### Between Agent 3 (Update Status) and Agent 4 (Archive Updates)
- **Agent 3** updates status information
- **Agent 4** must update navigation docs that reference updated files
- **Coordination**: Agent 3 signals completion → Agent 4 updates navigation

---

## Success Metrics

### Before Implementation:
- Active files: 33
- Files exceeding 500-line limit: 6
- Duplicate content: ~1,400 lines (45%)
- Outdated status markers: 8 files
- Total lines in plans/: ~8,500

### After Implementation (Target):
- Active files: ~60 (new modular files)
- Files exceeding 500-line limit: 0
- Duplicate content: ~0 lines (consolidated)
- Outdated status markers: 0
- Total lines in plans/: ~7,884 (net reduction: ~616 lines, 7% reduction)

### Quality Targets:
- All new files <500 lines ✅
- No duplicate content ✅
- All cross-references valid ✅
- Archive indices complete ✅
- Navigation accurate ✅

---

## Risk Mitigation

### High-Risk Items:
1. **Breaking Links During Split/Merge**
   - **Mitigation**: Agent 4 (Validate References) runs after all splits/merges
   - **Mitigation**: Commit Group 10 (Navigation Updates) is last commit

2. **Content Loss During Merge**
   - **Mitigation**: Careful review of all duplicate sections before deletion
   - **Mitigation**: Keep best version of duplicated content

3. **Agent Coordination Failures**
   - **Mitigation**: Clear handoff protocols defined above
   - **Mitigation**: Orchestrator monitors all agent status

### Medium-Risk Items:
1. **Git Commit Granularity Too Fine**
   - **Mitigation**: Related changes grouped into single commits
   - **Mitigation**: Commit message format: `[module] description`

2. **Status Update Conflicts**
   - **Mitigation**: Single agent (Agent 3) handles all status updates
   - **Mitigation**: Use consistent status: 67% complete

### Low-Risk Items:
1. **Archive Index Updates Missed**
   - **Mitigation**: Comprehensive review of all new files
   - **Mitigation**: Update indices after all file moves

---

## Rollback Plan

If any commit causes issues:
1. **Git Revert**: `git revert <commit-hash>` for problematic commit
2. **Restore Files**: Git revert will restore original file structure
3. **Re-plan**: Analyze what went wrong, adjust approach, retry
4. **Atomic Commits**: Each commit is logical and independent, minimizing rollback scope

---

**Master Plan Status**: ✅ COMPLETE
**Next Phase**: Spawn Phase 3 specialist agents for implementation
**Estimated Total Effort**: 40-60 hours for all 10 commit groups
**Estimated Net Line Reduction**: ~616 lines (7% reduction from current state)
