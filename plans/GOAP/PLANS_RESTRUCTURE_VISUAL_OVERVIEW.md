# Plans Directory Restructure - Visual Overview

## Current State → Target State Transformation

```
┌─────────────────────────────────────────────────────────────────┐
│                    CURRENT STATE (285 files)                      │
│                                                                  │
│  Top-level (39)                                                 │
│  ├─ Mixed active and archived content                            │
│  ├─ Many outdated files                                         │
│  └─ No clear organization                                       │
│                                                                  │
│  GOAP/ (36)                                                     │
│  ├─ Agent documentation                                         │
│  ├─ Execution plans (many completed)                            │
│  └─ Phase-specific files                                        │
│                                                                  │
│  ARCHITECTURE/ (5)                                              │
│  ├─ API_DOCUMENTATION.md (1407 lines ❌)                        │
│  ├─ ARCHITECTURE_* files                                        │
│  └─ Some exceed 500-line limit                                  │
│                                                                  │
│  CONFIGURATION/ (9)                                             │
│  ├─ Configuration guides                                         │
│  └─ Some exceed 500-line limit                                  │
│                                                                  │
│  ROADMAPS/ (4)                                                  │
│  ├─ Current, vision, history                                    │
│  └─ Some overlap and redundancy                                 │
│                                                                  │
│  STATUS/ (11)                                                   │
│  ├─ Multiple status reports (duplicate)                         │
│  ├─ Phase validation reports (completed)                        │
│  └─ MEMORY_MCP_VALIDATION_REPORT.md (1292 lines ❌)             │
│                                                                  │
│  research/ (27)                                                  │
│  ├─ Phase integration plans (completed)                         │
│  ├─ Best practices (large files)                                │
│  ├─ ets_forecasting_best_practices.md (1316 lines ❌)           │
│  └─ dbscan_anomaly_detection_best_practices.md (1243 lines ❌)  │
│                                                                  │
│  archive/ (154)                                                 │
│  ├─ Historical documents                                         │
│  ├─ Completed work                                               │
│  └─ Needs better organization                                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                    ┌─────────────────┐
                    │   TRANSFORM     │
                    │                 │
                    │ Phase 1-5       │
                    │ (3-4 hours)     │
                    └─────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    TARGET STATE (~90 files)                     │
│                                                                  │
│  Top-level (~20)                                                │
│  ├─ README.md (updated)                                         │
│  ├─ QUICK_START_PLANS.md                                        │
│  ├─ Only active, high-priority plans                            │
│  └─ Clear navigation                                            │
│                                                                  │
│  active/ (3-5)                                                  │
│  ├─ FEATURE_PLANS.md                                            │
│  ├─ QUALITY_IMPROVEMENTS.md                                     │
│  ├─ PERFORMANCE_OPTIMIZATION.md                                  │
│  └─ GAP_ANALYSIS.md                                             │
│                                                                  │
│  reference/ (15-20)                                             │
│  ├─ architecture/                                               │
│  │   ├─ ARCHITECTURE_CORE.md                                    │
│  │   ├─ ARCHITECTURE_PATTERNS.md                                │
│  │   ├─ STORAGE_ARCHITECTURE.md                                 │
│  │   ├─ API_OVERVIEW.md                                         │
│  │   ├─ EPISODE_API.md                                          │
│  │   ├─ PATTERN_API.md                                          │
│  │   ├─ STORAGE_API.md                                          │
│  │   ├─ RETRIEVAL_API.md                                        │
│  │   └─ CURRENT_CAPABILITIES.md                                  │
│  │                                                                │
│  ├─ embeddings/                                                  │
│  │   ├─ MULTI_PROVIDER_GUIDE.md                                  │
│  │   └─ VECTOR_SEARCH_OPTIMIZATION.md                           │
│  │                                                                │
│  ├─ mcp/                                                         │
│  │   ├─ MCP_SERVER_GUIDE.md                                     │
│  │   ├─ MCP_SECURITY_MODEL.md                                   │
│  │   ├─ MCP_VALIDATION_RESULTS.md                               │
│  │   └─ MCP_PERFORMANCE_ANALYSIS.md                             │
│  │                                                                │
│  └─ research/                                                   │
│      ├─ ETS_FORECASTING.md                                      │
│      └─ DBSCAN_ANOMALY_DETECTION.md                             │
│                                                                  │
│  roadmaps/ (3)                                                  │
│  ├─ ROADMAP_ACTIVE.md                                           │
│  ├─ ROADMAP_VISION.md                                           │
│  └─ ROADMAP_HISTORY.md                                          │
│                                                                  │
│  status/ (2-3)                                                  │
│  ├─ PROJECT_STATUS.md                                           │
│  ├─ IMPLEMENTATION_STATUS.md                                    │
│  └─ QUALITY_METRICS.md                                          │
│                                                                  │
│  goap/ (3)                                                       │
│  ├─ GOAP_AGENT_GUIDE.md                                          │
│  ├─ GOAP_EXECUTION_TEMPLATE.md                                  │
│  └─ GOAP_QUALITY_GATES.md                                        │
│                                                                  │
│  archive/ (180-190)                                              │
│  ├─ completed/                                                   │
│  │   ├─ 2025-12/audits/                                         │
│  │   ├─ 2025-12/status/                                         │
│  │   ├─ 2025-12/phase-validation/                               │
│  │   └── 2025-12/code-review/                                   │
│  │                                                                │
│  ├─ goap-plans/                                                  │
│  │   └── completed/2025-12/                                      │
│  │                                                                │
│  ├─ research/                                                    │
│  │   └── phase1-4-integration/                                  │
│  │                                                                │
│  └── legacy/                                                     │
│                                                                  │
│  ✓ All files <500 lines                                         │
│  ✓ No overlapping content                                       │
│  ✓ Clear separation: active vs archive                          │
│  ✓ Logical folder structure                                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Transformation Process Flow

```
START
  ↓
