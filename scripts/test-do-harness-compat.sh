#!/usr/bin/env bash
# do-harness Compatibility Contract Test
#
# WHY: scripts/check-do-harness-compat.sh and scripts/check-do-harness-doctrine.sh
# assert that the harness CLI honors the contract the agent workflow is written
# against. A probe that cannot fail is worthless - it is the same vacuous-green class
# that made the old harness-check.sh dangerous (issue #1246): the reader trusts the
# word "Passed" and never asks whether anything actually ran.
#
# So this file feeds both probes a stub do-harness and breaks one contract element per
# case, asserting the probe turns red *and names the right check*. Case 1 is a faithful
# stub that must go green - that is what stops an always-red fixture from looking like
# diligence. Every mutation is checked for having changed something, because a sed
# no-op reports "NOT-FIRED" as a pass (LESSON-063).
#
# Usage: bash scripts/test-do-harness-compat.sh
# Exit codes: 0 every contract case behaved, 1 at least one did not.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPAT="$ROOT/scripts/check-do-harness-compat.sh"
DOCTRINE="$ROOT/scripts/check-do-harness-doctrine.sh"

WORK=$(mktemp -d)
BIN=$WORK/bin
SHADOW=$WORK/shadow
FLOOR_PATH="/usr/bin:/bin"
PROBE=$BIN/do-harness
ROOT_OK=$WORK/repo
PASS=0
FAILURES=0
RUN_RC=0
RUN_OUT=""
PATH_EXTRA=""

mkdir -p "$BIN" "$SHADOW"
trap 'rm -rf "$WORK"' EXIT

# ------------------------------------------------------------------ the stub CLI
# Answers exactly the invocations the two probes make. STUB_* variables are the
# mutations; unset means "the contract is honored".
cat >"$PROBE" <<'STUB'
#!/usr/bin/env bash
root=""
declare -a args=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --root)
            root="$2"
            shift 2
            ;;
        -q) shift ;;
        *)
            args+=("$1")
            shift
            ;;
    esac
done
sub="${args[0]:-}"
rest="${args[*]:-}"
cfg="$root/do-harness.toml"

# A missing root is "could not run" for every subcommand.
if [[ -n "$root" && ! -d "$root" ]]; then
    echo "error: root is not a directory: $root" >&2
    exit "${STUB_BADROOT_RC:-2}"
fi

set_members() {
    [[ -f "$cfg" ]] || return 0
    sed -n '/^\[signal-sets\]/,/^\[\[/p' "$cfg" | tr -d '\n' |
        grep -oE "(^|[^a-zA-Z0-9_-])$1[[:space:]]*=[[:space:]]*\[[^]]*\]" |
        grep -oE '"[a-z0-9_-]+"' | tr -d '"' | LC_ALL=C sort
}

known_set() { # $1 = set name
    [[ -f "$cfg" ]] || return 1
    grep -qE "^$1[[:space:]]*=" <<<"$(sed -n '/^\[signal-sets\]/,/^\[\[/p' "$cfg")"
}

json_sensor() { # $1 name, $2 sensor-ok, $3 failed-entry (empty for none), $4 exit_code
    local failed='[]'
    [[ -z "$3" ]] || failed="[\"$3\"]"
    printf '{"ok":%s,"failed":%s,"sensors":[{"name":"%s","ok":%s,"exit_code":%s,"duration_ms":7,"allow_failure":false,"warned":false}]}\n' \
        "$2" "$failed" "$1" "$2" "$4"
}

write_evidence_if_asked() {
    [[ "$rest" == *"--evidence "* ]] || return 0
    local ev="${rest##*--evidence }"
    ev="${ev%% *}"
    [[ -n "$ev" ]] && printf '{"schema_version":4,"sensors":[{"name":"green","argv":["bash","-c","true"]}]}' >"$ev"
    return 0
}

