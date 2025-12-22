# Plans Folder Cleanup Plan

## Overview
This document outlines the cleanup strategy for the `/workspaces/feat-phase3/plans/` folder to remove duplicate and outdated files while preserving important historical records.

## Analysis Summary

### File Structure
```
/plans/
├── archive/                 # ✅ Keep as source of truth
│   ├── goap-plans/         # Contains latest versions
│   ├── legacy/             # Historical planning docs
│   ├── releases/           # Version-specific archives
│   └── research/           # Research artifacts
├── Root level files        # 🎯 Clean up duplicates
└── Documentation files     # ✅ Keep (unique content)
```

### Duplicate Files Identified (15 exact duplicates)

#### GitHub Actions Files (5 files)
- `github-actions-fix-2025.md` (149 lines) - duplicate in `archive/goap-plans/github-actions-2025/`
- `github-actions-issues-analysis.md` (206 lines) - duplicate in `archive/goap-plans/github-actions-2025/`
- `github-actions-update-plan.md` (393 lines) - duplicate in `archive/goap-plans/github-actions-2025/`
- `goap-github-actions-2025-update-report.md` (223 lines) - duplicate in `archive/goap-plans/github-actions-2025/`
- `goap-github-actions-optimization-plan.md` (283 lines) - duplicate in `archive/goap-plans/github-actions-2025/`

#### Loop Agent Files (3 files)
- `loop-agent-final-summary.md` (205 lines) - duplicate in `archive/goap-plans/`
- `loop-agent-github-actions-monitor.md` (178 lines) - duplicate in `archive/goap-plans/`
- `loop-iteration-1-results.md` (75 lines) - duplicate in `archive/goap-plans/`

#### Phase Implementation Files (5 files)
- `phase1-quick-wins-implementation.md` (471 lines) - duplicate in `archive/goap-plans/`
- `goap-phase2c-javy-plan.md` (582 lines) - duplicate in `archive/releases/v0.1.6/`
- `phase2c-javy-completion-final.md` (425 lines) - duplicate in `archive/releases/v0.1.6/`
- `phase2c-javy-completion-status.md` (213 lines) - duplicate in `archive/releases/v0.1.6/`
- `phase2c-javy-verification-report.md` (296 lines) - duplicate in `archive/releases/v0.1.6/`

#### General Files (2 files)
- `goap-execution-schedule.md` (321 lines) - duplicate in `archive/goap-plans/`
- `javy-research-findings.md` (165 lines) - duplicate in `archive/releases/v0.1.6/`

## Files to Keep (Unique Content)

### Documentation & Strategic Documents (11 files)
- `README.md` (358 lines) - Project overview and navigation
- `ROADMAP.md` (1029 lines) - Long-term project roadmap
- `14-v0.2.0-roadmap.md` (641 lines) - Version 0.2.0 specific roadmap
- `15-long-term-vision.md` (504 lines) - Long-term vision document
- `21-architecture-decision-records.md` (955 lines) - ADR documentation
- `PROJECT_STATUS.md` (301 lines) - Current project status
- `CHANGES_SUMMARY.md` (466 lines) - Summary of all changes
- `MISSING_IMPLEMENTATIONS_ANALYSIS.md` (739 lines) - Analysis of missing features
- `PLANS_FOLDER_OPTIMIZATION_RECOMMENDATIONS.md` (375 lines) - Previous optimization attempt
- `PLANS_FOLDER_CLEANUP_PLAN.md` - This file
- `v0.1.7-release-preparation-summary.md` (212 lines) - v0.1.7 release prep (unique)

### Archive Structure to Preserve
```
archive/
├── goap-plans/                 # Latest GOAP planning docs
│   └── github-actions-2025/    # 2025 GitHub Actions updates
├── legacy/                     # Historical planning workflows
├── releases/                   # Version-specific documentation
│   └── v0.1.6/                # v0.1.6 complete archive
└── research/                   # Research findings
```

## Cleanup Strategy

### Phase 1: Remove Duplicate Files (15 files)
Delete the following root-level files (keeping archive versions):
1. `github-actions-fix-2025.md`
2. `github-actions-issues-analysis.md`
3. `github-actions-update-plan.md`
4. `goap-github-actions-2025-update-report.md`
5. `goap-github-actions-optimization-plan.md`
6. `loop-agent-final-summary.md`
7. `loop-agent-github-actions-monitor.md`
8. `loop-iteration-1-results.md`
9. `phase1-quick-wins-implementation.md`
10. `goap-phase2c-javy-plan.md`
11. `phase2c-javy-completion-final.md`
12. `phase2c-javy-completion-status.md`
13. `phase2c-javy-verification-report.md`
14. `goap-execution-schedule.md`
15. `javy-research-findings.md`

### Phase 2: Update Navigation
- Update `README.md` to reflect new structure
- Ensure archive folders are well-organized

### Phase 3: Validation
- Verify all unique content preserved
- Check that documentation references are updated
- Create final cleanup summary

## Expected Results

### Before Cleanup
- Total .md files: ~50+
- Duplicate files: 15
- Root level clutter: High
- Navigation complexity: High

### After Cleanup
- Total .md files: ~35 (deduplicated)
- Duplicate files: 0
- Root level clutter: Low
- Navigation complexity: Low
- Archive structure: Clear and logical

## Benefits
1. **Reduced Clutter**: 15 duplicate files removed from root level
2. **Clear Hierarchy**: Archive folder becomes single source of truth
3. **Better Navigation**: Easier to find relevant documents
4. **Historical Preservation**: All content preserved in organized archives
5. **Maintenance**: Easier to maintain and update going forward

## Risk Mitigation
- All duplicates verified to be exact matches
- Archive versions confirmed to be identical or more comprehensive
- Backup strategy: Changes logged for potential rollback if needed

---
**Generated**: 2025-12-19
**Status**: Ready for Execution