┌─────────────────────┐
│  Phase 1: Archive   │
│  Outdated Content   │
│  (30-45 min)        │
└──────────┬──────────┘
           │
           ├─ Move version-specific roadmaps
           ├─ Move audit reports
           ├─ Move superseded status reports
           ├─ Move completed GOAP executions
           └─ Move research integration materials
           ↓
┌─────────────────────┐
│  Quality Gate:      │
│  - All archived     │
│  - No duplicates    │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│  Phase 2:           │
│  Consolidate         │
│  Overlapping        │
│  (45-60 min)        │
└──────────┬──────────┘
           │
           ├─ Merge status reports → PROJECT_STATUS.md
           ├─ Consolidate roadmaps → 3 files
           ├─ Consolidate embedding docs → 1 guide
           └─ Consolidate GOAP docs → 1 guide
           ↓
┌─────────────────────┐
│  Quality Gate:      │
│  - No overlap       │
│  - All <500 lines   │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│  Phase 3:           │
│  Update & Split     │
│  Large Files        │
│  (60-75 min)        │
└──────────┬──────────┘
           │
           ├─ Split API_DOCUMENTATION.md → 5 files
           ├─ Handle PHASE3_ACTION_PLAN.md
           ├─ Split MEMORY_MCP_VALIDATION_REPORT.md → 3-4 files
           ├─ Split research best practices
           └─ Update top-level analysis files
           ↓
┌─────────────────────┐
│  Quality Gate:      │
│  - All active <500  │
│  - No content loss  │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│  Phase 4: Create    │
│  New Reference      │
│  Documentation      │
│  (30-45 min)        │
└──────────┬──────────┘
           │
           ├─ Create CURRENT_CAPABILITIES.md
           ├─ Create QUICK_START_PLANS.md
           ├─ Update README.md
           └─ Update ARCHIVE_INDEX.md
           ↓
┌─────────────────────┐
│  Quality Gate:      │
│  - All guides <500  │
│  - README accurate   │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│  Phase 5:           │
│  Validate & Test    │
│  (20-30 min)        │
└──────────┬──────────┘
           │
           ├─ Validate links
           ├─ Check content preservation
           ├─ Verify file counts
           └─ Manual review
           ↓
┌─────────────────────┐
│  Quality Gate:      │
│  - All links work   │
│  - No info lost     │
│  - Targets met      │
└──────────┬──────────┘
           ↓
         COMPLETE
```

---

## File Flow Diagram

### Large Files (>500 lines) Handling

```
API_DOCUMENTATION.md (1407 lines)
    ↓ Split
