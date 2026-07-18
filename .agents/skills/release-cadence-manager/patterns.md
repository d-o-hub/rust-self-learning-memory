# Release Cadence Manager - Patterns

## Pattern 1: GOAP Orchestrator with Swarm Agents

**Use when**: Automating release cadence management with multiple agents

```
Task: Automate release cadence management

Phase 0: ADR Discovery [MANDATORY]
├─ Read: plans/adr/ADR-*.md
├─ Identify: Relevant ADRs for release management
├─ Check: ADR-022 (GOAP), ADR-023 (CI/CD), etc.
└─ Note: Architectural constraints and decisions
Quality Gate: Relevant ADRs identified and reviewed

Phase 1: Drift Detection [Skill]
├─ Skill(command="release-cadence-manager")
│  Mode: detect
│  → Analyze current release state
└─ Quality Gate: Drift severity determined

Phase 2: Strategy Selection [Skill + Analysis Swarm]
├─ Skill(command="analysis-swarm")
│  → Multi-perspective decision on resolution strategy
├─ Options:
│  ├─ Auto-label PR (if valid release PR)
│  ├─ Create release-preparation PR
│  ├─ Notify maintainer for manual intervention
│  └─ Skip (if false positive)
└─ Quality Gate: Strategy approved

Phase 3: Execution [Swarm Agents]
├─ Agent 1: Drift Detector Agent
│  └─ Monitors release cadence and detects drift
├─ Agent 2: Label Manager Agent
│  └─ Manages the `release-preparation` label
├─ Agent 3: Release Coordinator Agent
│  └─ Coordinates the release process
└─ Agent 4: Validation Agent
   └─ Validates that all steps are completed correctly

Phase 4: Validation [Skills]
├─ Skill(command="code-quality")
├─ Skill(command="architecture-validation")
└─ Skill(command="release-guard")
└─ Quality Gate: Release process validated

Phase 5: Update Plans & Documentation [Documentation]
├─ Update: AGENTS.md with new workflow
├─ Create/Update: Skill documentation
└─ Document: Lessons learned and patterns discovered
```

## Pattern 2: Drift Detection → Resolution

**Use when**: Release drift is detected and needs resolution

```
Phase 1: Detect Drift
├─ Run: ./scripts/check-release-drift.sh
├─ Analyze: severity and reason
└─ Quality Gate: Drift severity determined

Phase 2: Determine Resolution
├─ If severity == "clean": No action
├─ If severity == "warning": Notify maintainer
├─ If severity == "critical": Automated resolution
└─ Quality Gate: Resolution strategy determined

Phase 3: Execute Resolution
├─ If auto-label: Add release-preparation label
├─ If create PR: Create release-preparation PR
├─ If notify: Send notification to maintainer
└─ Quality Gate: Resolution executed

Phase 4: Validate Resolution
├─ Check: Label added correctly
├─ Check: PR created (if applicable)
├─ Check: Release process coordinated
└─ Quality Gate: Resolution validated
```

## Pattern 3: Swarm Agent Coordination

**Use when**: Multiple agents need to coordinate on release cadence management

```
Phase 1: Initialize Agents
├─ Drift Detector Agent: Ready
├─ Label Manager Agent: Ready
├─ Release Coordinator Agent: Ready
└─ Validation Agent: Ready

Phase 2: Parallel Execution
├─ Drift Detector Agent → Detect drift
├─ Label Manager Agent → Manage labels
├─ Release Coordinator Agent → Coordinate release
└─ Validation Agent → Validate steps

Phase 3: Synchronize Results
├─ Combine: Drift detection results
├─ Combine: Label management results
├─ Combine: Release coordination results
└─ Combine: Validation results

Phase 4: Validate Overall Success
├─ Check: All agents completed successfully
├─ Check: No errors or failures
├─ Check: Quality gates passed
└─ Quality Gate: Overall success validated
```

## Pattern 4: Integration with Existing Skills

**Use when**: Integrating release-cadence-manager with existing skills

