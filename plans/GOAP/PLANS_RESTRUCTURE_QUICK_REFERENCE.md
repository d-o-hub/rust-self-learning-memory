# Plans Directory Restructure - Quick Reference

**Execution Plan**: `GOAP/PLANS_DIRECTORY_RESTRUCTURE_EXECUTION_PLAN.md`
**Checklist**: `GOAP/PLANS_RESTRUCTURE_CHECKLIST.md`
**Helper Script**: `scripts/plans_restructure_helper.sh`

---

## Current State (2025-12-31)

```
Total Files: 285 markdown files

Distribution:
├── Top-level:        39 files (13.7%)
├── GOAP/:            36 files (12.6%)
├── ARCHITECTURE/:     5 files (1.8%)
├── CONFIGURATION/:    9 files (3.2%)
├── ROADMAPS/:         4 files (1.4%)
├── STATUS/:          11 files (3.9%)
├── research/:        27 files (9.5%)
├── archive/:        154 files (54.0%)
└── benchmark_results/: 1 file (0.4%)

Active files: 244 (excluding archive)
Target: <100 active files (59% reduction)
```

---

## Target Structure

```
plans/
├── README.md (updated)
│
├── active/                          # Active, in-progress plans
│   ├── FEATURE_PLANS.md
│   ├── QUALITY_IMPROVEMENTS.md
│   ├── PERFORMANCE_OPTIMIZATION.md
│   └── GAP_ANALYSIS.md
│
├── reference/                       # Current capability documentation
│   ├── architecture/
│   │   ├── ARCHITECTURE_CORE.md
│   │   ├── ARCHITECTURE_PATTERNS.md
│   │   ├── STORAGE_ARCHITECTURE.md
│   │   ├── API_OVERVIEW.md
│   │   ├── EPISODE_API.md
│   │   ├── PATTERN_API.md
│   │   ├── STORAGE_API.md
│   │   └── RETRIEVAL_API.md
│   ├── embeddings/
│   │   ├── MULTI_PROVIDER_GUIDE.md
│   │   └── VECTOR_SEARCH_OPTIMIZATION.md
│   ├── mcp/
│   │   ├── MCP_SERVER_GUIDE.md
│   │   ├── MCP_SECURITY_MODEL.md
│   │   ├── MCP_VALIDATION_RESULTS.md
│   │   └── MCP_PERFORMANCE_ANALYSIS.md
│   └── research/
│       ├── ETS_FORECASTING.md
│       └── DBSCAN_ANOMALY_DETECTION.md
│
├── roadmaps/
│   ├── ROADMAP_ACTIVE.md
│   ├── ROADMAP_VISION.md
│   └── ROADMAP_HISTORY.md
│
├── status/
│   ├── PROJECT_STATUS.md
│   ├── IMPLEMENTATION_STATUS.md
│   └── QUALITY_METRICS.md
│
├── goap/
│   ├── GOAP_AGENT_GUIDE.md
│   ├── GOAP_EXECUTION_TEMPLATE.md
│   └── GOAP_QUALITY_GATES.md
│
└── archive/                         # Historical documentation
    ├── completed/
    ├── research/
    └── legacy/
```

---

## Five-Phase Execution

### Phase 1: Archive Outdated Content (30-45 min)

**Goal**: Move clearly outdated files to archive

| Action | Time | Files to Move |
|--------|------|---------------|
| 1.1 Archive version-specific roadmaps | 10 min | `archive/v0.1.7-roadmap/*` |
| 1.2 Archive one-time audit reports | 5 min | `DOCUMENTATION_AUDIT_*.md`, etc. |
| 1.3 Archive superseded status reports | 10 min | `V019_STATUS_REPORT.md`, etc. |
| 1.4 Archive completed GOAP executions | 10 min | `*_EXECUTION_PLAN.md`, etc. |
| 1.5 Archive research integration materials | 10 min | `research/PHASE*_INTEGRATION_PLAN.md` |

**Quality Gate**: All version-specific docs in archive, no duplicate status reports

---

### Phase 2: Consolidate Overlapping Content (45-60 min)

**Goal**: Merge duplicate and overlapping information

| Action | Time | Merge From | To |
|--------|------|------------|-----|
| 2.1 Consolidate status reports | 15 min | `PROJECT_STATUS_UNIFIED.md` + others | `STATUS/PROJECT_STATUS.md` |
| 2.2 Consolidate roadmaps | 15 min | `ROADMAP_V030_VISION.md` + others | `ROADMAPS/ROADMAP_VISION.md` |
| 2.3 Consolidate embedding docs | 20 min | `EMBEDDINGS_REFACTOR_DESIGN.md` + others | `reference/embeddings/MULTI_PROVIDER_GUIDE.md` |
| 2.4 Consolidate GOAP docs | 15 min | `GOAP_AGENT_IMPROVEMENT_PLAN.md` + others | `goap/GOAP_AGENT_GUIDE.md` |

**Quality Gate**: No overlapping content, all files <500 lines

---

