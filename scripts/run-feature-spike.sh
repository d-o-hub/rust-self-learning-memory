#!/usr/bin/env bash
# run-feature-spike.sh — Produce a GO/NO-GO decision artifact for a feature spike
#
# Usage:
#   ./scripts/run-feature-spike.sh SPIKE_ID --config PATH --output PATH
#   ./scripts/run-feature-spike.sh --help
#
# Reads a TOML/JSON config if present, or creates a minimal in-memory config.
# Writes JSON decision artifact with schema:
#   {id, commit, timestamp, owner, commands, metrics, preapproved_thresholds,
#    result, decision, reviewers}
#
# decision is GO or NO-GO based on simple checks (required files exist,
# optional commands, force_decision override). Exit 0 always when the
# artifact is written unless fatal usage error (exit 2).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/run-feature-spike.sh SPIKE_ID --config PATH --output PATH
  ./scripts/run-feature-spike.sh --help

Arguments:
  SPIKE_ID           Spike identifier (e.g. F4.1, S1.1c)
  --config PATH      Spike config (TOML or JSON). Created minimally if missing.
  --output PATH      JSON decision artifact path (parent dirs created)

Config keys (TOML or JSON):
  id, owner, reviewers, commands, required_files, metrics,
  preapproved_thresholds, force_decision (GO|NO-GO optional)

Exit codes:
  0  Artifact written (decision may be GO or NO-GO)
  2  Usage / fatal argument error
EOF
}

SPIKE_ID=""
CONFIG_PATH=""
OUTPUT_PATH=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    --config)
      CONFIG_PATH="${2:-}"
      shift 2
      ;;
    --output)
      OUTPUT_PATH="${2:-}"
      shift 2
      ;;
    --*)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
    *)
      if [[ -z "$SPIKE_ID" ]]; then
        SPIKE_ID="$1"
        shift
      else
        echo "Unexpected argument: $1" >&2
        usage >&2
        exit 2
      fi
      ;;
  esac
done

if [[ -z "$SPIKE_ID" || -z "$CONFIG_PATH" || -z "$OUTPUT_PATH" ]]; then
  echo "FATAL: SPIKE_ID, --config, and --output are required" >&2
  usage >&2
  exit 2
fi

# ── Ensure config exists (minimal if absent) ──────────────────────────────────
if [[ ! -f "$CONFIG_PATH" ]]; then
  mkdir -p "$(dirname "$CONFIG_PATH")"
  cat >"$CONFIG_PATH" <<EOF
# Auto-generated minimal spike config for ${SPIKE_ID}
id = "${SPIKE_ID}"
owner = "goap-agent"
reviewers = ["maintainer"]
commands = []
required_files = []
force_decision = ""

[metrics]
notes = "minimal auto-config"

[preapproved_thresholds]
placeholder = true
EOF
  echo "NOTE: created minimal config at $CONFIG_PATH"
fi

# ── Parse config via Python (TOML via tomllib, or JSON) ───────────────────────
# shellcheck disable=SC2016
parse_out=$(python3 - "$CONFIG_PATH" "$SPIKE_ID" <<'PY'
import json, sys
from pathlib import Path

path = Path(sys.argv[1])
spike_id = sys.argv[2]
raw_bytes = path.read_bytes()

if path.suffix.lower() == ".json":
    data = json.loads(raw_bytes.decode("utf-8"))
else:
    try:
        import tomllib
    except ImportError:  # pragma: no cover
        import tomli as tomllib  # type: ignore
    data = tomllib.loads(raw_bytes.decode("utf-8"))

# Normalize list/dict fields
out = {
    "id": data.get("id", spike_id),
    "owner": data.get("owner", "goap-agent"),
    "reviewers": data.get("reviewers", ["maintainer"]),
    "commands": data.get("commands", []),
    "required_files": data.get("required_files", []),
    "metrics": data.get("metrics", {}),
    "preapproved_thresholds": data.get("preapproved_thresholds", {}),
    "force_decision": data.get("force_decision", ""),
    "result_notes": data.get("result_notes", data.get("notes", "")),
}
if not isinstance(out["reviewers"], list):
    out["reviewers"] = [str(out["reviewers"])]
if not isinstance(out["commands"], list):
    out["commands"] = [str(out["commands"])] if out["commands"] else []
if not isinstance(out["required_files"], list):
    out["required_files"] = [str(out["required_files"])] if out["required_files"] else []
if not isinstance(out["metrics"], dict):
    out["metrics"] = {"value": out["metrics"]}
if not isinstance(out["preapproved_thresholds"], dict):
    out["preapproved_thresholds"] = {"value": out["preapproved_thresholds"]}
print(json.dumps(out))
PY
)

