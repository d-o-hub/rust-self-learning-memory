#!/usr/bin/env bash
# do-harness CLI Compatibility Probe
#
# WHY (issue #1247): AGENTS.md, HARNESS.md and the harness skill all name a pinned
# compatible harness version, and the completion contract is written as `do-harness
# verify --set verification --strict --record`. Until this probe existed the claim was
# prose: nothing checked that the binary on PATH accepts those flags, returns the exit
# codes the doctrine assigns to them, or emits the JSON fields agent scripts parse. A
# pin nobody measures is a phantom affordance (LESSON-063), and the failure is silent -
# the documented command changes shape, an agent reads the error as a green run, and
# the contract stops being evidence.
#
# WHAT IT ASSERTS, all cheap (the full sensor suite stays CI's job):
#   1. one resolved binary, identified by version and digest, independent of PATH order;
#   2. that version satisfies scripts/do-harness-version.txt, and the prose pin equals
#      the same file;
#   3. every subcommand and flag this repository's contract cites is still offered;
#   4. the real do-harness.toml parses, both signal sets resolve to the declared
#      membership, and every sensor argv path exists on disk;
#   5. the exit-code doctrine (0 passed / 1 ran-and-failed / 2 could not run) and the
#      JSON schema agent scripts consume - asserted by the sibling probe
#      scripts/check-do-harness-doctrine.sh, which runs the same fixture against a
#      green, a deliberately failing and an unspawnable sensor in a throwaway root;
#   6. `doctor` (non-strict, as routed by AGENTS.md) runs against this workspace, and
#      one real cheap sensor runs at the workspace root and is reported faithfully.
#
# Exit codes: 0 compatible, 1 contract violation, 2 nothing verified (no binary, no jq).
# A probe that could not run must never be readable as a pass.
#
# Usage: bash scripts/check-do-harness-compat.sh [--bin <path>] [--root <dir>]
#            [--expect-version <v>] [--expect-sha256 <hex>] [--offline] [-h]
#   --bin <path>          probe this binary (default: $DO_HARNESS_BIN, then PATH)
#   --root <dir>          workspace to validate (default: this repository)
#   --expect-version <v>  require exactly this version (CI pins it)
#   --expect-sha256 <hex> require this digest for the resolved binary (CI verifies it)
#   --offline             skip the probe that executes a real workspace sensor
#                         (the sensor runs `cargo fmt`, so a cargo-free runner —
#                         including this repository's harness-compat CI job — must
#                         pass it; without it the sensor cannot spawn, exit_code is
#                         null, and the probe reports a violation, which is correct)

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$ROOT"
PIN_FILE="scripts/do-harness-version.txt"
CONFIG_FILE="do-harness.toml"
PROSE_FILES=(AGENTS.md HARNESS.md .agents/skills/harness/SKILL.md)

BIN="${DO_HARNESS_BIN:-}"
EXPECT_VERSION=""
EXPECT_SHA=""
OFFLINE=0

VIOLATIONS=0
RC=0
OUT=""
ERR=""
RESOLVED=""
VERSION=""

violation() {
    echo "HARNESS VIOLATION: harness-compat - $1" >&2
    VIOLATIONS=$((VIOLATIONS + 1))
}
ok() { echo "OK: $1"; }
note() { echo "NOTE: $1"; }
blocked() {
    echo "HARNESS BLOCKED: harness-compat - nothing was verified. $1" >&2
    exit 2
}
finish() {
    echo
    if [[ "$VIOLATIONS" -gt 0 ]]; then
        echo "HARNESS VIOLATION: harness-compat - $VIOLATIONS compatibility check(s) failed for ${VERSION:-unknown} at ${RESOLVED:-none}" >&2
        exit 1
    fi
    echo "=== do-harness CLI Compatibility Passed (${VERSION:-?} at $RESOLVED) ==="
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --bin) BIN="${2:?--bin needs a path}"; shift 2 ;;
        --root) REPO_ROOT="${2:?--root needs a directory}"; shift 2 ;;
        --expect-version) EXPECT_VERSION="${2:?--expect-version needs a version}"; shift 2 ;;
        --expect-sha256) EXPECT_SHA="${2:?--expect-sha256 needs a digest}"; shift 2 ;;
        --offline) OFFLINE=1; shift ;;
        -h | --help)
            sed -n '/^# Usage:/,/^#   --offline/p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown arg: $1" >&2
            exit 2
            ;;
    esac
