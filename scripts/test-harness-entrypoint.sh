#!/usr/bin/env bash
# test-harness-entrypoint.sh — contract fixtures for scripts/harness-check.sh
#
# WHY: the shim's whole job is to delegate, so the property worth testing is the
# delegation itself — which command it runs, with which argv, and what it reports when
# the delegate fails or is absent. Those are exactly the things that used to be wrong:
# a private clippy invocation with 27 suppressions, `deny: PASS` with no cargo-deny,
# `security: PASS` with no gitleaks, and an `arch` command naming a package that does
# not exist (issue #1246).
#
# Every case runs against stub binaries on a private PATH, so the assertions are about
# the contract and never about this machine's toolchain. No network, no Rust compile —
# CI runs it in .github/workflows/skill-evals.yml.
#
# Usage: bash scripts/test-harness-entrypoint.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SHIM="$ROOT/scripts/harness-check.sh"
REGISTRY="$ROOT/do-harness.toml"
DOCS="$ROOT/HARNESS.md"

# /usr/bin:/bin is the floor; it deliberately excludes ~/.cargo/bin and ~/.local/bin, so
# "tool absent" is the default state and each case opts in by dropping a stub in $BIN.
FLOOR_PATH="/usr/bin:/bin"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
BIN="$WORK/bin"
ARGV_LOG="$WORK/argv.log"
mkdir -p "$BIN"
export ARGV_LOG

RUN_OUT=""
RUN_RC=0
FAILURES=0
CASES=0

stub() {
    # stub <name> <body>
    local name="$1" body="$2"
    printf '#!/usr/bin/env bash\n%s\n' "$body" > "$BIN/$name"
    chmod +x "$BIN/$name"
}

clear_stubs() { find "$BIN" -mindepth 1 -delete; }

run_shim() {
    : > "$ARGV_LOG"
    RUN_RC=0
    RUN_OUT="$(PATH="$BIN:$FLOOR_PATH" bash "$SHIM" "$@" 2>&1)" || RUN_RC=$?
}

pass() { CASES=$((CASES + 1)); echo "OK   $1"; }

fail() {
    CASES=$((CASES + 1))
    FAILURES=$((FAILURES + 1))
    echo "FAIL $1" >&2
    if [[ -n "$RUN_OUT" ]]; then
        printf '%s\n' "----- last captured output -----$RUN_OUT-----" >&2
    fi
}

assert_exit() {
    if [[ "$RUN_RC" == "$2" ]]; then pass "$1 (exit $2)"; else fail "$1: expected exit $2, got $RUN_RC"; fi
}

assert_argv() {
    if grep -qxF "$2" "$ARGV_LOG"; then pass "$1"; else fail "$1 — argv log was: $(tr '\n' ' ' < "$ARGV_LOG")"; fi
}

assert_has() {
    if grep -qF -- "$2" <<<"$RUN_OUT"; then pass "$1"; else fail "$1 (missing: $2)"; fi
}

assert_no_pass() {
    if grep -q ': PASS' <<<"$RUN_OUT"; then fail "$1 — output claimed PASS"; else pass "$1"; fi
}

echo "=== harness-check.sh entrypoint contract ==="

# ------------------------------------------------------------- mapped sensor names
for sensor in fmt clippy deny test; do
    clear_stubs
    stub do-harness 'printf "do-harness %s\n" "$*" >> "$ARGV_LOG"; exit 0'
    run_shim "$sensor"
    assert_exit "$sensor delegates" 0
    assert_argv "$sensor argv" "do-harness verify --only $sensor --strict"
    assert_has "$sensor reports pass" "$sensor: PASS"
done

# The private clippy command must be gone, not merely shadowed by the delegate.
clear_stubs
stub do-harness 'printf "do-harness %s\n" "$*" >> "$ARGV_LOG"; exit 0'
run_shim clippy
if grep -qE 'clippy::|-A |--lib' "$ARGV_LOG"; then
    fail "no hand-rolled clippy invocation survives: $(tr '\n' ' ' < "$ARGV_LOG")"
else
    pass "no hand-rolled clippy invocation survives"
fi

# -------------------------------------------------------------------- failure paths
clear_stubs
stub do-harness 'printf "do-harness %s\n" "$*" >> "$ARGV_LOG"; exit 1'
run_shim fmt
assert_exit "failing sensor propagates" 1
assert_has "failing sensor is a violation" "HARNESS VIOLATION: [fmt]"
assert_no_pass "failing sensor never says PASS"

clear_stubs
run_shim fmt
assert_exit "absent CLI blocks" 2
assert_has "absent CLI names the remedy" "HARNESS BLOCKED: [do-harness]"
assert_has "absent CLI points at the release channel" "d-o-hub/do-harness"
assert_no_pass "absent CLI never says PASS"

clear_stubs
run_shim not-a-sensor
assert_exit "unknown subcommand rejected" 2
assert_has "unknown subcommand prints usage" "Usage: bash scripts/harness-check.sh"
assert_no_pass "unknown subcommand never says PASS"

# ------------------------------------------------------------------------------ all
clear_stubs
stub do-harness 'printf "do-harness %s\n" "$*" >> "$ARGV_LOG"; exit 0'
run_shim all
assert_exit "all runs the canonical set" 0
assert_argv "all argv" "do-harness verify --set verification --strict --record"

