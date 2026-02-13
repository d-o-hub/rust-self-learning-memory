# Skills Consolidation Plan

**Date**: 2026-02-13
**Status**: Ready for Execution
**Author**: Analysis Swarm

## Executive Summary

This plan consolidates skills from two directories into a unified `.agents/skills/` structure, preserving the best content from both sources while maintaining functionality and enabling future consolidation.

### Current State

| Directory | Skills | Structure | Avg Files/Skill |
|-----------|--------|-----------|-----------------|
| `.opencode/skill/` | 18 | Single comprehensive SKILL.md | 1 |
| `.claude/skills/` | 69 | Modular with supporting files | 1-11 |
| **Total Unique** | **72** | Mixed | **Varies** |

### Key Findings

1. **Content Richness**:
   - `.opencode`: Single comprehensive files (200-900 lines)
   - `.claude`: Entry point (40-100 lines) + supporting files

2. **Duplicate Skills** (15 exist in both):
   - `.opencode` versions are more comprehensive
   - `.claude` versions have better modular structure
   - Both have valuable content

3. **Unique Skills**:
   - `.opencode`: build-rust, memory-mcp (no, exists in .claude too), rust-code-quality
   - `.claude`: 40+ skills including agentdb-*, v3-*, codebase-*, quality-unit-testing

## 1. Consolidation Strategy

### 1.1 Core Principles

1. **Preserve Best Content**: Keep the most comprehensive version of each skill
2. **Maintain Modularity**: Use .claude's modular structure for rich skills
3. **Enable Symlinks**: Create flexible linking to both sources
4. **Future-Proof**: Design for gradual migration and deprecation

### 1.2 Merge Strategy by Type

#### Type A: Simple Skills (Both have only SKILL.md)

**Skills**: agent-coordination, clean-code-developer, code-quality, git-worktree-manager,
github-release-best-practices, perplexity-researcher-pro, perplexity-researcher-reasoning-pro

**Strategy**:
1. Compare content length and quality
2. Prefer the more comprehensive version
3. If equal length, prefer .claude (will be canonical source)

**Action**: Copy preferred SKILL.md to `.agents/skills/<skill>/SKILL.md`

#### Type B: Rich Skills (.claude has supporting files)

**Skills**:
- analysis-swarm (5 extra files)
- architecture-validation (6 extra files)
- debug-troubleshoot (5 extra files)
- feature-implement (5 extra files)
- github-workflows (5 extra files)
- goap-agent (7 extra files)

**Strategy**:
1. Use .claude's modular structure as base
2. Extract comprehensive content from .opencode SKILL.md
3. Merge content into appropriate supporting files
4. Update .claude's SKILL.md as entry point
5. Copy entire structure to `.agents/skills/<skill>/`

**Action**:
```bash
# For each rich skill
1. Copy .claude/skills/<skill>/ → .agents/skills/<skill>/
2. Read .opencode/skill/<skill>/SKILL.md
3. Merge sections into appropriate supporting files
4. Update SKILL.md entry point
```

#### Type C: Unique to .opencode

**Skills**:
- build-rust (note: .claude has build-compile, different skill)
- rust-code-quality (.claude also has this with 2 files)

**Strategy**:
1. If exists in .claude with different structure → Treat as Type B
2. If truly unique → Copy directly to `.agents/skills/`

**Action**: Copy entire directory to `.agents/skills/<skill>/`

#### Type D: Unique to .claude

**Skills**: 40+ skills including agentdb-*, v3-*, codebase-*, quality-unit-testing

**Strategy**:
1. Copy entire directory structure to `.agents/skills/`
2. Preserve all subdirectories (resources/, checklists/, etc.)
3. Maintain modular structure

**Action**: Copy entire directory to `.agents/skills/<skill>/`

## 2. Directory Structure Design

### 2.1 Target Structure