```
Phase 1: Skill Integration
├─ Integrate: release-guard skill
├─ Integrate: analysis-swarm skill
├─ Integrate: goap-agent skill
└─ Integrate: agent-coordination skill

Phase 2: Workflow Integration
├─ Integrate: release-drift.yml workflow
├─ Integrate: release.yml workflow
├─ Integrate: pr-readiness workflow
└─ Quality Gate: Workflow integration validated

Phase 3: Documentation Integration
├─ Update: AGENTS.md
├─ Create: Skill documentation
├─ Document: Integration patterns
└─ Quality Gate: Documentation integration complete
```

## Pattern 5: Error Handling and Recovery

**Use when**: Agents fail or errors occur

```
Phase 1: Detect Error
├─ Monitor: Agent execution
├─ Detect: Failures or errors
└─ Quality Gate: Error detected

Phase 2: Classify Error
├─ If agent failure: Retry or fallback
├─ If integration error: Manual intervention
├─ If validation error: Re-run validation
└─ Quality Gate: Error classified

Phase 3: Execute Recovery
├─ If retry: Re-run failed agent
├─ If fallback: Use alternative approach
├─ If manual: Notify maintainer
└─ Quality Gate: Recovery executed

Phase 4: Validate Recovery
├─ Check: Error resolved
├─ Check: Process completed
├─ Check: Quality gates passed
└─ Quality Gate: Recovery validated
```

## Pattern 6: Progressive Enhancement

**Use when**: Implementing release-cadence-manager incrementally

```
Phase 1: Basic Implementation
├─ Create: Minimal skill
├─ Implement: Basic drift detection
├─ Test: Core functionality
└─ Quality Gate: Basic implementation complete

Phase 2: Enhanced Implementation
├─ Add: Swarm agents
├─ Add: Automated resolution
├─ Add: Integration with existing skills
└─ Quality Gate: Enhanced implementation complete

Phase 3: Advanced Implementation
├─ Add: Multi-perspective analysis
├─ Add: Progressive disclosure
├─ Add: Advanced error handling
└─ Quality Gate: Advanced implementation complete

Phase 4: Production Implementation
├─ Add: Performance optimization
├─ Add: Comprehensive testing
├─ Add: Full documentation
└─ Quality Gate: Production implementation complete
```

## Pattern 7: Quality Gates

**Use when**: Ensuring quality at each phase

```
Quality Gate 1: ADR Discovery
├─ Check: Relevant ADRs identified
├─ Check: Architectural constraints noted
└─ Pass: ADRs reviewed and understood

Quality Gate 2: Skill Creation
├─ Check: SKILL.md created
├─ Check: Supporting files created
├─ Check: Documentation complete
└─ Pass: Skill created with all files

Quality Gate 3: Agent Implementation
├─ Check: All agents implemented
├─ Check: Agents tested
├─ Check: Agents integrated
└─ Pass: All agents implemented

Quality Gate 4: Workflow Update
├─ Check: Workflow updated
├─ Check: Integration tested
├─ Check: Manual override maintained
└─ Pass: Workflow updated and tested

Quality Gate 5: Documentation Update
├─ Check: AGENTS.md updated
├─ Check: Skill documentation created
├─ Check: Lessons learned documented
└─ Pass: Documentation updated
```

## Pattern 8: Monitoring and Observability

**Use when**: Tracking release cadence management

```
Phase 1: Define Metrics
├─ Drift detection accuracy
├─ Resolution success rate
├─ Agent execution time
└─ Quality gate pass rate

Phase 2: Implement Monitoring
├─ Log: Agent execution
├─ Log: Drift detection
├─ Log: Resolution actions
└─ Log: Quality gate results

Phase 3: Analyze Metrics
├─ Analyze: Drift patterns
├─ Analyze: Resolution effectiveness
├─ Analyze: Agent performance
└─ Analyze: Quality gate trends

Phase 4: Optimize Process
├─ Optimize: Drift detection
├─ Optimize: Resolution strategies
├─ Optimize: Agent coordination
└─ Optimize: Quality gates
```
