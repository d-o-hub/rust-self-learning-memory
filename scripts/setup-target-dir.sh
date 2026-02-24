#!/bin/bash
set -euo pipefail

# Setup isolated target directory for CI jobs (ADR-032 Phase 5).

JOB_NAME="${1:-default}"
BASE_DIR="${RUNNER_TEMP:-/tmp}"
RUN_SUFFIX="${GITHUB_RUN_ID:-local}"
TARGET_DIR="${BASE_DIR}/cargo-target-${JOB_NAME}-${RUN_SUFFIX}"

echo "=== Setting up target directory ==="
echo "Job name: ${JOB_NAME}"
echo "Base directory: ${BASE_DIR}"
echo "Target directory: ${TARGET_DIR}"

mkdir -p "${TARGET_DIR}"

if [ -z "${GITHUB_ENV:-}" ]; then
    echo "GITHUB_ENV is not set; export manually: CARGO_TARGET_DIR=${TARGET_DIR}"
    exit 0
fi

echo "CARGO_TARGET_DIR=${TARGET_DIR}" >> "${GITHUB_ENV}"
echo "TARGET_DIR=${TARGET_DIR}" >> "${GITHUB_ENV}"

echo "Target directory setup completed"
