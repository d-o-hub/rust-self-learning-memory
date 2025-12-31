# File Restructuring Checklist

**Purpose**: Track progress during plans directory restructuring
**Execution Plan**: `PLANS_DIRECTORY_RESTRUCTURE_EXECUTION_PLAN.md`
**Start Date**: _____
**Completion Date**: _____

---

## Phase 1: Archive Outdated Content (30-45 min)

### Action 1.1: Archive Version-Specific Roadmaps (10 min)
- [ ] Verify `archive/v0.1.7-roadmap/` exists
- [ ] Move any loose v0.1.7/v0.1.8 prep materials
- [ ] Verify no version-specific roadmaps in active directories
- [ ] Notes: _______________________________

### Action 1.2: Archive One-Time Audit Reports (5 min)
- [ ] Create `archive/completed/2025-12/audits/`
- [ ] Move `DOCUMENTATION_AUDIT_*.md` files
- [ ] Move `DOCUMENTATION_VERIFICATION_*.md` files
- [ ] Move `PLANS_VERIFICATION_SUMMARY_*.md` files
- [ ] Verify no audit reports in active directories
- [ ] Notes: _______________________________

### Action 1.3: Archive Superseded Status Reports (10 min)
- [ ] Keep: `PROJECT_STATUS_UNIFIED.md` (will be renamed)
- [ ] Archive: `V019_STATUS_REPORT.md` → `archive/completed/2025-12/status/`
- [ ] Archive: `PHASE*_VALIDATION_REPORT_*.md` → `archive/completed/2025-12/phase-validation/`
- [ ] Archive: `MEMORY_SYSTEM_VERIFICATION_REPORT_*.md` → `archive/completed/2025-12/verification/`
- [ ] Archive: `PHASE*_CODE_REVIEW_REPORT_*.md` → `archive/completed/2025-12/code-review/`
- [ ] Verify only one status report in STATUS/
- [ ] Notes: _______________________________

### Action 1.4: Archive Completed GOAP Executions (10 min)
- [ ] Create `archive/goap-plans/completed/2025-12/`
- [ ] Archive: `*_EXECUTION_PLAN.md` files
- [ ] Archive: `*_EXECUTION_SUMMARY.md` files
- [ ] Archive: `*_QA_REPORT.md` files
- [ ] Archive: `*_TEST_PLAN.md` files
- [ ] Archive: `*_PROGRESS_TRACKING.md` files
- [ ] Keep: GOAP guide files (to be consolidated)
- [ ] Verify only 3-4 core GOAP files remain
- [ ] Notes: _______________________________

### Action 1.5: Archive Research Integration Materials (10 min)
- [ ] Create `archive/research/phase1-4-integration/`
- [ ] Archive: `research/PHASE1_INTEGRATION_PLAN.md`
- [ ] Archive: `research/PHASE2_INTEGRATION_PLAN.md`
- [ ] Archive: `research/PHASE3_INTEGRATION_PLAN.md`
- [ ] Archive: `archive/completed/2025-12/*INTEGRATION*.md`
- [ ] Verify no phase-specific integration plans in active research/
- [ ] Notes: _______________________________

#### Phase 1 Quality Gate
- [ ] All version-specific docs in archive
- [ ] No duplicate status reports
- [ ] Archive properly categorized by type
- [ ] Archive index updated
- [ ] Time taken: _____ minutes

---

## Phase 2: Consolidate Overlapping Content (45-60 min)

### Action 2.1: Consolidate Status Reports (15 min)
- [ ] Create `STATUS/PROJECT_STATUS.md` (<500 lines)
- [ ] Extract from: `PROJECT_STATUS_UNIFIED.md`
- [ ] Extract from: `IMPLEMENTATION_STATUS.md`
- [ ] Extract from: `PROJECT_SUMMARY_2025-12.md`
- [ ] Archive originals
- [ ] Verify consolidated file is comprehensive
- [ ] Verify file <500 lines
- [ ] Notes: _______________________________

### Action 2.2: Consolidate Roadmaps (15 min)
- [ ] Keep: `ROADMAPS/ROADMAP_ACTIVE.md`
- [ ] Update: `ROADMAPS/ROADMAP_VISION.md` (merge ROADMAP_V030_VISION.md)
- [ ] Create: `ROADMAPS/ROADMAP_HISTORY.md` (condensed version history)
- [ ] Archive: `ROADMAPS/ROADMAP_V010_ARCHIVED.md`
- [ ] Archive: `ROADMAPS/ROADMAP_VERSION_HISTORY.md`
- [ ] Verify 3 roadmap files total
- [ ] Verify all <500 lines
- [ ] Notes: _______________________________

