# GOAP Execution Plan: Plans Directory Restructuring

**Plan ID**: GOAP_PLANS_RESTRUCTURE_2025-12-31
**Version**: 1.0
**Created**: 2025-12-31
**Status**: Ready for Execution
**Complexity**: High (Multi-phase, 285+ files, archival)
**Estimated Duration**: 3-4 hours

---

## Executive Summary

The `/workspaces/feat-phase3/plans` directory currently contains **285 markdown files**, many of which are outdated, duplicated, or exceed the project's 500-line per file policy. This GOAP execution plan will restructure the plans directory to:

1. **Reduce file count** from 285 to <100 active files
2. **Preserve critical information** through strategic consolidation
3. **Align with current capabilities** (v0.1.9: episodic memory, embeddings, multi-storage, MCP server)
4. **Enforce 500-line file limit** policy
5. **Separate active plans** from archived/legacy content
6. **Maintain navigability** with clear folder structure and documentation

---

## Current State Analysis

### File Distribution
```
Total Files: 285 markdown files

Top-level:        39 files (13.7%)
GOAP/:            34 files (11.9%)
ARCHITECTURE/:     5 files (1.8%)
CONFIGURATION/:    9 files (3.2%)
ROADMAPS/:         4 files (1.4%)
STATUS/:          11 files (3.9%)
research/:        27 files (9.5%)
archive/:        154 files (54.0%)
benchmark_results/: 1 file (0.4%)
test-reports/:     1 file (0.4%)
```

### Critical Issues

#### 1. File Size Violations (>500 lines)
```
Top 10 Violators:
- turso-local-ci-setup-plan.md: 1989 lines (archive)
- API_DOCUMENTATION.md: 1407 lines (ARCHITECTURE/)
- PHASE3_ACTION_PLAN.md: 1365 lines (GOAP/)
- ets_forecasting_best_practices.md: 1316 lines (research/)
- MEMORY_MCP_VALIDATION_REPORT.md: 1292 lines (STATUS/)
- PHASE3_INTEGRATION_PLAN.md: 1264 lines (research/)
- dbscan_anomaly_detection_best_practices.md: 1243 lines (research/)
- FINAL_RESEARCH_INTEGRATION_REPORT_OLD.md: 1197 lines (archive/)
- PHASE2_INTEGRATION_PLAN.md: 1161 lines (research/)
- 03-execute.md: 1143 lines (archive/legacy/)
```

#### 2. Content Overlap and Duplication
- Multiple version status reports (V019_STATUS_REPORT.md, PROJECT_STATUS_UNIFIED.md, etc.)
- Duplicate GOAP execution summaries across multiple phases
- Overlapping roadmap documentation
- Redundant implementation status reports

#### 3. Outdated Content
- v0.1.7 roadmaps and prep materials (superseded by ROADMAP_V010_ARCHIVED.md)
- Phase completion summaries that are no longer relevant
- One-time audit and verification reports
- Outdated optimization analyses (pre-embedding refactor)

#### 4. File Organization Issues
- Archive lacks clear categorization by relevance
- Research files mixed with active plans
- GOAP plans scattered across multiple directories
- No clear separation between "in progress" and "completed" active documents

---

## Target Structure

### Proposed New Directory Layout