```
.agents/skills/
├── agent-coordination/SKILL.md → .claude/skills/agent-coordination/SKILL.md
├── agentdb-advanced/SKILL.md → .claude/skills/agentdb-advanced/SKILL.md
├── agentdb-learning/SKILL.md → .claude/skills/agentdb-learning/SKILL.md
├── ...
├── analysis-swarm/
│   ├── SKILL.md (merged entry point)
│   ├── discourse.md → .claude/skills/analysis-swarm/discourse.md
│   ├── examples.md → .claude/skills/analysis-swarm/examples.md
│   ├── orchestration.md → .claude/skills/analysis-swarm/orchestration.md
│   └── personas.md → .claude/skills/analysis-swarm/personas.md
├── build-rust/SKILL.md → .opencode/skill/build-rust/SKILL.md
├── ...
└── quality-unit-testing/
    ├── SKILL.md → .claude/skills/quality-unit-testing/SKILL.md
    ├── checklists/ → .claude/skills/quality-unit-testing/checklists/
    ├── reference/ → .claude/skills/quality-unit-testing/reference/
    ├── scripts/ → .claude/skills/quality-unit-testing/scripts/
    └── templates/ → .claude/skills/quality-unit-testing/templates/
```

### 2.2 File Organization Rules

1. **Entry Point**: Every skill has `SKILL.md` at root
2. **Supporting Files**: Named clearly (e.g., methodology.md, patterns.md)
3. **Subdirectories**: For complex skills (resources/, checklists/, templates/)
4. **No Orphaned Files**: All files referenced from SKILL.md

## 3. Symlink Strategy

### 3.1 Direction Decision

**Primary Direction**: `.agents/skills/<skill>/` → SOURCE FILES

**Rationale**:
1. `.agents/skills/` is the canonical, unified location
2. Source directories (`.claude/skills/`, `.opencode/skill/`) can be deprecated later
3. Symlinks point FROM canonical TO current source
4. Allows independent updates to sources before migration

### 3.2 Symlink Patterns

#### Pattern 1: Simple Skills (Single File)

```bash
# Simple symlink to source file
.agents/skills/<skill>/SKILL.md → .claude/skills/<skill>/SKILL.md
```

**Examples**:
- agent-coordination
- clean-code-developer
- code-quality
- git-worktree-manager

#### Pattern 2: Rich Skills (Multiple Files)

```bash
# Directory-level symlink for entire skill
.agents/skills/<skill>/ → .claude/skills/<skill>/
```

**Examples**:
- analysis-swarm
- architecture-validation
- debug-troubleshoot
- feature-implement
- github-workflows
- goap-agent

#### Pattern 3: Merged Skills (Custom Content)

```bash
# No symlinks - actual files in .agents/skills/<skill>/
# Source: Combined from both .opencode and .claude
```

**Rationale**: These have custom merged content, not direct symlinks

**Examples**:
- analysis-swarm (after content merge)
- goap-agent (after content merge)

#### Pattern 4: Complex Skills with Subdirectories

```bash
# Directory-level symlink preserves structure
.agents/skills/<skill>/ → .claude/skills/<skill>/
```

**Examples**:
- quality-unit-testing
- episodic-memory-testing

### 3.3 Symlink Implementation

```bash
# Create relative symlinks for portability
ln -sr ../../claude/skills/<skill> .agents/skills/<skill>

# Or absolute symlinks if preferred
ln -s /absolute/path/to/.claude/skills/<skill> .agents/skills/<skill>
```

**Recommendation**: Use relative symlinks for repository portability

## 4. Conflict Resolution Rules

### 4.1 SKILL.md Content Conflicts

When the same skill exists in both directories with different SKILL.md content:

#### Rule 1: Length Heuristic

```bash
opencode_lines=$(wc -l < .opencode/skill/<skill>/SKILL.md)
claude_lines=$(wc -l < .claude/skills/<skill>/SKILL.md)

if [ $opencode_lines -gt $((claude_lines * 2)) ]; then
    prefer=.opencode  # Significantly more comprehensive
elif [ $claude_lines -gt $((opencode_lines * 2)) ]; then
    prefer=.claude    # Significantly more comprehensive
else
    prefer=.claude    # Default to .claude (canonical source)
fi
```