### Phase 3: Update and Split Large Files (60-75 min)

**Goal**: Enforce 500-line limit and update outdated content

| File (Current Lines) | Action | Target |
|---------------------|--------|--------|
| `API_DOCUMENTATION.md` (1407) | Split into 5 files | All <500 lines |
| `PHASE3_ACTION_PLAN.md` (1365) | Split or archive | All <500 lines |
| `MEMORY_MCP_VALIDATION_REPORT.md` (1292) | Split into 3-4 files | All <500 lines |
| `ets_forecasting_best_practices.md` (1316) | Split | <500 lines |
| `dbscan_anomaly_detection_best_practices.md` (1243) | Split | <500 lines |
| `PHASE2_INTEGRATION_PLAN.md` (1161) | Split or archive | All <500 lines |
| `PHASE1_INTEGRATION_PLAN.md` (1011) | Split or archive | All <500 lines |
| `EMBEDDINGS_REFACTOR_DESIGN.md` (994) | Split (if not consolidated) | <500 lines |

**Quality Gate**: All active files <500 lines, no content loss

---

### Phase 4: Create New Reference Documentation (30-45 min)

**Goal**: Create consolidated documentation for current capabilities

| Action | Time | Deliverable |
|--------|------|-------------|
| 4.1 Create current capability guides | 15 min | `reference/architecture/CURRENT_CAPABILITIES.md` (<500 lines) |
| 4.2 Create quick start guides | 10 min | `QUICK_START_PLANS.md` (<300 lines) |
| 4.3 Update README.md | 10 min | Updated structure and navigation |
| 4.4 Update archive index | 10 min | `archive/ARCHIVE_INDEX.md` updated |

**Quality Gate**: All new guides created, all <500 lines

---

### Phase 5: Validation and Testing (20-30 min)

**Goal**: Validate new structure and ensure nothing is lost

| Action | Time | Deliverable |
|--------|------|-------------|
| 5.1 Link validation | 10 min | All internal links working |
| 5.2 Content preservation check | 10 min | Critical info preserved checklist |
| 5.3 File count verification | 5 min | File count targets met |
| 5.4 Documentation review | 5 min | Structure validated |

**Quality Gate**: All links working, no info lost, targets met

---

## Helper Script Commands

```bash
# Create backup before starting
./scripts/plans_restructure_helper.sh backup

# Count files in each directory
./scripts/plans_restructure_helper.sh count

# Find files exceeding 500 lines
./scripts/plans_restructure_helper.sh find-large

# Check for broken links
./scripts/plans_restructure_helper.sh check-links

# Show restructuring progress
./scripts/plans_restructure_helper.sh progress

# Validate new structure
./scripts/plans_restructure_helper.sh validate
```

---

## Key Success Criteria

### Quantitative
- [ ] Active file count: <100 files (from 285, 72% reduction)
- [ ] No file exceeds 500 lines
- [ ] Archive contains all historical content
- [ ] No broken links

### Qualitative
- [ ] Clear separation between active and archived content
- [ ] Navigation is intuitive
- [ ] Critical information preserved
- [ ] Structure aligns with v0.1.9 capabilities

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Accidental deletion of critical info | Create backup before starting |
| Broken links after restructuring | Link validation in Phase 5 |
| Loss of historical context | Archive all content, never delete |
| Time overrun | Prioritize Phase 1-3 if time-constrained |

---

## Estimated Timeline

```
Phase 1: Archive Outdated           30-45 min
Phase 2: Consolidate Overlapping     45-60 min
Phase 3: Update and Split Large     60-75 min
Phase 4: Create New Reference       30-45 min
Phase 5: Validation and Testing    20-30 min
────────────────────────────────────────
Total:                             3-4 hours
```

---

## Post-Execution

1. **Backup**: `cp -r plans/ plans.backup/`
2. **Commit**: `refactor(plans): restructure directory for better organization`
3. **CI Check**: Verify no CI jobs reference moved files
4. **Team**: Announce new structure
5. **Maintain**: Quarterly cleanup reviews

---

## v0.1.9 Current Capabilities Reference

### Core Features
- ✅ Episode Lifecycle Management
- ✅ Pattern Extraction
- ✅ Heuristic Learning
- ✅ Dual Storage (Turso + redb)
- ✅ CLI Interface (24 commands)
- ✅ Circuit Breaker
- ✅ Connection Pooling

### Embeddings (Multi-Provider)
- ✅ OpenAI API embeddings
- ✅ Cohere API embeddings
- ✅ Ollama local embeddings
- ✅ Local CPU-based embeddings
- ✅ Custom providers
- ✅ Configuration caching
- ✅ Vector search optimization

### MCP Server
- ✅ 6-layer security sandbox
- ✅ Tool implementation
- ✅ Query caching (v0.1.12)

### Performance
- ✅ 10-100x faster than baseline
- ✅ 92.5% test coverage
- ✅ 99.3% test pass rate (424/427)
- ✅ Zero clippy warnings

---

**Last Updated**: 2025-12-31
**Version**: 1.0
