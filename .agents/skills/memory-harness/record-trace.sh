#!/usr/bin/env bash
# record-trace.sh — record a real coding session as a memory-harness trace.
#
# The trace schema is the one replay-trace.sh and export-fixtures.sh consume:
#   {version, episode_id, task, domain, project, outcome, steps[]}
#
# Usage:
#   record-trace.sh start --task "..." [--domain DOMAIN] [--project PROJECT] [--dir DIR]
#   record-trace.sh step --tool TOOL --action ACTION [--latency-ms N] [--success] [--observation TEXT]
#   record-trace.sh complete [success|partial|failed]
#   record-trace.sh status
#
# Environment:
#   DO_MEMORY_CLI    CLI binary/command (default: do-memory-cli)
#   MEMORY_DB_PATH   Forwarded to every CLI call so start/step/complete share one DB.
#   MEMORY_TRACE_DIR Trace directory (default: .memory-traces)
#
# The active session is tracked in "$MEMORY_TRACE_DIR/.active". One session at a
# time; use a distinct --dir for parallel sessions.
set -euo pipefail

TRACE_DIR="${MEMORY_TRACE_DIR:-.memory-traces}"
ACTIVE_FILE="$TRACE_DIR/.active"
IFS=' ' read -r -a CLI_CMD <<< "${DO_MEMORY_CLI:-do-memory-cli}"

usage() {
  sed -n '2,24p' "$0"
}

die() {
  echo "record-trace: $*" >&2
  exit 1
}

require_cli() {
  command -v "${CLI_CMD[0]}" >/dev/null 2>&1 ||
    die "CLI not found: ${CLI_CMD[0]} (set DO_MEMORY_CLI or build do-memory-cli)"
}

require_active() {
  [[ -f "$ACTIVE_FILE" ]] || die "no active session; run 'record-trace.sh start' first"
}

run_cli() {
  "${CLI_CMD[@]}" "$@"
}

slugify() {
  local value="$1"
  printf '%s' "$value" | tr -cs '[:alnum:]' '-' | sed 's/^-//; s/-$//' | cut -c1-40
}

json_value() {
  local key="$1"
  python3 -c 'import json,sys; print(json.load(sys.stdin).get(sys.argv[1], ""))' "$key"
}

cmd_start() {
  local task="" domain="general" project=""
  while [[ $# -gt 0 ]]; do
    local option="$1"
    case "$option" in
      --task) task="${2:?--task requires a value}"; shift 2 ;;
      --domain) domain="${2:?--domain requires a value}"; shift 2 ;;
      --project) project="${2:?--project requires a value}"; shift 2 ;;
      --dir) TRACE_DIR="${2:?--dir requires a value}"; shift 2; ACTIVE_FILE="$TRACE_DIR/.active" ;;
      *) die "unknown start option: $option" ;;
    esac
  done
  [[ -n "$task" ]] || die "start requires --task"
  [[ -n "$project" ]] || project="$(basename "$(git rev-parse --show-toplevel 2>/dev/null || pwd)")"

  require_cli
  mkdir -p "$TRACE_DIR"

  local create_json episode_id trace
  create_json="$(run_cli --format json episode create --task "$task" --domain "$domain")"
  episode_id="$(printf '%s' "$create_json" | json_value episode_id)"
  [[ -n "$episode_id" ]] || die "could not parse episode_id from: $create_json"

  trace="$TRACE_DIR/$(date -u +%Y%m%dT%H%M%SZ)-$(slugify "$task").json"
  TRACE_OUT="$trace" EPISODE_ID="$episode_id" TASK="$task" DOMAIN="$domain" PROJECT="$project" \
    python3 - <<'PY'
import json, os
trace = os.environ["TRACE_OUT"]
data = {
    "version": "1.0",
    "episode_id": os.environ["EPISODE_ID"],
    "task": os.environ["TASK"],
    "domain": os.environ["DOMAIN"],
    "project": os.environ["PROJECT"],
    "outcome": "in_progress",
    "steps": [],
}
with open(trace, "w", encoding="utf-8") as handle:
    json.dump(data, handle, indent=2)
PY

  ACTIVE_OUT="$ACTIVE_FILE" EPISODE_ID="$episode_id" TRACE_OUT="$trace" \
    python3 - <<'PY'
