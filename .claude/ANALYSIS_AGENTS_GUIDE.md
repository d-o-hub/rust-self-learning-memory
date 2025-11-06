# Analysis Agents & Skills Guide

This guide explains how to use the three specialized analysis agents and skills created for the Rust self-learning memory project.

## Overview

Three comprehensive analysis capabilities have been added to validate implementation against plans, ensure code quality, and verify architecture compliance:

| Agent | Skill | Purpose |
|-------|-------|---------|
| **plan-gap-analyzer** | plan-gap-analysis | Compare Phase 1-6 plans with actual implementation |
| **rust-quality-reviewer** | rust-code-quality | Review Rust code quality and best practices |
| **architecture-validator** | architecture-validation | Validate architecture compliance with design |

## 1. Plan Gap Analyzer

**Agent**: `.claude/agents/plan-gap-analyzer.md`
**Skill**: `.claude/skills/plan-gap-analysis/SKILL.md`

### Purpose
Systematically analyze the implementation status by comparing comprehensive plans in `plans/` (Phases 1-6) against the actual Rust codebase.

### What It Checks

#### Phase 1: UNDERSTAND
- All 47 core data structures from specification
- Episode, ExecutionStep, Pattern structures
- All required type definitions

#### Phase 2: PLAN
- Architectural decisions implementation
- Circuit breakers and resilience patterns
- Feature flags
- Telemetry and observability
- Success metrics tracking

#### Phase 3: EXECUTE
- Week 1-2: Storage layer (Turso + redb)
- Week 3-4: Learning cycle (start → log → complete)
- Week 5-6: Pattern extraction
- Week 7-8: MCP integration
- Week 9-10: Performance benchmarks
- Week 11-12: Production readiness

#### Phase 4: REVIEW
- Functional requirements (FR1-FR7)
- Non-functional requirements (NFR1-NFR6)
- Test coverage (target: >90%)
- Quality metrics

#### Phase 5: SECURE
- 5 attack surface mitigations
- Security test coverage
- Vulnerability prevention

#### Phase 6: FEEDBACK LOOP
- Edge case handling
- Performance refinements
- Two-phase commit
- Schema migration

### Usage

```bash
# In Claude Code, invoke the agent
Task: "Use the plan-gap-analyzer agent to analyze implementation gaps"

# Or use directly
/plan-gap-analyzer
```

### Output Format

The agent generates:
- **Summary statistics** (total requirements, implemented, gaps, completion %)
- **Detailed gap list by phase** (Critical → High → Medium → Low)
- **File locations** with exact line numbers
- **Plan references** linking back to source documents
- **Effort estimates** for each gap
- **Execution plan** with week-by-week priorities

### Example Output

```markdown
# Implementation Gap Analysis - TODO List

## Summary by Phase
- Phase 1 (UNDERSTAND): 5 gaps (2 critical)
- Phase 2 (PLAN): 8 gaps (1 critical)
- Phase 3 (EXECUTE): 12 gaps (3 critical)
- Phase 4 (REVIEW): 7 gaps (4 critical)
- Phase 5 (SECURE): 6 gaps (2 critical)
- Phase 6 (FEEDBACK): 4 gaps (0 critical)

## Phase 1: UNDERSTAND

### Critical Priority
- [ ] **Missing Pattern variant: ContextPattern**
  - **File**: memory-core/src/pattern.rs:45
  - **Plan Reference**: plans/01-understand.md:70-77
  - **Required**: Add ContextPattern { context_features, recommended_approach, evidence }
  - **Impact**: Cannot identify context-based patterns
  - **Effort**: 2-4 hours
```

## 2. Rust Quality Reviewer

**Agent**: `.claude/agents/rust-quality-reviewer.md`
**Skill**: `.claude/skills/rust-code-quality/SKILL.md`

### Purpose
Perform comprehensive Rust code quality reviews against industry best practices.

### What It Checks

#### 1. Project Structure (Score: X/10)
- Workspace organization
- Crate separation
- Module hierarchy
- File size limits (<500 LOC)
- Naming conventions

#### 2. Error Handling (Score: X/10)
- Custom Error enum with thiserror
- Result<T> usage
- Error propagation with `?`
- No unwrap() in production
- Meaningful error messages

#### 3. Async/Await Patterns (Score: X/10)
- Proper async fn usage
- Tokio runtime patterns
- spawn_blocking for sync operations
- No blocking in async context
- Concurrent operations (join!, try_join!)

