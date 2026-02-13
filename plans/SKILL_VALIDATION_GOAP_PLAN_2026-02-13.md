# GOAP Skill Validation Execution Plan

**Date**: 2026-02-13
**Status**: In Progress
**Method**: Goal-Oriented Action Planning (GOAP)
**Orchestrator**: goap-agent skill
**ADR Guidance**: ADR-022 (GOAP Agent System)

---

## 1. World State Analysis

### Current State (2026-02-13)

| Dimension | State | Details |
|-----------|-------|---------|
| **Skills Inventory** | 76 SKILL.md files | Found in .agents/skills/ |
| **Skill Locations** | Root + skill/ subdirectory | Some duplicates exist |
| **Validation Status** | Unknown | Need to verify all skills load correctly |
| **Consolidation Plan** | Ready | skills-consolidation.md outlines strategy |
| **GOAP Capability** | Available | ADR-022 implemented and working |

### Skills Categorization

Based on directory structure:
- **Simple Skills** (single SKILL.md): ~40
- **Rich Skills** (with supporting files): ~25
- **Complex Skills** (with subdirectories): ~10
- **v3 Skills** (future version): ~10

### Known Issues
1. Duplicate skills in both root and skill/ subdirectory
2. Some skill paths have spaces (playwright-cli)
3. Need to validate all skills load correctly

---

## 2. Goal State

| Goal | Measurable Outcome |
|------|-------------------|
| **All Skills Validated** | 100% of unique skills load successfully |
| **Duplicate Skills Identified** | Map root vs skill/ duplicates |
| **Validation Report Created** | Document in plans/ with results |
| **Consolidation Updated** | Update skills-consolidation.md with status |
| **Broken Skills Fixed** | Zero skills fail to load |

---

## 3. Gap Analysis

| Gap | Current | Target | Effort | Priority |
|-----|---------|--------|--------|----------|
| Unknown skill validation status | 0 validated | 76 validated | 1-2h | P0 |
| Duplicate skill paths | Unmapped | Mapped | 0.5h | P1 |
| No validation report | Missing | Created | 0.5h | P1 |
| Consolidation status not updated | Outdated | Current | 0.5h | P2 |

**Total Estimated Effort**: 2.5-3.5 hours

---

## 4. Action Plan

### Phase 1: Discovery & Categorization (P0)

**Tasks**:
1. Extract unique skill names (excluding skill/ subdirectory duplicates)
2. Categorize by complexity (simple, rich, complex)
3. Identify duplicate skills
4. Map skill names to full paths

**Acceptance**:
- All 76 SKILL.md files enumerated
- Duplicates identified
- Categorization complete

**Effort**: 30 minutes

---

### Phase 2: Skill Validation (P0)

**Strategy**: Sequential with batching

**Why Sequential**: Skill loading is fast (~100ms each), but parallel loading might cause resource contention. Sequential allows immediate error handling.

**Tasks**:
1. Load each skill using `skill` tool
2. Capture load result (success/failure)
3. Document any errors
4. Categorize failures

**Batching**:
- Validate 10 skills at a time
- Document results after each batch
- Continue until all validated

**Acceptance**:
- All 76 skills attempted
- Success/failure status recorded
- Errors documented

**Effort**: 1-1.5 hours

---

### Phase 3: Duplicate Analysis (P1)

**Tasks**:
1. Compare root vs skill/ subdirectory skills
2. Identify which is canonical
3. Document differences
4. Recommend consolidation approach

**Acceptance**:
- Duplicate map created
- Canonical source identified
- Recommendations documented

**Effort**: 30 minutes

---

### Phase 4: Documentation (P1)

**Tasks**:
1. Create validation report in plans/
2. Update skills-consolidation.md
3. Document recommendations
4. Create summary for AGENTS.md

**Deliverables**:
- `plans/SKILL_VALIDATION_REPORT_2026-02-13.md`
- Updated `plans/skills-consolidation.md`
- Summary section in AGENTS.md

**Acceptance**:
- All deliverables created
- Results synthesized
- Recommendations clear

**Effort**: 30 minutes

---

## 5. Execution Strategy

### Overall Strategy: Sequential with Quality Gates

```
Phase 1: Discovery & Categorization
  ↓ Quality Gate: All skills enumerated
Phase 2: Skill Validation (Sequential)
  ↓ Quality Gate: All skills validated
Phase 3: Duplicate Analysis
  ↓ Quality Gate: Duplicates mapped
Phase 4: Documentation
  ↓ Quality Gate: Reports created
```

### Rationale for Sequential Approach

1. **Fast Execution**: Skill loading is quick (~100ms each)
2. **Immediate Feedback**: Catch errors as they occur
3. **Simpler Debugging**: Easier to trace issues
4. **Resource Efficient**: No contention on skill tool

### Quality Gates

Between each phase:
- [ ] Phase 1: All 76 skills enumerated and categorized
- [ ] Phase 2: All skills validated with status recorded
- [ ] Phase 3: Duplicate map complete with recommendations
- [ ] Phase 4: All documentation created

---

## 6. Success Metrics

| Metric | Target | Measurement |
|--------|---------|-------------|
| Skills Validated | 76/76 (100%) | Validation report |
| Load Success Rate | ≥95% | Success count / total |
| Broken Skills Fixed | 0 remaining | Post-fix validation |
| Documentation Complete | 3 files | File count |
| Consolidation Updated | Yes | skills-consolidation.md |

---

## 7. Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Skill load failures block validation | High | Low | Fix on-the-fly, document workaround |
| Too many duplicates to reconcile | Medium | Medium | Prioritize by usage frequency |
| Documentation incomplete | Low | Low | Template-driven approach |
| Time estimate exceeded | Medium | Low | Phase 4 can be deferred |

---

## 8. Progress Tracking

### Phase 1: Discovery & Categorization - IN PROGRESS
- [x] Extract all SKILL.md files
- [ ] Categorize by complexity
- [ ] Identify duplicates
- [ ] Map skill paths

### Phase 2: Skill Validation - PENDING
- [ ] Validate skills 1-10
- [ ] Validate skills 11-20
- [ ] Validate skills 21-30
- [ ] Validate skills 31-40
- [ ] Validate skills 41-50
- [ ] Validate skills 51-60
- [ ] Validate skills 61-70
- [ ] Validate skills 71-76

### Phase 3: Duplicate Analysis - PENDING
- [ ] Compare root vs skill/
- [ ] Identify canonical sources
- [ ] Document differences

### Phase 4: Documentation - PENDING
- [ ] Create validation report
- [ ] Update skills-consolidation.md
- [ ] Update AGENTS.md

---

## 9. References

- **ADR-022**: GOAP Agent System for Multi-Agent Coordination
- **skills-consolidation.md**: Consolidation strategy and execution plan
- **AGENTS.md**: Agent coding guidelines and skill reference
- **goap-agent skill**: GOAP methodology and patterns

---

*Generated by GOAP Agent System (ADR-022) on 2026-02-13*