# ── Evaluate GO / NO-GO ───────────────────────────────────────────────────────
COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

eval_result=$(python3 - "$parse_out" "$ROOT" <<'PY'
import json, os, subprocess, sys
from pathlib import Path

cfg = json.loads(sys.argv[1])
root = Path(sys.argv[2])
checks = []
ok = True

force = str(cfg.get("force_decision") or "").strip().upper()
if force in ("GO", "NO-GO"):
    decision = force
    checks.append({"type": "force_decision", "value": force, "pass": True})
else:
    for rel in cfg.get("required_files") or []:
        p = root / rel if not os.path.isabs(rel) else Path(rel)
        exists = p.is_file()
        checks.append({"type": "required_file", "path": str(rel), "pass": exists})
        if not exists:
            ok = False

    for cmd in cfg.get("commands") or []:
        if not cmd:
            continue
        try:
            r = subprocess.run(
                cmd, shell=True, cwd=str(root),
                capture_output=True, text=True, timeout=120,
            )
            passed = r.returncode == 0
            checks.append({
                "type": "command",
                "command": cmd,
                "exit_code": r.returncode,
                "pass": passed,
            })
            if not passed:
                ok = False
        except Exception as e:
            checks.append({
                "type": "command",
                "command": cmd,
                "error": str(e),
                "pass": False,
            })
            ok = False

    # If no checks at all and no force, default NO-GO (incomplete spike)
    if not checks:
        ok = False
        checks.append({
            "type": "default",
            "note": "no required_files, commands, or force_decision — default NO-GO",
            "pass": False,
        })
    decision = "GO" if ok else "NO-GO"

result_summary = "pass" if decision == "GO" else "fail"
if force:
    result_summary = f"forced_{decision.lower()}"

print(json.dumps({
    "decision": decision,
    "result": result_summary,
    "checks": checks,
    "cfg": cfg,
}))
PY
)

# ── Write artifact ────────────────────────────────────────────────────────────
mkdir -p "$(dirname "$OUTPUT_PATH")"

python3 - "$eval_result" "$COMMIT" "$TIMESTAMP" "$OUTPUT_PATH" <<'PY'
import json, sys
from pathlib import Path

ev = json.loads(sys.argv[1])
commit = sys.argv[2]
timestamp = sys.argv[3]
out_path = Path(sys.argv[4])
cfg = ev["cfg"]

artifact = {
    "id": cfg.get("id"),
    "commit": commit,
    "timestamp": timestamp,
    "owner": cfg.get("owner", "goap-agent"),
    "commands": cfg.get("commands") or [],
    "metrics": {
        **(cfg.get("metrics") or {}),
        "checks": ev.get("checks") or [],
    },
    "preapproved_thresholds": cfg.get("preapproved_thresholds") or {},
    "result": ev.get("result"),
    "decision": ev.get("decision"),
    "reviewers": cfg.get("reviewers") or ["maintainer"],
}
if cfg.get("result_notes"):
    artifact["metrics"]["notes"] = cfg["result_notes"]

out_path.write_text(json.dumps(artifact, indent=2) + "\n", encoding="utf-8")
print(f"Wrote spike artifact: {out_path}")
print(f"decision={artifact['decision']} result={artifact['result']} id={artifact['id']}")
PY

exit 0
