#!/usr/bin/env bash
# dogfood-coding-workflow.sh — exercise the product as an end-to-end coding workflow.
#
# Verifies the pieces an agent relies on:
#   1. CLI health + episode lifecycle (create → steps → complete → view/list/search)
#   2. Cross-process pattern extraction and storage stats parity
#   3. Local Turso/SQLite durability (sibling .db) and redb cache
#   4. Tags and storage journal commands
#   5. MCP stdio handshake, tools/list, health_check, query_memory
#   6. Skill route/eval validation and the harness feedback set
#
# Usage:
#   ./scripts/dogfood-coding-workflow.sh [--quick] [--release] [--skip-mcp]
#                                        [--skip-harness] [--keep] [--db PATH]
#
# --quick    build the debug CLI/MCP (default) instead of release
# --release  build release binaries (slower, matches the skill docs)
# --keep     keep the temporary database/trace directory
#
# Environment:
#   DO_MEMORY_CLI        Use an existing CLI binary instead of building
#   DO_MEMORY_MCP        Use an existing MCP server binary instead of building
#   CARGO_TARGET_DIR     Forwarded to cargo when building
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="quick"
SKIP_MCP=0
SKIP_HARNESS=0
KEEP=0
DB_PATH=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --quick) MODE="quick"; shift ;;
    --release) MODE="release"; shift ;;
    --skip-mcp) SKIP_MCP=1; shift ;;
    --skip-harness) SKIP_HARNESS=1; shift ;;
    --keep) KEEP=1; shift ;;
    --db) DB_PATH="${2:?--db requires a path}"; shift 2 ;;
    -h | --help)
      sed -n '2,24p' "$0"
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 2
      ;;
  esac
done

PASS=0
FAIL=0
FAILURES=()

pass() {
  PASS=$((PASS + 1))
  printf '  \033[0;32mPASS\033[0m %s\n' "$1"
}

fail() {
  FAIL=$((FAIL + 1))
  FAILURES+=("$1")
  printf '  \033[0;31mFAIL\033[0m %s\n' "$1"
}

section() {
  printf '\n\033[1m== %s ==\033[0m\n' "$1"
}

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/do-memory-dogfood.XXXXXX")"
TRACE_DIR="$TMP_DIR/traces"
mkdir -p "$TRACE_DIR"

cleanup() {
  if [[ "$KEEP" -eq 0 ]]; then
    rm -rf "$TMP_DIR"
  else
    echo "kept: $TMP_DIR"
  fi
}
trap cleanup EXIT

if [[ -z "$DB_PATH" ]]; then
  DB_PATH="$TMP_DIR/memory.redb"
fi
DB_DIR="$(dirname "$DB_PATH")"
mkdir -p "$DB_DIR"

TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
[[ "$TARGET_DIR" = /* ]] || TARGET_DIR="$ROOT/$TARGET_DIR"

if [[ "$MODE" == "release" ]]; then
  CLI_DEFAULT="$TARGET_DIR/release/do-memory-cli"
  MCP_DEFAULT="$TARGET_DIR/release/do-memory-mcp-server"
  BUILD_FLAGS=(--release)
else
  CLI_DEFAULT="$TARGET_DIR/debug/do-memory-cli"
  MCP_DEFAULT="$TARGET_DIR/debug/do-memory-mcp-server"
  BUILD_FLAGS=()
fi

CLI_BIN="${DO_MEMORY_CLI:-$CLI_DEFAULT}"
MCP_BIN="${DO_MEMORY_MCP:-$MCP_DEFAULT}"

section "Build"
if [[ -x "$CLI_BIN" ]]; then
  pass "CLI present: $CLI_BIN"
else
  echo "  building do-memory-cli --features turso ..."
  if cargo build "${BUILD_FLAGS[@]}" -p do-memory-cli --features turso; then
    pass "built CLI: $CLI_BIN"
  else
    fail "CLI build failed"
    echo "Cannot continue without the CLI." >&2
    exit 1
  fi
fi

if [[ "$SKIP_MCP" -eq 0 && ! -x "$MCP_BIN" ]]; then
  echo "  building do-memory-mcp-server ..."
  if cargo build "${BUILD_FLAGS[@]}" -p do-memory-mcp; then
    pass "built MCP server: $MCP_BIN"
  else
    fail "MCP build failed"
    SKIP_MCP=1
  fi
fi

run_cli() {
  RUST_LOG=off "$CLI_BIN" --storage-mode local --db-path "$DB_PATH" "$@"
}

json_get() {
  local key_path="$1"
  # key path is dot separated; stdin is JSON
  python3 -c '
import json, sys
value = json.load(sys.stdin)
for key in sys.argv[1].split("."):
    if key:
        value = value[key] if isinstance(value, dict) else value[int(key)]
print(value)
' "$key_path"
}

section "CLI health"
if HEALTH_JSON="$(run_cli --format json health check 2>"$TMP_DIR/health.err")"; then
  if printf '%s' "$HEALTH_JSON" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert "overall_status" in d and "components" in d' 2>/dev/null; then
    pass "health check returned a component report"
  else
    fail "health check JSON missing overall_status/components"
  fi
else
  fail "health check exited non-zero: $(tr '\n' ' ' <"$TMP_DIR/health.err" | cut -c1-200)"
fi

section "Episode lifecycle"
TASK="Dogfood the coding workflow end to end"
DOMAIN="coding-workflow"
if CREATE_JSON="$(run_cli --format json episode create --task "$TASK" --domain "$DOMAIN")"; then
  EPISODE_ID="$(printf '%s' "$CREATE_JSON" | json_get episode_id)"
  if [[ -n "$EPISODE_ID" ]]; then
    pass "episode created: $EPISODE_ID"
  else
    fail "episode create JSON has no episode_id"
    EPISODE_ID=""
  fi
else
  fail "episode create failed"
  EPISODE_ID=""
fi

if [[ -n "$EPISODE_ID" ]]; then
  STEPS=(
    "read|inspect the coding workflow docs|12"
    "grep|locate skill command references|8"
    "edit|fix the memory-harness trace schema|21"
    "test|run the dogfood script|840"
    "harness|run the feedback sensor set|1200"
  )
  STEP_OK=0
  for entry in "${STEPS[@]}"; do
    IFS='|' read -r TOOL ACTION LATENCY <<<"$entry"
    if run_cli episode log-step "$EPISODE_ID" --tool "$TOOL" --action "$ACTION" \
      --latency-ms "$LATENCY" --success >/dev/null 2>&1; then
      STEP_OK=$((STEP_OK + 1))
    fi
  done
  if [[ "$STEP_OK" -eq "${#STEPS[@]}" ]]; then
    pass "logged $STEP_OK/${#STEPS[@]} execution steps"
  else
    fail "logged $STEP_OK/${#STEPS[@]} execution steps"
  fi

  if run_cli episode complete "$EPISODE_ID" success >/dev/null 2>&1; then
    pass "episode completed"
  else
    fail "episode complete failed"
  fi

  if VIEW_JSON="$(run_cli --format json episode view "$EPISODE_ID" 2>/dev/null)"; then
    if printf '%s' "$VIEW_JSON" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d.get("completed_at") is not None; assert d.get("reward") is not None; assert d.get("reflection") is not None' 2>/dev/null; then
      pass "episode view shows completed + reward + reflection"
    else
      fail "episode view missing completed_at/reward/reflection"
    fi
  else
    fail "episode view failed"
  fi

  if run_cli --format json episode list --limit 10 2>/dev/null | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d.get("total_count",0) >= 1' 2>/dev/null; then
    pass "episode list sees the new episode"
  else
    fail "episode list does not see the new episode"
  fi

  if run_cli --format json episode search "Dogfood" --limit 10 2>/dev/null | python3 -c 'import json,sys; d=json.load(sys.stdin); assert len(d.get("episodes",[])) >= 1' 2>/dev/null; then
    pass "episode search returns the episode"
  else
    fail "episode search returned no match"
  fi
fi

section "Patterns and storage parity"
PATTERN_TOTAL=""
if PATTERN_JSON="$(run_cli --format json pattern list --limit 1000 2>/dev/null)"; then
  PATTERN_TOTAL="$(printf '%s' "$PATTERN_JSON" | json_get total_count 2>/dev/null || echo "")"
  if [[ -n "$PATTERN_TOTAL" && "$PATTERN_TOTAL" -gt 0 ]]; then
    pass "pattern list extracted $PATTERN_TOTAL pattern(s)"
  else
    fail "pattern list extracted no patterns"
  fi
else
  fail "pattern list failed"
fi

STATS_JSON="$(run_cli --format json storage stats 2>/dev/null || true)"
if [[ -n "$STATS_JSON" ]]; then
  STATS_PATTERNS="$(printf '%s' "$STATS_JSON" | json_get patterns.total_count 2>/dev/null || echo "")"
  STATS_EPISODES="$(printf '%s' "$STATS_JSON" | json_get episodes.total_count 2>/dev/null || echo "")"
  if [[ -n "$PATTERN_TOTAL" && "$STATS_PATTERNS" == "$PATTERN_TOTAL" ]]; then
    pass "storage stats pattern count matches pattern list ($STATS_PATTERNS)"
  else
    fail "pattern count mismatch: list=$PATTERN_TOTAL stats=$STATS_PATTERNS"
  fi
  if [[ -n "$STATS_EPISODES" && "$STATS_EPISODES" -ge 1 ]]; then
    pass "storage stats reports at least one episode"
  else
    fail "storage stats episode count is zero/unavailable"
  fi
else
  fail "storage stats failed"
fi

section "Local database durability"
DB_FILE="$DB_PATH"
if [[ "$DB_FILE" == *.redb ]]; then
  SQLITE_FILE="${DB_FILE%.redb}.db"
else
  SQLITE_FILE="$DB_FILE"
fi
if [[ -f "$SQLITE_FILE" ]]; then
  if python3 - "$SQLITE_FILE" "$EPISODE_ID" <<'PY'
import sqlite3, sys
path, episode_id = sys.argv[1], sys.argv[2]
con = sqlite3.connect(path)
cur = con.cursor()
tables = {row[0] for row in cur.execute("SELECT name FROM sqlite_master WHERE type='table'")}
assert "episodes" in tables, tables
assert "patterns" in tables, tables
episodes = cur.execute("SELECT COUNT(*) FROM episodes").fetchone()[0]
assert episodes >= 1, episodes
if episode_id:
    found = cur.execute("SELECT 1 FROM episodes WHERE episode_id = ?", (episode_id,)).fetchone()
    assert found, episode_id
print(f"sqlite tables={sorted(tables)} episodes={episodes}")
PY
  then
    pass "durable SQLite sibling contains the episode and patterns tables"
  else
    fail "durable SQLite sibling is missing the episode/tables"
  fi
else
  fail "durable SQLite sibling not found: $SQLITE_FILE"
fi

section "Tags and journal"
if [[ -n "$EPISODE_ID" ]]; then
  if run_cli tag add "$EPISODE_ID" dogfood >/dev/null 2>&1; then
    pass "tag add"
  else
    fail "tag add failed"
  fi
  if run_cli --format json tag list --episode "$EPISODE_ID" 2>/dev/null | python3 -c 'import json,sys; d=json.load(sys.stdin); assert "dogfood" in d.get("tags",[])' 2>/dev/null; then
    pass "tag list returns the tag"
  else
    fail "tag list did not return the tag"
  fi
fi

if run_cli storage journal >/dev/null 2>&1; then
  pass "storage journal"
else
  fail "storage journal failed"
fi

section "MCP stdio"
if [[ "$SKIP_MCP" -eq 0 && -x "$MCP_BIN" ]]; then
  if python3 - "$MCP_BIN" "$DB_PATH" <<'PY'
import json, os, select, subprocess, sys, time

binary, db_path = sys.argv[1], sys.argv[2]
redb_path = db_path
db_file = db_path[:-5] + ".db" if db_path.endswith(".redb") else db_path
env = os.environ.copy()
env.update(
    MEMORY_STORAGE_MODE="local",
    MEMORY_DB_PATH=db_file,
    REDB_CACHE_PATH=redb_path,
    RUST_LOG="off",
)
proc = subprocess.Popen(
    [binary], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env
)

def send(message):
    body = json.dumps(message).encode()
    proc.stdin.write(b"Content-Length: " + str(len(body)).encode() + b"\r\n\r\n" + body)
    proc.stdin.flush()

def read_message(timeout=20):
    deadline = time.time() + timeout
    data = b""
    while time.time() < deadline:
        ready, _, _ = select.select([proc.stdout.fileno()], [], [], 0.5)
        if not ready:
            continue
        chunk = os.read(proc.stdout.fileno(), 65536)
        if not chunk:
            break
        data += chunk
        if b"\r\n\r\n" not in data:
            continue
        header, rest = data.split(b"\r\n\r\n", 1)
        length = None
        for line in header.split(b"\r\n"):
            if line.lower().startswith(b"content-length:"):
                length = int(line.split(b":", 1)[1].strip())
        if length is None:
            if b"\n" in rest:
                line, _ = rest.split(b"\n", 1)
                return json.loads(line)
            continue
        if len(rest) < length:
            continue
        return json.loads(rest[:length])
    raise TimeoutError("no MCP response")

def call(request_id, method, params=None):
    message = {"jsonrpc": "2.0", "id": request_id, "method": method}
    if params is not None:
        message["params"] = params
    send(message)
    response = read_message()
    if response.get("id") != request_id:
        raise RuntimeError(f"unexpected response id: {response}")
    if "error" in response:
        raise RuntimeError(f"{method} error: {response['error']}")
    return response.get("result")

try:
    call(1, "initialize", {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "dogfood", "version": "1"}})
    tools = call(2, "tools/list", {})
    names = {tool["name"] for tool in tools.get("tools", [])}
    assert "health_check" in names, sorted(names)
    assert "query_memory" in names, sorted(names)
    health = call(3, "tools/call", {"name": "health_check", "arguments": {}})
    assert health is not None
    assert not health.get("isError"), health
    query = call(4, "tools/call", {"name": "query_memory", "arguments": {"query": "dogfood"}})
    assert query is not None
    assert not query.get("isError"), query
    print(f"mcp tools={len(names)} health=ok query=ok")
finally:
    try:
        send({"jsonrpc": "2.0", "method": "shutdown"})
    except Exception:
        pass
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
    stderr = proc.stderr.read().decode(errors="replace").strip()
    if stderr:
        print(stderr, file=sys.stderr)
PY
  then
    pass "MCP initialize, tools/list, health_check, query_memory"
  else
    fail "MCP stdio dogfood failed"
  fi
else
  echo "  (skipped: MCP binary not built)"
fi

section "Memory-harness skill"
export DO_MEMORY_CLI="$CLI_BIN --storage-mode local --db-path $DB_PATH"
export MEMORY_TRACE_DIR="$TRACE_DIR"
RECORD="$ROOT/.agents/skills/memory-harness/record-trace.sh"
REPLAY="$ROOT/.agents/skills/memory-harness/replay-trace.sh"

if bash "$RECORD" start --task "Harness trace for the coding workflow" --domain coding-workflow >"$TMP_DIR/record-start.out" 2>&1; then
  if bash "$RECORD" step --tool read --action "inspect the workflow" --latency-ms 9 --success >/dev/null 2>&1 &&
    bash "$RECORD" step --tool test --action "run the dogfood gate" --latency-ms 700 --success >/dev/null 2>&1 &&
    bash "$RECORD" complete success >/dev/null 2>&1; then
    pass "record-trace start/step/complete"
    TRACE_FILE="$(find "$MEMORY_TRACE_DIR" -maxdepth 1 -name '*.json' ! -name '*.results.json' | head -1)"
    if [[ -n "$TRACE_FILE" ]] && python3 - "$TRACE_FILE" <<'PY'
import json, sys
trace = json.load(open(sys.argv[1], encoding="utf-8"))
assert trace.get("task"), trace
assert trace.get("domain"), trace
assert trace.get("outcome") == "success", trace
assert len(trace.get("steps") or []) == 2, trace
PY
    then
      pass "trace schema has task/domain/outcome/steps"
    else
      fail "trace schema is incomplete"
    fi
    if [[ -n "$TRACE_FILE" ]] && bash "$REPLAY" "$TRACE_FILE" >/dev/null 2>&1; then
      pass "replay-trace consumes the recorded trace"
    else
      fail "replay-trace rejected the recorded trace"
    fi
    if bash "$ROOT/.agents/skills/memory-harness/export-fixtures.sh" "$MEMORY_TRACE_DIR" "$TMP_DIR/fixtures" >/dev/null 2>&1 &&
      python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d and d[0].get("task") and d[0].get("steps")' "$TMP_DIR/fixtures/real_traces.json" 2>/dev/null; then
      pass "export-fixtures emits replayable fixtures"
    else
      fail "export-fixtures rejected the trace schema"
    fi
  else
    fail "record-trace start/step/complete"
  fi
else
  fail "record-trace start failed: $(tr '\n' ' ' <"$TMP_DIR/record-start.out" | cut -c1-200)"
fi
unset DO_MEMORY_CLI MEMORY_TRACE_DIR

section "Skills"
if ./scripts/validate-skill-routes.sh >/dev/null 2>&1; then
  pass "skill routes validate"
else
  fail "skill route validation failed"
fi
if ./scripts/run-evals.sh memory-harness >/dev/null 2>&1; then
  pass "memory-harness evals"
else
  fail "memory-harness evals failed"
fi
if ./scripts/run-evals.sh do-memory-cli-ops >/dev/null 2>&1; then
  pass "do-memory-cli-ops evals"
else
  fail "do-memory-cli-ops evals failed"
fi

section "Harness"
if [[ "$SKIP_HARNESS" -eq 0 ]]; then
  if command -v do-harness >/dev/null 2>&1; then
    if do-harness verify --only commit-ci-tokens --strict >/dev/null 2>&1; then
      pass "harness commit-ci-tokens sensor"
    else
      fail "harness commit-ci-tokens sensor"
    fi
    if do-harness verify --only agents-md-ids --strict >/dev/null 2>&1; then
      pass "harness agents-md-ids sensor"
    else
      fail "harness agents-md-ids sensor"
    fi
  else
    echo "  (skipped: do-harness not installed)"
  fi
else
  echo "  (skipped by --skip-harness)"
fi

section "Summary"
printf '  passed: %d\n  failed: %d\n' "$PASS" "$FAIL"
if [[ "$FAIL" -gt 0 ]]; then
  printf '  findings:\n'
  for finding in "${FAILURES[@]}"; do
    printf '   - %s\n' "$finding"
  done
  exit 1
fi
echo "  dogfood: PASS"
