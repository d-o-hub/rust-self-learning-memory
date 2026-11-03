#!/usr/bin/env bash
# Lockfile Sync and Alignment Gate
#
# Catches two failure modes locally that otherwise only surface as a red CI job
# with no obvious cause:
#
#   1. Stale lockfile. A dependency requirement changed but the matching
#      Cargo.lock was not re-resolved. The audit gate runs
#      `cargo deny --manifest-path fuzz/Cargo.toml check advisories`, which
#      lets cargo rewrite that stale lockfile mid-scan, so
#      scripts/check-dependency-audit.sh fails with
#      "ERROR: Lockfiles were modified during the audit scan!" even though it
#      reported 0 vulnerabilities. fuzz is `exclude`d from the root workspace,
#      so no other sensor ever looks at it.
#
#   2. Split alignment-constrained crate. `libsql` must resolve to exactly one
#      version per graph: chaotic_semantic_memory requires 0.9.x, and linking
#      two libsql versions under the `csm` feature duplicates libsql's exported
#      C symbols. A `cargo check` cannot see that - rlibs do not link - so the
#      conflict is detected from the lockfile instead.
#
# Fix a stale lock by re-resolving the graph it belongs to:
#   cargo metadata --format-version 1 >/dev/null            # root
#   cd fuzz && cargo metadata --format-version 1 >/dev/null # fuzz

set -euo pipefail

# Graphs to check: "<manifest>|<lockfile>"
GRAPHS=(
    "Cargo.toml|Cargo.lock"
    "fuzz/Cargo.toml|fuzz/Cargo.lock"
)

# Crates that must resolve to a single version in every graph, with the
# constraint that forces it.
ALIGNMENT_SINGLETONS=("libsql")

FAILED=0

echo "=== Lockfile Sync and Alignment ==="

# 1. Every graph's lockfile must already satisfy its manifests (--locked refuses
#    to rewrite, so a needed re-resolution becomes a failure here instead of a
#    mid-scan mutation of a tracked file).
for graph in "${GRAPHS[@]}"; do
    manifest="${graph%%|*}"
    lockfile="${graph##*|}"
    if [ ! -f "$lockfile" ]; then
        echo "ERROR: expected lockfile '$lockfile' is missing." >&2
        FAILED=1
        continue
    fi
    if cargo metadata --locked --manifest-path "$manifest" --format-version 1 >/dev/null; then
        echo "OK: $lockfile satisfies $manifest"
    else
        echo "ERROR: $lockfile is stale for $manifest - it would be rewritten during the audit scan." >&2
        echo "       Re-resolve with: cargo metadata --manifest-path $manifest --format-version 1 >/dev/null" >&2
        FAILED=1
    fi
done

# 2. Alignment-constrained crates must appear once per graph.
for graph in "${GRAPHS[@]}"; do
    lockfile="${graph##*|}"
    for crate in "${ALIGNMENT_SINGLETONS[@]}"; do
        versions=$(grep -A1 "^name = \"$crate\"" "$lockfile" |
            grep '^version = ' |
            sort -u |
            wc -l)
        if [ "$versions" -eq 1 ]; then
            echo "OK: $crate resolves to one version in $lockfile"
        else
            echo "ERROR: $crate resolves to $versions versions in $lockfile." >&2
            grep -A1 "^name = \"$crate\"" "$lockfile" | grep '^version = ' >&2
            echo "       Keep the workspace requirement aligned with the crate that constrains it (see the comment in Cargo.toml)." >&2
            FAILED=1
        fi
    done
done

if [ "$FAILED" -ne 0 ]; then
    echo "ERROR: Lockfile sync/alignment check failed." >&2
    exit 1
fi

echo "=== Lockfile Sync and Alignment Passed ==="