case "$sub" in
    version)
        printf '{"name":"do-harness","version":"%s","commit":"abc1234","commit_date":"2026-09-15","dirty":%s}\n' \
            "${STUB_VERSION:-0.1.0}" "${STUB_DIRTY:-false}"
        exit 0
        ;;
    --help)
        echo "Commands:"
        # shellcheck disable=SC2086  # intentional word splitting of the subcommand list
        for s in ${STUB_SUBS:-verify status explain list doctor version}; do
            echo "  $s    stub"
        done
        exit 0
        ;;
    verify)
        if [[ "$rest" == *"--help"* ]]; then
            echo "Options:"
            # shellcheck disable=SC2086  # intentional word splitting of the flag list
            for f in ${STUB_VERIFY_FLAGS:---set --only --strict --record --format --exclude --config --root}; do
                echo "      $f <V>"
            done
            exit 0
        fi
        if [[ "$rest" =~ --set[[:space:]]+([^[:space:]]+) ]]; then
            known_set "${BASH_REMATCH[1]}" ||
                {
                    echo "error: unknown signal set '${BASH_REMATCH[1]}'" >&2
                    exit "${STUB_SET_RC:-2}"
                }
        fi
        if [[ "$rest" =~ --only[[:space:]]+([^[:space:]]+) ]]; then
            s="${BASH_REMATCH[1]}"
            grep -qE "^name = \"$s\"" "$cfg" ||
                {
                    echo "error: unknown sensor(s): $s" >&2
                    exit "${STUB_ONLY_RC:-2}"
                }
        fi
        case "$rest" in
            *--only\ green*)
                write_evidence_if_asked
                json_sensor green true "" 0
                echo
                exit "${STUB_GREEN_RC:-0}"
                ;;
            *--only\ red*)
                json_sensor red false red 3
                echo
                exit "${STUB_RED_RC:-1}"
                ;;
            *--only\ ghost*)
                json_sensor ghost false ghost null
                echo
                if [[ "$rest" == *--strict* ]]; then
                    exit "${STUB_GHOST_STRICT_RC:-1}"
                fi
                exit "${STUB_GHOST_RC:-1}"
                ;;
            *--only\ fmt*)
                code="${STUB_FMT_CODE:-0}"
                ok=true
                [[ "$code" == 0 ]] || ok=false
                printf '{"ok":%s,"failed":[],"sensors":[{"name":"fmt","ok":%s,"exit_code":%s,"duration_ms":700,"allow_failure":false,"warned":false}]}\n' \
                    "$ok" "$ok" "$code"
                if [[ -n "${STUB_FMT_RC:-}" ]]; then
                    exit "$STUB_FMT_RC"
                fi
                [[ "$code" == 0 ]] && exit 0
                exit 1
                ;;
            *--set\ feedback\ --strict\ --record*)
                json_sensor green true "" 0
                echo
                exit "${STUB_RECORD_RC:-0}"
                ;;
            *)
                echo "error: unsupported invocation: $rest" >&2
                exit 2
                ;;
        esac
        ;;
    status)
        if [[ "$rest" == *"--help"* ]]; then
            echo "Options:"
            echo "      --set <V>"
            echo "      --format <V>"
            exit 0
        fi
        if [[ "$rest" =~ --set[[:space:]]+([^[:space:]]+) ]]; then
            known_set "${BASH_REMATCH[1]}" ||
                {
                    echo "error: unknown signal set '${BASH_REMATCH[1]}'" >&2
                    exit "${STUB_STATUS_SET_RC:-2}"
                }
        fi
        echo '{"state":"green","set":"feedback","reason":"current"}'
        exit 0
        ;;
    explain)
        if [[ "$rest" == *"--help"* ]]; then
            echo "Options:"
            echo "      --set <V>"
            echo "      --format <V>"
            echo "      --changed"
            exit 0
        fi
        [[ "$rest" =~ --set[[:space:]]+([^[:space:]]+) ]] || exit 2
        s="${BASH_REMATCH[1]}"
        known_set "$s" ||
            {
                echo "error: unknown signal set '$s'" >&2
                exit 2
            }
        members=$(set_members "$s")
        if [[ -n "${STUB_SETS_MISMATCH:-}" ]]; then
            members=$(grep -v "^${STUB_SETS_MISMATCH}$" <<<"$members" || true)
        fi
        printf '{"set":"%s","changed_files":[],"selected":[%s],"skipped":[]}\n' "$s" \
            "$(printf '%s\n' "$members" | grep . | sed 's/.*/{"name":"&","reason":"stub"}/' | paste -sd, -)"
        exit 0
        ;;
    list)
        [[ -f "$cfg" ]] || {
            echo "error: no do-harness.toml under $root" >&2
            exit 2
        }
        grep -oE '^name = "[a-z0-9_-]+"' "$cfg" | sed 's/.*"\(.*\)"/\1/'
        exit 0
        ;;
    doctor)
        echo "Doctor checks passed."
        exit "${STUB_DOCTOR_RC:-0}"
        ;;
    *)
        echo "error: unknown subcommand $sub" >&2
        exit 2
        ;;
esac
STUB
chmod +x "$PROBE"

