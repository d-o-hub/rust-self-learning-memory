# GOAP Agent Execution Template

- **Version**: 1.0
- **Last Updated**: 2026-03-06
- **Purpose**: Template for creating new GOAP execution plans

---

## Template

```markdown
# GOAP Execution Plan: [Title]

- **Date**: YYYY-MM-DD
- **Goal**: [One-line objective]
- **Strategy**: [Sequential | Parallel | Hybrid]
- **Related ADRs**: [ADR-XXX, ADR-YYY]

## Objective

[Clear statement of what this plan accomplishes]

## Current State

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| [Metric 1] | [Value] | [Target] | [Status] |
| [Metric 2] | [Value] | [Target] | [Status] |

## Phase 1: ANALYZE

### Baseline Assessment

[Describe current state and gaps]

### Gap Analysis

| Gap | Priority | Effort | Risk |
|-----|----------|--------|------|
| [Gap 1] | P0/P1/P2 | [Hours] | [Low/Med/High] |

## Phase 2: DECOMPOSE

### Work Packages

| ID | Task | Owner | Depends On | Exit Criteria |
|----|------|-------|------------|---------------|
| WP1 | [Task] | [Skill/Agent] | — | [Criteria] |

## Phase 3: STRATEGIZE

### Execution Strategy

[Describe approach: sequential, parallel, hybrid]

### Dependency Graph

```
[ASCII diagram of task dependencies]
```

## Phase 4: COORDINATE

### Resource Allocation

[Assign work packages to agents/skills]

## Phase 5: EXECUTE

### Execution Log

| Date | Action | Result | Notes |
|------|--------|--------|-------|
| [Date] | [Action] | [Pass/Fail] | [Notes] |

## Phase 6: SYNTHESIZE

### Outcomes

[Summarize results]

### Learning Delta

[What was learned during execution]

## Validation

- [ ] All exit criteria met
- [ ] CI checks passing
- [ ] State files updated
- [ ] Learning captured

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| [Risk] | [Low/Med/High] | [Low/Med/High] | [Action] |

## Rollback Plan

[Steps to revert changes if execution fails]

## Success Criteria

- [ ] [Criterion 1]
- [ ] [Criterion 2]
```

---

## Usage

1. Copy template to `plans/GOAP_EXECUTION_PLAN_[topic]_[date].md`
2. Fill in all sections
3. Run `./scripts/quality-gates.sh` to validate
4. Execute phases in order
5. Update state files after completion

## Section Guidelines

### Objective

- Single sentence
- Measurable outcome
- Time-bound if applicable

### Current State

- Use metrics from `GOAP_CODEBASE_ANALYSIS_*.md`
- Include verification commands
- Mark status with emoji: ✅ 🟡 🔴

### Risks & Mitigations

- At least 2 risks documented
- Each risk has mitigation
- Include rollback trigger

### Rollback Plan

- Specific commands to revert
- Data recovery steps if needed
- Communication plan for team

## Quality Checks

Before marking plan complete:

```bash
# Verify template sections
grep -q "^## Objective" plans/GOAP_EXECUTION_PLAN_*.md
grep -q "^## Validation" plans/GOAP_EXECUTION_PLAN_*.md
grep -q "^## Risks" plans/GOAP_EXECUTION_PLAN_*.md
grep -q "^## Rollback" plans/GOAP_EXECUTION_PLAN_*.md

# Verify file size
wc -l plans/GOAP_EXECUTION_PLAN_*.md  # Should be <500 lines
```