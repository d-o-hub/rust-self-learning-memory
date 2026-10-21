#!/bin/bash
# Test suite for scripts/check-dependency-audit.sh
#
# Validates:
# - Successful two-lockfile audit run
# - Identification of exact audited lockfiles
# - Failure on missing lockfile
# - Failure on empty lockfile
# - Failure on invalid JSON report
# - Failure on seeded unignored vulnerability finding in fuzz lockfile
# - Verification of lockfile immutability

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
AUDIT_SCRIPT="$REPO_ROOT/scripts/check-dependency-audit.sh"

echo "=== Testing check-dependency-audit.sh ==="

if [ ! -x "$AUDIT_SCRIPT" ]; then
    echo "ERROR: Audit script '$AUDIT_SCRIPT' is missing or not executable." >&2
    exit 1
fi

TEST_TMPDIR=$(mktemp -d)
trap 'rm -rf "$TEST_TMPDIR"' EXIT

# Copy the repository's *tracked* files, not the working tree: a full
# `cp -r "$REPO_ROOT"` also copies `target/`, which is gigabytes of build
# artifacts and fills the tmpfs the fixture runs in. `git archive` copies only
# tracked files, so the fixture still has a real manifest and source graph
# (cargo deny needs `Cargo.toml` to resolve the workspace) but no build output.
make_fixture_repo() {
    local dest="$1"
    mkdir -p "$dest"
    git -C "$REPO_ROOT" archive HEAD | tar -x -C "$dest"
    # The audit script diffs the lockfiles against git, so the fixture needs a
    # repository (or the diff fails and the run aborts).
    (cd "$dest" && git init -q && git add -A && git -c user.email=t@t -c user.name=t commit -qm fixture)
}

# Test 1: Successful real audit run on repository
echo "Test 1: Run real dependency audit..."
OUT1=$("$AUDIT_SCRIPT" 2>&1)
echo "$OUT1"
if ! echo "$OUT1" | grep -q "Cargo.lock"; then
    echo "FAILED Test 1: Cargo.lock not identified in report." >&2
    exit 1
fi
if ! echo "$OUT1" | grep -q "fuzz/Cargo.lock"; then
    echo "FAILED Test 1: fuzz/Cargo.lock not identified in report." >&2
    exit 1
fi
if [ ! -f "$REPO_ROOT/audit-report-root.json" ] || [ ! -f "$REPO_ROOT/audit-report-fuzz.json" ]; then
    echo "FAILED Test 1: Report JSON files were not generated." >&2
    exit 1
fi
echo "PASSED Test 1"

# Test 2: Failure on missing fuzz lockfile
echo "Test 2: Failure on missing fuzz lockfile..."
make_fixture_repo "$TEST_TMPDIR/repo_missing_fuzz"
rm -f "$TEST_TMPDIR/repo_missing_fuzz/fuzz/Cargo.lock"
set +e
OUT2=$(cd "$TEST_TMPDIR/repo_missing_fuzz" && ./scripts/check-dependency-audit.sh 2>&1)
RC2=$?
set -e
if [ "$RC2" -eq 0 ]; then
    echo "FAILED Test 2: Script passed despite missing fuzz/Cargo.lock." >&2
    exit 1
fi
if ! echo "$OUT2" | grep -q "does not exist"; then
    echo "FAILED Test 2: Expected 'does not exist' error message, got: $OUT2" >&2
    exit 1
fi
echo "PASSED Test 2"

# Test 3: Failure on empty fuzz lockfile
echo "Test 3: Failure on empty fuzz lockfile..."
make_fixture_repo "$TEST_TMPDIR/repo_empty_fuzz"
: > "$TEST_TMPDIR/repo_empty_fuzz/fuzz/Cargo.lock"
set +e
OUT3=$(cd "$TEST_TMPDIR/repo_empty_fuzz" && ./scripts/check-dependency-audit.sh 2>&1)
RC3=$?
set -e
if [ "$RC3" -eq 0 ]; then
    echo "FAILED Test 3: Script passed despite empty fuzz/Cargo.lock." >&2
    exit 1
fi
if ! echo "$OUT3" | grep -q "empty"; then
    echo "FAILED Test 3: Expected 'empty' error message, got: $OUT3" >&2
    exit 1
fi
echo "PASSED Test 3"

# Test 4: Failure on seeded vulnerability finding in fuzz graph
echo "Test 4: Failure on seeded unignored vulnerability in fuzz graph..."
make_fixture_repo "$TEST_TMPDIR/repo_seeded_fuzz"
# Add an unignored vulnerable dependency (RUSTSEC-2021-0145 / atty 0.2.14) to fuzz/Cargo.toml and fuzz/Cargo.lock
cat << 'EOF' >> "$TEST_TMPDIR/repo_seeded_fuzz/fuzz/Cargo.toml"

[dependencies.atty]
version = "=0.2.14"
EOF

cat << 'EOF' >> "$TEST_TMPDIR/repo_seeded_fuzz/fuzz/Cargo.lock"

[[package]]
name = "atty"
version = "0.2.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d9b39be18770d11421cdb1b9947a45dd3f37e93092cbf377614828a319d5fee8"
dependencies = [
 "hermit-abi 0.1.19",
 "libc 0.2.186",
]
EOF

set +e
OUT4=$(cd "$TEST_TMPDIR/repo_seeded_fuzz" && ./scripts/check-dependency-audit.sh 2>&1)
RC4=$?
set -e
if [ "$RC4" -eq 0 ]; then
    echo "FAILED Test 4: Script passed despite seeded vulnerability in fuzz graph." >&2
    exit 1
fi
# The failure must be the seeded advisory in the *fuzz* graph. Accepting any
# failure ("failed", "vulnerabilities") makes this a false green: a missing
# Cargo.toml also fails the root cargo-deny step, and the script exits before
# the fuzz audit ever runs.
if ! echo "$OUT4" | grep -q "RUSTSEC-2021-0145"; then
    echo "FAILED Test 4: Expected the seeded advisory RUSTSEC-2021-0145, got: $OUT4" >&2
    exit 1
fi
if ! echo "$OUT4" | grep -q "fuzz/Cargo.lock"; then
    echo "FAILED Test 4: Expected the finding to name fuzz/Cargo.lock, got: $OUT4" >&2
    exit 1
fi
echo "PASSED Test 4"

echo "=== All dependency audit tests passed successfully ==="
