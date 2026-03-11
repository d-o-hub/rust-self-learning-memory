# Skills Consolidation - Quick Start Guide

**Status**: Ready to Execute
**Plan Document**: `plans/skills-consolidation.md`
**Execution Script**: `scripts/consolidate-skills.sh`

## Quick Start

```bash
# 1. Review the plan
less plans/skills-consolidation.md

# 2. Review the script
less scripts/consolidate-skills.sh

# 3. Execute consolidation
./scripts/consolidate-skills.sh

# 4. Review results
cat .agents/skills/INVENTORY.md
```

## What Will Happen

### Phase 1: Preparation
- Create `.agents/skills/` directory
- Backup existing directories (`.claude/skills/`, `.opencode/skill/`)
- Initialize inventory

### Phase 2: Copy Unique Skills
- Copy 50+ skills unique to `.claude/skills/`
- Copy 3-4 skills unique to `.opencode/skill/`
- Result: ~54 unique skills

### Phase 3: Simple Duplicates
- Process 7 simple duplicate skills (both have only SKILL.md)
- Compare line counts, prefer longer version
- Result: 7 skills

### Phase 4: Rich Duplicates
- Process 6 rich duplicate skills (`.claude` has additional files)
- Copy `.claude` structure as base
- Create `MERGE_NOTES.md` with merge instructions
- Result: 6 skills requiring manual merge

### Phase 5: Symlinks (Optional)
- Ask if you want symlinks
- Create relative symlinks for simple skills
- Skip rich skills (pending merge)

### Phase 6: Validation
- Check all skills have SKILL.md
- Check for broken symlinks
- Count total skills
- Generate inventory

## Expected Results

### Total Skills: 72

| Category | Count | Source |
|----------|-------|--------|
| Unique to .claude | ~54 | Copied directly |
| Unique to .opencode | ~3 | Copied directly |
| Simple duplicates | 7 | Prefer longer version |
| Rich duplicates | 6 | Manual merge required |

### Files Created

```
.agents/skills/
├── INVENTORY.md
├── [70+ skill directories]
│   ├── SKILL.md (or symlink)
│   ├── [additional files]
│   └── MERGE_NOTES.md (for 6 rich skills)
├── .claude/skills.backup.YYYYMMDD_HHMMSS
└── .opencode/skill.backup.YYYYMMDD_HHMMSS
```

## Manual Merge Required

### 6 Rich Skills Need Manual Attention

1. **analysis-swarm**
   - .opencode: 515 lines (comprehensive)
   - .claude: 41 lines + 4 supporting files
   - Action: Merge .opencode content into .claude structure

2. **architecture-validation**
   - .opencode: ~300 lines
   - .claude: ~50 lines + 6 supporting files
   - Action: Merge .opencode content into .claude structure

3. **debug-troubleshoot**
   - .opencode: ~400 lines
   - .claude: ~50 lines + 5 supporting files
   - Action: Merge .opencode content into .claude structure

4. **feature-implement**
   - .opencode: ~400 lines
   - .claude: ~50 lines + 5 supporting files
   - Action: Merge .opencode content into .claude structure

5. **github-workflows**
   - .opencode: ~400 lines
   - .claude: ~50 lines + 5 supporting files
   - Action: Merge .opencode content into .claude structure

6. **goap-agent**
   - .opencode: 893 lines (very comprehensive)
   - .claude: 48 lines + 6 supporting files
   - Action: Merge .opencode content into .claude structure

### Merge Process

For each rich skill:

```bash
# 1. Read merge notes
cat .agents/skills/<skill>/MERGE_NOTES.md

# 2. Compare content
diff -u .claude/skills/<skill>/SKILL.md \
       .opencode/skill/<skill>/SKILL.md

# 3. Open both files in editor
# Extract sections from .opencode
# Merge into appropriate .claude supporting files
# Update SKILL.md entry point

# 4. Validate
# Check all internal links work
# Test skill loading with Skill tool

# 5. Remove merge notes when complete
rm .agents/skills/<skill>/MERGE_NOTES.md
```

## Validation Steps

After consolidation:

