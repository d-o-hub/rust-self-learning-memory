# Skills vs Task Agents

## Overview

Claude Code provides two primary mechanisms for extending capabilities:

1. **Skills** - Instruction sets loaded into the main conversation
2. **Task Agents** - Autonomous sub-processes with specialized capabilities

## Skills

### Definition
Skills are instruction sets that guide Claude's behavior within the current conversation context.

### When to Use Skills
- Quick, focused operations
- Instructional guidance needed
- No autonomous execution required
- Results needed immediately in context

### Examples
- `code-quality` - Guide formatting, linting, and Rust best practices
- `architecture-validation` - Validate against plans
- `plan-gap-analysis` - Analyze implementation gaps

### Invocation
```
Use the Skill tool with skill name
```

### Characteristics
- Runs in current context
- No separate process
- Direct tool access
- Immediate results

## Task Agents

### Definition
Task Agents are specialized autonomous subprocesses that execute complex multi-step tasks.

### When to Use Task Agents
- Multi-step autonomous work
- Parallel execution needed
- Specialized domain expertise
- Background processing

### Examples
- `code-reviewer` - Review code changes
- `test-runner` - Execute test suites
- `debugger` - Diagnose runtime issues
- `loop-agent` - Iterative refinement

### Invocation
```
Use the Task tool with subagent_type
```

### Characteristics
- Runs in separate context
- Autonomous execution
- Specialized tool subset
- Can run in background

## Decision Guide

| Need | Use |
|------|-----|
| Quick guidance | Skill |
| Format/lint | Skill |
| Multi-step task | Task Agent |
| Parallel work | Task Agent |
| Background processing | Task Agent |
| Domain expertise | Task Agent |
| Current context needed | Skill |

## Coordination

Skills and Agents can work together:

1. **Skill loads context** -> **Agent executes work** -> **Skill validates results**
2. **Multiple agents** coordinated via **agent-coordination skill**

## See Also

- [strategies.md](./strategies.md) - Execution patterns
- [quality-gates.md](./quality-gates.md) - Validation checkpoints
- [examples.md](./examples.md) - Real-world examples