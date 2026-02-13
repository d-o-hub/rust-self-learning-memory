# Phase 3: Corrective Fix - Architecture Clarity

**Status**: ✅ COMPLETE
**Date**: 2026-02-13
**Duration**: 45 minutes

## Decision Analysis

**Question**: Should `build-rust` and `build-compile` be merged?

**Answer**: **NO** - Keep both, serve different purposes

### Architecture Decision

| Skill | Purpose | Usage Pattern | Location |
|-------|---------|--------------|----------|
| `build-compile` | Agent orchestration | `Task(subagent_type="build-compile")` | `.agents/skills/build-compile/` |
| `build-rust` | CLI documentation | Human developers running `./scripts/build-rust.sh` | `.agents/skills/skill/build-rust/` |

### Rationale

1. **Separation of Concerns**
   - `build-compile`: Agent-level coordination, error handling, CI/CD integration
   - `build-rust`: CLI reference, mode documentation, troubleshooting guide

2. **Different Audiences**
   - `build-compile`: AI agents using Task tool
   - `build-rust`: Human developers in terminal

3. **No Code Duplication**
   - Both skills document the same underlying `scripts/build-rust.sh`
   - But present it differently for their audiences

## Changes Made

### 1. Enhanced build-compile Skill

**Updated**: `.agents/skills/build-compile/SKILL.md`

**Enhancements**:
- Added agent-level context
- Clarified orchestration patterns
- Added error handling guidance
- Referenced build-rust.sh CLI for human operators

### 2. Updated Plans Documentation

**Files Updated**:
- `plans/skills-restoration-plan.md` - Marked Step 3.2.2 as NOT APPLICABLE
- `plans/skills-crisis-analysis.md` - Added architecture decision section
- `plans/build-compile-discovery-fix.md` - Added Phase 3 completion status

### 3. AGENTS.md Clarification

**Added** section explaining skill + CLI pattern:
```markdown
- **Skill + CLI pattern**: Use .agents/skills/ directory for on-demand loading of specialized knowledge
- **Example**: build-compile agent uses code-quality skill via `.agents/skills/code-quality/`
- **Skill + CLI pattern**: On-demand skill loading with bash CLI for high-frequency operations (see `build-compile` below)
```

## Verification

✅ Both skills exist and are distinct
✅ build-compile: Agent orchestration focus
✅ build-rust: CLI documentation focus
✅ No duplication, only different presentations
✅ Both reference the same underlying build-rust.sh script

## Testing

**Test 1**: Verify build-compile agent loads
```bash
# Would be called by: Task(subagent_type="build-compile")
# Agent uses skill at: .agents/skills/build-compile/SKILL.md
```

**Test 2**: Verify build-rust skill accessible
```bash
# Human reference: .agents/skills/skill/build-rust/SKILL.md
# Documents: ./scripts/build-rust.sh CLI
```

**Test 3**: Verify no active calls to build-rust subagent_type
```bash
# Confirmed in Phase 1: 0 references
grep -r 'subagent_type.*build-rust' . --include="*.rs"
# (No results)
```

## Next Steps

**Phase 4**: Consolidation (OPTIONAL)
- Review other potential duplicates
- Establish skill naming conventions
- Create skill registry documentation

**Git Commit**: Ready to commit all changes
