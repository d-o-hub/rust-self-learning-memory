# Report Templates

Output format templates for different types of codebase consolidation reports.

## Template 1: Executive Summary Report

Use this for high-level overview and prioritized recommendations.

```markdown
# Codebase Analysis Report: [Project Name]
**Generated**: [Date]
**Analyzer**: Claude Code Consolidation Skill
**Lines of Code**: [Total LOC]
**Language**: [Primary Language]

## Executive Summary

**Health Score**: [X/100] 🟢/🟡/🔴

### Quick Stats
- **Strengths**: [Top 3 positive findings]
- **Critical Issues**: [Count] requiring immediate attention
- **Refactoring Opportunities**: [Count] identified
- **Technical Debt**: [Estimate in days/weeks]
- **Test Coverage**: [X%]
- **Security Score**: [X/10]

## Key Metrics Dashboard

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Total LOC | X | - | - |
| Test Coverage | X% | >90% | 🟡 |
| Duplicate Code | X% | <5% | 🔴 |
| Avg Complexity | X | <10 | 🟢 |
| Large Files (>500 LOC) | X | 0 | 🟡 |
| Security Vulnerabilities | X | 0 | 🟢 |
| Outdated Dependencies | X | 0 | 🟡 |
| Documentation Coverage | X% | 100% | 🟡 |

## Priority Recommendations

### 🔴 Critical (This Week)
1. **[Issue Title]** - [Brief description]
   - **Effort**: [X days]
   - **Impact**: High ([why it matters])
   - **Files**: [list of affected files]

### 🟡 High Priority (This Sprint)
2. **[Issue Title]** - [Brief description]
   - **Effort**: [X days]
   - **Impact**: Medium ([why it matters])

### 🟢 Medium Priority (This Quarter)
3. **[Issue Title]** - [Brief description]

## ROI Analysis

**Total Estimated Effort**: [X days]
**Expected Benefits**:
- Reduced maintenance time: [X%]
- Fewer bugs: [X% reduction]
- Faster onboarding: [X% time reduction]
- Improved performance: [metrics]

## Next Steps

1. [ ] Review and approve recommendations
2. [ ] Create tickets for critical issues
3. [ ] Schedule refactoring sprints
4. [ ] Set up monitoring for key metrics
```

## Template 2: Architecture Documentation

Use this for documenting system structure and design decisions.

```markdown
# Architecture Overview: [Project Name]

**Last Updated**: [Date]
**Version**: [Version]
**Author**: [Name/Team]

## System Architecture

### High-Level Component Diagram

```
[ASCII art diagram showing major components and their relationships]
```

### Component Catalog

#### [Component Name]
- **Purpose**: [One-line description]
- **Location**: `[path/to/component]`
- **Language/Tech**: [Rust, TypeScript, etc.]
- **Dependencies**: [list]
- **Exposes**: [API/interfaces]
- **Key Responsibilities**:
  - [Responsibility 1]
  - [Responsibility 2]

## Design Patterns

### [Pattern Name]
**Where Used**: [Components/files]
**Why**: [Rationale]
**Example**:
```[language]
[Code example]
```

## Data Flow

### [Flow Name] (e.g., "Episode Creation")

```
[Step 1] → [Step 2] → [Step 3] → [Step 4]
```

**Detailed Steps**:
1. **[Step 1]**: [Description]
   - Input: [data]
   - Output: [data]
   - Error Handling: [approach]

2. **[Step 2]**: [Description]
   ...

## Integration Points

### External Systems
- **[System Name]**: [How it integrates, protocols used]

### Internal APIs
- **[API Name]**: [Purpose and usage]

## Architecture Decision Records

### ADR-001: [Decision Title]
**Date**: [Date]
**Status**: [Accepted/Deprecated/Superseded]

**Context**: [What was the situation]

**Decision**: [What we decided]

**Consequences**: [Trade-offs and implications]

**Alternatives Considered**: [What else was considered and why rejected]

## Security Architecture

### Defense Layers
1. **[Layer Name]**: [Protection provided]
2. **[Layer Name]**: [Protection provided]

### Trust Boundaries
- [Boundary description and controls]

## Performance Characteristics

### Throughput
- [Metric]: [Value] under [conditions]

### Latency
- [Operation]: [P50/P95/P99] latency

### Scalability
- [How system scales, bottlenecks, limits]
```