done

command -v jq >/dev/null 2>&1 || blocked "jq is required to parse the CLI's JSON output."
[ -d "$REPO_ROOT" ] || blocked "workspace '$REPO_ROOT' is not a directory."
[ -f "$REPO_ROOT/$PIN_FILE" ] || blocked "$PIN_FILE is missing; there is no tracked version reference to check against."
[ -f "$REPO_ROOT/$CONFIG_FILE" ] || blocked "$REPO_ROOT/$CONFIG_FILE is missing; there is no sensor pack to validate."

# ---------------------------------------------------------------- binary resolution
# PATH shadowing is the specific hazard: an older do-harness earlier on PATH answers
# the documented commands with the older option surface, and every probe below would
# then be measuring the wrong binary.
resolve_binary() {
    local candidate dir uniq
    local -a hits=() path_dirs=()
    candidate="$BIN"
    [[ -n "$candidate" ]] || candidate="$(command -v do-harness 2>/dev/null || true)"
    [[ -n "$candidate" ]] ||
        blocked "no do-harness binary found. Install the version named in $PIN_FILE (.github/workflows/harness-compat.yml shows the checksum-pinned download), or pass --bin <path>."
    [[ -x "$candidate" ]] || blocked "'$candidate' is not an executable file."
    RESOLVED="$(realpath "$candidate" 2>/dev/null || echo "$candidate")"
    BIN="$RESOLVED"

    IFS=: read -ra path_dirs <<<"${PATH:-}"
    for dir in "${path_dirs[@]}"; do
        [[ -d "$dir" && -e "$dir/do-harness" ]] || continue
        hits+=("$(realpath "$dir/do-harness" 2>/dev/null || echo "$dir/do-harness")")
    done
    uniq=$(printf '%s\n' "${hits[@]+"${hits[@]}"}" | sort -u | grep -c . || true)
    if [[ "${uniq:-0}" -gt 1 ]]; then
        violation "$(printf '%s ' "${hits[@]}")- several do-harness binaries on PATH, so which one answers depends on ordering. Set DO_HARNESS_BIN or remove the stale one."
    fi
}

# -------------------------------------------------------------------- CLI invocation
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

help_text() { # $@ = argv; sets OUT to help text
    local f
    f=$(mktemp)
    "$BIN" "$@" >"$f" 2>&1 || true
    OUT=$(cat "$f")
    rm -f "$f"
}