#### 4. Memory & Performance (Score: X/10)
- Minimize allocations
- Avoid unnecessary clones
- Use borrowing over ownership
- Zero-copy where possible

#### 5. Testing (Score: X/10)
- Unit test coverage (target: >90%)
- Integration tests
- Benchmarks
- Test utilities
- Property-based tests

#### 6. Documentation (Score: X/10)
- Crate-level docs
- Module docs
- Function docs with examples
- README and CONTRIBUTING
- 100% public API coverage

#### 7. Type Safety (Score: X/10)
- Strong typing (newtype pattern)
- Builder patterns
- Default implementations
- Clear API contracts

#### 8. Security & Safety (Score: X/10)
- No unsafe code
- Input validation
- SQL parameterization
- Resource limits
- No hardcoded secrets

### Usage

```bash
# In Claude Code
Task: "Use the rust-quality-reviewer agent to review code quality"

# Or invoke directly
/rust-quality-reviewer
```

### Output Format

```markdown
# Rust Code Quality Review Report
**Overall Score**: 78/100

## Executive Summary
- **Critical Issues**: 2
- **Warnings**: 8
- **Best Practices Met**: 32/40

## Quality Dimensions

### 1. Project Structure: 8/10 ⭐⭐⭐⭐
✅ Good workspace organization
⚠️ memory-core/src/memory.rs: 623 lines (target: <500)

### 2. Error Handling: 9/10 ⭐⭐⭐⭐⭐
✅ Excellent use of thiserror
⚠️ 3 unwrap() calls in production:
  - memory-core/src/sync.rs:145

### 3. Async Patterns: 7/10 ⭐⭐⭐⭐
✅ Good async fn usage
❌ Blocking call: sync.rs:89 - std::fs::read

[... continues for all 8 dimensions ...]

## Action Items

### Critical
- [ ] Fix blocking call in sync.rs (Priority: High, Effort: 1 hour)

### High
- [ ] Increase test coverage to 90% (Priority: High, Effort: 2-3 days)
```

## 3. Architecture Validator

**Agent**: `.claude/agents/architecture-validator.md`
**Skill**: `.claude/skills/architecture-validation/SKILL.md`

### Purpose
Validate that the implementation adheres to documented architectural decisions and design patterns.

### What It Checks

#### 1. System Architecture
- Crate structure matches plan
- Component boundaries
- Separation of concerns

#### 2. Dependency Architecture
- Core is storage-agnostic (AGENTS.md rule)
- Storage depends only on core
- No circular dependencies
- test-utils is dev-only

#### 3. Storage Architecture
- Hybrid Turso + redb design
- Sync mechanism implementation
- Conflict resolution strategy
- All required tables exist

#### 4. Learning Cycle
- 5-phase implementation
- All API methods exist
- Reward, reflection, extraction integrated

#### 5. Pattern Extraction
- All 4 Pattern variants
- Extractor implementations
- Rule-based + embedding strategy

#### 6. MCP Integration
- Server implementation
- Sandbox security layers
- Resource limits
- Tool definitions

#### 7. Performance Targets
- Benchmarks for all metrics
- P95 latency tests
- Concurrent operation tests
- Memory profiling

#### 8. Security Architecture
- 5 attack surface mitigations
- Defense-in-depth implementation
- Security test coverage

### Usage

```bash
# In Claude Code
Task: "Use the architecture-validator agent to check architecture compliance"

# Or invoke directly
/architecture-validator
```

### Output Format

```markdown
# Architecture Validation Report

## Executive Summary
- **Overall Compliance**: 82% (7.5/9 dimensions)
- **Critical Violations**: 1
- **Architecture Drift**: 3 areas

## Compliance Dashboard

| Dimension | Status | Score | Notes |
|-----------|--------|-------|-------|
| System Architecture | ✅ | 10/10 | Fully compliant |
| Dependency Flow | ❌ | 6/10 | Core depends on Turso |
| Storage Layer | ⚠️ | 8/10 | Sync incomplete |
| Learning Cycle | ✅ | 10/10 | All phases implemented |

## Detailed Findings

### Critical Violations
1. **Dependency Violation**: Core depends on Turso
   - **Planned**: Core is storage-agnostic
   - **Fix**: Introduce StorageBackend trait

### Architecture Drift
1. **Sync Mechanism**: Two-phase commit missing
2. **Resource Limits**: Not enforced in sandbox

## Recommendations

### High Priority
- [ ] Remove Turso dependency from core
- [ ] Implement two-phase commit
- [ ] Enforce resource limits
```

## How to Use These Agents Together

