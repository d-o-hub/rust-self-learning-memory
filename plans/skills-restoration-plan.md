# Skills System Restoration Plan

**Created**: 2026-02-13
**Status**: Ready for Execution
**Priority**: P0 - Critical System Restoration
**Estimated Time**: 2 hours total (15 min discovery, 15 min rapid fix, 30 min corrective fix, 1 hr consolidation)

---

## Executive Summary

This plan provides **step-by-step instructions** to restore the skills system to full functionality. Based on the comprehensive analysis in `plans/skills-crisis-analysis.md`, we'll execute a **phased approach** that unblocks the system immediately while implementing correct long-term fixes.

**Critical Path**: Discovery → Rapid Fix → Corrective Fix → Consolidation

**Success Criteria**:
- ✅ No agent calls fail due to missing skills
- ✅ All documentation matches implementation
- ✅ Duplicate structure resolved
- ✅ No regressions introduced

---

## Phase 1: Discovery (15 minutes)

**Objective**: Gather evidence to determine correct fix strategy

**Lead**: SOCRATES (Questioning Facilitator)
**Timebox**: 15 minutes strict
**Decision Point**: Active calls found? → Yes (CRITICAL) / No (LOW)

### Step 1.1: Search for Active Agent Calls (5 minutes)

```bash
cd /home/do/rust-self-learning-memory

# Create output directory
mkdir -p plans/skills-restoration

# Search for build-compile references in code
grep -r "build-compile" . \
  --include="*.rs" \
  --include="*.md" \
  --include="*.json" \
  --include="*.yaml" \
  --include="*.yml" \
  --exclude-dir=node_modules \
  --exclude-dir=target \
  --exclude-dir=.git \
  > plans/skills-restoration/build-compile-references.txt

# Search for subagent_type calls
grep -r 'subagent_type.*build-compile' . \
  --include="*.rs" \
  --include="*.md" \
  --exclude-dir=node_modules \
  --exclude-dir=target \
  --exclude-dir=.git \
  >> plans/skills-restoration/build-compile-references.txt

# Count references
echo "=== Total References ===" >> plans/skills-restoration/build-compile-references.txt
wc -l plans/skills-restoration/build-compile-references.txt
```

**Expected Output**:
- File: `plans/skills-restoration/build-compile-references.txt`
- Count: Number of references found

### Step 1.2: Categorize References (5 minutes)

```bash
cd /home/do/rust-self-learning-memory

# Categorize by file type
echo "=== By File Type ===" >> plans/skills-restoration/build-compile-references.txt
grep -E "\.rs$" plans/skills-restoration/build-compile-references.txt | wc -l | xargs echo "Rust code:" >> plans/skills-restoration/build-compile-references.txt
grep -E "\.md$" plans/skills-restoration/build-compile-references.txt | wc -l | xargs echo "Documentation:" >> plans/skills-restoration/build-compile-references.txt
grep -E "\.(json|yaml|yml)$" plans/skills-restoration/build-compile-references.txt | wc -l | xargs echo "Config:" >> plans/skills-restoration/build-compile-references.txt

# Show top 10 files with most references
echo "=== Top 10 Files ===" >> plans/skills-restoration/build-compile-references.txt
grep -v "^===" plans/skills-restoration/build-compile-references.txt | cut -d: -f1 | sort | uniq -c | sort -rn | head -10 >> plans/skills-restoration/build-compile-references.txt
```

**Expected Output**:
- Categorization by type (code vs. docs vs. config)
- Top 10 files with references

### Step 1.3: Check for Active Calls (5 minutes)

```bash
cd /home/do/rust-self-learning-memory

# Look for actual Task tool invocations
echo "=== Active Agent Calls ===" > plans/skills-restoration/active-calls.txt
grep -r 'Task.*subagent_type.*build-compile' . \
  --include="*.rs" \
  --include="*.md" \
  --exclude-dir=node_modules \
  --exclude-dir=target \
  --exclude-dir=.git \
  >> plans/skills-restoration/active-calls.txt || echo "No active calls found" >> plans/skills-restoration/active-calls.txt

# Check if any are in actual Rust code (not just docs)
echo "=== Rust Code Calls ===" >> plans/skills-restoration/active-calls.txt
grep -r 'Task.*subagent_type.*build-compile' . \
  --include="*.rs" \
  --exclude-dir=target \
  --exclude-dir=.git \
  >> plans/skills-restoration/active-calls.txt || echo "No Rust code calls found" >> plans/skills-restoration/active-calls.txt
```