# --------------------------------------------------------------- test workspace
build_repo_fixture() {
    local r=$1
    rm -rf "$r"
    mkdir -p "$r/scripts/probe" "$r/.agents/skills/harness"
    cat >"$r/do-harness.toml" <<'TOML'
language = "rust"

[signal-sets]
feedback = ["probe-green"]
verification = ["probe-green", "probe-red"]

[[sensors]]
name = "probe-green"
argv = ["bash", "scripts/probe/green.sh"]

[[sensors]]
name = "probe-red"
argv = ["bash", "scripts/probe/red.sh"]

[[sensors]]
name = "fmt"
argv = ["cargo", "fmt", "--all", "--", "--check"]
TOML
    printf 'exit 0\n' >"$r/scripts/probe/green.sh"
    printf 'exit 1\n' >"$r/scripts/probe/red.sh"
    chmod +x "$r/scripts/probe/"*.sh
    printf 'pin = "0.1.0"\nvalidated_through = "0.4.1"\n' >"$r/scripts/do-harness-version.txt"
    printf 'Pinned compatible harness version: `v0.1.0`.\n' >"$r/AGENTS.md"
    printf 'Signal sets (pinned harness version `v0.1.0`):\n' >"$r/HARNESS.md"
    printf 'Pinned compatible harness version: `v0.1.0`.\n' >"$r/.agents/skills/harness/SKILL.md"
}
build_repo_fixture "$ROOT_OK"

clear_env() {
    PATH_EXTRA=""
    unset STUB_VERSION STUB_DIRTY STUB_SUBS STUB_VERIFY_FLAGS STUB_SET_RC STUB_ONLY_RC \
        STUB_GREEN_RC STUB_RED_RC STUB_GHOST_RC STUB_GHOST_STRICT_RC STUB_RECORD_RC \
        STUB_STATUS_SET_RC STUB_DOCTOR_RC STUB_SETS_MISMATCH STUB_FMT_RC STUB_FMT_CODE \
        STUB_BADROOT_RC 2>/dev/null || true
}

run_compat() {
    local r="$1"
    shift
    RUN_RC=0
    # env, never an assignment prefix or a subshell export: a newline ends a prefix list,
    # so a prefix spread across lines silently stops reaching the child and the probe
    # measures nothing, and `export` inside `$( )` only scopes to a subshell the parent
    # cannot read back.
    RUN_OUT=$(
        env PATH="$PATH_EXTRA$BIN:$FLOOR_PATH" DO_HARNESS_BIN="$PROBE" \
            bash "$COMPAT" --root "$r" "$@" 2>&1
    ) || RUN_RC=$?
}
run_doctrine() {
    RUN_RC=0
    RUN_OUT=$(
        env PATH="$BIN:$FLOOR_PATH" DO_HARNESS_BIN="$PROBE" \
            bash "$DOCTRINE" 2>&1
    ) || RUN_RC=$?
}

pass() {
    PASS=$((PASS + 1))
    echo "ok   - $1"
}
fail() {
    printf 'FAIL - %s\n' "$1" >&2
    if [[ -n "$RUN_OUT" ]]; then
        printf '%s\n' "$RUN_OUT" | sed 's/^/       /' >&2
    fi
    FAILURES=$((FAILURES + 1))
}
expect_red() { # label expected-substring
    if [[ "$RUN_RC" -eq 1 ]] && grep -qF -- "$2" <<<"$RUN_OUT"; then
        pass "$1"
    else
        fail "$1 (rc=$RUN_RC, wanted '$2')"
    fi
}
expect_compat_green() {
    if [[ "$RUN_RC" -eq 0 ]] && grep -q 'Compatibility Passed' <<<"$RUN_OUT"; then
        pass "$1"
    else
        fail "$1 (rc=$RUN_RC, expected green)"
    fi
}
expect_doctrine_green() {
    if [[ "$RUN_RC" -eq 0 ]] && grep -q 'Status Contract Passed' <<<"$RUN_OUT"; then
        pass "$1"
    else
        fail "$1 (rc=$RUN_RC, expected green)"
    fi
}

expect_blocked() { # label expected-substring
    if [[ "$RUN_RC" -eq 2 ]] && grep -qF -- "$2" <<<"$RUN_OUT"; then
        pass "$1"
    else
        fail "$1 (rc=$RUN_RC, expected 2 = nothing verified)"
    fi
}

echo "=== do-harness Compatibility Contract Cases ==="

