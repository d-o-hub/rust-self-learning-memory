---
name: learn
description: Extract non-obvious session learnings into scoped AGENTS.md files
category: knowledge-management
trigger: after non-trivial task completion
---

# Learn

Extract non-obvious session learnings into scoped `AGENTS.md` files to preserve knowledge across sessions.

## When to Use

Activate after completing a non-trivial task to capture insights that would otherwise be lost.

## Instructions

### What to Capture (Non-Obvious Only)

- Hidden relationships between files or scripts not obvious from code.
- Execution paths that differ from what the code appears to do.
- Non-obvious config, env vars, or flags (see `agent_docs/ENVIRONMENT_VARIABLES.md`).
- Debugging breakthroughs where error messages were misleading.
- Files that must change together (e.g., `AGENTS.md` + `agent_docs/AVAILABLE_SKILLS.md` when adding skills).
- Build/test commands not documented in README.
- Architectural constraints discovered at runtime.

### What NOT to Capture

- Obvious documentation or standard behavior.
- Duplicates of existing entries.
- Verbose explanations or session-specific notes.

### Scoping Rules

Place learnings in the most specific `AGENTS.md` file:
- **Project-wide**: Root `AGENTS.md`.
- **Script-specific**: `scripts/AGENTS.md`.
- **Skill-specific**: `.agents/skills/<name>/AGENTS.md`.

### Dual-Write Requirement

Every new non-obvious insight must be recorded in two places:
1. **Verbose Log**: Add a full `LESSON-NNN` entry to `agent_docs/LESSONS.md` with Issue/Root Cause/Solution.
2. **Distilled Note**: Add a 1–3 line note to the nearest `AGENTS.md` (this is what `learn` automates).

### Format

- 1–3 lines per insight in `AGENTS.md`.
- Fits within `MAX_LINES_AGENTS_MD=150` constraint.
- Bulleted list under a "Learnings" or "Context" section.

## Reference Files

- `agent_docs/LESSONS.md` - Project-wide lessons log.
- `AGENTS.md` - Root agent guidance and constraints.