```
plans/
├── README.md (updated)
│
├── active/                          # Active, in-progress plans
│   ├── FEATURE_PLANS.md             # Active feature development plans
│   ├── QUALITY_IMPROVEMENTS.md      # Active quality enhancement plans
│   └── PERFORMANCE_OPTIMIZATION.md  # Active performance work
│
├── reference/                       # Current capability documentation
│   ├── architecture/
│   │   ├── ARCHITECTURE_CORE.md     # Core architecture (split if >500 lines)
│   │   ├── ARCHITECTURE_PATTERNS.md # Architecture patterns (split if >500 lines)
│   │   └── STORAGE_ARCHITECTURE.md  # Storage layer details (NEW)
│   ├── embeddings/
│   │   ├── MULTI_PROVIDER_GUIDE.md  # Multi-provider embeddings (NEW)
│   │   └── VECTOR_SEARCH_OPTIMIZATION.md (consolidated from existing)
│   ├── mcp/
│   │   ├── MCP_SERVER_GUIDE.md      # MCP server documentation (NEW)
│   │   └── MCP_SECURITY_MODEL.md    # Security sandbox details (NEW)
│   └── storage/
│       ├── DUAL_STORAGE_DESIGN.md   # Turso + redb architecture (NEW)
│       └── CIRCUIT_BREAKER_GUIDE.md # Circuit breaker patterns (NEW)
│
├── roadmaps/
│   ├── ROADMAP_ACTIVE.md            # Current v0.1.x roadmap
│   ├── ROADMAP_VISION.md            # v1.0+ vision (consolidated)
│   └── ROADMAP_HISTORY.md           # Historical overview (condensed)
│
├── status/
│   ├── PROJECT_STATUS.md            # Current status (consolidated)
│   ├── IMPLEMENTATION_STATUS.md     # Implementation tracking
│   └── QUALITY_METRICS.md           # Quality gates and metrics (NEW)
│
├── goap/                            # GOAP agent documentation
│   ├── GOAP_AGENT_GUIDE.md          # GOAP agent usage guide (consolidated)
│   ├── GOAP_EXECUTION_TEMPLATE.md   # Execution template
│   └── GOAP_QUALITY_GATES.md        # Quality gates
│
└── archive/                         # Historical documentation
    ├── completed/                   # Completed work by version
    ├── research/                    # Archived research findings
    └── legacy/                      # Legacy planning framework
```

### File Count Targets
- **Before**: 285 files
- **After**: ~80-90 active files (72% reduction)
- **Archive**: 180-190 files (preserved but organized)

---

## Execution Plan

### Phase 1: Archive Outdated Content (30-45 minutes)

**Objective**: Move clearly outdated files to archive with proper categorization

#### Actions

**Action 1.1: Archive Version-Specific Roadmaps** (10 min)
- Move to `archive/completed/2025-12/v0.1.7-roadmap/`
- Files:
  - `archive/v0.1.7-roadmap/*` (already in archive, ensure proper location)
  - Any v0.1.8 prep materials
- **Success Criterion**: No version-specific roadmaps in active directories

**Action 1.2: Archive One-Time Audit Reports** (5 min)
- Move to `archive/completed/2025-12/audits/`
- Files:
  - `DOCUMENTATION_AUDIT_*.md`
  - `DOCUMENTATION_VERIFICATION_*.md`
  - `PLANS_VERIFICATION_SUMMARY_*.md`
- **Success Criterion**: No one-time audit reports in active directories

**Action 1.3: Archive Superseded Status Reports** (10 min)
- Keep only: `PROJECT_STATUS_UNIFIED.md` as single source of truth
- Archive superseded:
  - `V019_STATUS_REPORT.md` → `archive/completed/2025-12/status/`
  - `PHASE*_VALIDATION_REPORT_*.md` → `archive/completed/2025-12/phase-validation/`
  - `MEMORY_SYSTEM_VERIFICATION_REPORT_*.md` → `archive/completed/2025-12/verification/`
- **Success Criterion**: One status report file in STATUS/ directory

**Action 1.4: Archive Completed GOAP Executions** (10 min)
- Move completed execution plans to `archive/goap-plans/completed/2025-12/`
- Keep only:
  - `GOAP_AGENT_GUIDE.md` (to be created)
  - `GOAP_EXECUTION_TEMPLATE.md`
  - `GOAP_QUALITY_GATES.md`
- Archive:
  - `*_EXECUTION_PLAN.md` files
  - `*_EXECUTION_SUMMARY.md` files
  - `*_QA_REPORT.md` files
- **Success Criterion**: Only 3-4 core GOAP documentation files in goap/

