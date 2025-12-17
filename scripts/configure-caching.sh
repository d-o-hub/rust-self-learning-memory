#!/bin/bash
set -euo pipefail

# Cache configuration for multi-crate Rust workspace

echo "=== Configuring optimized caching ==="

# Generate cache keys based on workspace structure
WORKSPACE_HASH=$(sha256sum Cargo.toml Cargo.lock 2>/dev/null | sha256sum | cut -d' ' -f1)
TOOLCHAIN_HASH=$(rustc --version | sha256sum | cut -d' ' -f1)

# Detect runner OS (default to Linux if not in GitHub Actions)
if [[ -n "${RUNNER_OS:-}" ]]; then
    RUNNER_OS="${RUNNER_OS}"
else
    RUNNER_OS="Linux"
fi

# Set environment variables for cache keys
if [[ -n "${GITHUB_ENV:-}" ]]; then
    # Running in GitHub Actions
    echo "CARGO_REGISTRY_CACHE_KEY=${RUNNER_OS}-cargo-registry-${WORKSPACE_HASH}" >> $GITHUB_ENV
    echo "CARGO_INDEX_CACHE_KEY=${RUNNER_OS}-cargo-index-${WORKSPACE_HASH}" >> $GITHUB_ENV
    echo "CARGO_BUILD_CACHE_KEY=${RUNNER_OS}-cargo-build-${WORKSPACE_HASH}-${TOOLCHAIN_HASH}" >> $GITHUB_ENV
else
    # Running locally
    export CARGO_REGISTRY_CACHE_KEY="${RUNNER_OS}-cargo-registry-${WORKSPACE_HASH}"
    export CARGO_INDEX_CACHE_KEY="${RUNNER_OS}-cargo-index-${WORKSPACE_HASH}"
    export CARGO_BUILD_CACHE_KEY="${RUNNER_OS}-cargo-build-${WORKSPACE_HASH}-${TOOLCHAIN_HASH}"
fi

# Set cache save conditions
if [[ -n "${GITHUB_REF:-}" ]]; then
    # Running in GitHub Actions
    if [[ "${GITHUB_REF}" == "refs/heads/main" || "${GITHUB_REF}" == "refs/heads/develop" ]]; then
        CACHE_SAVE_CONDITION="true"
        if [[ -n "${GITHUB_ENV:-}" ]]; then
            echo "CACHE_SAVE_CONDITION=true" >> $GITHUB_ENV
        fi
    else
        CACHE_SAVE_CONDITION="false"
        if [[ -n "${GITHUB_ENV:-}" ]]; then
            echo "CACHE_SAVE_CONDITION=false" >> $GITHUB_ENV
        fi
    fi
else
    # Running locally, assume we can save
    CACHE_SAVE_CONDITION="true"
fi

echo "Cache configuration completed"
echo "Runner OS: ${RUNNER_OS}"
echo "Workspace hash: ${WORKSPACE_HASH:0:16}..."
echo "Toolchain hash: ${TOOLCHAIN_HASH:0:16}..."
echo "Cache save condition: ${CACHE_SAVE_CONDITION}"

# Display cache keys if available
if [[ -n "${CARGO_REGISTRY_CACHE_KEY:-}" ]]; then
    echo "Registry cache key: ${CARGO_REGISTRY_CACHE_KEY:0:50}..."
    echo "Index cache key: ${CARGO_INDEX_CACHE_KEY:0:50}..."
    echo "Build cache key: ${CARGO_BUILD_CACHE_KEY:0:50}..."
fi