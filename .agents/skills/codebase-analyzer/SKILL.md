---
name: codebase-analyzer
description: Analyze implementation details, trace data flow, explain technical workings, locate files, and consolidate codebases. Use when you need to understand HOW code works, find file locations, or assess technical debt.
---

# Codebase Analyzer

Analyze implementation details, trace data flow, explain technical workings, and locate files.

## Quick Reference

- **[analysis-dimensions.md](analysis-dimensions.md)** - 8 analysis dimensions with detailed criteria
- **[consolidation-patterns.md](consolidation-patterns.md)** - Common refactoring patterns with examples
- **[report-templates.md](report-templates.md)** - Output format templates

## When to Use

- Understanding how a specific feature works
- Finding where functionality is implemented
- Tracing data flow from entry to exit
- Assessing technical debt and refactoring opportunities
- Documenting API contracts and architecture
- Code review preparation

## Analysis Modes

| Mode | Purpose | Output |
|------|---------|--------|
| **Locator** | Find file locations | File paths with descriptions |
| **Analyzer** | Trace implementation | Data flow with file:line refs |
| **Consolidator** | Assess debt/refactor | Debt report + recommendations |

## Locator Strategy

### Search Patterns

| Pattern | Purpose |
|---------|---------|
| `*service*`, `*handler*` | Business logic |
| `*test*`, `*spec*` | Test files |
| `*.config.*` | Configuration |
| Rust: `src/`, `crates/` | Source files |

### Output Format

```markdown
## File Locations for [Feature]

### Implementation Files
- `src/services/feature.rs` - Main service logic

### Test Files
- `src/services/__tests__/feature.test.rs`

### Configuration
- `config/feature.json`
```

## Analyzer Strategy

### Step 1: Read Entry Points
- Start with main files mentioned
- Look for exports, public methods

### Step 2: Follow Code Path
- Trace function calls step by step
- Note data transformations

### Step 3: Document Key Logic
- Describe validation, error handling
- Explain complex algorithms

## Guidelines

### Do

✓ Include file:line references
✓ Read files thoroughly before explaining
✓ Trace actual code paths
✓ Group files logically by purpose
✓ Use multiple search patterns

### Don't

✗ Guess about implementation
✗ Skip error handling
✗ Make architectural recommendations without context
✗ Analyze without clear goals

## Remember

You are a **documentarian**. Map and explain, don't redesign.