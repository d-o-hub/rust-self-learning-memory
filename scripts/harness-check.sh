#!/usr/bin/env bash
# harness-check.sh — compatibility shim onto the canonical do-harness sensors.
#
# It used to run its own weaker copies of the gates: a clippy pass over
# `--lib --tests` with 27 `-A clippy::…` suppressions, `deny: PASS` when cargo-deny
# was absent, `security: PASS` when gitleaks was absent. Two notions of green is the
# bug — an agent that read `PASS` here could claim CI readiness while CI's own
# `clippy` sensor (`scripts/code-quality.sh clippy --workspace`, three suppressions)
# was red. Everything now delegates, so there is exactly one definition of green
# (issue #1246).
#
# Usage:
#   bash scripts/harness-check.sh <fmt|clippy|deny|test|arch|security|all>
#
#   fmt | clippy | deny | test   do-harness verify --only <sensor> --strict
#   all                          do-harness verify --set verification --strict --record
#   arch                         focused arch_fitness run (a subset of `test`)
#   security                     local gitleaks diagnostic; CI owns the authoritative scan
#
# Exit codes: 0 the delegated check passed; 1 it ran and failed; 2 it could not run at
# all (CLI or tool absent). Exit 2 is reported as BLOCKED, never as success — a check
# that silently skipped is how an unscanned tree gets shipped.
#
# This wrapper is NOT registered as a sensor, so `do-harness status` does not reflect
# it; the completion evidence boundary stays `do-harness verify --set verification
# --strict --record` followed by `do-harness status --set verification`.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

usage() {
    echo "Usage: bash scripts/harness-check.sh <fmt|clippy|deny|test|arch|security|all>" >&2
}

violation() {
    local category="$1" hint="$2"
    echo -e "${RED}HARNESS VIOLATION: [${category}] — ${hint}${NC}" >&2
}

blocked() {
    local category="$1" remedy="$2"
    echo -e "${RED}HARNESS BLOCKED: [${category}] — nothing was checked. ${remedy}${NC}" >&2
}

info() {
    echo -e "${YELLOW}Running: $1${NC}" >&2
}

# Delegation target shared by the mapped subcommands and `all`.
require_cli() {
    command -v do-harness >/dev/null 2>&1 || {
        blocked "do-harness" "the pinned harness CLI is not on PATH. Install a release from
    https://github.com/d-o-hub/do-harness/releases onto PATH, then re-run \`do-harness doctor\`
    in this workspace. The completion contract is do-harness.toml's \`verification\` set;
    running the old hand-rolled commands instead would reproduce the two-notions-of-green
    bug this shim exists to remove."
        exit 2
    }
}

verify_one() {
    local sensor="$1"
    require_cli
    info "do-harness verify --only ${sensor} --strict"
    if do-harness verify --only "${sensor}" --strict; then
        echo -e "${GREEN}${sensor}: PASS (canonical sensor)${NC}" >&2
        return 0
    fi
    violation "${sensor}" "the canonical ${sensor} sensor failed; fix the cause — re-running this
        wrapper cannot produce a different verdict"
    return 1
}

check_arch() {
    # The arch_fitness target lives in package `e2e-tests` (tests/Cargo.toml declares it as
    # `[[test]] name = "arch_fitness"`). The old command named `-p tests`, which matches no
    # package in the workspace: cargo resolved nothing and the run reported a verdict about
    # nothing. `cargo nextest run --profile ci --workspace` (the `test` sensor) already builds
    # and runs it, so this stays a focused diagnostic, not completion evidence.
    info "cargo nextest run -p e2e-tests --test arch_fitness (subset of the test sensor)"
    if ! command -v cargo >/dev/null 2>&1 || ! cargo nextest --version >/dev/null 2>&1; then
        blocked "arch" "cargo-nextest is not installed. \`cargo install --locked cargo-nextest\`,
        or run the authoritative \`do-harness verify --only test --strict\`, which includes this target."
        return 2
    fi
    if cargo nextest run -p e2e-tests --test arch_fitness; then
        echo -e "${GREEN}arch: PASS (focused diagnostic — the test sensor is the gate)${NC}" >&2
        return 0
    fi
    violation "arch" "crate-layering fitness failed; see tests/arch_fitness.rs for the violated rule"
    return 1
}

check_security() {
    # Advisory locally, mandatory in CI: .github/workflows/security.yml runs gitleaks on every
    # push/PR. Absent tool here used to print `security: PASS`, which is the false green.
    info "gitleaks detect (local diagnostic; CI owns the authoritative scan)"
    if ! command -v gitleaks >/dev/null 2>&1; then
        blocked "security" "gitleaks is not installed, so nothing was scanned. Install it from
        https://github.com/gitleaks/gitleaks/releases, or rely on the Security workflow, which
        gates the merge — this script can never substitute for it."
        return 2
    fi
    if gitleaks detect --config .gitleaks.toml --source . --redact --no-git; then
        echo -e "${GREEN}security: PASS (local diagnostic)${NC}" >&2
        return 0
    fi
    violation "security" "a secret was found; move it to an environment variable and scrub it from history"
    return 1
}

if [[ $# -ne 1 ]]; then
    usage
    exit 2
fi

COMMAND="$1"
RC=0

case "$COMMAND" in
    fmt | clippy | deny | test) verify_one "$COMMAND" || RC=$? ;;
    arch) check_arch || RC=$? ;;
    security) check_security || RC=$? ;;
    all)
        # The canonical completion set, not a parallel weaker one.
        require_cli
        info "do-harness verify --set verification --strict --record"
        if do-harness verify --set verification --strict --record; then
            echo -e "${GREEN}verification set: PASS — confirm with 'do-harness status --set verification'${NC}" >&2
        else
            violation "verification" "a sensor in the completion set failed; fix it, then re-run
                'do-harness verify --set verification --strict --record && do-harness status --set verification'"
            RC=1
        fi
        ;;
    *)
        echo "Unknown command: $COMMAND" >&2
        usage
        exit 2
        ;;
esac

exit "$RC"