## Template 3: Refactoring Roadmap

Use this for planning and tracking refactoring work.

```markdown
# Refactoring Roadmap: [Project Name]

**Timeline**: [Q1 2025 or Sprint 23-26]
**Total Effort**: [X developer days]
**Team**: [Team name/members]

## Overview

### Goals
1. [Primary goal]
2. [Secondary goal]
3. [Tertiary goal]

### Success Criteria
- [ ] [Measurable criterion 1]
- [ ] [Measurable criterion 2]
- [ ] [Measurable criterion 3]

## Phase 1: [Name] (Week 1-2)

### Objectives
- [Objective 1]
- [Objective 2]

### Tasks

#### Task 1.1: [Task Name]
**Priority**: 🔴 Critical
**Effort**: [X days]
**Owner**: [Name]
**Impact**: [High/Medium/Low] - [Description]

**Current State**:
- [Problem description]
- [Code locations]
- [Impact on system]

**Target State**:
- [Desired outcome]
- [New structure]
- [Benefits]

**Action Items**:
- [ ] [Specific action 1]
- [ ] [Specific action 2]
- [ ] [Test coverage added]
- [ ] [Documentation updated]

**Files Affected**:
- `[file1]` ([changes])
- `[file2]` ([changes])

**Dependencies**:
- Depends on: [Task X]
- Blocks: [Task Y]

**Risks**:
- [Risk 1]: [Mitigation]
- [Risk 2]: [Mitigation]

### Phase Completion Criteria
- [ ] All critical tasks complete
- [ ] Tests passing
- [ ] Code reviewed
- [ ] Deployed to staging

## Phase 2: [Name] (Week 3-4)

[Similar structure to Phase 1]

## Phase 3: [Name] (Week 5-6)

[Similar structure to Phase 1]

## Rollback Plan

If issues arise:
1. [Rollback step 1]
2. [Rollback step 2]
3. [Communication plan]

## Progress Tracking

| Week | Planned | Completed | % Complete | Notes |
|------|---------|-----------|------------|-------|
| 1 | [tasks] | [tasks] | X% | [notes] |
| 2 | [tasks] | [tasks] | X% | [notes] |
| ... | ... | ... | ... | ... |

## Metrics to Monitor

Track these metrics before, during, and after refactoring:

| Metric | Baseline | Target | Current |
|--------|----------|--------|---------|
| Test Coverage | X% | Y% | Z% |
| Build Time | X min | Y min | Z min |
| Cyclomatic Complexity | X | Y | Z |
| Duplicate Code % | X% | Y% | Z% |
| LOC | X | Y | Z |
```

## Template 4: Consolidation Opportunities Report

Use this for identifying and prioritizing duplicate code.

```markdown
# Code Consolidation Opportunities: [Project Name]

**Analysis Date**: [Date]
**Scope**: [Entire codebase / Specific modules]
**Total Opportunities**: [X]

## Summary

**Potential LOC Reduction**: [X lines] ([Y%] of codebase)
**Estimated Effort**: [X developer days]
**Expected ROI**: [High/Medium/Low]

### By Priority
- 🔴 Critical: [X opportunities] ([Y LOC], [Z days])
- 🟡 High: [X opportunities] ([Y LOC], [Z days])
- 🟢 Medium: [X opportunities] ([Y LOC], [Z days])
- ⚪ Low: [X opportunities] ([Y LOC], [Z days])

## Detailed Findings

### 1. [Duplicate Pattern Name]

**Severity**: 🔴 Critical
**Category**: [Security/Business Logic/Utilities]
**Duplicate Instances**: [X]
**LOC per Instance**: ~[Y]
**Total Wasted LOC**: ~[Z]
**Effort to Fix**: [X days]

#### Locations

**Instance 1**: `[file:line]`
```[language]
[Code snippet]
```

**Instance 2**: `[file:line]`
```[language]
[Code snippet]
```

**Instance 3**: `[file:line]`
[etc.]

#### Analysis

**Similarities**:
- [What's the same]

**Differences**:
- [What varies between instances]

#### Recommendation

**Proposed Refactoring**:
```[language]
[Consolidated code example]
```

**Refactoring Pattern**: [Extract Function/Strategy Pattern/etc.]

**Benefits**:
- ✅ [Benefit 1]
- ✅ [Benefit 2]
- ✅ [Benefit 3]

**Migration Path**:
1. [Step 1]
2. [Step 2]
3. [Step 3]

**Risks**:
- [Risk 1]: [Mitigation]

---

### 2. [Next Duplicate Pattern]

[Similar structure]

## Consolidation Strategy

### Phase 1: Critical Duplicates (Week 1-2)
- [ ] [Duplicate pattern 1]
- [ ] [Duplicate pattern 2]

### Phase 2: High Priority (Week 3-4)
- [ ] [Duplicate pattern 3]
- [ ] [Duplicate pattern 4]

### Phase 3: Medium Priority (Month 2)
- [ ] [Remaining patterns]

## Success Metrics

Track before and after consolidation:

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Total LOC | X | Y | -Z% |
| Duplicate % | X% | Y% | -Z% |
| Test Time | X min | Y min | -Z% |
| Maintenance Burden | High | Low | Improved |
```

