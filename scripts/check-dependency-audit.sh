#!/bin/bash
# Security Dependency Audit Runner
#
# Audits both root Cargo.lock and fuzz/Cargo.lock dependency graphs
# using cargo-deny (blocking advisory gate) and cargo-audit (structured reports).

set -euo pipefail

# Expected lockfiles to audit
LOCKFILES=("Cargo.lock" "fuzz/Cargo.lock")

echo "=== Dependency Security Audit ==="

# 1. Validate inputs exist and are non-empty
for lf in "${LOCKFILES[@]}"; do
    if [ ! -f "$lf" ]; then
        echo "ERROR: Tracked lockfile '$lf' does not exist." >&2
        exit 1
    fi
    if [ ! -s "$lf" ]; then
        echo "ERROR: Tracked lockfile '$lf' is empty." >&2
        exit 1
    fi
    echo "Found tracked lockfile: $lf"
done

# 2. Check required security tools
MISSING_TOOLS=()
if ! command -v cargo-deny &>/dev/null && ! cargo deny --version &>/dev/null; then
    MISSING_TOOLS+=("cargo-deny")
fi
if ! command -v cargo-audit &>/dev/null && ! cargo audit --version &>/dev/null; then
    MISSING_TOOLS+=("cargo-audit")
fi

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo "ERROR: Required security audit tool(s) missing: ${MISSING_TOOLS[*]}" >&2
    echo "Install with: cargo install cargo-deny cargo-audit --locked" >&2
    exit 1
fi

# 3. Save initial git status/diff for lockfiles to detect mutations
INITIAL_DIFF=$(git diff -- "${LOCKFILES[@]}")

# 4. Authoritative cargo-deny check for both graphs
echo "Running cargo deny advisory checks..."
if ! cargo deny check advisories; then
    echo "ERROR: Root workspace cargo-deny check failed." >&2
    exit 1
fi

if ! cargo deny --manifest-path fuzz/Cargo.toml check advisories; then
    echo "ERROR: Fuzz graph cargo-deny check failed." >&2
    exit 1
fi

# 5. Run cargo-audit structured report for each lockfile
AUDIT_FAILED=0

run_audit_for_lockfile() {
    local lockfile="$1"
    local report_file="$2"

    echo "Auditing lockfile: $lockfile -> $report_file"

    set +e
    cargo audit --file "$lockfile" --json > "$report_file" 2> "${report_file}.err"
    local audit_rc=$?
    set -e

    if [ ! -f "$report_file" ] || [ ! -s "$report_file" ]; then
        echo "ERROR: Audit report '$report_file' was not produced or is empty (rc=$audit_rc)." >&2
        if [ -f "${report_file}.err" ]; then
            cat "${report_file}.err" >&2
        fi
        return 1
    fi

    # Validate JSON syntax
    if ! jq -e . "$report_file" >/dev/null 2>&1; then
        echo "ERROR: Audit report '$report_file' contains invalid JSON." >&2
        return 1
    fi

    local count
    count=$(jq -r '.vulnerabilities.count // empty' "$report_file" 2>/dev/null || echo "")

    if [ -z "$count" ] || ! [[ "$count" =~ ^[0-9]+$ ]]; then
        echo "ERROR: Could not parse numeric vulnerability count from '$report_file'." >&2
        return 1
    fi

    echo "Audit result for $lockfile: $count vulnerabilities found"

    if [ "$count" -gt 0 ]; then
        echo "ERROR: Security vulnerabilities detected in $lockfile:" >&2
        jq '.vulnerabilities.list[]? | {id: .advisory.id, package: .package.name, title: .advisory.title}' "$report_file" >&2
        return 1
    fi

    return 0
}

if ! run_audit_for_lockfile "Cargo.lock" "audit-report-root.json"; then
    AUDIT_FAILED=1
fi

if ! run_audit_for_lockfile "fuzz/Cargo.lock" "audit-report-fuzz.json"; then
    AUDIT_FAILED=1
fi

# 6. Verify lockfiles were not mutated during scan
CURRENT_DIFF=$(git diff -- "${LOCKFILES[@]}")
if [ "$INITIAL_DIFF" != "$CURRENT_DIFF" ]; then
    echo "ERROR: Lockfiles were modified during the audit scan!" >&2
    git diff -- "${LOCKFILES[@]}" >&2
    exit 1
fi

if [ "$AUDIT_FAILED" -ne 0 ]; then
    echo "ERROR: Dependency security audit failed." >&2
    exit 1
fi

echo "=== Dependency Security Audit Passed ==="
echo "Audited lockfiles: Cargo.lock, fuzz/Cargo.lock"
echo "Root vulnerabilities: $(jq -r '.vulnerabilities.count // 0' audit-report-root.json)"
echo "Fuzz vulnerabilities: $(jq -r '.vulnerabilities.count // 0' audit-report-fuzz.json)"