```bash
# 1. Check skill count
ls -d .agents/skills/*/ | wc -l
# Expected: 72

# 2. Check for SKILL.md
find .agents/skills -name SKILL.md | wc -l
# Expected: 72

# 3. Check for broken symlinks
find .agents/skills -type l ! -exec test -e {} \; -print
# Expected: (empty)

# 4. Check merge notes
find .agents/skills -name MERGE_NOTES.md
# Expected: 6 files (to process)

# 5. Review inventory
cat .agents/skills/INVENTORY.md
```

## Rollback

If something goes wrong:

```bash
# Remove consolidated directory
rm -rf .agents/skills

# Restore from backup
mv .claude/skills.backup.* .claude/skills
mv .opencode/skill.backup.* .opencode/skill
```

## Symlink Strategy

### Decision Made: Relative Symlinks

**Direction**: `.agents/skills/<skill>/SKILL.md` → `../../.claude/skills/<skill>/SKILL.md`

**Rationale**:
- `.agents/skills/` is the canonical, unified location
- Source directories are temporary
- Relative symlinks for repository portability
- Allows independent updates to sources

### Implementation

```bash
# Simple skills: file-level symlinks
ln -sr ../../.claude/skills/agent-coordination/SKILL.md \
      .agents/skills/agent-coordination/SKILL.md

# Rich skills: directory-level symlinks (after merge)
ln -sr ../../.claude/skills/analysis-swarm \
      .agents/skills/analysis-swarm
```

## Future Migration Path

### Phase 1: Symlinks (Current)
- `.agents/skills/` → `.claude/skills/` and `.opencode/skill/`
- Maintains compatibility
- Low risk
- **Current Status**: Ready to execute

### Phase 2: Migration (6 months)
- Copy all content to `.agents/skills/`
- Remove symlinks
- Deprecate source directories
- Update all references
- **Action Item**: Schedule for August 2026

### Phase 3: Cleanup (12 months)
- Remove `.claude/skills/`
- Remove `.opencode/skill/`
- `.agents/skills/` is sole source
- **Action Item**: Schedule for February 2027

## Questions?

**Q: Will this break existing functionality?**
A: No. This consolidates skills into a new location (`.agents/skills/`). Original directories remain intact with backups.

**Q: Can I skip symlinks and just copy files?**
A: Yes. The script asks if you want symlinks. You can skip and copy instead.

**Q: How long does consolidation take?**
A: ~5 seconds for automation. Manual merges will take 1-2 hours total.

**Q: What if I make a mistake during manual merge?**
A: Backups are created automatically. You can restore and try again.

**Q: Can I do this incrementally?**
A: Yes. You can run the script multiple times. It won't overwrite existing `.agents/skills/` unless you confirm.

**Q: How do I test if skills work after consolidation?**
A: Use the Skill tool to load a skill and verify it works:
```
Use Skill: codebase-analyzer
```

## Summary

| Aspect | Details |
|--------|---------|
| **Duration** | 5 seconds (automation) + 1-2 hours (manual merges) |
| **Risk Level** | Low (backups created, no source modification) |
| **Skills Affected** | 72 total (6 require manual merge) |
| **Storage Needed** | ~50MB (duplicates during transition) |
| **Rollback** | Simple (restore from backups) |

## Execution Checklist

Before executing:
- [ ] Read `plans/skills-consolidation.md`
- [ ] Read `scripts/consolidate-skills.sh`
- [ ] Confirm you have ~50MB free space
- [ ] Ensure no other processes are modifying skill directories
- [ ] Plan 1-2 hours for manual merges

During execution:
- [ ] Review prompts carefully
- [ ] Confirm symlinks if desired
- [ ] Note any warnings

After execution:
- [ ] Review `INVENTORY.md`
- [ ] Process 6 `MERGE_NOTES.md` files
- [ ] Validate all skills accessible
- [ ] Test skill loading functionality
- [ ] Document any deviations

Success criteria:
- [ ] 72 skills in `.agents/skills/`
- [ ] Zero broken symlinks
- [ ] All `MERGE_NOTES.md` resolved
- [ ] Skills load and function correctly
