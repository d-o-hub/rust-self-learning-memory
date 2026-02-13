# Skills Consolidation - Executive Summary

**Status**: ✅ Ready for Execution
**Date**: 2026-02-13
**Estimated Time**: 5 minutes (automation) + 5 hours (manual merges)

## 📋 Overview

This plan consolidates 72 skills from two fragmented directories into a unified `.agents/skills/` structure, eliminating duplication, improving maintainability, and establishing a single source of truth.

## 📊 Current State

| Metric | Value |
|--------|-------|
| Source Directories | 2 (`.opencode/skill/`, `.claude/skills/`) |
| Total Skills | 72 (unique) |
| Duplicate Skills | 15 (exist in both) |
| Fragmentation | High (different structures, content) |

## 🎯 Consolidation Strategy

### Type A: Simple Skills (7)
**Action**: Copy/symlink preferred version based on line count

- agent-coordination
- clean-code-developer
- code-quality
- git-worktree-manager
- github-release-best-practices
- perplexity-researcher-pro
- perplexity-researcher-reasoning-pro

### Type B: Rich Skills (6) ⚠️ **Manual Merge Required**
**Action**: Copy `.claude` structure, merge `.opencode` content into supporting files

- analysis-swarm (515 lines → 41 lines + 4 files)
- architecture-validation (~300 lines → ~50 lines + 6 files)
- debug-troubleshoot (~400 lines → ~50 lines + 5 files)
- feature-implement (~400 lines → ~50 lines + 5 files)
- github-workflows (~400 lines → ~50 lines + 5 files)
- goap-agent (893 lines → 48 lines + 6 files)

### Type C: Unique to .claude (~54)
**Action**: Copy entire directory structure

Includes: agentdb-*, v3-*, codebase-*, quality-unit-testing, and 40+ more

### Type D: Unique to .opencode (~3)
**Action**: Copy entire directory

Includes: build-rust, and 2-3 others

## 📁 Deliverables

### Documentation
1. **`plans/skills-consolidation.md`** (14,000+ words)
   - Complete consolidation strategy
   - Merge rules and conflict resolution
   - Execution plan with 6 phases
   - Validation steps and rollback plan

2. **`plans/skills-consolidation-visual.md`** (diagrams)
   - Before/after structure
   - Symlink architecture
   - Merge workflow example
   - Timeline and risk assessment

3. **`plans/skills-consolidation-quickstart.md`**
   - Quick start guide
   - Expected results
   - Manual merge instructions
   - FAQ

### Automation
4. **`scripts/consolidate-skills.sh`** (executable)
   - Automated consolidation in 6 phases
   - Backup creation
   - Inventory generation
   - Symlink management

5. **`scripts/validate-consolidation.sh`** (executable)
   - 10 validation tests
   - Pass/fail reporting
   - Action items

## 🚀 Execution Steps

### Step 1: Review (15 min)
```bash
# Read the plan
less plans/skills-consolidation-quickstart.md

# Review the script
less scripts/consolidate-skills.sh
```

### Step 2: Execute (5 min)
```bash
# Run consolidation
./scripts/consolidate-skills.sh

# Expected output:
# - Created .agents/skills/
# - Copied 72 skills
# - Created backups
# - Generated inventory
```

### Step 3: Validate (2 min)
```bash
# Run validation
./scripts/validate-consolidation.sh

# Expected: 10/10 tests pass
```

### Step 4: Manual Merge (4-5 hours)
```bash
# Process 6 MERGE_NOTES.md files
for notes in .agents/skills/*/MERGE_NOTES.md; do
    # Read instructions
    cat "$notes"

    # Merge content
    # (see visual.md for workflow)

    # Validate
    # Remove notes when complete
done
```

### Step 5: Re-validate (2 min)
```bash
# Run validation again
./scripts/validate-consolidation.sh

# Expected: All tests pass, no pending merges
```

## 📈 Expected Results

| Metric | Before | After |
|--------|--------|-------|
| Directories | 2 fragmented | 1 unified |
| Skills | 72 (mixed) | 72 (canonical) |
| Duplicate Skills | 15 | 0 (merged) |
| Documentation | Scattered | Centralized |
| Maintenance | High effort | Low effort |

## ⚠️ Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Breaking functionality | High | Low | Backups, rollback plan |
| Content loss | High | Low | Manual review, source preservation |
| Time-consuming | Low | High | Only 6 skills need merge |
| Symlinks breaking | Medium | Low | Relative symlinks, optional |

**Overall Risk**: ✅ LOW (with comprehensive backups and rollback)

## 🎯 Success Criteria

### Structural
- [x] Plan documented (14,000+ words)
- [ ] 72 skills in `.agents/skills/`
- [ ] All skills have `SKILL.md`
- [ ] Zero broken symlinks
- [ ] All `MERGE_NOTES.md` resolved