### Action 2.3: Consolidate Embedding Documentation (20 min)
- [ ] Create directory: `reference/embeddings/`
- [ ] Create: `reference/embeddings/MULTI_PROVIDER_GUIDE.md` (<500 lines)
- [ ] Extract from: `EMBEDDINGS_REFACTOR_DESIGN.md`
- [ ] Extract from: `EMBEDDING_CONFIGURATION_REFACTOR_SUMMARY.md`
- [ ] Extract from: `MULTI_EMBEDDING_PROVIDER_COMPLETION_GUIDE.md`
- [ ] Extract from: `EMBEDDINGS_COMPLETION_ROADMAP.md`
- [ ] Extract from: `EMBEDDINGS_INTEGRATION_ANALYSIS.md`
- [ ] Archive superseded files
- [ ] Verify guide is comprehensive
- [ ] Verify <500 lines
- [ ] Notes: _______________________________

### Action 2.4: Consolidate GOAP Documentation (15 min)
- [ ] Create: `goap/GOAP_AGENT_GUIDE.md` (<500 lines)
- [ ] Extract from: `GOAP_AGENT_IMPROVEMENT_PLAN.md`
- [ ] Extract from: `GOAP_AGENT_CODEBASE_VERIFICATION.md`
- [ ] Extract from: `GOAP_AGENT_ROADMAP.md`
- [ ] Keep: `GOAP_EXECUTION_TEMPLATE.md`
- [ ] Keep: `GOAP_QUALITY_GATES.md`
- [ ] Archive: remaining GOAP agent files
- [ ] Verify guide is comprehensive
- [ ] Verify 3 GOAP files total
- [ ] Notes: _______________________________

#### Phase 2 Quality Gate
- [ ] No overlapping content across files
- [ ] All consolidated files <500 lines
- [ ] Critical information preserved
- [ ] Navigation updated in README.md
- [ ] Time taken: _____ minutes

---

## Phase 3: Update and Split Large Files (60-75 min)

### Action 3.1: Split API_DOCUMENTATION.md (20 min)
- [ ] Create directory: `reference/architecture/`
- [ ] Split into: `API_OVERVIEW.md` (<400 lines)
- [ ] Split into: `EPISODE_API.md` (<400 lines)
- [ ] Split into: `PATTERN_API.md` (<400 lines)
- [ ] Split into: `STORAGE_API.md` (<400 lines)
- [ ] Split into: `RETRIEVAL_API.md` (<400 lines)
- [ ] Archive original: `ARCHITECTURE/API_DOCUMENTATION.md`
- [ ] Verify no content loss
- [ ] Verify all files <500 lines
- [ ] Notes: _______________________________

### Action 3.2: Handle PHASE3_ACTION_PLAN.md (15 min)
- [ ] Assess: Is this still active or complete?
- [ ] If active:
  - [ ] Split into logical phases/components
  - [ ] Create multiple <500 line files
- [ ] If complete:
  - [ ] Archive to `archive/completed/2025-12/phase3/`
- [ ] Verify decision is appropriate
- [ ] Notes: _______________________________

### Action 3.3: Split MEMORY_MCP_VALIDATION_REPORT.md (15 min)
- [ ] Create directory: `reference/mcp/`
- [ ] Extract: `MCP_SECURITY_MODEL.md` (<500 lines)
- [ ] Extract: `MCP_VALIDATION_RESULTS.md` (<500 lines)
- [ ] Extract: `MCP_PERFORMANCE_ANALYSIS.md` (<500 lines)
- [ ] Archive original: `STATUS/MEMORY_MCP_VALIDATION_REPORT.md`
- [ ] Verify no content loss
- [ ] Verify all files <500 lines
- [ ] Notes: _______________________________

### Action 3.4: Update and Split Research Files (20 min)
- [ ] Create directory: `reference/research/`
- [ ] Handle: `ets_forecasting_best_practices.md`
  - [ ] Extract active best practices → `ETS_FORECASTING.md`
  - [ ] Archive historical sections
- [ ] Handle: `dbscan_anomaly_detection_best_practices.md`
  - [ ] Extract active best practices → `DBSCAN_ANOMALY_DETECTION.md`
  - [ ] Archive historical sections
- [ ] Handle: Remaining phase integration plans
  - [ ] Extract ongoing work → `active/RESEARCH_INTEGRATION.md`
  - [ ] Archive completed phases
- [ ] Verify all active files <500 lines
- [ ] Notes: _______________________________

### Action 3.5: Update Top-Level Files (15 min)
- [ ] Extract from: `GAP_ANALYSIS_REPORT_2025-12-29.md`
  - [ ] Create: `active/GAP_ANALYSIS.md`
  - [ ] Archive original
- [ ] Extract from: `IMPLEMENTATION_PRIORITY_PLAN_2025-12-29.md`
  - [ ] Create: `active/IMPLEMENTATION_PRIORITIES.md`
  - [ ] Archive original
- [ ] Extract from: `OPTIMIZATION_ANALYSIS_2025-12-29.md`
  - [ ] Create: `active/PERFORMANCE_OPTIMIZATION.md`
  - [ ] Archive original