├── API_OVERVIEW.md
├── EPISODE_API.md
├── PATTERN_API.md
├── STORAGE_API.md
└── RETRIEVAL_API.md

MEMORY_MCP_VALIDATION_REPORT.md (1292 lines)
    ↓ Split
├── MCP_SECURITY_MODEL.md
├── MCP_VALIDATION_RESULTS.md
└── MCP_PERFORMANCE_ANALYSIS.md

ets_forecasting_best_practices.md (1316 lines)
    ↓ Extract + Archive
├── ETS_FORECASTING.md (active)
└── Historical sections (archived)

dbscan_anomaly_detection_best_practices.md (1243 lines)
    ↓ Extract + Archive
├── DBSCAN_ANOMALY_DETECTION.md (active)
└── Historical sections (archived)

EMBEDDINGS_REFACTOR_DESIGN.md (994 lines)
    ↓ Consolidate (Phase 2)
→ MULTI_PROVIDER_GUIDE.md (merged content)
```

### Status Reports Consolidation

```
Multiple Status Reports:
├── PROJECT_STATUS_UNIFIED.md
├── IMPLEMENTATION_STATUS.md
├── PROJECT_SUMMARY_2025-12.md
├── V019_STATUS_REPORT.md
└── Various phase validation reports
    ↓ Consolidate
┌──────────────────┐
│ PROJECT_STATUS.md │ (single source of truth)
└──────────────────┘
    ↓ Archive originals
archive/completed/2025-12/status/
```

### GOAP Documentation Consolidation

```
GOAP Documentation:
├── GOAP_AGENT_IMPROVEMENT_PLAN.md
├── GOAP_AGENT_CODEBASE_VERIFICATION.md
├── GOAP_AGENT_ROADMAP.md
├── GOAP_EXECUTION_TEMPLATE.md
└── GOAP_QUALITY_GATES.md
    ↓ Consolidate
┌─────────────────────────┐
│ GOAP_AGENT_GUIDE.md     │ (merged agent docs)
├─────────────────────────┤
│ GOAP_EXECUTION_TEMPLATE │ (keep)
└─────────────────────────┘
│ GOAP_QUALITY_GATES.md   │ (keep)
└─────────────────────────┘
```

---

## Directory Creation Sequence

```
plans/
├── backup/                 (created first)
│   └── Original files
│
├── active/                 (Phase 3)
│   ├── FEATURE_PLANS.md
│   ├── QUALITY_IMPROVEMENTS.md
│   ├── PERFORMANCE_OPTIMIZATION.md
│   └── GAP_ANALYSIS.md
│
├── reference/              (Phase 3)
│   ├── architecture/      (Phase 3.1)
│   ├── embeddings/        (Phase 2.3)
│   ├── mcp/               (Phase 3.3)
│   └── research/          (Phase 3.4)
│
├── roadmaps/               (existing)
│   ├── ROADMAP_ACTIVE.md
│   ├── ROADMAP_VISION.md
│   └── ROADMAP_HISTORY.md
│
├── status/                 (Phase 2.1)
│   ├── PROJECT_STATUS.md
│   └── QUALITY_METRICS.md
│
├── goap/                   (Phase 2.4)
│   ├── GOAP_AGENT_GUIDE.md
│   ├── GOAP_EXECUTION_TEMPLATE.md
│   └── GOAP_QUALITY_GATES.md
│
└── archive/                (Phase 1)
    ├── completed/
    ├── goap-plans/
    ├── research/
    └── legacy/
```

---

## Metrics Tracking

```
Phase 1 (Archive):
  Files moved: ~100
  Active files: 244 → ~144

Phase 2 (Consolidate):
  Files merged: ~15
  Active files: ~144 → ~129

Phase 3 (Split):
  Large files split: ~8
  Files created: ~20
  Active files: ~129 → ~141

Phase 4 (Create):
  New guides: ~4
  Active files: ~141 → ~145

Phase 5 (Archive remaining):
  Archive remaining completed work
  Active files: ~145 → ~90 (target)

Total Reduction: 285 → ~90 (68% reduction)
```

---

**Visual Guide Version**: 1.0
**Last Updated**: 2025-12-31
