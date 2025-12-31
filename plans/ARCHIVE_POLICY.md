# Archive Policy and Retention Guidelines

**Purpose**: Define document lifecycle and retention rules for the plans/ folder
**Effective Date**: 2025-12-31
**Version**: 1.0

---

## Document Lifecycle

### 1. Active Planning Phase
**Duration**: During active development
**Location**: Root directories (ARCHITECTURE/, CONFIGURATION/, GOAP/, ROADMAPS/, STATUS/, research/)
**Status**: Current and actively maintained

**Characteristics**:
- Document is being actively updated
- Describes ongoing or planned work
- Referenced by team members
- Single source of truth for topic

### 2. Implementation Phase
**Duration**: During feature implementation
**Action**: Move implementation details to `../docs/` or `../agent_docs/`

**When to Move**:
- Design is finalized and implemented in code
- Implementation guides are complete
- Planning document becomes historical record

### 3. Completion Phase
**Duration**: Immediately after feature completion
**Action**: Archive to `archive/completed/` or appropriate subdirectory

**What to Archive**:
- Completion reports and summaries
- Superseded status reports
- Completed execution plans
- Phase validation reports
- Research integration materials

### 4. Retention Phase
**Duration**: Varies by category (see Retention Categories below)
**Action**: Keep in archive for defined period

### 5. Preservation or Deletion
**Action**: Based on retention category (Keep Forever, Archive 1 Year, Delete 6 Months)

---

## Retention Categories

### Category 1: Keep Forever (1% of documents)

**Documents with permanent value** - Never delete

**Includes**:
- Architecture decision records
- Key design documents explaining system rationale
- Research findings that inform future decisions
- Security policies and procedures
- Major architectural diagrams
- Original research on core algorithms

**Storage**: `archive/keep-forever/` (if not already in appropriate archive)

**Examples**:
- `ARCHITECTURE/ARCHITECTURE_DECISION_RECORDS.md` (when superseded)
- Research on core algorithms (PREMem, GENESIS, etc.)
- Security policies

### Category 2: Archive 1 Year (20% of documents)

**Documents with recent reference value** - Keep for 1 year, then review

**Includes**:
- Completed execution plans
- Phase completion reports
- Version-specific roadmaps and status reports
- Performance benchmarks (latest 4 quarters)
- Recent research findings
- Implementation summaries

**Storage**: `archive/` with appropriate subdirectory

**Retention Schedule**:
- Created: Keep for 12 months
- Review: At 12 months, evaluate if still needed
- Action: Move to "Keep Forever" or delete

**Examples**:
- `STATUS/V011_STATUS_REPORT.md` (after v0.2.0 release)
- `GOAP/PHASE1_EXECUTION_PLAN.md` (after completion)
- Phase completion reports (after 1 year)

### Category 3: Delete 6 Months (79% of documents)

**Documents with temporary value** - Delete after 6 months

**Includes**:
- One-time audit reports
- Consolidation reports (meta-planning)
- Temporary analysis documents
- Draft planning documents
- Superseded status reports
- Duplicate execution summaries
- Version-specific preparation materials

**Storage**: `archive/temporary/` during retention period

**Retention Schedule**:
- Created: Keep for 6 months
- Review: At 6 months, confirm no longer needed
- Action: Delete

**Examples**:
- Consolidation reports about consolidating (e.g., `2025-12-consolidation/`)
- One-time audit reports
- Temporary analysis
- Superseded execution summaries

---

## Archive Organization

### Primary Categories

**archive/completed/**: Implementation summaries and completion reports
**archive/goap-plans/**: Historical GOAP execution plans (by date)
**archive/research/**: Completed research findings
**archive/legacy/**: Historical planning framework (for reference)
**archive/v{version}/**: Version-specific documentation

### Retention Markers

**Documents in archive should include**:
- Creation date in header
- Original location (if moved)
- Retention category (Keep Forever / Archive 1 Year / Delete 6 Months)
- Superseded by (link to newer document, if applicable)

**Example Header**:
```markdown
# Historical Document

**Status**: ARCHIVED
**Archived**: 2025-12-31
**Original Location**: STATUS/PHASE1_VALIDATION_REPORT_2025-12-25.md
**Retention Category**: Archive 1 Year
**Superseded By**: STATUS/PROJECT_STATUS_UNIFIED.md
**Purpose**: Historical reference for Phase 1 completion
```

---

## Automated Maintenance

### Scripts

**Archive Old Plans**: `scripts/archive-old-plans.sh`
- Automatically move completed plans to archive
- Add retention markers
- Update archive index

**Expire Old Archive**: `scripts/expire-old-archive.sh`
- Delete documents past retention period
- Generate deletion report
- Update archive index

**Archive Index Update**: `scripts/update-archive-index.sh`
- Generate archive index from filesystem
- Include file age, size, category
- Auto-update on archive changes

### Triggers

**Automatic** (via pre-commit hook):
- When execution plan marked complete → Move to archive
- When status report superseded → Archive old version
- When version released → Move version-specific plans to archive

**Manual** (as needed):
- Weekly: Review and archive completed work
- Monthly: Run expiration script
- Quarterly: Review retention policy effectiveness

---

## Quality Gates

### Before Archiving
- [ ] All important information captured in newer documents
- [ ] Links to archived documents updated where needed
- [ ] Archive index updated
- [ ] Retention category assigned

### Before Deletion
- [ ] Document is past retention period
- [ ] No active references in other documents
- [ ] Not marked as "Keep Forever"
- [ ] Deletion logged for audit trail

---

## Exceptions

### Override Retention Period

**To extend retention** (move from 6 months → 1 year → keep forever):
1. Add `[KEEP FOREVER]` or `[EXTEND RETENTION]` tag to document header
2. Explain reason in comment
3. Update retention category

**Example**:
```markdown
# [EXTEND RETENTION] Phase 1 Completion Report

**Retention Category**: Keep Forever (overridden from Delete 6 Months)
**Reason**: Reference implementation for all future phases
```

### Emergency Restoration

**If archived document needed**:
1. Restore from git history (archive is versioned)
2. Move to appropriate location
3. Update links as needed

---

## Compliance and Audit

### Audit Trail
- All archival actions are logged
- Deletion reports retained for audit
- Git history provides complete record

### Access Control
- Archive is read-only for most contributors
- Only maintainers can delete expired documents
- All archive operations require review

---

## Metrics and Monitoring

### Track Monthly
- Archive file count (alert at 150 files)
- Documents deleted (past retention)
- Documents restored (from archive)
- Archive maintenance time (alert at >10 hours/week)

### Quarterly Review
- Retention policy effectiveness
- Categories needing adjustment
- Automation opportunities
- Navigation improvements

---

## Policy Updates

**Review Schedule**: Every 6 months
**Update Process**: Team discussion → PR → Merge
**Version Control**: Maintain policy version in header

---

## Contact

**Questions about this policy**: Ask project maintainers
**Archive access issues**: Contact team lead
**Policy change requests**: Open issue for discussion

---

**Version**: 1.0
**Created**: 2025-12-31
**Next Review**: 2025-06-30
**Owner**: Project Team