**Action 1.5: Archive Research Integration Materials** (10 min)
- Move to `archive/research/phase1-4-integration/`
- Files:
  - `research/PHASE*_INTEGRATION_PLAN.md`
  - `archive/completed/2025-12/*INTEGRATION*.md`
- **Success Criterion**: No phase-specific integration plans in active research/

#### Quality Gate
- [ ] All version-specific docs in archive
- [ ] No duplicate status reports
- [ ] Archive properly categorized by type
- [ ] Archive index updated (ARCHIVE_INDEX.md)

---

### Phase 2: Consolidate Overlapping Content (45-60 minutes)

**Objective**: Merge duplicate and overlapping information

#### Actions

**Action 2.1: Consolidate Status Reports** (15 min)
- Create new: `STATUS/PROJECT_STATUS.md`
- Content from:
  - `PROJECT_STATUS_UNIFIED.md`
  - `IMPLEMENTATION_STATUS.md` (merge sections)
  - `PROJECT_SUMMARY_2025-12.md` (extract current info)
- Archive originals
- **Success Criterion**: Single, comprehensive status file (<500 lines)

**Action 2.2: Consolidate Roadmaps** (15 min)
- Keep: `ROADMAPS/ROADMAP_ACTIVE.md`
- Update: `ROADMAPS/ROADMAP_VISION.md` (merge ROADMAP_V030_VISION.md)
- Create: `ROADMAPS/ROADMAP_HISTORY.md` (condensed version history)
- Archive: `ROADMAPS/ROADMAP_V010_ARCHIVED.md`, `ROADMAP_VERSION_HISTORY.md`
- **Success Criterion**: 3 roadmap files total, all <500 lines

**Action 2.3: Consolidate Embedding Documentation** (20 min)
- Create: `reference/embeddings/MULTI_PROVIDER_GUIDE.md`
- Merge content from:
  - `EMBEDDINGS_REFACTOR_DESIGN.md` (extract active info)
  - `EMBEDDING_CONFIGURATION_REFACTOR_SUMMARY.md`
  - `MULTI_EMBEDDING_PROVIDER_COMPLETION_GUIDE.md`
  - `EMBEDDINGS_COMPLETION_ROADMAP.md` (keep current roadmap)
  - `EMBEDDINGS_INTEGRATION_ANALYSIS.md` (extract relevant parts)
- Archive superseded files
- **Success Criterion**: One consolidated guide <500 lines

**Action 2.4: Consolidate GOAP Documentation** (15 min)
- Create: `goap/GOAP_AGENT_GUIDE.md`
- Merge content from:
  - `GOAP_AGENT_IMPROVEMENT_PLAN.md` (extract guide sections)
  - `GOAP_AGENT_CODEBASE_VERIFICATION.md` (extract verification patterns)
  - `GOAP_AGENT_ROADMAP.md` (extract roadmap)
- Keep: `GOAP_EXECUTION_TEMPLATE.md`, `GOAP_QUALITY_GATES.md`
- Archive: Remaining GOAP agent files
- **Success Criterion**: 3 GOAP files, guide <500 lines

#### Quality Gate
- [ ] No overlapping content across files
- [ ] All consolidated files <500 lines
- [ ] Critical information preserved
- [ ] Navigation updated in README.md

---

### Phase 3: Update and Split Large Files (60-75 minutes)

**Objective**: Enforce 500-line file limit and update outdated content

#### Actions

**Action 3.1: Split API_DOCUMENTATION.md** (20 min)
- Current: 1407 lines (violates 500-line limit)
- Split into:
  - `reference/architecture/API_OVERVIEW.md` (intro and concepts, <400 lines)
  - `reference/architecture/EPISODE_API.md` (episode operations, <400 lines)
  - `reference/architecture/PATTERN_API.md` (pattern operations, <400 lines)
  - `reference/architecture/STORAGE_API.md` (storage operations, <400 lines)
  - `reference/architecture/RETRIEVAL_API.md` (retrieval operations, <400 lines)
- **Success Criterion**: All API files <500 lines, no content loss

