# Playwright Skill Investigation and Resolution Plan

**Created**: 2026-02-13
**Version**: 1.0
**Status**: Active
**Method**: Goal-Oriented Action Planning (GOAP)
**Priority**: P0 (Critical - breaks skill loading)

---

## Executive Summary

**Issue**: Invalid skill folder name `playwright-cli ` (with trailing space) causing skill loading failures and path resolution issues.

**Impact**: Skill loading system cannot properly locate or load the playwright-cli skill, breaking browser automation capabilities.

**Root Cause**: Directory created with trailing space character, likely due to shell completion or copy-paste error.

**Resolution Strategy**: Atomic rename operation with comprehensive validation and regression testing.

**Estimated Effort**: 3-5 hours

---

## 1. World State Analysis

### Current State (2026-02-13)

| Dimension | State | Details |
|-----------|-------|---------|
| **Skill Directory** | ❌ Invalid | `.agents/skills/playwright-cli ` (trailing space) |
| **SKILL.md Location** | ⚠️ Misleading | Shows as `playwright-cli /` in available_skills |
| **Skill Loading** | ❌ Broken | Path resolution fails for skill system |
| **References** | ⚠️ Unknown | Need to audit all references |
| **Validation** | ❌ Missing | No pre-commit hooks for directory naming |
| **Browser Automation** | ❌ Unavailable | Feature completely broken |

### Affected Systems

1. **Skill Loading**: Cannot locate playwright-cli skill
2. **CLI Invocation**: Bash tool paths fail resolution
3. **Documentation**: Misleading location path shown to users
4. **CI/CD**: Potential failures in skill validation checks
5. **Agent Coordination**: Agents cannot use browser automation

---

## 2. Goal State

### Target State (Post-Resolution)

| Goal | Measurable Outcome |
|------|-------------------|
| **Valid Directory Name** | `.agents/skills/playwright-cli` (no trailing space) |
| **Correct Path References** | All references updated to `playwright-cli` |
| **Skill Loading Functional** | Skill system loads playwright-cli successfully |
| **Zero Broken References** | 0 references to old path in codebase |
| **Prevention Measures** | Git hooks prevent future invalid directory names |
| **Browser Automation Restored** | All playwright-cli commands functional |

---

## 3. Gap Analysis

| Gap | Current | Target | Effort | Priority |
|-----|---------|--------|--------|----------|
| Invalid directory name | `playwright-cli ` | `playwright-cli` | 0.5h | P0 |
| Path references in codebase | Unknown | All updated | 1-2h | P0 |
| SKILL.md location field | `playwright-cli /` | `playwright-cli` | 0.5h | P0 |
| Documentation references | Unknown | All updated | 0.5h | P0 |
| Git validation hooks | None | Implemented | 1h | P1 |
| Regression testing | None | Comprehensive | 0.5h | P1 |

**Total Estimated Effort**: 3-5 hours

---

## 4. Research and Investigation Phase

### Phase 1.1: Root Cause Analysis (30 minutes)

**Objectives**:
- Identify when and how the invalid directory was created
- Check git history for the creation event
- Determine if this is an isolated incident or systemic issue

**Actions**:
```bash
# Check git history for the directory
git log --follow --all -- ".agents/skills/playwright-cli /"
git log --follow --all -- ".agents/skills/playwright-cli "

# Check when the SKILL.md was created
git log --diff-filter=A --find-renames=100% -- ".agents/skills/playwright*"

# Search for any other directories with trailing spaces
find .agents/skills -type d -name "* " -o -name " *" 2>/dev/null
```

**Success Criteria**:
- [ ] Root cause identified (human error, script bug, or tool issue)
- [ ] Timeline of creation documented
- [ ] Confirmation of isolated vs. systemic issue

**Deliverables**:
- Investigation report (`plans/playwright-skill-investigation-report.md`)

---

### Phase 1.2: Impact Assessment (45 minutes)

**Objectives**:
- Identify all code references to the invalid path
- Check for broken imports or usages
- Assess impact on agent workflows

