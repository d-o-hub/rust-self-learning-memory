---
name: agents-update
description: Update AGENTS.md and agent_docs/ following best practices. Use when modifying agent guidelines, adding new documentation, or restructuring agent instructions.
---

# AGENTS.md Update

Keep AGENTS.md under 140 LOC. Move detailed content to `agent_docs/`, don't just delete.

## Core Rule

**AGENTS.md ≤ 140 LOC**: Quick reference + links only
**agent_docs/**: Complete instructions with examples

## Section Mapping

| AGENTS.md Section | Target agent_docs/ File | Status |
|-------------------|------------------------|--------|
| Quick Reference | `scripts/` | ✓ commands |
| Change Workflow | `agent_docs/code_conventions.md` | ✓ |
| Core Invariants | `agent_docs/code_conventions.md` | ✓ |
| Common Pitfalls | `agent_docs/code_conventions.md` | ✓ |
| Tool Selection | `agent_docs/code_conventions.md` | ✓ |
| Atomic Change Rules | `agent_docs/git_workflow.md` | ✓ |
| Required Checks | `agent_docs/building_the_project.md` | ✓ |
| Git Workflow | `agent_docs/git_workflow.md` | ✓ |
| Feature Flags | `agent_docs/building_the_project.md` | ✓ |
| Security | `agent_docs/code_conventions.md` | ✓ |
| Environment Variables | `agent_docs/code_conventions.md` | ✓ |
| Performance Targets | `agent_docs/code_conventions.md` | ✓ |
| Disk Space | `agent_docs/building_the_project.md` | ✓ |
| Planning & Decisions | `plans/ROADMAPS/ROADMAP_ACTIVE.md` | ✓ |

## Workflow

### 1. Read Current State
- Read AGENTS.md and relevant agent_docs/ files
- Check Section Mapping table
- For complex changes, use `goap-agent` skill to decompose

### 2. Move Content (Not Delete!)
- Find target location from mapping
- Add full content with examples
- Ensure standalone usability

### 3. Update AGENTS.md
- Replace with brief summary + reference link
- Keep under 140 LOC

### 4. Validate
```bash
wc -l AGENTS.md  # Must be ≤140
ls agent_docs/   # Verify all files exist
```

## Quality Checks

- [ ] `wc -l AGENTS.md` ≤ 140
- [ ] All sections mapped to agent_docs/
- [ ] agent_docs/ files have code examples
- [ ] Can follow agent_docs/ instructions independently
- [ ] Cross-References table lists all agent_docs/

## Best Practices

- Follow [agentskills.io spec](https://agentskills.io/specification)
- Keep SKILL.md under 250 LOC
- agent_docs/ files must be usable as standalone instructions

See [references/AGENTS_UPDATE_REFERENCE.md](references/AGENTS_UPDATE_REFERENCE.md) for detailed workflow.
