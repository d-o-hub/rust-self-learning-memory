#!/bin/bash
# Plans Directory Restructuring Helper Script
# Usage: ./plans_restructure_helper.sh <command>

set -e

PLANS_DIR="/workspaces/feat-phase3/plans"
BACKUP_DIR="/workspaces/feat-phase3/plans.backup"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

echo_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create backup
create_backup() {
    echo_info "Creating backup of plans directory..."
    if [ -d "$BACKUP_DIR" ]; then
        echo_warn "Backup already exists at $BACKUP_DIR"
        read -p "Overwrite existing backup? (y/n): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo_info "Skipping backup creation"
            return
        fi
        rm -rf "$BACKUP_DIR"
    fi
    cp -r "$PLANS_DIR" "$BACKUP_DIR"
    echo_info "Backup created at $BACKUP_DIR"
}

# Count files in directories
count_files() {
    echo_info "File count analysis:"
    echo ""

    total=0

    # Top-level files
    count=$(find "$PLANS_DIR" -maxdepth 1 -name "*.md" | wc -l)
    echo "Top-level: $count files"
    total=$((total + count))

    # Each subdirectory
    for dir in active reference roadmaps status goap archive; do
        if [ -d "$PLANS_DIR/$dir" ]; then
            count=$(find "$PLANS_DIR/$dir" -name "*.md" | wc -l)
            echo "$dir/: $count files"
            total=$((total + count))
        else
            echo "$dir/: (not yet created)"
        fi
    done

    echo ""
    echo "Total active files: $total"
    echo ""

    # Large files (>500 lines)
    echo_info "Checking for files exceeding 500 lines..."
    large_files=$(find "$PLANS_DIR" -name "*.md" ! -path "*/archive/*" -exec sh -c 'wc -l < "$1" | awk -v f="$1" "\$1 > 500 {print f}"' _ {} \;)
    if [ -z "$large_files" ]; then
        echo_info "No active files exceed 500 lines ✓"
    else
        echo_warn "Files exceeding 500 lines:"
        echo "$large_files"
    fi
}

# Find broken links
check_links() {
    echo_info "Checking for broken markdown links..."

    # Find all markdown files
    find "$PLANS_DIR" -name "*.md" ! -path "*/archive/*" -print0 | while IFS= read -r -d '' file; do
        # Extract all [text](link) patterns
        grep -o '\[.*\]([^)]*)' "$file" | while IFS= read -r link; do
            # Extract the link path
            link_path=$(echo "$link" | sed 's/.*](//' | sed 's/)$//')

            # Skip external links (http/https) and anchor links
            if [[ "$link_path" =~ ^https?:// ]] || [[ "$link_path" =~ ^# ]]; then
                continue
            fi

            # Resolve relative path
            dir=$(dirname "$file")
            target="$dir/$link_path"

            # Check if file exists
            if [ ! -f "$target" ]; then
                echo_warn "Broken link in $file: $link_path"
            fi
        done
    done

    echo_info "Link check complete"
}

# Find large files
find_large_files() {
    echo_info "Finding files exceeding 500 lines..."
    echo ""

    find "$PLANS_DIR" -name "*.md" -exec sh -c 'lines=$(wc -l < "$1"); if [ "$lines" -gt 500 ]; then echo "$lines $1"; fi' _ {} \; | sort -rn
}

# Archive old files by pattern
archive_by_pattern() {
    pattern="$1"
    dest_dir="$2"

    echo_info "Archiving files matching: $pattern"

    if [ ! -d "$dest_dir" ]; then
        mkdir -p "$dest_dir"
        echo_info "Created directory: $dest_dir"
    fi

    count=0
    for file in find "$PLANS_DIR" -name "$pattern" ! -path "*/archive/*" -print; do
        if [ -f "$file" ]; then
            mv "$file" "$dest_dir/"
            echo_info "Moved: $file"
            count=$((count + 1))
        fi
    done

    echo_info "Archived $count files"
}

# Show plan progress
show_progress() {
    echo_info "Restructuring Progress:"
    echo ""

    # Check if new directories exist
    dirs=("active" "reference" "roadmaps" "status" "goap")
    for dir in "${dirs[@]}"; do
        if [ -d "$PLANS_DIR/$dir" ]; then
            count=$(find "$PLANS_DIR/$dir" -name "*.md" 2>/dev/null | wc -l)
            echo -e "  ${GREEN}✓${NC} $dir/ ($count files)"
        else
            echo -e "  ${YELLOW}○${NC} $dir/ (not created)"
        fi
    done

    echo ""

    # Show statistics
    total_active=$(find "$PLANS_DIR" -name "*.md" ! -path "*/archive/*" | wc -l)
    total_archive=$(find "$PLANS_DIR/archive" -name "*.md" 2>/dev/null | wc -l)

    echo "Active files: $total_active (target: <100)"
    echo "Archived files: $total_archive"
}

# Validate structure
validate_structure() {
    echo_info "Validating new structure..."
    echo ""

    errors=0

    # Check 500-line limit
    large_files=$(find "$PLANS_DIR" -name "*.md" ! -path "*/archive/*" -exec sh -c 'wc -l < "$1" | awk -v f="$1" "\$1 > 500 {print f}"' _ {} \;)
    if [ -n "$large_files" ]; then
        echo_error "Files exceed 500 lines:"
        echo "$large_files"
        errors=$((errors + 1))
    else
        echo_info "✓ All files under 500 lines"
    fi

    # Check file count
    total_active=$(find "$PLANS_DIR" -name "*.md" ! -path "*/archive/*" | wc -l)
    if [ "$total_active" -gt 100 ]; then
        echo_error "Too many active files: $total_active (target: <100)"
        errors=$((errors + 1))
    else
        echo_info "✓ File count acceptable: $total_active"
    fi

    # Check essential files exist
    essential_files=(
        "$PLANS_DIR/README.md"
        "$PLANS_DIR/QUICK_START_PLANS.md"
    )

    for file in "${essential_files[@]}"; do
        if [ ! -f "$file" ]; then
            echo_error "Missing essential file: $file"
            errors=$((errors + 1))
        fi
    done

    echo ""
    if [ $errors -eq 0 ]; then
        echo_info "✓ Structure validation passed"
        return 0
    else
        echo_error "✗ Structure validation failed with $errors error(s)"
        return 1
    fi
}

# Main command handler
case "$1" in
    backup)
        create_backup
        ;;
    count)
        count_files
        ;;
    check-links)
        check_links
        ;;
    find-large)
        find_large_files
        ;;
    progress)
        show_progress
        ;;
    validate)
        validate_structure
        ;;
    *)
        echo "Plans Directory Restructuring Helper"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  backup      Create backup of plans directory"
        echo "  count       Count files in each directory"
        echo "  check-links  Check for broken markdown links"
        echo "  find-large  Find files exceeding 500 lines"
        echo "  progress    Show restructuring progress"
        echo "  validate     Validate new structure"
        echo ""
        echo "Example:"
        echo "  $0 backup      # Create backup before starting"
        echo "  $0 find-large  # Find files that need splitting"
        echo "  $0 validate    # Check if structure is correct"
        ;;
esac