## Template 5: Technical Debt Report

Use this for quantifying and prioritizing technical debt.

```markdown
# Technical Debt Assessment: [Project Name]

**Assessment Date**: [Date]
**Total Debt**: [X developer days]
**Debt-to-Codebase Ratio**: [X days per 1K LOC]

## Debt Categories

### 1. Code Debt

**Total**: [X developer days]
**Interest Rate**: [High/Medium/Low]

| Item | Count | Effort Each | Total | Priority |
|------|-------|-------------|-------|----------|
| TODOs | X | 30 min | Y hours | 🟡 |
| FIXMEs | X | 2 hours | Y hours | 🔴 |
| Dead Code | X files | 1 hour | Y hours | 🟢 |
| Complex Functions | X | 4 hours | Y hours | 🟡 |

### 2. Test Debt

**Total**: [X developer days]
**Interest Rate**: Medium (compounds with each change)

| Item | Count | Effort Each | Total | Priority |
|------|-------|-------------|-------|----------|
| Missing Tests | X files | 3 hours | Y days | 🔴 |
| Skipped Tests | X tests | 1 hour | Y hours | 🟡 |
| Flaky Tests | X tests | 2 hours | Y hours | 🔴 |
| Low Coverage | X modules | 4 hours | Y days | 🟡 |

### 3. Documentation Debt

**Total**: [X developer days]
**Interest Rate**: Low (constant cost)

| Item | Count | Effort Each | Total | Priority |
|------|-------|-------------|-------|----------|
| Undocumented APIs | X functions | 30 min | Y hours | 🟡 |
| Missing ADRs | X decisions | 2 hours | Y hours | 🟢 |
| Outdated Docs | X files | 1 hour | Y hours | 🟢 |

### 4. Dependency Debt

**Total**: [X developer days]
**Interest Rate**: High (security risk)

| Item | Count | Effort Each | Total | Priority |
|------|-------|-------------|-------|----------|
| Security Vulns | X | 4 hours | Y hours | 🔴 |
| Outdated Deps | X | 2 hours | Y hours | 🟡 |
| Deprecated APIs | X usages | 3 hours | Y hours | 🟡 |

## Interest Calculation

**Monthly Interest** (cost of not fixing):

| Debt Type | Monthly Cost | Annual Cost |
|-----------|-------------|-------------|
| Security Vulns | High | [X days] |
| Missing Tests | [Y hours] | [Z days] |
| Poor Documentation | [Y hours] | [Z days] |
| **Total Interest** | **[X days/month]** | **[Y days/year]** |

## Payoff Strategy

### Quick Wins (< 1 week effort, high impact)
1. [Item]
2. [Item]

### Strategic Investments (> 1 week effort, high impact)
1. [Item]
2. [Item]

### Nice-to-Haves (low effort, low impact)
1. [Item]
2. [Item]

## Debt Reduction Plan

**Target**: Reduce total debt by [X%] in [timeframe]

### Q1 Goals
- [ ] Pay down [X days] of critical debt
- [ ] Prevent new [type] debt
- [ ] Set up monitoring for [metric]

### Q2 Goals
- [ ] [Goal]

## Monitoring

Track debt accumulation rate:
- New debt added per sprint: [X hours]
- Debt paid down per sprint: [Y hours]
- Net debt change: [Z hours] (target: negative)
```