### Recommended Workflow

1. **Start with Plan Gap Analyzer** - Get the complete picture
   ```bash
   Task: "Use plan-gap-analyzer to identify all missing implementations"
   ```

2. **Then Rust Quality Reviewer** - Check code quality
   ```bash
   Task: "Use rust-quality-reviewer to assess code quality"
   ```

3. **Finally Architecture Validator** - Verify architecture compliance
   ```bash
   Task: "Use architecture-validator to check architecture compliance"
   ```

4. **Combine Results** - Create comprehensive action plan
   - Critical items from all three reports
   - Prioritize by impact and effort
   - Create weekly execution plan

### Example Combined Analysis

```bash
# Run all three in sequence
Task: "Use plan-gap-analyzer, rust-quality-reviewer, and architecture-validator to perform comprehensive analysis. Generate a consolidated TODO list prioritized by phase and criticality."
```

## Understanding the Output

### Priority Levels

- **Critical** 🔴: Blocks production, security risks, data corruption
- **High** 🟡: Affects quality, performance targets not met
- **Medium** 🟢: Technical debt, optimization opportunities
- **Low** ⚪: Nice-to-have, future enhancements

### Status Indicators

- ✅ **Compliant**: Fully implemented as planned
- ⚠️ **Partial**: Partially implemented, needs work
- ❌ **Missing**: Not implemented, gap identified
- 🔴 **Violation**: Contradicts documented architecture

### Effort Estimates

- **1-2 hours**: Quick fixes, simple additions
- **Half day**: Small features, refactoring
- **1-2 days**: Medium features, significant refactoring
- **3-5 days**: Large features, architecture changes
- **1-2 weeks**: Major features, system redesign

## Integration with TodoWrite

All three agents integrate with TodoWrite to create trackable tasks:

```markdown
# After analysis, agents create todos like:

### Critical Priority Tasks
- [ ] Fix blocking call in sync.rs (rust-quality-reviewer)
- [ ] Implement missing ContextPattern variant (plan-gap-analyzer)
- [ ] Remove Turso dependency from core (architecture-validator)

### High Priority Tasks
- [ ] Increase test coverage to 90% (rust-quality-reviewer)
- [ ] Implement two-phase commit (architecture-validator)
- [ ] Add NFR1 benchmark for retrieval latency (plan-gap-analyzer)
```

## Best Practices

1. **Run Regularly**: After major changes or weekly
2. **Address Critical First**: Focus on 🔴 items immediately
3. **Track Progress**: Use TodoWrite integration
4. **Update Plans**: If intentional changes, update architecture docs
5. **Iterate**: Re-run after fixes to verify

## Troubleshooting

### Agent Not Found
```bash
# Check agent exists
ls -la .claude/agents/plan-gap-analyzer.md

# Verify skill exists
ls -la .claude/skills/plan-gap-analysis/
```

### Agent Produces No Output
- Ensure all plan files exist in `plans/` directory
- Check that Rust source files are present
- Verify permissions on `.claude/` directory

### Incomplete Analysis
- Agent may need more time for large codebases
- Check for errors in agent execution
- Verify all dependencies are installed

## Files Created

### Skills
- `.claude/skills/plan-gap-analysis/SKILL.md` - Plan comparison methodology
- `.claude/skills/rust-code-quality/SKILL.md` - Code quality criteria
- `.claude/skills/architecture-validation/SKILL.md` - Architecture compliance checks

### Agents
- `.claude/agents/plan-gap-analyzer.md` - Plan gap analysis agent
- `.claude/agents/rust-quality-reviewer.md` - Code quality review agent
- `.claude/agents/architecture-validator.md` - Architecture validation agent

### Documentation
- `.claude/ANALYSIS_AGENTS_GUIDE.md` - This guide

## Next Steps

1. **Run Initial Analysis**: Use all three agents to get baseline
2. **Create Action Plan**: Prioritize findings by phase and criticality
3. **Execute Incrementally**: Work through TODO list systematically
4. **Re-validate**: Run agents again after major changes
5. **Maintain**: Keep running regularly to prevent drift

## Contributing

When modifying these agents:
1. Update both the agent and skill files
2. Test with actual codebase
3. Update this guide with new features
4. Document any new output formats

## Support

For issues or questions:
- Check agent execution logs
- Review skill documentation
- Verify plan files are up to date
- Ensure codebase structure matches expectations

---

**Last Updated**: 2025-11-06
**Version**: 1.0.0
**Maintained By**: Claude Code Agent System
