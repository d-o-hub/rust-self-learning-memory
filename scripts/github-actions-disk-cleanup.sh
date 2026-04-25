#!/bin/bash
set -euo pipefail

# Comprehensive disk space cleanup for GitHub Actions
# Optimized for Rust workflows with multi-crate workspaces

echo "=== GitHub Actions Disk Space Cleanup ==="
echo "Initial disk usage:"
df -h

# Function to safely remove directories
safe_remove() {
    local dir="$1"
    if [[ -d "$dir" && -n "$(ls -A "$dir" 2>/dev/null)" ]]; then
        echo "Cleaning: $dir ($(du -sh "$dir" 2>/dev/null | cut -f1))"
        rm -rf "$dir"/* 2>/dev/null || true
        rm -rf "$dir"/.* 2>/dev/null || true
    fi
}

# 1. GitHub hosted toolcache cleanup (most impactful)
echo "Cleaning GitHub hosted toolcache..."
safe_remove "/opt/hostedtoolcache"
safe_remove "/opt/ghc"

# 2. DotNet runtime (large, not needed for Rust)
echo "Cleaning .NET runtime..."
safe_remove "/usr/share/dotnet"

# 3. Android SDK (not needed for Rust)
echo "Cleaning Android SDK..."
safe_remove "/usr/local/lib/android"

# 4. Docker images and containers (if present)
if command -v docker >/dev/null 2>&1; then
    echo "Cleaning Docker resources..."
    docker system prune -af --volumes || true
fi

# 5. Package manager caches
echo "Cleaning package manager caches..."
apt-get clean -y || true
apt-get autoremove -y || true
rm -rf /var/lib/apt/lists/* 2>/dev/null || true

# 6. Rust-specific cleanup
echo "Cleaning Rust caches..."
# Clean cargo registry cache selectively
if [[ -d "$HOME/.cargo/registry/cache" ]]; then
    # Keep only recent cache entries (last 7 days)
    find "$HOME/.cargo/registry/cache" -type f -mtime +7 -delete 2>/dev/null || true
fi

# Clean cargo git db
safe_remove "$HOME/.cargo/git/db"

# 7. Temporary files
echo "Cleaning temporary files..."
safe_remove "/tmp"
safe_remove "/var/tmp"

# 8. Log files
echo "Cleaning log files..."
find /var/log -type f -name "*.log" -mtime +1 -delete 2>/dev/null || true
journalctl --vacuum-time=1d || true

# 9. Miniconda/conda (if present)
safe_remove "$HOME/miniconda3"
safe_remove "$HOME/conda"
safe_remove "$HOME/.conda"

# 10. Node.js (if present)
safe_remove "$HOME/.npm"
safe_remove "$HOME/.node_repl_history"

# 11. Python (if present)
safe_remove "$HOME/.cache/pip"
safe_remove "$HOME/.local/share/virtualenvs"

# Final disk usage report
echo "=== Final disk usage ==="
df -h

# Calculate space recovered
echo "=== Space recovery summary ==="
echo "Cleanup completed successfully"

# Check minimum available space
AVAILABLE_SPACE=$(df / | awk 'NR==2 {print $4}')
MIN_SPACE_MB=10240  # 10GB minimum

if [[ $AVAILABLE_SPACE -lt $MIN_SPACE_MB ]]; then
    echo "WARNING: Low disk space detected (${AVAILABLE_SPACE}KB available)"
    echo "Consider additional cleanup or increasing runner resources"
else
    echo "Sufficient disk space available"
fi