import json, os
with open(os.environ["ACTIVE_OUT"], "w", encoding="utf-8") as handle:
    json.dump(
        {"episode_id": os.environ["EPISODE_ID"], "trace": os.environ["TRACE_OUT"]},
        handle,
        indent=2,
    )
PY

  echo "episode: $episode_id"
  echo "trace:   $trace"
}

cmd_step() {
  local tool="" action="" latency=0 success=false observation=""
  while [[ $# -gt 0 ]]; do
    local option="$1"
    case "$option" in
      --tool) tool="${2:?--tool requires a value}"; shift 2 ;;
      --action) action="${2:?--action requires a value}"; shift 2 ;;
      --latency-ms) latency="${2:?--latency-ms requires a value}"; shift 2 ;;
      --observation) observation="${2:?--observation requires a value}"; shift 2 ;;
      --success) success=true; shift ;;
      *) die "unknown step option: $option" ;;
    esac
  done
  [[ -n "$tool" && -n "$action" ]] || die "step requires --tool and --action"
  require_cli
  require_active

  local episode_id trace
  read -r episode_id trace < <(
    ACTIVE_IN="$ACTIVE_FILE" python3 - <<'PY'
import json, os
with open(os.environ["ACTIVE_IN"], encoding="utf-8") as handle:
    data = json.load(handle)
print(data["episode_id"], data["trace"])
PY
  )

  local -a cli_args=(episode log-step "$episode_id" --tool "$tool" --action "$action" --latency-ms "$latency")
  if [[ "$success" == true ]]; then
    cli_args+=(--success)
  fi
  if [[ -n "$observation" ]]; then
    cli_args+=(--observation "$observation")
  fi
  if ! run_cli "${cli_args[@]}" >/dev/null; then
    die "episode log-step failed for episode $episode_id"
  fi

  TRACE_OUT="$trace" TOOL="$tool" ACTION="$action" LATENCY="$latency" \
    SUCCESS="$success" OBSERVATION="$observation" python3 - <<'PY'
import json, os
trace = os.environ["TRACE_OUT"]
with open(trace, encoding="utf-8") as handle:
    data = json.load(handle)
step = {
    "tool": os.environ["TOOL"],
    "action": os.environ["ACTION"],
    "latency_ms": int(os.environ["LATENCY"]),
    "success": os.environ["SUCCESS"] == "true",
}
if os.environ["OBSERVATION"]:
    step["observation"] = os.environ["OBSERVATION"]
data.setdefault("steps", []).append(step)
with open(trace, "w", encoding="utf-8") as handle:
    json.dump(data, handle, indent=2)
PY

  echo "step: $tool / $action (recorded)"
}

cmd_complete() {
  local outcome="${1:-success}"
  case "$outcome" in
    success | partial | failed) ;;
    *) die "outcome must be success, partial, or failed" ;;
  esac
  require_cli
  require_active

  local episode_id trace
  read -r episode_id trace < <(
    ACTIVE_IN="$ACTIVE_FILE" python3 - <<'PY'
import json, os
with open(os.environ["ACTIVE_IN"], encoding="utf-8") as handle:
    data = json.load(handle)
print(data["episode_id"], data["trace"])
PY
  )

  if ! run_cli episode complete "$episode_id" "$outcome" >/dev/null; then
    die "episode complete failed for episode $episode_id"
  fi

  TRACE_OUT="$trace" OUTCOME="$outcome" python3 - <<'PY'
import json, os
trace = os.environ["TRACE_OUT"]
with open(trace, encoding="utf-8") as handle:
    data = json.load(handle)
data["outcome"] = os.environ["OUTCOME"]
with open(trace, "w", encoding="utf-8") as handle:
    json.dump(data, handle, indent=2)
PY
  rm -f "$ACTIVE_FILE"
  echo "completed: $outcome"
  echo "trace:     $trace"
}

cmd_status() {
  if [[ ! -f "$ACTIVE_FILE" ]]; then
    echo "no active session"
    return 0
  fi
  cat "$ACTIVE_FILE"
}

main() {
  local cmd="${1:-}"
  [[ -n "$cmd" ]] || { usage; exit 2; }
  shift
  case "$cmd" in
    start) cmd_start "$@" ;;
    step) cmd_step "$@" ;;
    complete) cmd_complete "$@" ;;
    status) cmd_status "$@" ;;
    -h | --help | help) usage ;;
    *) die "unknown command: $cmd" ;;
  esac
}

main "$@"
