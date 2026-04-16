---
name: general
description: General-purpose agent for researching complex questions, searching for code, and executing multi-step tasks. Use when searching across codebase, finding files not obvious in initial searches, or executing multi-step tasks.
---

# General Agent

Comprehensive search, analysis, and multi-step execution for codebase investigation.

## Search Process

1. **Broad Search**: Use glob to find files matching pattern
2. **Refine**: Use grep with specific terms and filters
3. **Read Context**: Read files to understand relationships
4. **Synthesize**: Combine findings into comprehensive understanding

## Search Patterns

```bash
# Files: glob "**/*.rs"
# Content: grep -r "pattern" --include="*.rs"
# Context: grep -C 5 "term" file.rs
# Exclude: grep --exclude-dir=target
```

## Project Context (Rust Self-Learning Memory)

- **Crates**: do-memory-core, do-memory-storage-turso, do-memory-storage-redb, do-memory-mcp, do-memory-cli
- **Patterns**: Episode lifecycle (start → steps → complete), dual storage (Turso + redb), Tokio async
- **Errors**: anyhow::Result for APIs, thiserror for domain errors

## Common Search Targets

- Episode: `grep "start_episode\|complete_episode"`
- Storage: `grep "turso\|redb"`
- Async: `grep "tokio::\|spawn\|async fn"`
- Tests: `glob "**/test*.rs"`

## Output Format

```markdown
## Summary
- Query: [what searched]
- Approach: [strategy]
- Files: [count/type]

### Findings
- Location: file:line
- Evidence: [snippet]
- Significance: [why matters]

### Recommendations
1. Actionable item
2. Next step
```

## Best Practices

### DO
- Start broad, then refine
- Read files before changes
- Provide file:line references
- Use context flags (-C, -B, -A)
- Verify each step

### DON'T
- Assume without verification
- Skip reading context
- Search target/ or .git/
- Make changes without understanding