#### Rule 2: Structure Heuristic

- If .claude has supporting files → Use modular structure
- Merge comprehensive .opencode content into supporting files
- Update SKILL.md as entry point

#### Rule 3: Manual Review List

Skills requiring manual comparison:
- analysis-swarm (515 vs 41 lines + 4 supporting files)
- goap-agent (893 vs 48 lines + 6 supporting files)
- memory-mcp (475 vs 67 lines + 5 supporting files)

### 4.2 Supporting File Conflicts

If both have supporting files with different content:

1. **Compare filenames**: Identify unique vs overlapping
2. **Merge content**: Combine unique sections
3. **Prefer .claude structure**: Use .claude's organization
4. **Document differences**: Create MERGE_NOTES.md if significant

## 5. Execution Plan

### 5.1 Phase 1: Preparation

```bash
# 1. Create target directory
mkdir -p .agents/skills

# 2. Create inventory
echo "# Skills Inventory" > .agents/skills/INVENTORY.md
echo "Generated: $(date)" >> .agents/skills/INVENTORY.md

# 3. Backup existing directories (optional)
# cp -r .claude/skills .claude/skills.backup
# cp -r .opencode/skill .opencode/skill.backup
```

### 5.2 Phase 2: Copy Unique Skills

```bash
# Copy skills unique to .claude
for skill in .claude/skills/*/; do
    skill_name=$(basename "$skill")
    if [ ! -d ".opencode/skill/$skill_name" ]; then
        echo "Copying unique .claude skill: $skill_name"
        cp -r "$skill" ".agents/skills/"
    fi
done

# Copy skills unique to .opencode
for skill in .opencode/skill/*/; do
    skill_name=$(basename "$skill")
    if [ ! -d ".claude/skills/$skill_name" ]; then
        echo "Copying unique .opencode skill: $skill_name"
        cp -r "$skill" ".agents/skills/"
    fi
done
```

### 5.3 Phase 3: Handle Simple Duplicates

```bash
# For simple skills (both have only SKILL.md)
simple_skills=(
    "agent-coordination"
    "clean-code-developer"
    "code-quality"
    "git-worktree-manager"
    "github-release-best-practices"
    "perplexity-researcher-pro"
    "perplexity-researcher-reasoning-pro"
)

for skill in "${simple_skills[@]}"; do
    opencode_lines=$(wc -l < ".opencode/skill/$skill/SKILL.md" 2>/dev/null || echo 0)
    claude_lines=$(wc -l < ".claude/skills/$skill/SKILL.md" 2>/dev/null || echo 0)

    mkdir -p ".agents/skills/$skill"

    if [ $opencode_lines -gt $claude_lines ]; then
        echo "Copying $skill from .opencode ($opencode_lines lines)"
        cp ".opencode/skill/$skill/SKILL.md" ".agents/skills/$skill/SKILL.md"
    else
        echo "Copying $skill from .claude ($claude_lines lines)"
        cp ".claude/skills/$skill/SKILL.md" ".agents/skills/$skill/SKILL.md"
    fi
done
```

### 5.4 Phase 4: Handle Rich Duplicates (Manual Merge)

For rich skills requiring manual merge:

```bash
# Create merge script stub
rich_skills=(
    "analysis-swarm"
    "architecture-validation"
    "debug-troubleshoot"
    "feature-implement"
    "github-workflows"
    "goap-agent"
)

for skill in "${rich_skills[@]}"; do
    echo "Processing rich skill: $skill"

    # 1. Copy .claude structure
    cp -r ".claude/skills/$skill" ".agents/skills/$skill"

    # 2. Extract content from .opencode
    # TODO: Manual merge required
    # Read .opencode/skill/$skill/SKILL.md
    # Identify sections not in .claude
    # Merge into appropriate supporting files
    # Update SKILL.md entry point

    # Create merge notes
    cat > ".agents/skills/$skill/MERGE_NOTES.md" << EOF
# Merge Notes for $skill

## Sources
- .claude/skills/$skill/ (base structure)
- .opencode/skill/$skill/SKILL.md (comprehensive content)

## Action Required
- [ ] Compare SKILL.md files
- [ ] Merge comprehensive content into supporting files
- [ ] Update entry point SKILL.md
- [ ] Test skill loading
- [ ] Remove this file when complete
EOF
done
```

