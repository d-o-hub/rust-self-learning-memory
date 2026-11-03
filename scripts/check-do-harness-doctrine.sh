#!/usr/bin/env bash
# do-harness Status Contract (exit codes + JSON schema) Probe
#
# Companion to scripts/check-do-harness-compat.sh. That file asks "is the installed
# binary the one the docs promise, and does it accept the documented flags?". This one
# asks the harder question, because the flags can all exist while the answers change:
# does the CLI still honor the status contract the whole agent workflow is written
# against?
#
#   exit 0  -> every sensor ran and passed
#   exit 1  -> a sensor ran and failed (or --strict refused weak evidence)
#   exit 2  -> nothing ran (unknown set, unknown sensor, bad root, bad flag)
#
# Those three tiers are the doctrine in AGENTS.md and .agents/skills/harness/SKILL.md.
# If a release collapses "could not run" into "failed", an agent reads a config typo as
# a real gate red and fixes code that was never wrong; if it collapses the other way, a
# broken sensor certifies silently. Both are the vacuous-pass class, so the cases below
# are exercised against a green sensor, a deliberately failing sensor, and an
# unspawnable one, in a throwaway root - never against this repository's real suite.
#
# Exit codes: 0 contract holds, 1 contract broken, 2 nothing verified (no binary/jq).
#
# Usage: bash scripts/check-do-harness-doctrine.sh [--bin <path>] [--expect-version <v>]
#   --bin <path>          probe this binary (default: $DO_HARNESS_BIN, then PATH)
#   --expect-version <v>  name this version in the report (CI pins it)
#   -h, --help            this text

set -uo pipefail

BIN="${DO_HARNESS_BIN:-}"
EXPECT_VERSION=""
VIOLATIONS=0
RC=0
OUT=""
ERR=""
FIX=""
RESOLVED=""

violation() {
    echo "HARNESS VIOLATION: harness-doctrine - $1" >&2
    VIOLATIONS=$((VIOLATIONS + 1))
}
ok() { echo "OK: $1"; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --bin) BIN="${2:?--bin needs a path}"; shift 2 ;;
        --expect-version) EXPECT_VERSION="${2:?--expect-version needs a version}"; shift 2 ;;
        -h | --help)
            sed -n '/^# Usage:/,/^#   -h/p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown arg: $1" >&2
            exit 2
            ;;
    esac
done

command -v jq >/dev/null 2>&1 || {
    echo "HARNESS BLOCKED: harness-doctrine - nothing was verified. jq is required." >&2
    exit 2
}
candidate="$BIN"
[[ -n "$candidate" ]] || candidate="$(command -v do-harness 2>/dev/null || true)"
[[ -n "$candidate" ]] || {
    echo "HARNESS BLOCKED: harness-doctrine - nothing was verified. No do-harness binary on PATH; see .github/workflows/harness-compat.yml for the pinned download." >&2
    exit 2
}
[[ -x "$candidate" ]] || {
    echo "HARNESS BLOCKED: harness-doctrine - '$candidate' is not executable." >&2
    exit 2
}
BIN="$candidate"
RESOLVED="$(realpath "$BIN" 2>/dev/null || echo "$BIN")"

run() { # $1 = root, or "-" for none; rest = CLI args. Sets RC/OUT/ERR.
    local root="$1"
    shift
    local of ef
    of=$(mktemp)
    ef=$(mktemp)
    local -a argv=()
    [[ "$root" == "-" ]] || argv=(--root "$root")
    "$BIN" "${argv[@]+"${argv[@]}"}" -q "$@" >"$of" 2>"$ef"
    RC=$?
    OUT=$(cat "$of")
    ERR=$(cat "$ef")
    rm -f "$of" "$ef"
}

is_json() { jq -e . >/dev/null 2>&1 <<<"$OUT"; }

# Three sensors, three distinct states: green, a real non-zero exit, and an argv that
# cannot be spawned at all. A fixture with only a green sensor would prove nothing
# about the red path, which is the one agents act on.
FIX=$(mktemp -d)
git init -q "$FIX" 2>/dev/null || true
cat >"$FIX/do-harness.toml" <<'TOML'
language = "rust"

[signal-sets]
feedback = ["green"]
verification = ["green", "red", "ghost"]

[[sensors]]
name = "green"
argv = ["bash", "-c", "echo green-sensor-ran; exit 0"]

[[sensors]]
name = "red"
argv = ["bash", "-c", "echo red-sensor-ran; exit 3"]

[[sensors]]
name = "ghost"
argv = ["do-harness-probe-binary-that-does-not-exist"]
TOML

expect_rc() { # root expected_rc label [args...] -> 0 when the contract held
    local root="$1" want="$2" label="$3"
    shift 3
    run "$root" "$@"
    if [[ "$RC" -ne "$want" ]]; then
        violation "$label exited $RC, contract says $want (0 passed / 1 ran-and-failed / 2 could not run); cli said: $(head -c 160 <<<"$ERR")"
        return 1
    fi
    return 0
}

echo "=== do-harness Status Contract ==="
VERSION=""
if run "-" version --format json && jq -e . >/dev/null 2>&1 <<<"$OUT"; then
    VERSION=$(jq -r '.version // empty' <<<"$OUT")
fi
echo "BINARY: $RESOLVED"
echo "VERSION: ${VERSION:-unknown}${EXPECT_VERSION:+ (this run verifies $EXPECT_VERSION)}"