**Actions**:
```bash
# Search for references in code
grep -r "playwright-cli " . --include="*.rs" --include="*.md" --include="*.yaml" --include="*.yml"

# Search for references in documentation
grep -r "playwright-cli" docs/ plans/ --include="*.md"

# Check skill loading logic
grep -r "load.*skill" .agents/ --include="*.rs" --include="*.ts" --include="*.js"

# Search for tool invocations
grep -r "playwright-cli:" . --include="*.md"
```

**Success Criteria**:
- [ ] All file references catalogued
- [ ] Broken usages identified
- [ ] Impact scope documented

**Deliverables**:
- Impact assessment report with file list and severity ratings

---

## 5. Resolution Phase

### Phase 2.1: Atomic Directory Rename (30 minutes)

**Objectives**:
- Safely rename directory without data loss
- Maintain git history continuity
- Ensure atomic operation (no partial state)

**Actions**:
```bash
# Step 1: Verify current state
ls -la .agents/skills/ | grep playwright
git status

# Step 2: Create backup (safety measure)
cp -r ".agents/skills/playwright-cli " ".agents/skills/playwright-cli.backup "

# Step 3: Atomic rename using git mv (preserves history)
cd .agents/skills
git mv "playwright-cli " playwright-cli

# Step 4: Verify the rename
ls -la .agents/skills/playwright-cli/
git status

# Step 5: Remove backup once verified
rm -rf ".agents/skills/playwright-cli.backup "
```

**Acceptance Criteria**:
- [ ] Directory renamed to `playwright-cli` (no trailing space)
- [ ] Git history preserved via `git mv`
- [ ] All files intact in new location
- [ ] No git merge conflicts introduced

**Quality Gates**:
- [ ] `git status` shows clean rename
- [ ] `ls -la` confirms valid directory name
- [ ] `git log` shows continuous history

---

### Phase 2.2: Update All References (1-2 hours)

**Objectives**:
- Update all code references to new path
- Update documentation
- Update SKILL.md location fields
- Update any configuration files

**Actions**:

#### 2.2.1: Update SKILL.md Location Field

**File**: `.agents/skills/playwright-cli/SKILL.md`
- **Current**: Location field shows invalid path
- **Target**: Update to `file:///home/do/rust-self-learning-memory/.agents/skills/playwright-cli/SKILL.md`

```yaml
# In available_skills listing, update location from:
location: file:///home/do/rust-self-learning-memory/.agents/skills/playwright-cli /SKILL.md
# To:
location: file:///home/do/rust-self-learning-memory/.agents/skills/playwright-cli/SKILL.md
```

#### 2.2.2: Update Code References

Search and replace in all Rust, TypeScript, and JavaScript files:
```bash
# Find all occurrences
grep -rn "playwright-cli /" --include="*.rs" --include="*.ts" --include="*.js"

# Replace using sed (dry run first)
sed --dry-run 's|playwright-cli /|playwright-cli/|g' [files]

# Actual replacement
sed -i 's|playwright-cli /|playwright-cli/|g' [files]
```

#### 2.2.3: Update Documentation

Update all markdown documentation:
```bash
# Find all references
grep -rn "playwright-cli " docs/ plans/ --include="*.md"

# Update references
sed -i 's|playwright-cli |playwright-cli|g' docs/**/*.md plans/**/*.md
```

#### 2.2.4: Update Configuration Files

Check and update any YAML/config files:
```bash
# Search configs
grep -rn "playwright-cli " . --include="*.yaml" --include="*.yml" --include="*.toml"
```

**Acceptance Criteria**:
- [ ] 0 references to old path `playwright-cli ` or `playwright-cli /`
- [ ] All references point to `playwright-cli` (no trailing space)
- [ ] Documentation updated consistently
- [ ] Configuration files updated

**Quality Gates**:
- [ ] `grep -r "playwright-cli /" .` returns no results
- [ ] `grep -r "playwright-cli " .` returns no results (except valid usage)
- [ ] All modified files pass formatting/linting

---

## 6. Validation Phase

### Phase 3.1: Functional Testing (30 minutes)

**Objectives**:
- Verify skill loading works
- Test skill invocation
- Validate browser automation functionality

**Test Plan**:

#### Test 3.1.1: Skill Loading
```bash
# Test skill can be loaded by the system
# (Implementation depends on skill loading mechanism)

# Expected: Skill loads without errors
# Actual: [Record result]
```

#### Test 3.1.2: Skill Listing
```bash
# Test skill appears correctly in available_skills
grep -A 2 "name: playwright-cli" .agents/skills/playwright-cli/SKILL.md

# Expected: Clean skill name and location
# Actual: [Record result]
```

#### Test 3.1.3: Documentation Path
```bash
# Verify location field in skill metadata
# Expected: file:///home/do/rust-self-learning-memory/.agents/skills/playwright-cli/SKILL.md
# Actual: [Record result]
```

#### Test 3.1.4: CLI Tool Access
```bash
# Test that playwright-cli commands can be invoked
# Example: Try to load the skill in a test invocation

# Expected: Skill loads and commands are available
# Actual: [Record result]
```

**Success Criteria**:
- [ ] Skill loads without path resolution errors
- [ ] Location field shows correct path
- [ ] All playwright-cli commands are accessible
- [ ] No console errors or warnings about invalid paths

**Deliverables**:
- Test execution report (`plans/playwright-skill-test-report.md`)

---

### Phase 3.2: Regression Prevention (1 hour)

**Objectives**:
- Implement git hooks to prevent invalid directory names
- Add validation to skill creation process
- Update documentation with best practices

#### 3.2.1: Git Pre-Commit Hook

**File**: `.git/hooks/pre-commit` (or `.husky/pre-commit` if using husky)

```bash
#!/bin/bash
# Check for directories with trailing spaces
if find .agents/skills -type d -name "* " -o -name " *" 2>/dev/null | grep -q .; then
  echo "❌ Error: Found directories with trailing spaces in .agents/skills/"
  echo "Please rename directories to remove trailing spaces."
  find .agents/skills -type d -name "* " -o -name " *"
  exit 1
fi
```

**Installation**:
```bash
# Make executable
chmod +x .git/hooks/pre-commit

# Or if using husky (recommended for teams)
npx husky set .husky/pre-commit '. "$(dirname "$0")_/validate-skill-dirs.sh"'
```

#### 3.2.2: Validation Script

**File**: `.agents/scripts/validate-skill-dirs.sh`

```bash
#!/bin/bash
# Validate skill directory names

SKILLS_DIR=".agents/skills"
ERRORS=0

# Check for directories with trailing spaces
echo "Checking for invalid directory names in $SKILLS_DIR..."

if find "$SKILLS_DIR" -type d -name "* " -o -name " *" 2>/dev/null | grep -q .; then
  echo "❌ Found directories with trailing spaces:"
  find "$SKILLS_DIR" -type d -name "* " -o -name " *"
  ERRORS=$((ERRORS + 1))
fi

# Check for directories with leading spaces
if find "$SKILLS_DIR" -maxdepth 1 -type d -name " *" 2>/dev/null | grep -q .; then
  echo "❌ Found directories with leading spaces:"
  find "$SKILLS_DIR" -maxdepth 1 -type d -name " *"
  ERRORS=$((ERRORS + 1))
fi

# Check for invalid characters (only alphanumeric, hyphen, underscore allowed)
for dir in "$SKILLS_DIR"/*/; do
  dirname=$(basename "$dir")
  if [[ ! "$dirname" =~ ^[a-zA-Z0-9_-]+$ ]]; then
    echo "❌ Invalid directory name: $dirname (only alphanumeric, hyphen, underscore allowed)"
    ERRORS=$((ERRORS + 1))
  fi
done

if [ $ERRORS -gt 0 ]; then
  echo ""
  echo "❌ Validation failed with $ERRORS error(s)"
  exit 1
fi

echo "✅ All skill directory names are valid"
exit 0
```

#### 3.2.3: Update Documentation

**File**: `AGENT_SKILL_GUIDELINES.md` (create if not exists)