### 5.5 Phase 5: Create Symlinks

```bash
# Option A: Symlink simple skills
for skill in "${simple_skills[@]}"; do
    # Determine source
    if [ -f ".agents/skills/$skill/SKILL.md" ]; then
        # Create directory structure for symlink
        source_dir="../.claude/skills/$skill"
        if [ -f ".opencode/skill/$skill/SKILL.md" ] && \
           [ $(wc -l < ".opencode/skill/$skill/SKILL.md") -gt \
             $(wc -l < ".claude/skills/$skill/SKILL.md") ]; then
            source_dir="../.opencode/skill/$skill"
        fi

        # Create symlink
        echo "Creating symlink for $skill → $source_dir"
        # ln -s "$source_dir" ".agents/skills/$skill"
    fi
done

# Option B: Symlink rich skills (after merge is complete)
for skill in "${rich_skills[@]}"; do
    if [ -f ".agents/skills/$skill/MERGE_NOTES.md" ]; then
        echo "Skipping symlink for $skill (merge pending)"
    else
        echo "Creating symlink for $skill"
        # ln -s "../.claude/skills/$skill" ".agents/skills/$skill"
    fi
done
```

### 5.6 Phase 6: Validation

```bash
# 1. Verify all skills have SKILL.md
echo "Checking for missing SKILL.md files..."
for skill_dir in .agents/skills/*/; do
    if [ ! -f "$skill_dir/SKILL.md" ]; then
        echo "ERROR: Missing SKILL.md in $skill_dir"
    fi
done

# 2. Verify symlinks
echo "Checking symlinks..."
find .agents/skills -type l | while read link; do
    if [ ! -e "$link" ]; then
        echo "ERROR: Broken symlink: $link"
    fi
done

# 3. Count skills
echo "Total skills in .agents/skills/: $(ls -d .agents/skills/*/ | wc -l)"

# 4. Generate inventory
cat > .agents/skills/INVENTORY.md << 'EOF'
# Skills Inventory

Generated: $(date)

## Summary
- Total Skills: X
- From .claude: Y
- From .opencode: Z
- Merged: N

## Skills List
EOF

for skill_dir in .agents/skills/*/; do
    skill_name=$(basename "$skill_dir")
    file_count=$(find "$skill_dir" -type f | wc -l)
    echo "- **$skill_name**: $file_count files" >> .agents/skills/INVENTORY.md
done
```

## 6. Validation Steps

### 6.1 Structural Validation

```bash
# 1. All skills have SKILL.md
find .agents/skills -mindepth 1 -maxdepth 1 -type d | while read dir; do
    if [ ! -f "$dir/SKILL.md" ]; then
        echo "FAIL: $dir missing SKILL.md"
    fi
done

# 2. No broken symlinks
find .agents/skills -type l ! -exec test -e {} \; -print

# 3. All referenced files exist
grep -h '^\[' .agents/skills/*/SKILL.md | grep -oP '\([^)]+\)' | \
    grep -oP '[^/]+\.md' | sort -u
```

### 6.2 Content Validation

```bash
# 1. Verify skill frontmatter
for skill in .agents/skills/*/SKILL.md; do
    if ! grep -q '^name:' "$skill"; then
        echo "WARNING: Missing 'name' in $skill"
    fi
    if ! grep -q '^description:' "$skill"; then
        echo "WARNING: Missing 'description' in $skill"
    fi
done

# 2. Check for obvious merge artifacts
grep -r "TODO.*merge" .agents/skills/
grep -r "MERGE_NOTES" .agents/skills/
```

