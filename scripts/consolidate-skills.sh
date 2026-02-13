#!/usr/bin/env bash
# Skills Consolidation Execution Script
# Source: plans/skills-consolidation.md
# Created: 2026-02-13

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
AGENTS_DIR=".agents/skills"
CLAUDE_DIR=".claude/skills"
OPENCODE_DIR=".opencode/skill"
BACKUP_SUFFIX=".backup.$(date +%Y%m%d_%H%M%S)"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Phase 1: Preparation
phase1_prepare() {
    log_info "Phase 1: Preparation"

    # Create target directory
    if [ -d "$AGENTS_DIR" ]; then
        log_warning "$AGENTS_DIR already exists"
        read -p "Remove and recreate? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$AGENTS_DIR"
        else
            log_error "Aborting due to existing directory"
            exit 1
        fi
    fi

    mkdir -p "$AGENTS_DIR"
    log_success "Created $AGENTS_DIR"

    # Create backup
    log_info "Creating backups..."
    if [ -d "$CLAUDE_DIR" ] && [ ! -d "${CLAUDE_DIR}${BACKUP_SUFFIX}" ]; then
        cp -r "$CLAUDE_DIR" "${CLAUDE_DIR}${BACKUP_SUFFIX}"
        log_success "Backed up $CLAUDE_DIR"
    fi

    if [ -d "$OPENCODE_DIR" ] && [ ! -d "${OPENCODE_DIR}${BACKUP_SUFFIX}" ]; then
        cp -r "$OPENCODE_DIR" "${OPENCODE_DIR}${BACKUP_SUFFIX}"
        log_success "Backed up $OPENCODE_DIR"
    fi

    # Initialize inventory
    cat > "$AGENTS_DIR/INVENTORY.md" << EOF
# Skills Inventory

Generated: $(date)

## Summary
- Total Skills:
- From .claude:
- From .opencode:
- Merged:

## Execution Log
EOF
}

# Phase 2: Copy Unique Skills
phase2_copy_unique() {
    log_info "Phase 2: Copying Unique Skills"

    claude_count=0
    opencode_count=0

    # Copy skills unique to .claude
    log_info "Copying skills unique to .claude..."
    for skill in "$CLAUDE_DIR"/*/; do
        skill_name=$(basename "$skill")
        if [ ! -d "$OPENCODE_DIR/$skill_name" ]; then
            log_info "  - $skill_name"
            cp -r "$skill" "$AGENTS_DIR/"
            ((claude_count++))
        fi
    done
    log_success "Copied $claude_count skills from .claude"

    # Copy skills unique to .opencode
    log_info "Copying skills unique to .opencode..."
    for skill in "$OPENCODE_DIR"/*/; do
        skill_name=$(basename "$skill")
        if [ ! -d "$CLAUDE_DIR/$skill_name" ]; then
            log_info "  - $skill_name"
            cp -r "$skill" "$AGENTS_DIR/"
            ((opencode_count++))
        fi
    done
    log_success "Copied $opencode_count skills from .opencode"

    # Update inventory
    sed -i "/From .claude:/s/-/$claude_count/" "$AGENTS_DIR/INVENTORY.md"
    sed -i "/From .opencode:/s/-/$opencode_count/" "$AGENTS_DIR/INVENTORY.md"
}

# Phase 3: Handle Simple Duplicates
phase3_simple_duplicates() {
    log_info "Phase 3: Handling Simple Duplicates"

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
        # Check if both exist
        if [ ! -f "$OPENCODE_DIR/$skill/SKILL.md" ] || [ ! -f "$CLAUDE_DIR/$skill/SKILL.md" ]; then
            log_warning "Skipping $skill (not in both directories)"
            continue
        fi

        opencode_lines=$(wc -l < "$OPENCODE_DIR/$skill/SKILL.md" 2>/dev/null || echo 0)
        claude_lines=$(wc -l < "$CLAUDE_DIR/$skill/SKILL.md" 2>/dev/null || echo 0)

        mkdir -p "$AGENTS_DIR/$skill"

        if [ "$opencode_lines" -gt "$claude_lines" ]; then
            log_info "  - $skill: .opencode ($opencode_lines > $claude_lines lines)"
            cp "$OPENCODE_DIR/$skill/SKILL.md" "$AGENTS_DIR/$skill/SKILL.md"
        else
            log_info "  - $skill: .claude ($claude_lines >= $opencode_lines lines)"
            cp "$CLAUDE_DIR/$skill/SKILL.md" "$AGENTS_DIR/$skill/SKILL.md"
        fi
    done

    log_success "Processed ${#simple_skills[@]} simple duplicate skills"
}