- [ ] Verify current plans extracted
- [ ] Verify archives organized
- [ ] Notes: _______________________________

#### Phase 3 Quality Gate
- [ ] All active files <500 lines
- [ ] Large files properly split or archived
- [ ] No content loss during splitting
- [ ] Cross-references updated
- [ ] Time taken: _____ minutes

---

## Phase 4: Create New Reference Documentation (30-45 min)

### Action 4.1: Create Current Capability Guides (15 min)
- [ ] Create: `reference/architecture/CURRENT_CAPABILITIES.md` (<500 lines)
- [ ] Include: Core Memory System section
- [ ] Include: Storage Layer section
- [ ] Include: Embeddings section
- [ ] Include: MCP Server section
- [ ] Include: CLI Interface section
- [ ] Include: Performance Characteristics section
- [ ] Verify comprehensive coverage
- [ ] Verify <500 lines
- [ ] Notes: _______________________________

### Action 4.2: Create Quick Start Guides (10 min)
- [ ] Create: `QUICK_START_PLANS.md` (<300 lines)
- [ ] Include: New feature development guidance
- [ ] Include: Bug fix process
- [ ] Include: Performance issue workflow
- [ ] Include: Security issue checklist
- [ ] Verify clear and actionable
- [ ] Notes: _______________________________

### Action 4.3: Update README.md (10 min)
- [ ] Update structure section
- [ ] Add navigation to new folders
- [ ] Update file counts
- [ ] Add quick links to key documents
- [ ] Verify accuracy
- [ ] Verify all links work
- [ ] Notes: _______________________________

### Action 4.4: Update Archive Index (10 min)
- [ ] Update: `archive/ARCHIVE_INDEX.md`
- [ ] Add entries for newly archived content
- [ ] Organize by category and date
- [ ] Verify completeness
- [ ] Notes: _______________________________

#### Phase 4 Quality Gate
- [ ] All new guides created
- [ ] All files <500 lines
- [ ] README.md updated and accurate
- [ ] Archive index complete
- [ ] Time taken: _____ minutes

---

## Phase 5: Validation and Testing (20-30 min)

### Action 5.1: Link Validation (10 min)
- [ ] Check all internal links in remaining files
- [ ] Update broken links to reflect new structure
- [ ] Verify all key documents accessible
- [ ] Run link checker (if available)
- [ ] Notes: _______________________________

### Action 5.2: Content Preservation Check (10 min)
- [ ] Architecture decisions preserved
- [ ] Current status preserved
- [ ] Active roadmaps preserved
- [ ] Quality gates preserved
- [ ] Critical API documentation preserved
- [ ] MCP security documentation preserved
- [ ] Create preservation checklist
- [ ] Notes: _______________________________

### Action 5.3: File Count Verification (5 min)
- [ ] Count top-level files: _____ (target: <20)
- [ ] Count active/ files: _____ (target: 3-5)
- [ ] Count reference/ files: _____ (target: 15-20)
- [ ] Count roadmaps/ files: _____ (target: 3)
- [ ] Count status/ files: _____ (target: 2-3)
- [ ] Count goap/ files: _____ (target: 3)
- [ ] Count archive/ files: _____ (target: 180-190)
- [ ] Total active files: _____ (target: <100)
- [ ] Notes: _______________________________

### Action 5.4: Documentation Review (5 min)
- [ ] Read through new structure
- [ ] Check for consistency
- [ ] Identify any remaining issues
- [ ] Document any post-execution tasks
- [ ] Notes: _______________________________

#### Phase 5 Quality Gate
- [ ] All links working
- [ ] No critical information lost
- [ ] File count targets met
- [ ] Structure validated
- [ ] Time taken: _____ minutes

---

## Overall Quality Gates

### Quantitative
- [ ] Active file count: <100 files
- [ ] No file exceeds 500 lines
- [ ] Archive contains all historical content
- [ ] No broken links

### Qualitative
- [ ] Clear separation between active and archived content
- [ ] Navigation is intuitive
- [ ] Critical information preserved
- [ ] Structure aligns with v0.1.9 capabilities

---

## Post-Execution Tasks

- [ ] Create backup before starting: `cp -r plans/ plans.backup/`
- [ ] Commit changes with message: `refactor(plans): restructure directory for better organization`
- [ ] Verify no CI jobs reference moved files
- [ ] Update any hardcoded paths
- [ ] Announce new structure to team
- [ ] Document any issues encountered
- [ ] Schedule quarterly cleanup reviews

---

## Issues Encountered

Track any issues or deviations from the plan:

1. _______________________________
2. _______________________________
3. _______________________________

---

## Lessons Learned

What went well:
1. _______________________________
2. _______________________________

What could be improved:
1. _______________________________
2. _______________________________

---

**Total Execution Time**: _____ hours _____ minutes
**Actual vs Estimated**: ___________
**Recommendations for Future Restructuring**: _______________________________