clear_stubs
stub do-harness 'printf "do-harness %s\n" "$*" >> "$ARGV_LOG"; exit 1'
run_shim all
assert_exit "all propagates a failed set" 1
assert_has "all reports the failed set" "HARNESS VIOLATION: [verification]"

# ------------------------------------------------------------------------ security
clear_stubs
run_shim security
assert_exit "security without gitleaks blocks" 2
assert_has "security states nothing was scanned" "nothing was scanned"
assert_no_pass "security never passes on an absent tool"

clear_stubs
stub gitleaks 'exit 1'
run_shim security
assert_exit "a gitleaks finding fails" 1
assert_has "a gitleaks finding is a violation" "HARNESS VIOLATION: [security]"

clear_stubs
stub gitleaks 'exit 0'
run_shim security
assert_exit "clean gitleaks passes" 0
assert_has "clean gitleaks is labelled a diagnostic" "local diagnostic"

# ---------------------------------------------------------------------------- arch
clear_stubs
stub cargo 'printf "cargo %s\n" "$*" >> "$ARGV_LOG"
if [[ "$1" == "nextest" && "$2" == "--version" ]]; then echo "cargo-nextest 0.9.0"; exit 0; fi
exit 0'
run_shim arch
assert_exit "arch runs the fitness test" 0
assert_argv "arch targets the real package" "cargo nextest run -p e2e-tests --test arch_fitness"
assert_has "arch labels itself a diagnostic" "subset of the test sensor"

clear_stubs
run_shim arch
assert_exit "arch without cargo-nextest blocks" 2
assert_has "arch names the authoritative sensor" "verify --only test --strict"
assert_no_pass "arch never passes without the tool"

# ------------------------------------------------- registry and documentation parity
# A mapped name that is not a real sensor would resolve to nothing and report a verdict
# about nothing — the `-p tests` bug in a new costume.
missing=""
for sensor in fmt clippy deny test; do
    grep -qE "^name = \"$sensor\"$" "$REGISTRY" || missing="$missing $sensor"
done
if [[ -z "$missing" ]]; then
    pass "every mapped name is a sensor in do-harness.toml"
else
    fail "mapped names absent from do-harness.toml:$missing"
fi

# The subcommand list an operator reads must be the list the script actually handles.
documented=$(sed -n '/^usage() {/,/^}/p' "$SHIM" | grep -oE '<[a-z][a-z|]*>' | head -1 \
    | tr -d '<>' | tr '|' ' ' | tr -s ' ' '\n' | LC_ALL=C sort | tr '\n' ' ')
in_docs=$(sed -n '/harness-check\.sh/p' "$DOCS" | grep -oE '<[a-z][a-z|]*>' | head -1 | tr -d '<>' | tr '|' ' ' \
    | tr -s ' ' '\n' | LC_ALL=C sort | tr '\n' ' ')
implemented=$(sed -n '/^case "\$COMMAND" in/,/^esac/p' "$SHIM" \
    | grep -oE '^[[:space:]]+[a-z][a-z-]*( \| [a-z][a-z-]*)*\)' | sed 's/)[[:space:]]*$//' \
    | tr '|' ' ' | tr -s ' \t' '\n' | grep -v '^$' | LC_ALL=C sort | tr '\n' ' ')
expected=$(printf 'all\narch\nclippy\ndeny\nfmt\nsecurity\ntest\n' | LC_ALL=C sort | tr '\n' ' ')

for pair in "usage() text:$documented" "HARNESS.md:$in_docs" "case handlers:$implemented"; do
    label="${pair%%:*}" actual="${pair#*:}"
    if [[ "$actual" == "$expected" ]]; then
        pass "$label lists exactly the contracted subcommands"
    else
        fail "$label: got '$actual', expected '$expected'"
    fi
done

# `all` is only a completion contract if the verification set names real sensors: a
# dangling entry, or an empty set, would let the delegate exit 0 having run nothing —
# the vacuous-pass class the harness skill calls out.
set_names=$(sed -n '/^verification = \[/,/^\]/p' "$REGISTRY" | grep -oE '"[a-z][a-z-]*"' | tr -d '"')
set_count=$(echo "$set_names" | grep -c . || true)
if [[ "$set_count" -gt 0 ]]; then
    pass "the verification set is not empty ($set_count sensors)"
else
    fail "the verification set is empty: 'all' would certify nothing"
fi
dangling=""
for name in $set_names; do
    grep -qE "^name = \"$name\"$" "$REGISTRY" || dangling="$dangling $name"
done
if [[ -z "$dangling" ]]; then
    pass "every verification-set name is a defined sensor"
else
    fail "verification set names no sensor:$dangling"
fi

# The dead command must not come back: `-p tests` matches no package in this workspace.
# Whole-line comments are stripped first, because the shim's header documents the old
# invocation — prose about a prohibition must not trip the gate that enforces it
# (LESSON-079).
if grep -vE '^[[:space:]]*#' "$SHIM" | grep -qF -- '-p tests'; then
    fail "the dead \`-p tests\` arch command is back"
else
    pass "no \`-p tests\` package reference in code (the package is e2e-tests)"
fi

echo "=== entrypoint contract: $CASES cases, $FAILURES failure(s) ==="
if [[ "$FAILURES" != "0" ]]; then
    echo "HARNESS VIOLATION: harness-entrypoint - $FAILURES contract case(s) failed" >&2
    exit 1
fi
exit 0