bad=0
if expect_rc "$FIX" 0 "'verify --only green'" verify --only green --format json; then
    jq -e '.ok==true and (.failed|length)==0' <<<"$OUT" >/dev/null 2>&1 ||
        violation "a passing sensor did not report {ok:true, failed:[]}: $(head -c 160 <<<"$OUT")"
else
    bad=1
fi
if expect_rc "$FIX" 1 "'verify --only red'" verify --only red --format json; then
    jq -e '.ok==false and (.failed==["red"]) and (.sensors[0].exit_code==3)' <<<"$OUT" >/dev/null 2>&1 ||
        violation "a failing sensor did not report {ok:false, failed:[red], exit_code:3}: $(head -c 160 <<<"$OUT")"
else
    bad=1
fi
# An unspawnable argv reports exit_code null. The exit code alone cannot tell
# "gate red" from "gate never ran", so the null is the only signal available to a
# caller - if it disappears, a config typo is silently readable as a genuine failure.
if expect_rc "$FIX" 1 "'verify --only ghost'" verify --only ghost --format json; then
    jq -e '.sensors[0].exit_code==null' <<<"$OUT" >/dev/null 2>&1 ||
        violation "an unspawnable sensor no longer reports exit_code null: $(head -c 160 <<<"$OUT")"
else
    bad=1
fi
expect_rc "$FIX" 2 "'verify --set nosuchset'" verify --set nosuchset --format json || bad=1
expect_rc "$FIX" 2 "'verify --only nosuchsensor'" verify --only nosuchsensor --format json || bad=1
expect_rc "/tmp/do-harness-probe-no-such-dir" 2 "a run against a missing root" list || bad=1
[[ "$bad" -eq 0 ]] && ok "exit-code doctrine holds: 0 pass / 1 ran-and-failed / 2 could-not-run"

run "$FIX" verify --only green --format json
if ! is_json; then
    violation "'-q verify --format json' mixed non-JSON text into stdout: $(head -c 120 <<<"$OUT")"
elif ! jq -e '
    (type=="object") and (.ok|type=="boolean") and (.failed|type=="array") and
    (.sensors|type=="array") and ((.sensors|length)>0) and
    ([.sensors[] | has("name") and has("ok") and has("exit_code") and
               has("duration_ms") and has("allow_failure") and has("warned")] | all)
' <<<"$OUT" >/dev/null 2>&1; then
    violation "verify JSON lost a field the contract depends on (name/ok/exit_code/duration_ms/allow_failure/warned)"
else
    ok "verify JSON schema intact for the fields this repo consumes"
fi
# `severity` is version-dependent, not contract-critical: measured absent on 0.1.0,
# present from 0.1.1 on. Reporting it keeps the difference visible so nobody writes an
# agent script that depends on a field the pinned floor does not provide.
if jq -e '.sensors[0] | has("severity")' <<<"$OUT" >/dev/null 2>&1; then
    echo "REPORT: .sensors[].severity present (value $(jq -r '.sensors[0].severity' <<<"$OUT"))"
else
    echo "REPORT: .sensors[].severity absent on this binary - do not depend on it"
fi

# --strict is what `verify --set verification --strict --record` rests on: weak
# evidence has to be refused rather than reported as a pass.
run "$FIX" verify --only green --strict --format json
if [[ "$RC" -eq 0 ]]; then
    ok "--strict accepts a complete sensor record"
else
    violation "'verify --only green --strict' exited $RC; the completion contract's own flag is refused"
fi
run "$FIX" verify --only ghost --strict --format json
if [[ "$RC" -eq 1 ]]; then
    ok "--strict rejects weak evidence (unspawnable sensor)"
else
    violation "'--strict' exited $RC on an unspawnable sensor; weak evidence is being accepted"
fi

# schema_version grows across releases (3 -> 4 -> 5 measured), so the assertion names
# the fields consumed, never an exact schema number.
ev="$FIX/evidence.json"
run "$FIX" verify --only green --evidence "$ev" --format json
if [[ -s "$ev" ]] && jq -e '.sensors[0].argv|type=="array"' "$ev" >/dev/null 2>&1; then
    ok "--evidence artifact records sensor argv (schema_version $(jq -r '.schema_version // "?"' "$ev"))"
else
    violation "--evidence wrote no parseable artifact to $ev"
fi

run "$FIX" status --set nosuchset --format json
[[ "$RC" -eq 2 ]] || violation "'status --set nosuchset' exited $RC, expected 2"
run "$FIX" verify --set feedback --strict --record --format json
[[ "$RC" -eq 0 ]] || violation "recording a green fixture set exited $RC"
run "$FIX" status --set feedback --format json
if jq -e '.state=="green" and .set=="feedback"' <<<"$OUT" >/dev/null 2>&1; then
    ok "record then status --set feedback reports state green"
else
    violation "after a recorded green run, status reported: $(head -c 160 <<<"$OUT")"
fi

rm -rf "$FIX"
echo
if [[ "$VIOLATIONS" -gt 0 ]]; then
    echo "HARNESS VIOLATION: harness-doctrine - $VIOLATIONS status-contract check(s) failed for ${VERSION:-unknown} at $RESOLVED" >&2
    exit 1
fi
echo "=== do-harness Status Contract Passed (${VERSION:-?} at $RESOLVED) ==="
