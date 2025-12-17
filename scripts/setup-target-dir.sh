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