```markdown
## Skill Directory Naming Guidelines

### Allowed Characters
- Alphanumeric: a-z, A-Z, 0-9
- Hyphen: -
- Underscore: _

### Forbidden Patterns
- ❌ Trailing spaces: `skill-name ` (will cause path resolution issues)
- ❌ Leading spaces: ` skill-name` (will cause loading issues)
- ❌ Special characters: `skill@name`, `skill.name`, etc.
- ❌ Case sensitivity issues: Avoid names that differ only in case

### Examples
✅ Good:
- `playwright-cli`
- `github-workflows`
- `v3-core-implementation`

❌ Bad:
- `playwright-cli ` (trailing space)
- ` playwright-cli` (leading space)
- `playwright-cli@v2` (special character)
- `Playwright-CLI` (inconsistent case)

### Validation
Run validation script before committing:
```bash
bash .agents/scripts/validate-skill-dirs.sh
```
```

**Acceptance Criteria**:
- [ ] Git hook prevents commits with invalid directory names
- [ ] Validation script checks for all invalid patterns
- [ ] Documentation updated with clear guidelines
- [ ] CI/CD pipeline includes validation check

**Quality Gates**:
- [ ] Attempt to create `test-dir ` fails pre-commit hook
- [ ] Validation script passes on current state
- [ ] Documentation reviewed and approved

---

## 7. Cleanup and Documentation Phase

### Phase 4.1: Git Commit and Cleanup (30 minutes)

**Objectives**:
- Create atomic commit for the fix
- Update CHANGELOG
- Create summary report

**Actions**:

#### 4.1.1: Commit the Fix

```bash
# Stage all changes
git add .

# Create atomic commit
git commit -m "fix(skills): rename playwright-cli directory to remove trailing space

- Rename .agents/skills/playwright-cli (trailing space) to playwright-cli
- Update all path references in documentation and code
- Add pre-commit hook to prevent invalid directory names
- Add validation script for skill directory naming

Fixes: Invalid skill folder name causing loading failures
Breaking: None (backward compatible)
"

# Verify commit
git log -1 --stat
```

#### 4.1.2: Update CHANGELOG

**File**: `CHANGELOG.md`

```markdown
## [Unreleased]

### Fixed
- **skills**: Rename playwright-cli directory to remove trailing space (#XXX)
  - Fixes skill loading failures due to invalid directory name
  - Adds pre-commit validation for skill directory names
  - Updates all path references across codebase
```

#### 4.1.3: Create Summary Report

**File**: `plans/playwright-skill-resolution-summary.md`

Include:
- Root cause analysis
- Impact assessment
- Changes made
- Test results
- Prevention measures implemented

**Acceptance Criteria**:
- [ ] Atomic commit created with clear message
- [ ] CHANGELOG updated
- [ ] Summary report complete
- [ ] No merge conflicts introduced

---

## 8. Success Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| **Directory Name Valid** | ❌ Invalid (trailing space) | ✅ Valid | `ls -la .agents/skills/` |
| **Skill Loading** | ❌ Fails | ✅ Works | Load skill successfully |
| **Path References** | ❌ Broken | ✅ All updated | `grep -r "playwright-cli /"` returns 0 |
| **Git History** | ⚠️ Unknown | ✅ Preserved | `git log` shows continuous history |
| **Prevention Measures** | ❌ None | ✅ Implemented | Pre-commit hook active |
| **Test Coverage** | ⚠️ Unknown | ✅ 100% | All tests pass |

---

## 9. Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| **Git history breakage** | High | Low | Use `git mv` for atomic rename |
| **Missed references** | Medium | Medium | Comprehensive grep search + code review |
| **Skill system regression** | High | Low | Functional testing before commit |
| **Merge conflicts** | Medium | Low | Atomic commit in feature branch |
| **Pre-commit hook bypass** | Low | Medium | Add CI check as backup |
| **Data loss during rename** | Critical | Very Low | Backup created before rename |

**Contingency Plan**:
1. If issues arise, revert commit and restore from backup
2. Rollback procedure documented in summary report
3. Feature branch used to isolate changes until verified

---

## 10. Execution Timeline