### 6.3 Functional Validation

```bash
# 1. Test skill loading (if tool exists)
# skill --list

# 2. Verify skill count matches expected
expected=72
actual=$(ls -d .agents/skills/*/ | wc -l)
if [ $actual -ne $expected ]; then
    echo "WARNING: Expected $expected skills, found $actual"
fi
```

## 7. Rollback Plan

If consolidation fails:

```bash
# Restore from backup
rm -rf .claude/skills
rm -rf .opencode/skill
mv .claude/skills.backup .claude/skills
mv .opencode/skill.backup .opencode/skill

# Or remove .agents and start over
rm -rf .agents/skills
```

## 8. Migration Path (Future)

### Phase 1: Symlinks (Current)
- `.agents/skills/` → `.claude/skills/` and `.opencode/skill/`
- Maintains compatibility
- Low risk

### Phase 2: Migration (6 months)
- Copy all content to `.agents/skills/`
- Remove symlinks
- Deprecate source directories
- Update all references

### Phase 3: Cleanup (12 months)
- Remove `.claude/skills/`
- Remove `.opencode/skill/`
- `.agents/skills/` is sole source

## 9. Success Criteria

- [ ] All 72 unique skills present in `.agents/skills/`
- [ ] No broken symlinks
- [ ] All SKILL.md files have valid frontmatter
- [ ] Merge notes created for all rich skills
- [ ] Inventory document generated
- [ ] Zero content loss from source directories
- [ ] Backups created before execution

## 10. Execution Checklist

### Pre-Execution
- [ ] Review this plan with stakeholders
- [ ] Create backups of source directories
- [ ] Document any additional conflict resolution rules
- [ ] Prepare rollback procedure

### Execution
- [ ] Phase 1: Create .agents/skills/ directory
- [ ] Phase 2: Copy unique skills
- [ ] Phase 3: Handle simple duplicates
- [ ] Phase 4: Handle rich duplicates (manual merge)
- [ ] Phase 5: Create symlinks
- [ ] Phase 6: Validate structure and content

### Post-Execution
- [ ] Test skill loading functionality
- [ ] Verify all skills accessible
- [ ] Document any deviations from plan
- [ ] Create migration plan for source deprecation
- [ ] Update documentation referencing old paths

## Appendix A: Skill Classification

### Simple Skills (1 file each)
- agent-coordination
- clean-code-developer
- code-quality
- git-worktree-manager
- github-release-best-practices
- perplexity-researcher-pro
- perplexity-researcher-reasoning-pro

### Rich Skills (5+ files each)
- analysis-swarm (5)
- architecture-validation (6)
- debug-troubleshoot (5)
- episodic-memory-testing (4 + resources/)
- feature-implement (5)
- github-workflows (5)
- goap-agent (7)
- loop-agent (5)
- memory-cli-ops (4)
- quality-unit-testing (11 + subdirs)
- rust-async-testing (4 + resources/)
- skill-creator (7)
- test-optimization (4 + resources/)
- web-search-researcher (5)

### Complex Skills (with subdirectories)
- episodic-memory-testing (resources/)
- quality-unit-testing (checklists/, reference/, scripts/, templates/)
- rust-async-testing (resources/)
- test-optimization (resources/)

### Unique to .opencode
- build-rust (different from .claude's build-compile)

### Unique to .claude
- agentdb-*
- v3-*
- codebase-*
- All other skills not in .opencode

## Appendix B: Manual Merge Guidelines

For each rich skill requiring manual merge:

1. **Read both SKILL.md files completely**
2. **Identify unique sections in .opencode version**
3. **Map sections to .claude supporting files**
4. **Merge content preserving best of both**
5. **Update cross-references and links**
6. **Validate all internal links work**
7. **Test skill loading with Skill tool**
8. **Document merge decisions**

Example: `analysis-swarm`
- .opencode has comprehensive persona definitions (515 lines)
- .claude has separate personas.md file
- Merge: Extract persona sections from .opencode, enhance personas.md