**Expected Output**:
- File: `plans/skills-restoration/active-calls.txt`
- Boolean: Active calls exist? (Yes/No)

### Step 1.4: Make Go/No-Go Decision

**Decision Matrix**:

| Evidence | Action | Priority |
|----------|--------|----------|
| **Active calls in Rust code** | Implement Rapid Fix (Option A) | **CRITICAL** |
| **References only in docs** | Implement Rapid Fix (Option B) | LOW |
| **No references found** | Implement Rapid Fix (Option B) | LOW |
| **External references found** | Expand scope, include external updates | **CRITICAL** |

**Output**:
- Decision document: `plans/skills-restoration/decision.txt`
- Selected option: A or B
- Justification: Evidence summary

---

## Phase 2: Rapid Fix (15 minutes)

**Objective**: Unblock the system immediately

**Lead**: FLASH (Rapid Innovator)
**Timebox**: 15 minutes strict
**Success**: No agent calls fail

### Option A: Active Calls Found (CRITICAL)

**Action**: Copy build-rust to build-compile as temporary unblocker

```bash
cd /home/do/rust-self-learning-memory

# Step 2.1: Copy skill file
cp .agents/skills/skill/build-rust/SKILL.md .agents/skills/build-compile/SKILL.md

# Step 2.2: Add temporary marker
sed -i '1i\
<!-- \
TODO: Temporary fix - copied from build-rust on 2026-02-13\
See: plans/skills-crisis-analysis.md\
This will be removed after corrective fix (Phase 3)\
-->\
' .agents/skills/build-compile/SKILL.md

# Step 2.3: Verify copy
ls -lh .agents/skills/build-compile/SKILL.md
echo "First 10 lines:"
head -10 .agents/skills/build-compile/SKILL.md

# Step 2.4: Test with dummy invocation
echo "Testing build-compile skill exists..."
test -f .agents/skills/build-compile/SKILL.md && echo "✅ PASS" || echo "❌ FAIL"
```

**Expected Output**:
- File exists: `.agents/skills/build-compile/SKILL.md`
- File has temporary TODO marker
- Test passes: ✅ PASS

**Validation**:
```bash
# Verify the skill is now present
grep -c "^name: build-rust" .agents/skills/build-compile/SKILL.md
# Should output: 1

# Verify TODO marker present
grep -c "TODO: Temporary fix" .agents/skills/build-compile/SKILL.md
# Should output: 1
```

### Option B: No Active Calls Found (LOW)

**Action**: Remove empty build-compile directory

```bash
cd /home/do/rust-self-learning-memory

# Step 2.1: Confirm directory is empty
ls -la .agents/skills/build-compile/
echo "=== Directory should be empty ==="

# Step 2.2: Remove directory
rmdir .agents/skills/build-compile/

# Step 2.3: Verify removal
test -d .agents/skills/build-compile/ && echo "❌ Directory still exists" || echo "✅ Directory removed"
```

**Expected Output**:
- Directory removed
- Test passes: ✅ Directory removed

**Validation**:
```bash
# Verify directory doesn't exist
test ! -d .agents/skills/build-compile/ && echo "✅ PASS" || echo "❌ FAIL"
```

### Step 2.5: Document Rapid Fix

```bash
cd /home/do/rust-self-learning-memory

# Create rapid fix record
cat > plans/skills-restoration/rapid-fix-record.txt << 'EOF'
Rapid Fix Execution Record
=========================

Date: 2026-02-13
Time: [Fill in after execution]
Option: A (Active Calls) or B (No Active Calls)
Executor: [Fill in]

Actions Taken:
--------------

[Option A]
✅ Copied build-rust/SKILL.md to build-compile/SKILL.md
✅ Added temporary TODO marker
✅ Verified file exists and is readable
✅ Tested with dummy invocation

[Option B]
✅ Confirmed build-compile directory empty
✅ Removed empty directory
✅ Verified removal successful

Results:
--------
- No agent calls fail: [Verify]
- System unblocked: [Yes/No]
- Temporary debt created: [Yes/No]

Next Steps:
-----------
1. Execute Phase 3: Corrective Fix
2. Remove temporary fix (if Option A)
3. Update all documentation
4. Validate with real agent calls

EOF

cat plans/skills-restoration/rapid-fix-record.txt
```

