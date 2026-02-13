# Self-Learning Framework

Enable **self-learning and continuous improvement** from validation results.

## Learning Cycle

```
Validate → Identify Issues → Analyze Root Cause → Update Documentation → Re-validate
    ↑                                                                            ↓
    └────────────────────────── Feedback Loop ──────────────────────────────────┘
```

## Learning Triggers

### Trigger 1: Repeated Violations
- Same violation appears 3+ times
- May indicate plan is outdated or unrealistic
- Action: Review and potentially update plan

### Trigger 2: False Positives
- Validator reports violations for correct code
- Indicates validation logic needs refinement
- Action: Update agent/skill validation patterns

### Trigger 3: New Patterns Emerge
- Implementation uses patterns not documented
- Patterns appear beneficial
- Action: Document new patterns in plans

### Trigger 4: Plan-Reality Mismatch
- Consistent drift between plan and implementation
- Implementation is actually better
- Action: Update plan to reflect reality

## Self-Update Protocol

### Phase 1: Detect Learning Opportunity
```bash
# After validation, analyze:
# - Number of violations by type
# - Pattern frequency
# - False positive rate
# - User feedback on findings
```

### Phase 2: Root Cause Analysis
```bash
# Determine root cause:
# - Is the plan outdated? → Update plan
# - Is validation incomplete? → Update agent/skill
# - Is implementation wrong? → Report to user
# - Is this a new valid pattern? → Document pattern
```

### Phase 3: Update Documentation

**Option A: Update Plans**
- plans/00-overview.md: If project scope changed
- plans/01-understand.md: If requirements changed
- plans/02-plan.md: If architecture evolved
- plans/03-execute.md: If implementation patterns changed

**Option B: Update Agent/Skill**
- .claude/agents/architecture-validator.md: Update validation logic
- .claude/skills/architecture-validation/: Update patterns

### Phase 4: Verification
```bash
# After updates:
# 1. Re-run validation
# 2. Confirm issue resolved
# 3. Check for new issues
# 4. Document learning
```

## Learning Examples

### Example 1: Outdated Dependency Rule
```
Violation: "Core depends on storage implementation"
Frequency: 10 occurrences
Analysis: Dependency is intentional and beneficial
Learning: Rule too strict for current architecture
Action:
  1. Edit plans/02-plan.md: Update dependency rules
  2. Document rationale: "Direct dependency acceptable"
  3. Edit architecture-validator.md: Remove check
  4. Document in plans/06-feedback-loop.md
```

### Example 2: Missing Validation Pattern
```
Issue: New async pattern not validated
Frequency: 5 instances found manually
Analysis: Validation extraction patterns incomplete
Learning: Need to check for async patterns
Action:
  1. Edit architecture-validator.md: Add async checks
  2. Edit SKILL.md: Document async validation
  3. Re-run validation: Confirm patterns detected
```

### Example 3: New Architecture Pattern
```
Discovery: Code uses Circuit Breaker pattern
Status: Not documented in plans
Analysis: Pattern is beneficial, should be standard
Learning: Update plans to include pattern
Action:
  1. Edit plans/02-plan.md: Add Circuit Breaker section
  2. Edit plans/03-execute.md: Document implementation
  3. Edit architecture-validator.md: Validate circuit breakers
```

## Learning Metrics

- **Learning Rate**: Updates per week
- **False Positive Reduction**: % decrease over time
- **Coverage Improvement**: New patterns detected
- **Plan Accuracy**: Plan-reality alignment
- **Validation Quality**: User satisfaction

## Learning History Format

In `plans/06-feedback-loop.md`:
```markdown
## Architecture Validator Learnings

### [Date]: Dependency Rule Refinement
**Issue**: Core-Storage dependency flagged incorrectly
**Analysis**: Rule too strict, dependency is intentional
**Action**: Updated plans/02-plan.md lines 45-50
**Result**: False positives reduced from 10 to 0
**Status**: Verified
```

## Continuous Improvement

**Weekly Review**:
- Review validation results
- Identify improvement opportunities
- Update documentation
- Refine validation logic

**Monthly Retrospective**:
- Assess learning metrics
- Major architecture changes
- Plan substantial updates
- Validate learning effectiveness

**Quarterly Audit**:
- Comprehensive review of plans
- Major agent/skill updates
- Architecture evolution assessment
- Long-term pattern analysis
