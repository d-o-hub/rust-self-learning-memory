# GOAP Agents Reference

## Agent Capability Matrix

| Agent | Capabilities | Tools | Best For |
|-------|--------------|-------|----------|
| **feature-implementer** | Design, implement, test, integrate | Read, Write, Edit, Bash, Glob, Grep | New functionality, modules, APIs |
| **debugger** | Diagnose runtime issues, async problems | Read, Bash, Grep, Edit | Bug fixes, deadlocks, performance |
| **test-runner** | Execute tests, diagnose failures | Bash, Read, Grep, Edit | Test validation |
| **refactorer** | Improve structure, eliminate duplication | Read, Edit, Bash, Grep, Glob | Code quality, modernization |
| **code-reviewer** | Review quality, standards, security | Read, Glob, Grep, Bash | Quality assurance |
| **loop-agent** | Iterative refinement, convergence | Task, Read, TodoWrite, Glob, Grep | Progressive improvements |
| **agent-creator** | Create new Task Agents | Write, Read, Glob, Grep, Edit | New capabilities |
| **Explore** | Fast codebase exploration | All tools | Finding files, architecture |
| **memory-cli** | CLI development, testing | Read, Write, Edit, Bash, Glob, Grep | CLI features |

## Assignment Principles

1. Match agent capabilities to task requirements
2. Balance workload across agents
3. Consider agent specialization
4. Plan for quality validation

## Execution Agents

- **feature-implementer**: Design, implement, test new features
- **refactorer**: Improve code quality, structure
- **debugger**: Diagnose runtime issues

## Validation Agents

- **code-reviewer**: Review quality, correctness
- **test-runner**: Execute tests, diagnose failures

## Meta Agents

- **agent-creator**: Create new Task Agents
- **goap-agent**: Complex multi-step planning
- **loop-agent**: Execute workflows iteratively
- **Explore**: Fast codebase exploration

## Integration with Self-Learning Memory

### Starting a GOAP Episode
```
Use: Skill(command="episode-start")
Context: {language: "coordination", domain: "goap", tags: ["multi-agent"]}
Description: "GOAP coordination for [task]"
```

### Logging GOAP Steps
```
Use: Skill(command="episode-log-steps")
Log: Decomposition decisions, agent assignments, strategy selection
```

### Completing a GOAP Episode
```
Use: Skill(command="episode-complete")
Score based on: Goal achievement, efficiency, quality, adaptability
```

### Retrieving Past Context
```
Use: Skill(command="context-retrieval")
Query: Similar coordination tasks, past decisions
```

## Error Handling

### Agent Failure Recovery
1. Log failure reason
2. Check quality gate status
3. Options:
   - Retry same agent (transient error)
   - Assign to different agent
   - Modify task requirements
   - Escalate to user

### Quality Gate Failure
1. Identify failing criteria
2. Diagnose root cause
3. Options:
   - Re-run previous phase with fixes
   - Adjust quality criteria
   - Change strategy

### Blocked Dependencies
1. Identify blocking task
2. Prioritize unblocking
3. Options:
   - Execute dependency first
   - Remove dependency
   - Parallel work on independent tasks
