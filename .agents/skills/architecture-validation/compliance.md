# Report Format

```markdown
# Architecture Validation Report
**Date**: [Date]
**Project**: [Name]
**Plans**: [List of plan files]

## Executive Summary
- Overall Compliance: X%
- Critical Issues: N
- Warnings: M
- Info: K

## Plans Analyzed
1. plans/00-overview.md - Project overview
2. plans/01-understand.md - Requirements
...

## Architectural Elements Discovered
[Dynamic list based on plan extraction]

### Components
- Component A: Compliant
- Component B: Partial
- Component C: Missing

### Dependencies
- Rule 1: Compliant
- Rule 2: Violated

### Performance
- Target 1 (<100ms): Untested
- Target 2 (>1000 ops/s): Met

### Security
- Requirement 1: Implemented
- Requirement 2: Partial

## Detailed Findings

### Fully Compliant
[List compliant aspects]

### Partial Compliance
[List partial implementations with details]

### Non-Compliant
[List violations with:
- Plan reference (file:line)
- Expected vs Actual
- Impact assessment
- Priority
- Recommended action]

## Architecture Drift
[List intentional or unintentional deviations]

## Recommendations

### High Priority
[Critical items]

### Medium Priority
[Important items]

### Low Priority
[Nice to have items]

## Next Steps
[Actionable next steps]
```

## Metrics

Track validation quality:
- **Plan Coverage**: % of plan files analyzed
- **Element Coverage**: % of architectural elements checked
- **Validation Depth**: How thoroughly each element validated
- **Finding Quality**: Specificity and actionability
- **Report Completeness**: All sections filled out

## Example Usage

### Scenario 1: Initial Validation
```bash
# After reading all plan files
# Extract 50+ architectural elements
# Validate against codebase
# Generate report: 75% compliance, 5 critical issues
```

### Scenario 2: Post-Refactoring
```bash
# Re-validate after major changes
# Check for new violations
# Verify planned improvements implemented
# Generate diff report
```

### Scenario 3: Architecture Review Prep
```bash
# Comprehensive validation
# Document all drift
# Prepare justifications
# Generate presentation-ready report
```
