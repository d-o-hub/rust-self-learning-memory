# Skill Templates

## Template 1: Process Skill

```markdown
---
name: process-name
description: [Action] [what] [when to use]
---

# Process Name

Brief description of what this process achieves.

## When to Use

- Scenario 1
- Scenario 2

## Prerequisites

- Requirement 1
- Requirement 2

## Process Steps

### Step 1: [Action]
Instructions for step 1

### Step 2: [Action]
Instructions for step 2

## Quality Checklist

- [ ] Check 1
- [ ] Check 2

## Examples

### Example 1: [Scenario]
[Concrete example]

## Best Practices

✓ Do this
✗ Don't do this
```

## Template 2: Knowledge Skill

```markdown
---
name: domain-knowledge
description: [Topic] knowledge and guidance for [use case]
---

# Domain Knowledge

Overview of knowledge domain.

## Core Concepts

### Concept 1
Explanation

### Concept 2
Explanation

## Guidelines

### Guideline 1
Details

## Patterns

### Pattern 1: [Name]
**Use When**: [scenario]
**Implementation**: [how-to]

## Anti-Patterns

### Anti-Pattern 1: [Name]
**Problem**: [what's wrong]
**Solution**: [correct approach]

## References

- Related skill 1
- Related skill 2
```

## Template 3: Tool Skill

```markdown
---
name: tool-usage
description: Use [tool] for [purpose]. Invoke when [scenarios]
---

# Tool Usage: [Tool Name]

Guide for effectively using [tool].

## When to Use

- Use case 1
- Use case 2

## Basic Usage

### Command Structure
```
tool [options] [arguments]
```

### Common Operations

#### Operation 1
```
tool command1
```

#### Operation 2
```
tool command2
```

## Advanced Usage

### Advanced Operation 1
Details and examples

## Troubleshooting

### Issue 1
**Symptom**: [what happens]
**Solution**: [how to fix]

## Best Practices

✓ Recommendation 1
✓ Recommendation 2
```

## Template 4: Agent Coordination Skill

```markdown
---
name: agent-coordination
description: Coordinate multiple [agent-type] agents through [strategy]. Use when [scenarios].
---

# Agent Coordination: [Strategy]

Guide for coordinating multiple agents to accomplish complex tasks.

## When to Use

- Scenario requiring multiple specialized agents
- Tasks with dependent subtasks
- Parallel execution opportunities

## Coordination Strategies

### Sequential
One agent completes before next starts.

### Parallel
Multiple agents work simultaneously.

### Iterative
Agents refine results through feedback.

### Hybrid
Combination of above strategies.

## Process

### Phase 1: Planning
Define agent roles and dependencies

### Phase 2: Execution
Launch agents according to strategy

### Phase 3: Integration
Combine agent results

## Quality Gates

- [ ] Agent outputs validated
- [ ] No conflicting actions
- [ ] Dependencies resolved
- [ ] Results integrated correctly
```

## Template Selection Guide

| Type | Use When |
|------|----------|
| Process | Step-by-step workflows |
| Knowledge | Teaching concepts/expertise |
| Tool | Using external tools/APIs |
| Agent Coordination | Managing multiple agents |