---

## Phase 3: Corrective Fix (30 minutes)

**Objective**: Implement correct long-term solution

**Lead**: RYAN (Methodical Analyst)
**Timebox**: 30 minutes
**Success**: No temporary fixes remain, documentation accurate

### Step 3.1: Determine Rename Intent (5 minutes)

```bash
cd /home/do/rust-self-learning-memory

# Examine git history for intent
git log --oneline --all --grep="build-rust\|build-compile" | head -20

# Check commit that performed rename
git show e734961 --stat

# Look for related commits
git log --oneline --all --since="2025-02-09" --until="2025-02-11" | grep -i build

# Decision: Was rename intentional?
echo "=== Intent Analysis ===" > plans/skills-restoration/intent-analysis.txt
echo "Commit e734961 message:" >> plans/skills-restoration/intent-analysis.txt
git log -1 --format=%B e734961 >> plans/skills-restoration/intent-analysis.txt
echo "" >> plans/skills-restoration/intent-analysis.txt
echo "Evidence for intentional rename:" >> plans/skills-restoration/intent-analysis.txt
echo "- Deleted .claude/skills/build-compile/SKILL.md" >> plans/skills-restoration/intent-analysis.txt
echo "- Added .opencode/skill/build-rust/SKILL.md" >> plans/skills-restoration/intent-analysis.txt
echo "- Added scripts/build-rust.sh" >> plans/skills-restoration/intent-analysis.txt
echo "" >> plans/skills-restoration/intent-analysis.txt
echo "Conclusion: Rename was INTENTIONAL" >> plans/skills-restoration/intent-analysis.txt
```

**Expected Output**:
- File: `plans/skills-restoration/intent-analysis.txt`
- Conclusion: Intentional or accidental

### Step 3.2: Update References to build-rust (15 minutes)

**IF Intentional Rename** (Expected):

```bash
cd /home/do/rust-self-learning-memory

# Step 3.2.1: Update AGENTS.md
echo "Updating AGENTS.md..."
sed -i 's/build-compile/build-rust/g' AGENTS.md

# Step 3.2.2: Update other documentation files
for file in $(grep -l "build-compile" plans/skills-restoration/build-compile-references.txt | grep "\.md$"); do
  echo "Updating $file..."
  sed -i 's/build-compile/build-rust/g' "$file"
done

# Step 3.2.3: Update any config files (if found)
for file in $(grep -l "build-compile" plans/skills-restoration/build-compile-references.txt | grep -E "\.(json|yaml|yml)$"); do
  echo "Updating $file..."
  sed -i 's/build-compile/build-rust/g' "$file"
done

# Step 3.2.4: Verify updates
echo "=== Verification ===" > plans/skills-restoration/update-verification.txt
grep -r "build-compile" . --include="*.md" --exclude-dir=.git | wc -l | xargs echo "Remaining build-compile references:" >> plans/skills-restoration/update-verification.txt
grep -r "build-rust" . --include="*.md" --exclude-dir=.git | wc -l | xargs echo "Total build-rust references:" >> plans/skills-restoration/update-verification.txt
```

**Expected Output**:
- AGENTS.md updated
- All doc files updated
- Verification shows few/no build-compile references remaining

**IF Accidental Rename** (Unlikely):

```bash
cd /home/do/rust-self-learning-memory

# Step 3.2.1: Restore build-compile from git history
git show e734961~1:.claude/skills/build-compile/SKILL.md > .agents/skills/build-compile/SKILL.md

# Step 3.2.2: Remove build-rust
rm -rf .agents/skills/skill/build-rust/

# Step 3.2.3: Update documentation to reference build-compile
# (Similar to above, but reversing direction)

# Step 3.2.4: Verify
echo "=== Verification ===" > plans/skills-restoration/update-verification.txt
grep -r "build-rust" . --include="*.md" --exclude-dir=.git | wc -l | xargs echo "Remaining build-rust references:" >> plans/skills-restoration/update-verification.txt
grep -r "build-compile" . --include="*.md" --exclude-dir=.git | wc -l | xargs echo "Total build-compile references:" >> plans/skills-restoration/update-verification.txt
```

