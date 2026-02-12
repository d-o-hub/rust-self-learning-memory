#!/bin/bash
# Generate dynamic codebase statistics for AGENTS.md
set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Output file
STATS_FILE="$PROJECT_ROOT/stats.md"
AGENT_FILE="$PROJECT_ROOT/AGENTS.md"

echo "🔢 Generating codebase statistics..."

# Get version from Cargo.toml
if command -v jq >/dev/null 2>&1; then
    VERSION=$(cargo metadata --no-deps --format-version 1 2>/dev/null | jq -r '.packages[] | select(.name == "rust-self-learning-memory") | .version' || echo "unknown")
else
    VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "//; s/"//')
fi

# Count Rust files (excluding target/)
RUST_FILES=$(find "$PROJECT_ROOT" -name "*.rs" -not -path "*/target/*" -not -path "*/.git/*" | wc -l)

# Count total lines of Rust code
TOTAL_LINES=$(find "$PROJECT_ROOT" -name "*.rs" -not -path "*/target/*" -not -path "*/.git/*" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}' || echo "0")

# Count test files
TEST_FILES=$(find "$PROJECT_ROOT" -name "*test*.rs" -not -path "*/target/*" -not -path "*/.git/*" | wc -l)

# Count workspace members
if command -v cargo >/dev/null 2>&1 && [ -r "$PROJECT_ROOT/Cargo.toml" ]; then
    # Count members directly from Cargo.toml workspace.members section
    WORKSPACE_MEMBERS=$(awk '/^\[workspace\]/ {in_workspace=1; next} in_workspace && /^\[.*\]/ {exit} in_workspace && /^members/ {print $0}' "$PROJECT_ROOT/Cargo.toml" | grep -o '=' | wc -l || echo "0")
    if [ "$WORKSPACE_MEMBERS" -eq 0 ]; then
        # Fallback: count lines under [workspace] that contain member paths
        WORKSPACE_MEMBERS=$(awk '/^\[workspace\]/ {in_workspace=1; next} in_workspace && /^\[.*\]/ {exit} in_workspace && /"/ {count++} END {print count+0}' "$PROJECT_ROOT/Cargo.toml")
    fi
else
    WORKSPACE_MEMBERS="unknown"
fi

# Current date
CURRENT_DATE=$(date +%Y-%m-%d)

# Format lines with K suffix if large
if [ "$TOTAL_LINES" -gt 1000 ]; then
    FORMATTED_LINES="$(($TOTAL_LINES / 1000))K"
else
    FORMATTED_LINES="$TOTAL_LINES"
fi

# Generate stats content
cat > "$STATS_FILE" <<EOF
**Last Updated**: $CURRENT_DATE (v$VERSION)

**Codebase Stats**: $RUST_FILES Rust files, ~$FORMATTED_LINES LOC, $TEST_FILES+ test files, $WORKSPACE_MEMBERS workspace members
EOF

echo "✅ Generated statistics:"
cat "$STATS_FILE"
echo ""

# Update AGENTS.md if it has include placeholder
if grep -q "<!-- INCLUDE:stats.md -->" "$AGENT_FILE"; then
    echo "🔄 Updating AGENTS.md with generated stats..."
    
    # Create temp file with updated content
    awk '
    BEGIN { in_stats_block = 0; stats_line = "" }
    /<!-- INCLUDE:stats.md -->/ {
        # Read the stats file
        while ((getline line < "'"$STATS_FILE"'") > 0) {
            if (stats_line != "") stats_line = stats_line "\\n" line
            else stats_line = line
        }
        close("'$STATS_FILE'")
        print stats_line
        in_stats_block = 1
        next
    }
    {
        if (in_stats_block) {
            in_stats_block = 0
        }
        print
    }
    ' "$AGENT_FILE" > "$AGENT_FILE.tmp" && mv "$AGENT_FILE.tmp" "$AGENT_FILE"
    
    echo "✅ AGENTS.md updated"
else
    echo "ℹ️  AGENTS.md does not have <!-- INCLUDE:stats.md --> placeholder"
    echo "   Add this placeholder where you want dynamic stats to appear"
fi

echo ""
echo "📊 Summary:"
echo "  Version: v$VERSION"
echo "  Rust files: $RUST_FILES"
echo "  Total lines: $TOTAL_LINES"
echo "  Test files: $TEST_FILES"
echo "  Workspace members: $WORKSPACE_MEMBERS"
echo "  Stats file: $STATS_FILE"