# 1. The control. A faithful stub must go green, or every case below proves nothing.
clear_env
run_compat "$ROOT_OK" --expect-version 0.1.0
expect_compat_green "faithful stub: compat passes"
run_doctrine
expect_doctrine_green "faithful stub: doctrine passes"

# 2. Binary older than the tracked pin.
clear_env
export STUB_VERSION=0.0.9
run_compat "$ROOT_OK"
expect_red "older-than-pin binary is refused" "older than the tracked pin 0.1.0"

# 3. A routing doc naming a version the pin file does not.
clear_env
printf 'Pinned compatible harness version: `v9.9.9`.\n' >"$ROOT_OK/AGENTS.md"
run_compat "$ROOT_OK"
expect_red "prose pin drift is caught" "claims a pinned harness version 9.9.9"
printf 'Pinned compatible harness version: `v0.1.0`.\n' >"$ROOT_OK/AGENTS.md"

# 4. A documented subcommand disappears.
clear_env
export STUB_SUBS="verify status list doctor version"
run_compat "$ROOT_OK"
expect_red "dropped 'explain' subcommand is caught" "not offered by '--help'"

# 5. verify stops documenting --strict.
clear_env
export STUB_VERIFY_FLAGS="--set --only --record --format --exclude --config --root"
run_compat "$ROOT_OK"
expect_red "dropped '--strict' is caught" "verify no longer documents: --strict"

# 6. The CLI and do-harness.toml disagree about set membership.
clear_env
export STUB_SETS_MISMATCH="probe-red"
run_compat "$ROOT_OK"
expect_red "set-membership drift is caught" "differs from do-harness.toml"

# 7. A sensor's argv script has been renamed away.
clear_env
rm -f "$ROOT_OK/scripts/probe/red.sh"
run_compat "$ROOT_OK"
expect_red "missing sensor argv path is caught" "missing script(s)"
printf 'exit 1\n' >"$ROOT_OK/scripts/probe/red.sh"
chmod +x "$ROOT_OK/scripts/probe/red.sh"

# 8. fmt is red but the CLI reports the run as green.
clear_env
export STUB_FMT_CODE=1 STUB_FMT_RC=0
run_compat "$ROOT_OK"
expect_red "a red sensor reported as green is caught" "reported as green"

# 9. PATH shadowing: two distinct binaries, so ordering decides which one answers.
clear_env
cp "$PROBE" "$SHADOW/do-harness"
chmod +x "$SHADOW/do-harness"
PATH_EXTRA="$SHADOW:" run_compat "$ROOT_OK"
expect_red "two do-harness binaries on PATH is caught" "several do-harness binaries on PATH"
rm -f "$SHADOW/do-harness"

# 10. The binary is not the artifact CI checksum-verified.
clear_env
run_compat "$ROOT_OK" --expect-version 0.1.0 --expect-sha256 deadbeef
expect_red "checksum mismatch is caught" "not the checksum this run was told to expect"

# 11. A dirty build cannot back published compat evidence.
clear_env
export STUB_DIRTY=true
run_compat "$ROOT_OK" --expect-version 0.1.0
expect_red "a dirty build is refused in a pinned run" "dirty build"

# 12. Nothing to verify must never read as a pass: missing version file, no binary.
clear_env
rm -f "$ROOT_OK/scripts/do-harness-version.txt"
run_compat "$ROOT_OK"
expect_blocked "a missing version file blocks the probe" "no tracked version reference"
build_repo_fixture "$ROOT_OK"
RUN_RC=0
RUN_OUT=$(
    env PATH="$FLOOR_PATH" DO_HARNESS_BIN="$WORK/no-such-binary" \
        bash "$COMPAT" --root "$ROOT_OK" 2>&1
) || RUN_RC=$?
expect_blocked "an unusable binary blocks rather than passes" "HARNESS BLOCKED"

# 13. Doctrine mutations: the three exit tiers and the --strict evidence gate.
clear_env
export STUB_RED_RC=0
run_doctrine
expect_red "a failing sensor that exits 0 is caught" "contract says 1"

clear_env
export STUB_GHOST_STRICT_RC=0
run_doctrine
expect_red "--strict accepting weak evidence is caught" "weak evidence is being accepted"

clear_env
export STUB_SET_RC=1
run_doctrine
expect_red "an unknown set that does not exit 2 is caught" "'verify --set nosuchset' exited 1"

clear_env
export STUB_GREEN_RC=1
run_doctrine
expect_red "a passing sensor that exits 1 is caught" "contract says 0"