# Phase 4: Handle Rich Duplicates
phase4_rich_duplicates() {
    log_info "Phase 4: Handling Rich Duplicates (Requires Manual Merge)"

    rich_skills=(
        "analysis-swarm"
        "architecture-validation"
        "debug-troubleshoot"
        "feature-implement"
        "github-workflows"
        "goap-agent"
    )

    for skill in "${rich_skills[@]}"; do
        # Check if both exist
        if [ ! -d "$CLAUDE_DIR/$skill" ]; then
            log_warning "Skipping $skill (not in .claude)"
            continue
        fi

        log_info "  - $skill"

        # Copy .claude structure as base
        cp -r "$CLAUDE_DIR/$skill" "$AGENTS_DIR/$skill"

        # Count files in each
        claude_files=$(find "$CLAUDE_DIR/$skill" -name "*.md" | wc -l)
        opencode_lines=$(wc -l < "$OPENCODE_DIR/$skill/SKILL.md" 2>/dev/null || echo 0)

        # Create merge notes
        cat > "$AGENTS_DIR/$skill/MERGE_NOTES.md" << EOF
# Merge Notes for $skill

## Sources
- **Base**: .claude/skills/$skill/ ($claude_files files)
- **Additional**: .opencode/skill/$skill/SKILL.md ($opencode_lines lines)

## Comparison
\`\`\`bash
# Line counts
wc -l .opencode/skill/$skill/SKILL.md
wc -l .claude/skills/$skill/*.md

# Preview differences
diff -u .claude/skills/$skill/SKILL.md .opencode/skill/$skill/SKILL.md
\`\`\`

## Action Required
- [ ] Compare SKILL.md files
- [ ] Extract unique sections from .opencode version
- [ ] Merge into appropriate supporting files
- [ ] Update entry point SKILL.md
- [ ] Verify all internal links work
- [ ] Test skill loading
- [ ] Remove this file when complete

## Files in This Skill
EOF

        # List files
        find "$AGENTS_DIR/$skill" -name "*.md" -type f | while read -r file; do
            echo "- $(basename "$file")" >> "$AGENTS_DIR/$skill/MERGE_NOTES.md"
        done

        log_warning "    Created MERGE_NOTES.md for $skill"
    done

    log_success "Processed ${#rich_skills[@]} rich duplicate skills"
    log_warning "MANUAL MERGE REQUIRED for these skills"
}

# Phase 5: Create Symlinks
phase5_symlinks() {
    log_info "Phase 5: Creating Symlinks"

    # Ask user if they want symlinks
    read -p "Create symlinks to source files? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_warning "Skipping symlink creation"
        return
    fi

    # For simple skills, create file-level symlinks
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
        # Determine source
        if [ ! -f "$AGENTS_DIR/$skill/SKILL.md" ]; then
            continue
        fi

        # Check if it's a copy or already symlink
        if [ -L "$AGENTS_DIR/$skill/SKILL.md" ]; then
            log_warning "  Skipping $skill (already symlink)"
            continue
        fi

        # Determine source by comparing line counts
        opencode_lines=$(wc -l < "$OPENCODE_DIR/$skill/SKILL.md" 2>/dev/null || echo 0)
        claude_lines=$(wc -l < "$CLAUDE_DIR/$skill/SKILL.md" 2>/dev/null || echo 0)

        if [ "$opencode_lines" -gt "$claude_lines" ]; then
            source="$OPENCODE_DIR/$skill/SKILL.md"
        else
            source="$CLAUDE_DIR/$skill/SKILL.md"
        fi

        # Remove copy and create symlink
        rm "$AGENTS_DIR/$skill/SKILL.md"
        ln -s "$source" "$AGENTS_DIR/$skill/SKILL.md"
        log_info "  - $skill → $source"
    done

    log_success "Created symlinks for simple skills"

    # Note: Rich skills skipped (need manual merge first)
    log_warning "Rich skills skipped (merge pending)"
}

# Phase 6: Validation
phase6_validate() {
    log_info "Phase 6: Validation"

    errors=0
    warnings=0

    # Check 1: All skills have SKILL.md
    log_info "Checking for SKILL.md in all skills..."
    for skill_dir in "$AGENTS_DIR"/*/; do
        skill_name=$(basename "$skill_dir")
        if [ ! -f "$skill_dir/SKILL.md" ]; then
            log_error "  Missing SKILL.md: $skill_name"
            ((errors++))
        fi
    done

    # Check 2: No broken symlinks
    log_info "Checking for broken symlinks..."
    broken_symlinks=$(find "$AGENTS_DIR" -type l ! -exec test -e {} \; -print 2>/dev/null || true)
    if [ -n "$broken_symlinks" ]; then
        log_error "Broken symlinks found:"
        echo "$broken_symlinks" | while read -r link; do
            log_error "  $link"
        done
        ((errors++))
    else
        log_success "No broken symlinks"
    fi

    # Check 3: Count skills
    total_skills=$(ls -d "$AGENTS_DIR"/*/ 2>/dev/null | wc -l)
    log_info "Total skills in $AGENTS_DIR: $total_skills"

    # Check 4: Check for merge notes
    merge_pending=$(find "$AGENTS_DIR" -name "MERGE_NOTES.md" | wc -l)
    if [ "$merge_pending" -gt 0 ]; then
        log_warning "$merge_pending skills require manual merge"
        ((warnings++))
    fi

    # Update inventory
    sed -i "/Total Skills:/s/-/$total_skills/" "$AGENTS_DIR/INVENTORY.md"
    sed -i "/Merged:/s/-/0 (pending)/" "$AGENTS_DIR/INVENTORY.md"

    # Summary
    cat >> "$AGENTS_DIR/INVENTORY.md" << EOF

## Skills List
EOF

    for skill_dir in "$AGENTS_DIR"/*/; do
        skill_name=$(basename "$skill_dir")
        file_count=$(find "$skill_dir" -type f | wc -l)
        echo "- **$skill_name**: $file_count files" >> "$AGENTS_DIR/INVENTORY.md"
    done

    # Final report
    echo ""
    log_info "=== Validation Summary ==="
    log_info "Total skills: $total_skills"
    log_info "Errors: $errors"
    log_info "Warnings: $warnings"

    if [ "$errors" -gt 0 ]; then
        log_error "Validation failed with $errors errors"
        return 1
    else
        log_success "Validation passed!"
    fi

    if [ "$warnings" -gt 0 ]; then
        log_warning "Review $warnings warnings"
    fi
}

# Main execution
main() {
    log_info "Starting Skills Consolidation"
    log_info "Source: plans/skills-consolidation.md"
    log_info ""

    # Confirm execution
    read -p "Proceed with consolidation? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Aborted"
        exit 0
    fi

    # Execute phases
    phase1_prepare
    phase2_copy_unique
    phase3_simple_duplicates
    phase4_rich_duplicates
    phase5_symlinks
    phase6_validate

    # Final summary
    echo ""
    log_success "=== Consolidation Complete ==="
    log_info "Location: $AGENTS_DIR"
    log_info "Inventory: $AGENTS_DIR/INVENTORY.md"
    log_info "Backups:"
    log_info "  - ${CLAUDE_DIR}${BACKUP_SUFFIX}"
    log_info "  - ${OPENCODE_DIR}${BACKUP_SUFFIX}"

    # Next steps
    echo ""
    log_info "=== Next Steps ==="
    log_info "1. Review MERGE_NOTES.md in rich skills"
    log_info "2. Manually merge content for rich skills"
    log_info "3. Test skill loading functionality"
    log_info "4. Remove backups when satisfied"
    log_info "5. Plan deprecation of source directories"
}

# Run main
main "$@"