### Step 3.3: Remove Temporary Fix (5 minutes)

**IF Option A was used in Phase 2**:

```bash
cd /home/do/rust-self-learning-memory

# Step 3.3.1: Remove temporary build-compile
# (If we updated all references to build-rust)
rm .agents/skills/build-compile/SKILL.md
rmdir .agents/skills/build-compile/

# Step 3.3.2: Verify removal
test -d .agents/skills/build-compile/ && echo "❌ Directory still exists" || echo "✅ Directory removed"

# Step 3.3.3: Verify no references remain
grep -r "build-compile" . --include="*.md" --exclude-dir=.git | grep -v plans/ || echo "✅ No references outside plans/"
```

**Expected Output**:
- Temporary build-compile removed
- No references remaining (except in plans/)

**IF Option B was used in Phase 2**:

```bash
# No temporary fix to remove
# Already cleaned up in Phase 2
echo "✅ No temporary fix to remove"
```

### Step 3.4: Update build-rust Documentation (5 minutes)

```bash
cd /home/do/rust-self-learning-memory

# Ensure build-rust skill references the correct script
cat .agents/skills/skill/build-rust/SKILL.md | grep -E "build-rust\.sh|cargo build"

# Add note about rename history if not present
if ! grep -q "formerly build-compile" .agents/skills/skill/build-rust/SKILL.md; then
  cat >> .agents/skills/skill/build-rust/SKILL.md << 'EOF'

## History

This skill was renamed from `build-compile` in commit e734961 to provide more specific
Rust-focused build operations with specialized CLI tooling.
EOF
fi

# Verify update
tail -10 .agents/skills/skill/build-rust/SKILL.md
```

---

## Phase 4: Consolidation (1 hour)

**Objective**: Eliminate duplicate structure and prevent future issues

**Lead**: All Personas (Swarm Coordination)
**Timebox**: 1 hour
**Success**: Clean, documented architecture

### Step 4.1: Investigate Duplicate Purpose (15 minutes)

```bash
cd /home/do/rust-self-learning-memory

# Step 4.1.1: Compare content for differences
echo "=== Content Comparison ===" > plans/skills-restoration/duplicate-analysis.txt
for skill in .agents/skills/skill/*/; do
  name=$(basename "$skill")
  if [ -f ".agents/skills/$name/SKILL.md" ]; then
    echo "Comparing: $name" >> plans/skills-restoration/duplicate-analysis.txt
    diff "$skill/SKILL.md" ".agents/skills/$name/SKILL.md" > /dev/null
    if [ $? -eq 0 ]; then
      echo "  ✅ Identical" >> plans/skills-restoration/duplicate-analysis.txt
    else
      echo "  ⚠️  DIFFERS" >> plans/skills-restoration/duplicate-analysis.txt
      echo "  Lines in parent: $(wc -l < .agents/skills/$name/SKILL.md)" >> plans/skills-restoration/duplicate-analysis.txt
      echo "  Lines in subdirectory: $(wc -l < $skill/SKILL.md)" >> plans/skills-restoration/duplicate-analysis.txt
    fi
  fi
done

# Step 4.1.2: Search for references to skill/ subdirectory
echo "" >> plans/skills-restoration/duplicate-analysis.txt
echo "=== References to skill/ subdirectory ===" >> plans/skills-restoration/duplicate-analysis.txt
grep -r "skills/skill/" . --include="*.rs" --include="*.md" --exclude-dir=.git | head -20 >> plans/skills-restoration/duplicate-analysis.txt || echo "None found" >> plans/skills-restoration/duplicate-analysis.txt

# Step 4.1.3: Check for configuration files
echo "" >> plans/skills-restoration/duplicate-analysis.txt
echo "=== Configuration Files ===" >> plans/skills-restoration/duplicate-analysis.txt
find . -name "*.json" -o -name "*.yaml" -o -name "*.yml" | xargs grep -l "skills/skill/" 2>/dev/null || echo "None found" >> plans/skills-restoration/duplicate-analysis.txt
```