# 14. Pack input contract. A sensor that declares no `inputs` gets the
# `input:git-index-and-nonignored-untracked` fallback recorded as a null digest,
# which makes `do-harness status --set <set>` report
# `stale (inputs_changed_or_unknown)` for every change and breaks the documented
# "confirm green local evidence boundary" line. The probe asserts the pack of the
# repository it lives in, so these cases run a copy of it inside a fixture whose
# pack is the mutation.
build_pack_fixture() {
    local r=$1
    rm -rf "$r"
    mkdir -p "$r/scripts"
    cp "$DOCTRINE" "$r/scripts/check-do-harness-doctrine.sh"
}

build_pack_fixture "$WORK/pack-no-inputs"
cat >"$WORK/pack-no-inputs/do-harness.toml" <<'TOML'
[[sensors]]
name = "probe-green"
argv = ["bash", "scripts/probe/green.sh"]
TOML
clear_env
RUN_RC=0
RUN_OUT=$(
    env PATH="$BIN:$FLOOR_PATH" DO_HARNESS_BIN="$PROBE" \
        bash "$WORK/pack-no-inputs/scripts/check-do-harness-doctrine.sh" 2>&1
) || RUN_RC=$?
expect_red "a sensor that declares no 'inputs' is caught" \
    "sensors declare no inputs (status can never report green): probe-green"

build_pack_fixture "$WORK/pack-empty-inputs"
cat >"$WORK/pack-empty-inputs/do-harness.toml" <<'TOML'
[[sensors]]
name = "probe-green"
argv = ["bash", "scripts/probe/green.sh"]
inputs = []
TOML
clear_env
RUN_RC=0
RUN_OUT=$(
    env PATH="$BIN:$FLOOR_PATH" DO_HARNESS_BIN="$PROBE" \
        bash "$WORK/pack-empty-inputs/scripts/check-do-harness-doctrine.sh" 2>&1
) || RUN_RC=$?
expect_red "an empty 'inputs' list is caught" \
    "sensors declare an empty inputs list, which upstream treats as the omitted case"

build_pack_fixture "$WORK/pack-empty-glob"
cat >"$WORK/pack-empty-glob/do-harness.toml" <<'TOML'
[[sensors]]
name = "probe-green"
argv = ["bash", "scripts/probe/green.sh"]
inputs = ["no/such/glob/**/*.rs"]
TOML
clear_env
RUN_RC=0
RUN_OUT=$(
    env PATH="$BIN:$FLOOR_PATH" DO_HARNESS_BIN="$PROBE" \
        bash "$WORK/pack-empty-glob/scripts/check-do-harness-doctrine.sh" 2>&1
) || RUN_RC=$?
expect_red "an input glob matching no file is caught" \
    "input patterns match no file (record a null digest): probe-green:no/such/glob/**/*.rs"

# 15. The pack section's interpreter must fail closed. The probe runs under
# `set -uo pipefail` with no `-e`, so an absent, failing or silent python3 would
# otherwise leave the read loop with nothing to consume and let the probe print
# "Passed" without asserting the pack at all - the vacuous-green class this file
# exists to prevent.
mkdir -p "$WORK/py-fail" "$WORK/py-silent"
printf '#!/usr/bin/env bash\necho "no tomllib" >&2\nexit 3\n' >"$WORK/py-fail/python3"
printf '#!/usr/bin/env bash\nexit 0\n' >"$WORK/py-silent/python3"
chmod +x "$WORK/py-fail/python3" "$WORK/py-silent/python3"

clear_env
RUN_RC=0
RUN_OUT=$(
    env PATH="$WORK/py-fail:$BIN:$FLOOR_PATH" DO_HARNESS_BIN="$PROBE" bash "$DOCTRINE" 2>&1
) || RUN_RC=$?
expect_red "a failing python3 blocks the pack contract" "could not be evaluated (python3 exited 3)"

clear_env
RUN_RC=0
RUN_OUT=$(
    env PATH="$WORK/py-silent:$BIN:$FLOOR_PATH" DO_HARNESS_BIN="$PROBE" bash "$DOCTRINE" 2>&1
) || RUN_RC=$?
expect_red "a silent python3 blocks the pack contract" "could not be evaluated (it produced no output)"

echo
if [[ "$FAILURES" -gt 0 ]]; then
    echo "HARNESS VIOLATION: harness-compat-contract - $FAILURES contract case(s) failed" >&2
    exit 1
fi
echo "=== do-harness Compatibility Contract Passed ($PASS cases) ==="