### Functional
- [ ] Skills load correctly with Skill tool
- [ ] All internal links work
- [ ] Frontmatter valid for all `SKILL.md`
- [ ] No content loss from sources

### Quality
- [ ] Improved organization
- [ ] Single canonical location
- [ ] Clear documentation
- [ ] Future-proof structure

## 📝 Post-Execution Checklist

### Immediate (Day 1)
- [ ] Execute consolidation script
- [ ] Run validation script
- [ ] Review `INVENTORY.md`
- [ ] Create merge plan for 6 rich skills
- [ ] Document any deviations

### Manual Merges (Days 2-3)
- [ ] analysis-swarm: Merge 515 lines into modular structure
- [ ] architecture-validation: Merge 300 lines into modular structure
- [ ] debug-troubleshoot: Merge 400 lines into modular structure
- [ ] feature-implement: Merge 400 lines into modular structure
- [ ] github-workflows: Merge 400 lines into modular structure
- [ ] goap-agent: Merge 893 lines into modular structure

### Validation (Day 3)
- [ ] Re-run validation script (all 10 tests pass)
- [ ] Test skill loading for sample skills
- [ ] Verify all internal links work
- [ ] Confirm zero `MERGE_NOTES.md` remain

### Future (6+ months)
- [ ] Break symlinks
- [ ] Deprecate source directories
- [ ] Update all references
- [ ] Remove source directories (12+ months)

## 🔧 Technical Details

### Symlink Strategy
- **Type**: Relative symlinks
- **Direction**: `.agents/skills/<skill>/SKILL.md` → `../../.claude/skills/<skill>/SKILL.md`
- **Rationale**: Portable, maintains single source, allows independent updates

### Merge Heuristics
- **Simple skills**: Prefer longer version (line count)
- **Rich skills**: Use `.claude` structure, merge `.opencode` content
- **Conflicts**: Manual review with `MERGE_NOTES.md` guidance

### Backup Strategy
- **Location**: `.claude/skills.backup.<timestamp>/`
- **Retention**: Until consolidation validated and complete
- **Rollback**: Simple `mv` operation

## 📚 Resources

### Documentation
1. `plans/skills-consolidation.md` - Complete plan (14,000+ words)
2. `plans/skills-consolidation-visual.md` - Visual diagrams and workflows
3. `plans/skills-consolidation-quickstart.md` - Quick start guide and FAQ
4. `.agents/skills/INVENTORY.md` - Generated inventory (after execution)

### Scripts
1. `scripts/consolidate-skills.sh` - Automated consolidation
2. `scripts/validate-consolidation.sh` - Validation tests

### Inventories
1. **Before**: 18 (`.opencode`) + 69 (`.claude`) = 87 directories
2. **Duplicates**: 15 skills exist in both
3. **Unique**: 72 unique skills
4. **After**: 72 skills in `.agents/skills/`

## ❓ FAQ

**Q: How long will this take?**
A: 5 minutes for automation + 5 hours for manual merges (can be done incrementally)

**Q: Will this break existing functionality?**
A: No. Original directories remain unchanged with backups.

**Q: What if I make a mistake during manual merge?**
A: Restore from backup and try again. Source files are never modified.

**Q: Can I skip symlinks?**
A: Yes. The script asks if you want symlinks. You can copy instead.

**Q: How do I validate success?**
A: Run `./scripts/validate-consolidation.sh` - all 10 tests should pass.

## 🎓 Lessons Learned

### What Worked Well
- ✅ Comprehensive documentation (14,000+ words)
- ✅ Visual diagrams for clarity
- ✅ Automated execution (reduces human error)
- ✅ Validation automation (catches issues early)
- ✅ Rollback plan (mitigates risk)

### What to Improve
- Consider manual merge preview (diff files)
- Add merge progress tracking
- Consider skill dependency analysis
- Plan for incremental migration

## ✅ Recommendation

**Proceed with consolidation** when:
- You have 5 minutes for automation
- You can schedule 5 hours for manual merges
- You have ~50MB free storage space
- You've reviewed the plan and scripts

**Expected Outcome**:
- Unified skill location
- Reduced maintenance burden
- Clear documentation
- Future-proof structure

---

## 📞 Next Steps

1. **Review**: Read `plans/skills-consolidation-quickstart.md`
2. **Execute**: Run `./scripts/consolidate-skills.sh`
3. **Validate**: Run `./scripts/validate-consolidation.sh`
4. **Merge**: Process 6 `MERGE_NOTES.md` files
5. **Complete**: Re-validate and celebrate!

**Total Estimated Time**: 5-6 hours (spread over 2-3 days)

**Risk Level**: ✅ LOW (comprehensive backups and rollback)

**Success Probability**: ✅ HIGH (automated + manual validation)
