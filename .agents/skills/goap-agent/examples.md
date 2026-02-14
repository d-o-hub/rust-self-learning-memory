# GOAP Examples

## Example 1: Implement Authentication System

```markdown
Task: Implement authentication system

## GOAP Plan

### Phase 1: Analysis (Sequential)
- goap-agent → Define requirements
- Quality Gate: Requirements clear

### Phase 2: Implementation (Parallel)
- Agent A → User model + database
- Agent B → Auth middleware
- Agent C → API endpoints
- Quality Gate: All components implemented

### Phase 3: Integration (Sequential)
- feature-implementer → Wire components
- test-runner → Integration tests
- Quality Gate: Tests pass

### Phase 4: Validation (Sequential)
- code-reviewer → Security review
- Quality Gate: Approved for deployment
```

## Example 2: Learning-Enabled GOAP

```markdown
Task: Implement authentication system

Phase 0: Retrieve Context
└─ Skill(command="context-retrieval")
   Query: "authentication implementation"
   → Found: 3 past auth implementations
   → Pattern: Parallel (model + middleware + endpoints)
   → Lesson: Sequential integration after parallel build

Phase 1: Start Episode
└─ Skill(command="episode-start")
   Context: {domain: "goap", tags: ["auth", "parallel"]}

Phase 2-N: Execute with logging
└─ Skill(command="episode-log-steps")
   Log: Decomposition, assignment, quality gates

Phase Final: Complete Episode
└─ Skill(command="episode-complete")
   Score: High (reused successful pattern)
   Pattern: Confirmed parallel → sequential strategy
```

## Example 3: Creating Custom Capability

```markdown
Problem: Need specialized security audit for auth code

Step 1: Identify gap
→ No existing Skill covers auth security audit

Step 2: Create Skill
└─ Skill(command="skill-creator")
   Name: "auth-security-audit"
   Purpose: "Audit auth code for vulnerabilities"

Step 3: Integrate into GOAP
→ Add to Quality & Validation Skills
→ Add to Phase 3 and Phase 7 recommendations

Step 4: Use in workflow
└─ Phase 3: Skill(command="auth-security-audit")
   → Validates auth design before implementation
```

## Example 4: Full Stack Feature Implementation

```markdown
Task: Add real-time notifications

## Execution Plan

### Overview
- Strategy: Hybrid
- Total Tasks: 8
- Quality Gates: 3

### Phase 1: Research (Parallel)
- web-search-researcher → Best practices
- context-retrieval → Similar implementations
- Explore → Current architecture
Quality Gate: Requirements clear

### Phase 2: Decision
- analysis-swarm → Architectural approach
Quality Gate: Design approved

### Phase 3: Implementation (Parallel)
- feature-implementer → WebSocket server
- feature-implementer → Notification service
- feature-implementer → Client integration
Quality Gate: All services implemented

### Phase 4: Testing
- test-runner → Unit tests
- test-runner → Integration tests
Quality Gate: All tests pass

### Phase 5: Validation
- rust-code-quality → Code review
- code-reviewer → Final check
Quality Gate: Ready for merge
```

## Example 5: ADR-Driven CI/CD Remediation

```markdown
Task: Fix CI/CD failures following ADR-023

## Pre-Planning: ADR Discovery
├─ Read: plans/adr/ADR-023-CI-CD-GitHub-Actions-Remediation.md
├─ Key Decisions:
│  - Upgrade free-disk-space to v2
│  - Exclude heavy directories from coverage
│  - Document Dependabot resolution strategy
└─ Constraints:
   - Must maintain zero clippy warnings
   - All quality gates must pass

## GOAP Execution Plan

### Phase 1: Analysis
- Check: Current CI status
- Review: ADR-023 requirements
- Identify: Specific failures to fix
Quality Gate: Root causes identified

### Phase 2: Create Execution Plan
- Create: plans/GOAP_EXECUTION_PLAN_2026-02-14.md
- Link: Reference ADR-023
- Define: Tasks aligned with ADR decisions
Quality Gate: Plan created and linked to ADR

### Phase 3: Implementation (Sequential)
- Task 1: Upgrade free-disk-space action
  └─ Edit .github/workflows/coverage.yml
- Task 2: Optimize coverage exclusions
  └─ Add --exclude for benches, examples, test-utils
- Task 3: Document Dependabot resolution
  └─ Update plans/DEPENDABOT_TRIAGE_REPORT_*.md
Quality Gate: All changes implemented

### Phase 4: Validation
- Run: cargo fmt --all -- --check
- Run: cargo clippy --all -- -D warnings
- Run: cargo test --all
Quality Gate: Zero warnings, all tests pass

### Phase 5: Update Documentation
- Update: ADR-023 with implementation status
- Update: Execution plan with completion
- Create: Phase completion summary
Quality Gate: Documentation current

### Phase 6: Learning
- Skill(command="episode-complete")
- Score: High (followed ADR, all gates passed)
- Pattern: ADR-driven sequential execution works for CI fixes
```

## Success Metrics

### Planning Quality
- Clear decomposition with measurable tasks
- Realistic estimates
- Appropriate strategy selection
- Well-defined quality gates

### Execution Quality
- Tasks completed as planned
- Quality gates passed
- Minimal re-work required
- Efficient resource utilization

### Learning
- Document what worked well
- Identify improvement areas
- Update patterns for future use