```
Phase 1: Investigation (Day 1, Morning)
├── 1.1: Root Cause Analysis (30 min)
└── 1.2: Impact Assessment (45 min)

Phase 2: Resolution (Day 1, Afternoon)
├── 2.1: Directory Rename (30 min)
└── 2.2: Update References (1-2h)

Phase 3: Validation (Day 2, Morning)
├── 3.1: Functional Testing (30 min)
└── 3.2: Regression Prevention (1h)

Phase 4: Cleanup (Day 2, Afternoon)
└── 4.1: Commit and Documentation (30 min)

Total Timeline: 1.5-2 days (3-5 hours actual work)
```

---

## 11. Dependencies and Prerequisites

### Required
- [ ] Git access to repository
- [ ] Bash shell access
- [ ] Write permissions to `.agents/skills/`
- [ ] Ability to run test suite

### Recommended
- [ ] Feature branch for isolation
- [ ] Code reviewer available
- [ ] CI/CD pipeline access for hook installation

### External Dependencies
- None (self-contained fix)

---

## 12. Quality Gates

### Per-Phase Gates

**Phase 1 (Investigation)**:
- [ ] Root cause identified
- [ ] All references catalogued
- [ ] Impact documented

**Phase 2 (Resolution)**:
- [ ] Directory renamed successfully
- [ ] All references updated
- [ ] Zero broken paths remain

**Phase 3 (Validation)**:
- [ ] Skill loading works
- [ ] All tests pass
- [ ] Prevention measures implemented

**Phase 4 (Cleanup)**:
- [ ] Clean atomic commit
- [ ] Documentation updated
- [ ] Summary report complete

### Release Gates

- [ ] All phases complete
- [ ] Zero references to old path
- [ ] All tests pass (`cargo test --all`)
- [ ] Zero clippy warnings
- [ ] Code formatted (`cargo fmt --all`)
- [ ] Pre-commit hook active
- [ ] CHANGELOG updated

---

## 13. Handoff and Coordination

### Stakeholders
- **Development Team**: Implementation and testing
- **Documentation Team**: Update guidelines
- **DevOps Team**: CI/CD hook installation
- **Code Reviewers**: Review atomic commit

### Communication Plan
1. **Start**: Notify team of investigation phase
2. **Resolution**: Share draft plan for feedback
3. **Testing**: Invite team to validate fix
4. **Completion**: Share summary report and lessons learned

---

## 14. Post-Resolution Actions

### Immediate (After Merge)
1. **Monitor**: Watch for any skill loading issues in logs
2. **Verify**: Confirm all agent workflows working
3. **Celebrate**: Document successful resolution

### Short-term (1-2 weeks)
1. **Audit**: Check for other similar issues (leading spaces, special chars)
2. **Training**: Share lessons learned with team
3. **Process**: Incorporate guidelines into onboarding

### Long-term (1-3 months)
1. **Review**: Evaluate effectiveness of prevention measures
2. **Enhance**: Consider additional validation (e.g., linting tool)
3. **Automate**: Expand validation to cover more edge cases

---

## 15. Appendix

### A. Commands Reference

```bash
# Check for trailing spaces in directories
ls -la .agents/skills/ | grep " "

# Find all invalid directories
find .agents/skills -type d -name "* " -o -name " *" 2>/dev/null

# Search for old path references
grep -rn "playwright-cli /" . --include="*.rs" --include="*.md" --include="*.yaml"

# Atomic rename with git
git mv "playwright-cli " playwright-cli

# Run validation
bash .agents/scripts/validate-skill-dirs.sh

# Test skill loading (implementation-dependent)
# [Insert skill loading test command]
```

### B. Related Documents

- `plans/GOAP_EXECUTION_PLAN_2026-02-12.md` - GOAP methodology reference
- `AGENT_SKILL_GUIDELINES.md` - Skill creation guidelines (to be created)
- `CHANGELOG.md` - Project changelog
- `.agents/scripts/validate-skill-dirs.sh` - Validation script (to be created)

### C. Contact and Support

**Plan Author**: General Agent (system)
**Questions**: Refer to `AGENTS.md` for coordination patterns
**Issues**: Create GitHub issue with label `skill-validation`

---

*End of Playwright Skill Investigation and Resolution Plan*

*Generated by GOAP Agent System on 2026-02-13*
*Following Goal-Oriented Action Planning methodology (ADR-022)*
