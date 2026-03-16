# AGENTS.md Update Reference

Detailed workflow for updating AGENTS.md and agent_docs/ following best practices.

## Core Rule

**AGENTS.md must stay under 140 LOC**. All detailed content goes to `agent_docs/` or `scripts/`.

## Update Workflow

### Step 1: Assess the Change

1. Read current `AGENTS.md`
2. Read relevant `agent_docs/` files
3. Identify what type of change is needed

### Step 2: Move Content, Don't Delete

When content needs to be removed from AGENTS.md:

1. **Identify target location** in agent_docs/:
   - Tool selection → `agent_docs/code_conventions.md`
   - Security → `agent_docs/code_conventions.md`
   - Environment vars → `agent_docs/code_conventions.md`
   - Performance → `agent_docs/code_conventions.md`
   - Disk space → `agent_docs/building_the_project.md`
   - Git workflow details → `agent_docs/git_workflow.md`
   - Tests → `agent_docs/running_tests.md`

2. **Add content** to the target agent_docs/ file (don't just link)

3. **Replace** AGENTS.md section with brief summary + reference

### Step 3: Validate

```bash
wc -l AGENTS.md  # Must show ≤140
```

### Step 4: Verify Agent Docs

Ensure agent_docs/ files are complete:
- Has clear sections
- Has code examples
- Can be used independently as instructions

## Content Migration Guide

| AGENTS.md Section | Target agent_docs/ File |
|-------------------|------------------------|
| Tool Selection | `agent_docs/code_conventions.md` |
| Security | `agent_docs/code_conventions.md` |
| Environment Variables | `agent_docs/code_conventions.md` |
| Performance Targets | `agent_docs/code_conventions.md` |
| Disk Space | `agent_docs/building_the_project.md` |
| Git Workflow details | `agent_docs/git_workflow.md` |
| Test details | `agent_docs/running_tests.md` |

## AGENTS.md Template

Keep only:

```markdown
# Agent Coding Guidelines

## Quick Reference
- Commands only (build, test, quality)

## Project Overview
- 1-2 lines description

## Skill + CLI Pattern
- Table of skill → CLI mappings

## Change Workflow
- Numbered list, max 10 steps

## Core Invariants
- Bullet list, max 8 items

## Common Pitfalls
- Compact table

## Atomic Change Rules
- Summary bullets

## Required Checks
- Checklist format

## Cross-References
- Table linking to agent_docs/
```

## Quality Checks

- [ ] `wc -l AGENTS.md` ≤ 140
- [ ] All removed content exists in agent_docs/
- [ ] agent_docs/ files have code examples
- [ ] Cross-References table is up to date
- [ ] Can follow agent_docs/ instructions independently