**Action 3.2: Split PHASE3_ACTION_PLAN.md** (15 min)
- Current: 1365 lines (violates 500-line limit)
- Assess: Is this still active?
- If still active: Split into logical phases/components
- If complete: Move to `archive/completed/2025-12/phase3/`
- **Success Criterion**: File either split or archived

**Action 3.3: Split MEMORY_MCP_VALIDATION_REPORT.md** (15 min)
- Current: 1292 lines (violates 500-line limit)
- Extract: `reference/mcp/MCP_SECURITY_MODEL.md` (security architecture)
- Extract: `reference/mcp/MCP_VALIDATION_RESULTS.md` (test results)
- Extract: `reference/mcp/MCP_PERFORMANCE_ANALYSIS.md` (performance metrics)
- Archive: Original comprehensive report
- **Success Criterion**: 3-4 focused files, all <500 lines

**Action 3.4: Update and Split Research Files** (20 min)
- Review: `research/ets_forecasting_best_practices.md` (1316 lines)
  - Extract active best practices → `reference/research/ETS_FORECASTING.md`
  - Archive: Historical analysis sections
- Review: `research/dbscan_anomaly_detection_best_practices.md` (1243 lines)
  - Extract active best practices → `reference/research/DBSCAN_ANOMALY_DETECTION.md`
  - Archive: Historical analysis sections
- Review: `research/PHASE*_INTEGRATION_PLAN.md` files
  - Extract ongoing work → `active/RESEARCH_INTEGRATION.md`
  - Archive: Completed phases
- **Success Criterion**: Active research files <500 lines, completed archived

**Action 3.5: Update Top-Level Files** (15 min)
- Review and update: `GAP_ANALYSIS_REPORT_2025-12-29.md`
  - Extract current gaps → `active/GAP_ANALYSIS.md`
  - Archive: Full historical report
- Review and update: `IMPLEMENTATION_PRIORITY_PLAN_2025-12-29.md`
  - Extract current priorities → `active/IMPLEMENTATION_PRIORITIES.md`
  - Archive: Full historical plan
- Review and update: `OPTIMIZATION_ANALYSIS_2025-12-29.md`
  - Extract active optimizations → `active/PERFORMANCE_OPTIMIZATION.md`
  - Archive: Full analysis
- **Success Criterion**: Current plans extracted, archives organized

#### Quality Gate
- [ ] All active files <500 lines
- [ ] Large files properly split or archived
- [ ] No content loss during splitting
- [ ] Cross-references updated

---

### Phase 4: Create New Reference Documentation (30-45 minutes)

**Objective**: Create consolidated documentation for current capabilities

#### Actions

**Action 4.1: Create Current Capability Guides** (15 min)

Create `reference/architecture/CURRENT_CAPABILITIES.md` (<500 lines):
```
Sections:
1. Core Memory System
   - Episode lifecycle
   - Pattern extraction
   - Heuristic learning

2. Storage Layer
   - Dual storage (Turso + redb)
   - Circuit breaker
   - Connection pooling

3. Embeddings
   - Multi-provider support (OpenAI, Cohere, Ollama, Local)
   - Configuration caching
   - Vector search optimization

4. MCP Server
   - 6-layer security sandbox
   - Tool implementation
   - Query caching

5. CLI Interface
   - 24 commands with aliases
   - Configuration management
   - Operations monitoring

6. Performance Characteristics
   - Current metrics (table)
   - Benchmarks summary
```

**Action 4.2: Create Quick Start Guides** (10 min)

Create `QUICK_START_PLANS.md` (<300 lines):
```
Sections:
1. For New Feature Development
   - Where to start
   - Which plan template to use
   - Reference documentation links

2. For Bug Fixes
   - Troubleshooting process
   - Quality gates

3. For Performance Issues
   - Benchmarking workflow
   - Optimization guidance

4. For Security Issues
   - Security checklist
   - Reporting procedures
```

**Action 4.3: Update README.md** (10 min)
- Update structure section
- Add navigation to new folders
- Update file counts
- Add quick links to key documents
- **Success Criterion**: README reflects new structure