# ---------------------------------------------------------------------- version/pin
semver_ge() { # 0 when $1 >= $2, componentwise
    local -a a b
    local i x y
    IFS=. read -ra a <<<"${1#v}"
    IFS=. read -ra b <<<"${2#v}"
    for i in 0 1 2; do
        x="${a[i]:-0}"
        y="${b[i]:-0}"
        [[ "$x" =~ ^[0-9]+$ ]] || x=0
        [[ "$y" =~ ^[0-9]+$ ]] || y=0
        if ((10#$x > 10#$y)); then return 0; fi
        if ((10#$x < 10#$y)); then return 1; fi
    done
    return 0
}

PIN=""
CEILING=""
read_pin_file() {
    PIN=$(sed -n 's/^pin[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$REPO_ROOT/$PIN_FILE" | head -1)
    CEILING=$(sed -n 's/^validated_through[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$REPO_ROOT/$PIN_FILE" | head -1)
    [[ -n "$PIN" ]] || violation "$PIN_FILE has no parseable pin = \"x.y.z\" line."
    [[ -n "$CEILING" ]] || violation "$PIN_FILE has no parseable validated_through = \"x.y.z\" line."
}

COMMIT=""
DIRTY=""
resolve_version() {
    run "$REPO_ROOT" version --format json
    if [[ "$RC" -ne 0 ]]; then
        violation "'version --format json' exited $RC; the documented version probe is not supported by this binary"
        return 1
    fi
    if ! jq -e '.name=="do-harness"' >/dev/null 2>&1 <<<"$OUT"; then
        violation "'version --format json' did not report .name do-harness: $(head -c 120 <<<"$OUT")"
        return 1
    fi
    VERSION=$(jq -r '.version // empty' <<<"$OUT")
    COMMIT=$(jq -r '.commit // empty' <<<"$OUT")
    DIRTY=$(jq -r '.dirty | tostring' <<<"$OUT")
    [[ -n "$VERSION" ]] || {
        violation "'version --format json' returned no .version field"
        return 1
    }
    return 0
}

check_version_against_pin() {
    [[ -n "$VERSION" ]] || return 0
    echo "BINARY: $RESOLVED"
    echo "VERSION: $VERSION (commit ${COMMIT:-unknown}, dirty ${DIRTY:-unknown})"
    if [[ -n "$EXPECT_VERSION" ]]; then
        if [[ "$VERSION" == "$EXPECT_VERSION" ]]; then
            ok "binary is exactly the requested $EXPECT_VERSION"
        else
            violation "this run was asked to verify $EXPECT_VERSION but resolved $VERSION."
        fi
        # A dirty build is not the published artifact, so its checksum means nothing
        # and the compat evidence cannot be reproduced by anyone else.
        if [[ "$DIRTY" == "true" ]]; then
            violation "the resolved $VERSION binary is a dirty build; compat evidence must come from a released artifact"
        fi
    elif semver_ge "$VERSION" "$PIN"; then
        ok "binary $VERSION satisfies the tracked pin $PIN"
    else
        violation "binary $VERSION is older than the tracked pin $PIN in $PIN_FILE. The docs contract against $PIN; install it rather than downgrading the pin."
    fi
    if [[ -n "$CEILING" ]] && ! semver_ge "$CEILING" "$VERSION"; then
        note "binary $VERSION is newer than validated_through $CEILING; raise the ceiling only on a measured compat run"
    fi
    if [[ -n "$EXPECT_SHA" ]]; then
        local actual
        actual=$(sha256sum "$RESOLVED" | cut -d' ' -f1)
        if [[ "$actual" == "$EXPECT_SHA" ]]; then
            ok "binary digest matches the pinned checksum ($actual)"
        else
            violation "binary digest $actual is not the checksum this run was told to expect ($EXPECT_SHA)"
        fi
    fi
}

# Prose drift is the failure this whole file exists to prevent. Only lines containing
# "pinned" are treated as claims, so release notes may name other versions freely.
check_prose_pin() {
    local file claim claims seen=0 examined=0
    for file in "${PROSE_FILES[@]}"; do
        if [[ ! -f "$REPO_ROOT/$file" ]]; then
            violation "routing doc $file is missing; its harness pin cannot be checked"
            continue
        fi
        examined=$((examined + 1))
        claims=$(grep -i 'pinned' "$REPO_ROOT/$file" | grep -oE 'v?[0-9]+\.[0-9]+\.[0-9]+' | sed 's/^v//')
        [[ -n "$claims" ]] || continue
        seen=1
        while IFS= read -r claim; do
            [[ -n "$claim" ]] || continue
            if [[ "$claim" == "$PIN" ]]; then
                ok "$file pins $claim, equal to $PIN_FILE"
            else
                violation "$file claims a pinned harness version $claim while $PIN_FILE pins $PIN - one of the two is unmeasured"
            fi
        done <<<"$claims"
    done
    if [[ "$examined" -gt 0 && "$seen" -eq 0 ]]; then
        violation "no routing doc names a pinned harness version, so $PIN_FILE is enforcing nothing"
    fi
}

# --------------------------------------------------------------- documented surface
# The subcommands and verify flags this repository's own contract cites. A rename or
# removal breaks every documented invocation at once, so each is a violation.
REQUIRED_SUBCOMMANDS=(verify status explain list doctor version)
REQUIRED_VERIFY_FLAGS=(--set --only --strict --record --format --exclude --config --root)

check_option_surface() {
    local missing="" sc fl vmissing=""
    help_text --help
    for sc in "${REQUIRED_SUBCOMMANDS[@]}"; do
        grep -qE "(^|[[:space:]])${sc}([[:space:]]|$)" <<<"$OUT" || missing="$missing $sc"
    done
    if [[ -z "$missing" ]]; then
        ok "subcommand surface intact: ${REQUIRED_SUBCOMMANDS[*]}"
    else
        violation "subcommand(s) the routing docs cite are not offered by '--help':${missing}"
    fi

    help_text verify --help
    for fl in "${REQUIRED_VERIFY_FLAGS[@]}"; do
        grep -qF -- "$fl" <<<"$OUT" || vmissing="$vmissing $fl"
    done
    if [[ -z "$vmissing" ]]; then
        ok "verify offers every flag the contract cites: ${REQUIRED_VERIFY_FLAGS[*]}"
    else
        violation "verify no longer documents:${vmissing} - 'do-harness verify --set verification --strict --record' cannot work"
    fi

    help_text status --help
    grep -qF -- '--set' <<<"$OUT" ||
        violation "status no longer documents --set, but the completion contract runs 'status --set verification'"
    help_text explain --help
    grep -qF -- '--set' <<<"$OUT" ||
        violation "explain no longer documents --set; this probe cannot resolve signal sets without it"
}

# ------------------------------------------------------------------ real sensor pack
sensor_names() {
    grep -oE '^name = "[a-z0-9_-]+"' "$REPO_ROOT/$CONFIG_FILE" | sed 's/.*"\(.*\)"/\1/' | LC_ALL=C sort
}
set_declared_names() {
    sed -n '/^\[signal-sets\]/,/^\[\[/p' "$REPO_ROOT/$CONFIG_FILE" |
        grep -oE '^[a-z][a-z0-9_-]*[[:space:]]*=' | tr -d ' =' | LC_ALL=C sort
}
set_members() { # $1 = set name -> sorted member names
    # The block is flattened first, so a set name can be preceded by ']' rather than
    # whitespace - anchor on "not an identifier character", not on a space.
    sed -n '/^\[signal-sets\]/,/^\[\[/p' "$REPO_ROOT/$CONFIG_FILE" |
        tr -d '\n' |
        grep -oE "(^|[^a-zA-Z0-9_-])$1[[:space:]]*=[[:space:]]*\[[^]]*\]" |
        grep -oE '"[a-z0-9_-]+"' | tr -d '"' | LC_ALL=C sort
}

check_repo_config() {
    local s expected actual dangling sensor missing="" path
    dangling=$(comm -23 <({ set_declared_names | while IFS= read -r s; do set_members "$s"; done; } | LC_ALL=C sort -u) <(sensor_names))
    if [[ -n "$dangling" ]]; then
        violation "signal sets reference sensors that do not exist: $(tr '\n' ' ' <<<"$dangling")"
    else
        ok "every signal-set member is a defined sensor"
    fi

    while IFS= read -r s; do
        [[ -n "$s" ]] || continue
        expected=$(set_members "$s")
        if [[ -z "$expected" ]]; then
            violation "signal set '$s' is empty; a set that certifies nothing is worse than no gate"
            continue
        fi
        run "$REPO_ROOT" explain --set "$s" --format json
        if [[ "$RC" -ne 0 ]]; then
            violation "the CLI does not resolve signal set '$s' (exit $RC): $(head -c 160 <<<"$ERR")"
            continue
        fi
        actual=$(jq -r '.selected[].name' <<<"$OUT" | LC_ALL=C sort)
        if [[ "$actual" == "$expected" ]]; then
            ok "set '$s' resolves to the $(echo "$expected" | grep -c .) sensors declared in $CONFIG_FILE"
        else
            violation "set '$s' differs from $CONFIG_FILE: cli=[$(tr '\n' ' ' <<<"$actual")] file=[$(tr '\n' ' ' <<<"$expected")]"
        fi
    done < <(set_declared_names)

    # A renamed script turns a sensor into an unspawnable argv, which the CLI reports
    # exactly like a genuine gate red (see the ghost fixture above). Catch it here.
    while IFS=$'\t' read -r sensor path; do
        [[ -n "$path" ]] || continue
        [ -f "$REPO_ROOT/$path" ] || missing="$missing $sensor=$path"
    done < <(awk '
        /^name = "/ { n=$0; gsub(/[^a-zA-Z0-9_-]/,"",n); collecting=0; next }
        /^argv = /  {
            line=$0
            sub(/^argv = /,"",line)
            if (line ~ /\]/) { buf=line; collecting=0 } else { buf=line; collecting=1 }
            next
        }
        collecting && /^\]/  { collecting=0 }
        collecting           { buf=buf" "$0 }
        !collecting && buf!="" {
            out=buf
            gsub(/[\[\]"]/,"",out)
            gsub(/,[[:space:]]*/," ",out)
            split(out,tok," ")
            if ((tok[1]=="bash" || tok[1]=="python3" || tok[1]=="node") && tok[2] ~ /\//) {
                print n"\t"tok[2]
            }
            buf=""
        }
        END {
            gsub(/[\[\]"]/,"",buf)
            gsub(/,[[:space:]]*/," ",buf)
            split(buf,tok," ")
            if ((tok[1]=="bash" || tok[1]=="python3" || tok[1]=="node") && tok[2] ~ /\//) {
                print n"\t"tok[2]
            }
        }' "$REPO_ROOT/$CONFIG_FILE")
    if [[ -z "$missing" ]]; then
        ok "every sensor argv path exists"
    else
        violation "sensor argv references missing script(s):${missing} - those sensors can only report a red they never ran"
    fi
}

check_documented_doctor() {
    # AGENTS.md routes agents to `do-harness doctor`. Non-strict only: delegated-hook
    # --strict behaviour is upstream d-o-hub/do-harness#428, and treating that known
    # limitation as an adopter regression would make this gate unpassable.
    run "$REPO_ROOT" doctor
    if [[ "$RC" -eq 0 ]]; then
        ok "doctor (non-strict) runs against this workspace"
    else
        violation "'do-harness doctor' exited $RC; AGENTS.md routes agents to it: $(head -c 160 <<<"$ERR")"
    fi
}

# One real sensor at the workspace root, so the probe is not only ever testing a
# fixture. Compatibility is judged on self-consistency, not on whether fmt is green:
# an unrelated fmt red must not read as a tooling incompatibility.
check_real_sensor() {
    if [[ "$OFFLINE" -eq 1 ]]; then
        note "--offline: skipped executing a real workspace sensor"
        return
    fi
    run "$REPO_ROOT" verify --only fmt --strict --format json
    if [[ "$RC" -eq 2 ]]; then
        violation "'verify --only fmt' could not run (exit 2): $(head -c 160 <<<"$ERR")"
        return
    fi
    is_json || {
        violation "'verify --only fmt --format json' returned no JSON (exit $RC)"
        return
    }
    local code
    code=$(jq -r '.sensors[0].exit_code // "null"' <<<"$OUT")
    if [[ "$code" == "null" ]]; then
        violation "the fmt sensor did not spawn: $(head -c 160 <<<"$ERR")"
    elif [[ "$RC" -eq 0 && "$code" != "0" ]]; then
        violation "verify exited 0 while fmt exited $code - a red sensor was reported as green"
    elif [[ "$RC" -ne 0 && "$code" == "0" ]]; then
        violation "verify exited $RC while fmt exited 0 - a green sensor was reported as red"
    elif [[ "$RC" -eq 0 ]]; then
        ok "real sensor 'fmt' ran at the workspace root and was reported faithfully"
    else
        note "fmt is red at the workspace root (exit $code); the CLI reported it faithfully, so compatibility itself holds"
    fi
}

main() {
    echo "=== do-harness CLI Compatibility ==="
    resolve_binary
    read_pin_file
    resolve_version
    check_version_against_pin
    check_prose_pin
    check_option_surface
    check_repo_config
    check_documented_doctor
    check_real_sensor
    finish
}

main "$@"