**Expected Output**:
- File: `plans/skills-restoration/duplicate-analysis.txt`
- Comparison of all duplicate skills
- Any references to skill/ subdirectory
- Any configuration using skill/ subdirectory

### Step 4.2: Make Consolidation Decision (5 minutes)

**Decision Matrix**:

| Evidence | Action | Rationale |
|----------|--------|-----------|
| **All duplicates identical + no references** | Delete `.agents/skills/skill/` | Single source of truth |
| **Content differs + references exist** | Keep both, document purpose | Maintain separation |
| **Content differs + no references** | Investigate purpose first | Avoid breaking unknown feature |

**Output**: Decision document with rationale

### Step 4.3: Execute Consolidation (20 minutes)

**IF Delete Duplicates**:

```bash
cd /home/do/rust-self-learning-memory

# Step 4.3.1: Backup first
tar -czf plans/skills-restoration/skill-subdirectory-backup.tar.gz .agents/skills/skill/

# Step 4.3.2: Remove subdirectory
rm -rf .agents/skills/skill/

# Step 4.3.3: Verify
test -d .agents/skills/skill/ && echo "❌ Still exists" || echo "✅ Removed"

# Step 4.3.4: Count skills
echo "Skills remaining:"
ls -1 .agents/skills/ | grep -v "^plans$" | wc -l
```

**IF Keep Both**:

```bash
cd /home/do/rust-self-learning-memory

# Step 4.3.1: Create architecture documentation
cat > .agents/skills/skill/README.md << 'EOF'
# Skills Subdirectory

This subdirectory contains a subset of skills that are [PURPOSE - e.g., public, experimental, etc.].

## Structure

- Parent directory (`.agents/skills/`): [Description]
- Subdirectory (`skill/`): [Description]

## Rationale

[Explain why two locations exist]

## Maintenance

- Updates must be synchronized between locations
- OR: One location is authoritative, other is generated
- [Document the process]
EOF

# Step 4.3.2: Add to main README
echo "" >> .agents/skills/README.md
echo "## Skill Subdirectories" >> .agents/skills/README.md
echo "See [skill/README.md](skill/README.md) for documentation of skill subdirectories." >> .agents/skills/README.md
```

### Step 4.4: Prevent Future Issues (20 minutes)

```bash
cd /home/do/rust-self-learning-memory

# Step 4.4.1: Create migration checklist
cat > plans/skills-restoration/migration-checklist.md << 'EOF'
# Skill Migration Checklist

Use this checklist when moving or renaming skills to prevent the issues identified
in the skills crisis of 2026-02-13.

## Pre-Migration

- [ ] Document reason for migration
- [ ] Identify all references to old name/location
- [ ] Create migration plan with timeline
- [ ] Backup current state

## During Migration

- [ ] Move/rename skill file
- [ ] Update ALL references in code
- [ ] Update ALL references in documentation
- [ ] Update ALL references in configuration
- [ ] Test with real agent calls
- [ ] Verify no regressions

## Post-Migration

- [ ] Update AGENTS.md
- [ ] Update relevant README files
- [ ] Add migration note to skill file history
- [ ] Remove old location/file
- [ ] Update any external documentation
- [ ] Monitor for errors (24 hours)

## Validation

- [ ] All tests pass
- [ ] No agent call failures
- [ ] Documentation matches implementation
- [ ] No broken references

## Rollback Plan

If issues occur:
1. Revert to pre-migration state
2. Investigate root cause
3. Update migration plan
4. Retry migration
EOF

# Step 4.4.2: Create validation script
cat > scripts/validate-skills.sh << 'EOF'
#!/bin/bash
# Validate skills system integrity

echo "Skills System Validation"
echo "========================"

# Check 1: No empty skill directories
echo ""
echo "Check 1: Empty skill directories"
empty_dirs=$(find .agents/skills/ -maxdepth 1 -type d -empty | grep -v "^$")
if [ -n "$empty_dirs" ]; then
  echo "❌ FAIL: Found empty directories:"
  echo "$empty_dirs"
  exit 1
else
  echo "✅ PASS: No empty directories"
fi

# Check 2: All skill directories have SKILL.md
echo ""
echo "Check 2: Missing SKILL.md files"
missing=0
for dir in .agents/skills/*/; do
  name=$(basename "$dir")
  if [ "$name" != "skill" ] && [ ! -f "$dir/SKILL.md" ]; then
    echo "❌ FAIL: $name/SKILL.md missing"
    missing=1
  fi
done
if [ $missing -eq 0 ]; then
  echo "✅ PASS: All directories have SKILL.md"
else
  exit 1
fi

# Check 3: No duplicate structure (unless intentional)
echo ""
echo "Check 3: Duplicate skill structure"
if [ -d ".agents/skills/skill/" ]; then
  echo "⚠️  WARN: Duplicate subdirectory exists"
  echo "   Verify this is intentional per architecture docs"
else
  echo "✅ PASS: No duplicate structure"
fi

echo ""
echo "Validation complete"
EOF

chmod +x scripts/validate-skills.sh

# Step 4.4.3: Run validation
./scripts/validate-skills.sh
```