**Action 4.4: Create Archive Index** (10 min)
- Update `archive/ARCHIVE_INDEX.md`
- Add entries for newly archived content
- Organize by category and date
- **Success Criterion**: All archived content indexed

#### Quality Gate
- [ ] All new guides created and <500 lines
- [ ] README.md updated and accurate
- [ ] Archive index complete
- [ ] Cross-references working

---

### Phase 5: Validation and Testing (20-30 minutes)

**Objective**: Validate new structure and ensure nothing is lost

#### Actions

**Action 5.1: Link Validation** (10 min)
- Check all internal links in remaining files
- Update broken links to reflect new structure
- Verify all key documents accessible
- **Success Criterion**: All links resolve, no broken references

**Action 5.2: Content Preservation Check** (10 min)
- Verify critical information preserved:
  - Architecture decisions
  - Current status
  - Active roadmaps
  - Quality gates
- Create preservation checklist
- **Success Criterion**: All critical info accounted for

**Action 5.3: File Count Verification** (5 min)
- Count files in each directory
- Verify target counts met:
  - Top-level: <20 files
  - active/: 3-5 files
  - reference/: 15-20 files
  - roadmaps/: 3 files
  - status/: 2-3 files
  - goap/: 3 files
  - archive/: 180-190 files
- **Success Criterion**: File count targets met

**Action 5.4: Documentation Review** (5 min)
- Read through new structure
- Check for consistency
- Identify any remaining issues
- **Success Criterion**: Structure logical and complete

#### Quality Gate
- [ ] All links working
- [ ] No critical information lost
- [ ] File count targets met
- [ ] Structure validated by manual review

---

## Dependencies

```
Phase 1 (Archive Outdated)
    ↓
Phase 2 (Consolidate Overlapping)
    ↓
Phase 3 (Update and Split Large Files)
    ↓
Phase 4 (Create New Reference Documentation)
    ↓
Phase 5 (Validation and Testing)
```

**Critical Path**: Phase 1 → 2 → 3 → 4 → 5
**Parallel Opportunities**: None (sequential due to dependencies)

---

## Risk Mitigation

### Risk 1: Accidental Deletion of Critical Information
- **Mitigation**: Create backup before restructuring
- **Action**: `cp -r /workspaces/feat-phase3/plans /workspaces/feat-phase3/plans.backup`

### Risk 2: Broken Links After Restructuring
- **Mitigation**: Link validation in Phase 5
- **Action**: Use script to find and update links automatically

### Risk 3: Loss of Historical Context
- **Mitigation**: Archive all superseded content, not delete
- **Action**: Maintain detailed archive index

### Risk 4: Time Overrun
- **Mitigation**: Prioritize by impact
- **Action**: If time-constrained, complete Phase 1-3 (core restructure), defer Phase 4-5

---

## Success Criteria

### Quantitative
- [ ] Active file count: <100 files (72% reduction from 285)
- [ ] No file exceeds 500 lines
- [ ] Archive contains all historical content
- [ ] No broken links

### Qualitative
- [ ] Clear separation between active and archived content
- [ ] Navigation is intuitive (verified by manual review)
- [ ] Critical information preserved (verified by checklist)
- [ ] Structure aligns with current v0.1.9 capabilities

---

## Resource Requirements

### Tools
- Standard shell commands (find, mv, cp, grep)
- Text editor (vim, nano, or similar)
- Git (for version control)

### Skills Required
- Understanding of markdown structure
- Knowledge of project capabilities
- Attention to detail for content preservation
- Basic shell scripting (optional, for link validation)

### Estimated Effort
- **Phase 1**: 30-45 minutes
- **Phase 2**: 45-60 minutes
- **Phase 3**: 60-75 minutes
- **Phase 4**: 30-45 minutes
- **Phase 5**: 20-30 minutes
- **Total**: 3-4 hours

---

## Post-Execution Activities

