# Phase 1 Quick Wins Implementation Guide

## Overview

This guide provides detailed implementation instructions for the Phase 1 quick wins that will immediately address disk space issues in GitHub Actions workflows while maintaining minimal risk and rapid deployment.

## 1. Enhanced Disk Space Management (DevOps Engineer)

### Implementation Script

Create `scripts/github-actions-disk-cleanup.sh`:

```bash
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
```

### Workflow Integration

Update each workflow file to use the cleanup script:

```yaml
- name: Free disk space
  if: runner.os == 'Linux'
  run: |
    chmod +x ./scripts/github-actions-disk-cleanup.sh
    ./scripts/github-actions-disk-cleanup.sh
```

### File Updates Required

#### `.github/workflows/ci.yml`
Replace existing disk cleanup steps (lines 85-90, 145-150, 241-248, 355-362):

```yaml
- name: Free disk space
  if: runner.os == 'Linux'
  run: |
    chmod +x ./scripts/github-actions-disk-cleanup.sh
    ./scripts/github-actions-disk-cleanup.sh
```

#### `.github/workflows/benchmarks.yml`
Add before benchmark steps:

```yaml
- name: Free disk space
  if: runner.os == 'Linux'
  run: |
    chmod +x ./scripts/github-actions-disk-cleanup.sh
    ./scripts/github-actions-disk-cleanup.sh
```

## 2. Optimized Caching Strategy (Performance Specialist)

### Workspace-Aware Cache Configuration

Create `scripts/configure-caching.sh`:

```bash
#!/bin/bash
set -euo pipefail

# Cache configuration for multi-crate Rust workspace

echo "=== Configuring optimized caching ==="

# Generate cache keys based on workspace structure
WORKSPACE_HASH=$(sha256sum Cargo.toml Cargo.lock 2>/dev/null | sha256sum | cut -d' ' -f1)
TOOLCHAIN_HASH=$(rustc --version | sha256sum | cut -d' ' -f1)

# Set environment variables for cache keys
echo "CARGO_REGISTRY_CACHE_KEY=${{ runner.os }}-cargo-registry-${WORKSPACE_HASH}" >> $GITHUB_ENV
echo "CARGO_INDEX_CACHE_KEY=${{ runner.os }}-cargo-index-${WORKSPACE_HASH}" >> $GITHUB_ENV
echo "CARGO_BUILD_CACHE_KEY=${{ runner.os }}-cargo-build-${WORKSPACE_HASH}-${TOOLCHAIN_HASH}" >> $GITHUB_ENV

# Set cache save conditions
if [[ "${{ github.ref }}" == "refs/heads/main" || "${{ github.ref }}" == "refs/heads/develop" ]]; then
    echo "CACHE_SAVE_CONDITION=true" >> $GITHUB_ENV
else
    echo "CACHE_SAVE_CONDITION=false" >> $GITHUB_ENV
fi

echo "Cache configuration completed"
```

### Updated Workflow Cache Configuration

#### `.github/workflows/ci.yml`

Replace all existing cache steps with optimized configuration:

```yaml
- name: Configure caching
  run: |
    chmod +x ./scripts/configure-caching.sh
    ./scripts/configure-caching.sh

- name: Cache cargo registry
  uses: actions/cache@v4.3.0
  with:
    path: ~/.cargo/registry
    key: ${{ env.CARGO_REGISTRY_CACHE_KEY }}
    restore-keys: |
      ${{ runner.os }}-cargo-registry-

- name: Cache cargo index
  uses: actions/cache@v4.3.0
  with:
    path: ~/.cargo/git
    key: ${{ env.CARGO_INDEX_CACHE_KEY }}
    restore-keys: |
      ${{ runner.os }}-cargo-index-

- name: Cache cargo build
  uses: actions/cache@v4.3.0
  with:
    path: |
      target/
      ~/.cargo/bin/
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
    key: ${{ env.CARGO_BUILD_CACHE_KEY }}
    restore-keys: |
      ${{ runner.os }}-cargo-build-
      ${{ runner.os }}-cargo-
```

#### `.github/workflows/benchmarks.yml`

```yaml
- name: Configure caching
  run: |
    chmod +x ./scripts/configure-caching.sh
    ./scripts/configure-caching.sh

- name: Cache cargo registry
  uses: actions/cache@v4.3.0
  with:
    path: ~/.cargo/registry
    key: ${{ env.CARGO_REGISTRY_CACHE_KEY }}
    restore-keys: |
      ${{ runner.os }}-cargo-registry-

- name: Cache cargo index
  uses: actions/cache@v4.3.0
  with:
    path: ~/.cargo/git
    key: ${{ env.CARGO_INDEX_CACHE_KEY }}
    restore-keys: |
      ${{ runner.os }}-cargo-index-

- name: Cache cargo build
  uses: actions/cache@v4.3.0
  with:
    path: |
      target/
      ~/.cargo/bin/
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
    key: ${{ env.CARGO_BUILD_CACHE_KEY }}
    restore-keys: |
      ${{ runner.os }}-cargo-build-
      ${{ runner.os }}-cargo-
```

## 3. Target Directory Optimization (Systems Architect)

### Standardized Target Directory Configuration

Update all workflow jobs to use `/tmp/target` consistently:

#### Environment Variables
Add to workflow level environment variables:

```yaml
env:
  CARGO_TARGET_DIR: /tmp/target
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"
```

### Job-Specific Target Isolation

Create `scripts/setup-target-dir.sh`:

```bash
#!/bin/bash
set -euo pipefail

# Setup isolated target directory for each job

JOB_NAME="${1:-default}"
TIMESTAMP=$(date +%s)
TARGET_DIR="/tmp/target-${JOB_NAME}-${TIMESTAMP}"

echo "=== Setting up target directory ==="
echo "Job name: $JOB_NAME"
echo "Target directory: $TARGET_DIR"

# Create isolated target directory
mkdir -p "$TARGET_DIR"

# Set environment variables
echo "CARGO_TARGET_DIR=$TARGET_DIR" >> $GITHUB_ENV
echo "TARGET_DIR=$TARGET_DIR" >> $GITHUB_ENV

# Cleanup function
cleanup_target() {
    echo "Cleaning up target directory: $TARGET_DIR"
    rm -rf "$TARGET_DIR" || true
}

# Trap for cleanup
trap cleanup_target EXIT

echo "Target directory setup completed"
```

### Workflow Integration Examples

#### Test Jobs
```yaml
- name: Setup target directory
  run: |
    chmod +x ./scripts/setup-target-dir.sh
    ./scripts/setup-target-dir.sh test-${{ matrix.os || 'ubuntu' }}
```

#### Build Jobs
```yaml
- name: Setup target directory
  run: |
    chmod +x ./scripts/setup-target-dir.sh
    ./scripts/setup-target-dir.sh build-${{ matrix.rust || 'stable' }}
```

#### Coverage Jobs
```yaml
- name: Setup target directory
  run: |
    chmod +x ./scripts/setup-target-dir.sh
    ./scripts/setup-target-dir.sh coverage
```

## 4. Basic Monitoring Setup (QA Engineer)

### Disk Usage Monitoring Script

Create `scripts/monitor-disk-usage.sh`:

```bash
#!/bin/bash
set -euo pipefail

# Monitor disk usage during CI/CD jobs

MONITOR_INTERVAL=30  # seconds
MAX_DISK_USAGE=85    # percentage
LOG_FILE="/tmp/disk-monitor.log"

echo "=== Starting disk usage monitoring ==="
echo "Monitoring interval: ${MONITOR_INTERVAL}s"
echo "Max disk usage threshold: ${MAX_DISK_USAGE}%"
echo "Log file: $LOG_FILE"

# Function to log disk usage
log_usage() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local usage=$(df / | awk 'NR==2 {print $5}' | sed 's/%//')
    local available=$(df -h / | awk 'NR==2 {print $4}')
    
    echo "$timestamp: Disk usage: ${usage}%, Available: ${available}" >> "$LOG_FILE"
    
    if [[ $usage -gt $MAX_DISK_USAGE ]]; then
        echo "$timestamp: WARNING: Disk usage above threshold (${usage}%)" >> "$LOG_FILE"
        echo "::warning::Disk usage is ${usage}%, approaching limit"
    fi
}

# Initial log
log_usage

# Start monitoring in background
(
    while true; do
        sleep $MONITOR_INTERVAL
        log_usage
    done
) &
MONITOR_PID=$!

echo "Monitoring started (PID: $MONITOR_PID)"

# Stop monitoring function
stop_monitoring() {
    echo "Stopping disk usage monitoring..."
    kill $MONITOR_PID 2>/dev/null || true
    
    # Generate final report
    echo "=== Disk Usage Summary ===" >> "$LOG_FILE"
    echo "Final disk usage:" >> "$LOG_FILE"
    df -h >> "$LOG_FILE"
    
    echo "Disk monitoring completed. Log saved to: $LOG_FILE"
}

# Trap for cleanup
trap stop_monitoring EXIT

echo "Monitoring setup completed"
```

### Workflow Integration

Add to workflow steps:

```yaml
- name: Start disk monitoring
  run: |
    chmod +x ./scripts/monitor-disk-usage.sh
    ./scripts/monitor-disk-usage.sh

# ... other job steps ...

- name: Upload disk usage log
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: disk-usage-log-${{ github.job }}
    path: /tmp/disk-monitor.log
    retention-days: 7
```

## Implementation Checklist

### Files to Create
- [ ] `scripts/github-actions-disk-cleanup.sh`
- [ ] `scripts/configure-caching.sh`
- [ ] `scripts/setup-target-dir.sh`
- [ ] `scripts/monitor-disk-usage.sh`

### Files to Modify
- [ ] `.github/workflows/ci.yml`
- [ ] `.github/workflows/benchmarks.yml`
- [ ] `.github/workflows/release.yml`
- [ ] `.github/workflows/quick-check.yml`

### Testing Procedures
1. **Local Testing**: Run scripts locally to verify functionality
2. **Branch Testing**: Create feature branch and test workflows
3. **Partial Rollout**: Test with subset of workflows first
4. **Full Deployment**: Deploy to all workflows after validation

### Success Metrics
1. **Disk Space Recovery**: >30% additional space before critical jobs
2. **Cache Hit Rate**: >80% on subsequent runs
3. **Target Isolation**: No conflicts between concurrent jobs
4. **Monitoring Coverage**: 100% of jobs have disk usage monitoring

### Rollback Plan
If any issues arise:
1. Revert workflow changes to previous version
2. Remove created scripts
3. Monitor for any remaining issues
4. Analyze root cause before retry

## Next Steps

After Phase 1 implementation:
1. Monitor performance for 48 hours
2. Collect metrics on disk space and build times
3. Validate all workflows still pass
4. Proceed to Phase 2 after confirmation of success

This comprehensive Phase 1 implementation provides immediate relief from disk space issues while establishing a foundation for more advanced optimizations in subsequent phases.