**Expected Output**:
- Migration checklist created
- Validation script created
- Validation script runs successfully

---

## Validation & Testing

### End-to-End Validation

```bash
cd /home/do/rust-self-learning-memory

# Test 1: Verify no empty skill directories
echo "Test 1: No empty directories"
for dir in .agents/skills/*/; do
  name=$(basename "$dir")
  if [ "$name" != "skill" ] && [ ! -f "$dir/SKILL.md" ]; then
    echo "❌ FAIL: $name is empty"
    exit 1
  fi
done
echo "✅ PASS"

# Test 2: Verify build-rust accessible
echo ""
echo "Test 2: build-rust skill accessible"
test -f .agents/skills/skill/build-rust/SKILL.md && echo "✅ PASS" || echo "❌ FAIL"

# Test 3: Verify no build-compile references (if intentional rename)
echo ""
echo "Test 3: No build-compile references"
remaining=$(grep -r "build-compile" . --include="*.md" --exclude-dir=.git --exclude-dir=plans | wc -l)
if [ $remaining -eq 0 ]; then
  echo "✅ PASS"
else
  echo "⚠️  WARN: $remaining references remain (review needed)"
fi

# Test 4: Verify duplicate structure resolved
echo ""
echo "Test 4: Duplicate structure resolved"
if [ -d ".agents/skills/skill/" ]; then
  echo "⚠️  WARN: Duplicate subdirectory still exists"
  echo "   Verify this is intentional"
else
  echo "✅ PASS: No duplicate structure"
fi

# Test 5: Run validation script
echo ""
echo "Test 5: Validation script"
./scripts/validate-skills.sh || echo "❌ FAIL: Validation script failed"
```

### Documentation Verification

```bash
cd /home/do/rust-self-learning-memory

# Verify AGENTS.md matches implementation
echo "Checking AGENTS.md..."
grep -q "build-rust" AGENTS.md && echo "✅ AGENTS.md mentions build-rust" || echo "⚠️  AGENTS.md may need update"

# Verify no orphaned references
echo ""
echo "Checking for orphaned references..."
grep -r "build-compile" . --include="*.md" --exclude-dir=.git --exclude-dir=plans | head -5
```

---

## Monitoring Plan

### Immediate Monitoring (First 24 Hours)

```bash
# Create monitoring script
cat > plans/skills-restoration/monitor-first-24h.sh << 'EOF'
#!/bin/bash
# Monitor skills system for first 24 hours

echo "Skills System Monitoring - First 24 Hours"
echo "========================================="
echo "Start: $(date)"
echo ""

# Check 1: Error logs
echo "Check 1: Recent error logs"
journalctl -u "*agent*" --since "1 hour ago" | grep -i "skill\|build" || echo "No recent errors"
echo ""

# Check 2: Agent call success rate
echo "Check 2: Agent calls (if metrics available)"
# [Add actual metrics collection when available]
echo ""

# Check 3: File system
echo "Check 3: File system integrity"
./scripts/validate-skills.sh
echo ""

echo "End: $(date)"
EOF

chmod +x plans/skills-restoration/monitor-first-24h.sh

# Set up cron job (if desired)
# crontab -e
# Add: 0 * * * * /home/do/rust-self-learning-memory/plans/skills-restoration/monitor-first-24h.sh >> /tmp/skills-monitor.log 2>&1
```

### Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| No empty skill directories | 0 | ⬜ Verify |
| All agent calls succeed | 100% | ⬜ Verify |
| Documentation accuracy | 100% | ⬜ Verify |
| No regressions | 0 issues | ⬜ Verify |
| Duplicate structure resolved | Yes/No | ⬜ Decide |

---

## Rollback Plan

### If Critical Issues Occur

```bash
cd /home/do/rust-self-learning-memory

# Rollback Step 1: Restore from backup (if created)
if [ -f "plans/skills-restoration/skill-subdirectory-backup.tar.gz" ]; then
  echo "Restoring from backup..."
  tar -xzf plans/skills-restoration/skill-subdirectory-backup.tar.gz
fi

# Rollback Step 2: Restore build-compile (if deleted)
if [ ! -d ".agents/skills/build-compile/" ]; then
  echo "Restoring build-compile..."
  git show e734961~1:.claude/skills/build-compile/SKILL.md > .agents/skills/build-compile/SKILL.md
fi

# Rollback Step 3: Revert documentation changes
git checkout HEAD -- AGENTS.md
# (Or restore from backup)

# Rollback Step 4: Verify
./scripts/validate-skills.sh
```

### Rollback Decision Criteria

Rollback if:
- 🔴 **CRITICAL**: Agent system completely broken
- 🔴 **CRITICAL**: >50% of agent calls fail
- 🟡 **WARNING**: Unexpected errors in production
- 🟡 **WARNING**: Performance degradation >50%

Do NOT rollback for:
- Minor documentation issues
- Non-critical path failures
- Expected teething problems

---

## Execution Timeline

### Summary

| Phase | Duration | Lead | Success Criteria |
|-------|----------|------|------------------|
| Phase 1: Discovery | 15 min | SOCRATES | Evidence gathered, decision made |
| Phase 2: Rapid Fix | 15 min | FLASH | System unblocked |
| Phase 3: Corrective Fix | 30 min | RYAN | No temporary fixes remain |
| Phase 4: Consolidation | 60 min | All | Clean architecture |
| **Total** | **2 hours** | | **System restored** |

### Prerequisites

Before starting:
- ✅ Read `plans/skills-crisis-analysis.md`
- ✅ Verify write access to codebase
- ✅ Create git branch for restoration work
- ✅ Ensure backup system available

### Post-Execution

After completing all phases:
- ✅ Update this plan with actual execution times
- ✅ Document any deviations from plan
- ✅ Create follow-up issues for any remaining work
- ✅ Share results with team

---

## Appendix: Quick Reference

### Common Commands

```bash
# Find all skill directories
find .agents/skills/ -maxdepth 1 -type d | sort

# Check for empty directories
find .agents/skills/ -maxdepth 1 -type d -empty

# Count skills with SKILL.md
find .agents/skills/ -maxdepth 2 -name "SKILL.md" | wc -l

# Search for references
grep -r "build-compile" . --include="*.md" --exclude-dir=.git

# Validate skills system
./scripts/validate-skills.sh

# Compare duplicate skills
diff .agents/skills/agent-coordination/SKILL.md .agents/skills/skill/agent-coordination/SKILL.md
```

### Decision Trees

```
Start: Discovery
│
├─ Active calls found?
│  ├─ Yes → Option A (Copy build-rust → build-compile)
│  └─ No  → Option B (Remove empty build-compile)
│
├─ Rename intentional?
│  ├─ Yes → Update all references to build-rust
│  └─ No  → Revert to build-compile
│
└─ Duplicates identical?
   ├─ Yes → Delete .agents/skills/skill/
   └─ No  → Document purpose, keep both
```

### Contact Points

For questions during execution:
- **RYAN**: Architecture questions, validation, risk assessment
- **FLASH**: Rapid fixes, prioritization, timebox decisions
- **SOCRATES**: Decision clarification, evidence gathering, facilitating consensus

---

**Plan Status**: Ready for Execution
**Next Action**: Execute Phase 1 (Discovery)
**Owner**: Development Team
**Review Date**: After execution completion

**Remember**: The goal is to unblock the system immediately while implementing correct long-term fixes. Use the phased approach to balance speed with correctness.