1. **Commit Changes**
   - Message: `refactor(plans): restructure directory for better organization`
   - Include summary of changes in commit message

2. **Update CI/CD**
   - Verify no CI jobs reference moved files
   - Update any hardcoded paths

3. **Team Communication**
   - Announce new structure to team
   - Provide quick start guide for navigation

4. **Maintenance**
   - Regular review of active/ vs archive/
   - Quarterly cleanup of completed work
   - Update archive index after each archival

---

## Appendix: Detailed File Lists

### Files to Archive (Phase 1)

**Version-Specific Roadmaps**:
- `archive/v0.1.7-roadmap/*` (ensure in proper location)

**One-Time Audit Reports**:
- `DOCUMENTATION_AUDIT_*.md`
- `DOCUMENTATION_VERIFICATION_*.md`
- `PLANS_VERIFICATION_SUMMARY_*.md`

**Superseded Status Reports**:
- `V019_STATUS_REPORT.md`
- `PHASE*_VALIDATION_REPORT_*.md`
- `MEMORY_SYSTEM_VERIFICATION_REPORT_*.md`
- `PHASE*_CODE_REVIEW_REPORT_*.md`

**Completed GOAP Executions**:
- `*_EXECUTION_PLAN.md`
- `*_EXECUTION_SUMMARY.md`
- `*_QA_REPORT.md`
- `*_TEST_PLAN.md`
- `*_PROGRESS_TRACKING.md`

**Research Integration Materials**:
- `research/PHASE1_INTEGRATION_PLAN.md`
- `research/PHASE2_INTEGRATION_PLAN.md`
- `research/PHASE3_INTEGRATION_PLAN.md`

### Files to Consolidate (Phase 2)

**Status Reports**:
- Merge: `PROJECT_STATUS_UNIFIED.md` + `IMPLEMENTATION_STATUS.md` + `PROJECT_SUMMARY_2025-12.md`
- Into: `STATUS/PROJECT_STATUS.md`

**Roadmaps**:
- Keep: `ROADMAPS/ROADMAP_ACTIVE.md`
- Merge: `ROADMAPS/ROADMAP_V030_VISION.md` + other vision docs
- Into: `ROADMAPS/ROADMAP_VISION.md`
- Create: `ROADMAPS/ROADMAP_HISTORY.md` (condensed)

**Embedding Documentation**:
- Merge: `EMBEDDINGS_REFACTOR_DESIGN.md` + `EMBEDDING_CONFIGURATION_REFACTOR_SUMMARY.md` +
        `MULTI_EMBEDDING_PROVIDER_COMPLETION_GUIDE.md` + `EMBEDDINGS_INTEGRATION_ANALYSIS.md`
- Into: `reference/embeddings/MULTI_PROVIDER_GUIDE.md`

**GOAP Documentation**:
- Merge: `GOAP_AGENT_IMPROVEMENT_PLAN.md` + `GOAP_AGENT_CODEBASE_VERIFICATION.md` +
        `GOAP_AGENT_ROADMAP.md`
- Into: `goap/GOAP_AGENT_GUIDE.md`

### Files to Split (Phase 3)

**Large Files (>500 lines)**:
- `ARCHITECTURE/API_DOCUMENTATION.md` (1407 lines) → 5 files
- `GOAP/PHASE3_ACTION_PLAN.md` (1365 lines) → split or archive
- `STATUS/MEMORY_MCP_VALIDATION_REPORT.md` (1292 lines) → 3-4 files
- `research/ets_forecasting_best_practices.md` (1316 lines) → split
- `research/PHASE2_INTEGRATION_PLAN.md` (1161 lines) → split or archive
- `research/PHASE1_INTEGRATION_PLAN.md` (1011 lines) → split or archive
- `EMBEDDINGS_REFACTOR_DESIGN.md` (994 lines) → split (if not consolidated)

---

**Plan Version**: 1.0
**Author**: GOAP Agent
**Review Status**: Pending Team Review
**Execution Priority**: High (before v0.1.13 development)