## Template 6: Onboarding Document

Use this for new developer onboarding.

```markdown
# [Project Name] - Developer Onboarding

**Welcome!** This document will help you understand the codebase and start contributing quickly.

## Project Overview

**Purpose**: [What this project does]
**Tech Stack**: [Languages, frameworks, tools]
**Architecture**: [High-level architecture summary]

## Getting Started

### Prerequisites
- [Tool 1] version X+
- [Tool 2] version Y+
- [Access to resource Z]

### Setup (< 15 minutes)

```bash
# Clone repository
git clone [repo-url]

# Install dependencies
[commands]

# Run tests
[commands]

# Start development server
[commands]
```

**Expected Output**: [What success looks like]

## Codebase Structure

```
project/
├── [dir1]/ - [Purpose]
├── [dir2]/ - [Purpose]
└── [dir3]/ - [Purpose]
```

### Key Files to Understand

1. **`[file1]`** - [Purpose and when you'll touch it]
2. **`[file2]`** - [Purpose and when you'll touch it]
3. **`[file3]`** - [Purpose and when you'll touch it]

## Common Development Tasks

### Adding a New Feature

1. [Step 1]
2. [Step 2]
3. [Step 3]

**Example**: [Link to recent PR]

### Fixing a Bug

1. [Step 1]
2. [Step 2]
3. [Step 3]

**Example**: [Link to recent PR]

### Running Tests

```bash
# All tests
[command]

# Specific test
[command]

# With coverage
[command]
```

### Debugging Tips

- **Common Issue 1**: [How to fix]
- **Common Issue 2**: [How to fix]

## Architecture Patterns

### Pattern 1: [Name]
**Used for**: [Use case]
**Example**: [Location in code]

## Code Style

- Follow [style guide]
- Run `[formatter]` before committing
- Lint with `[linter]`

## Development Workflow

1. Create feature branch from `main`
2. Make changes following patterns
3. Add tests (required)
4. Run full test suite
5. Open PR with description
6. Address review comments
7. Merge when approved

## Resources

- **Documentation**: [Link]
- **API Docs**: [Link]
- **Team Chat**: [Link]
- **CI/CD**: [Link]

## Getting Help

- **Quick questions**: [Slack/Teams channel]
- **Detailed help**: [Forum/Email]
- **Bugs**: [Issue tracker]

## Your First Contribution

We recommend starting with:
1. **Good First Issue**: [Link to issues]
2. **Documentation**: Improve docs as you learn
3. **Tests**: Add tests to increase coverage

**Goal**: Make your first PR within first week!
```

## Choosing the Right Template

| Need | Template |
|------|----------|
| Executive briefing for management | Executive Summary |
| Technical documentation | Architecture Documentation |
| Planning refactoring work | Refactoring Roadmap |
| Finding duplicate code | Consolidation Opportunities |
| Quantifying tech debt | Technical Debt Report |
| Onboarding new developers | Onboarding Document |

## Customization Guidelines

### Adapt to Audience

**For Management**:
- Focus on ROI, business impact, risks
- Use simple language
- Highlight quick wins
- Provide timeline and resource needs

**For Developers**:
- Include technical details
- Provide code examples
- Reference specific files and line numbers
- Include implementation guidance

**For New Team Members**:
- Start with high-level overview
- Provide step-by-step instructions
- Include common pitfalls
- Link to additional resources

### Adapt to Project Size

**Small Projects (< 10K LOC)**:
- Combine templates
- Less formal structure
- Focus on immediate actions

**Medium Projects (10K-100K LOC)**:
- Use full templates
- Progressive detail
- Track metrics

**Large Projects (> 100K LOC)**:
- Split by module/component
- Executive summary + detailed sections
- Automated metrics where possible

## Report Generation Tips

1. **Start with data collection** - Run automated tools first
2. **Use progressive disclosure** - Summary → Details → Deep dives
3. **Prioritize ruthlessly** - Not everything is urgent
4. **Include examples** - Show, don't just tell
5. **Make it actionable** - Every finding needs a recommended action
6. **Estimate effort** - Help with prioritization
7. **Track over time** - Compare with previous reports
8. **Get feedback** - Iterate on report format based on usefulness
