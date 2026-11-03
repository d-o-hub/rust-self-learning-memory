#!/usr/bin/env bash
# check-lesson-distillation.sh — every logged lesson must be reachable
#
# agent_docs/LESSONS.md opens with the contract "Pair each entry here with a
# short distilled note in the nearest AGENTS.md", but nothing enforced it: 41 of
# 77 entries were cited nowhere outside the log, and agents only read the log
# when they already know to look. A lesson nobody routes is a lesson that will
# be re-learned as an outage.
#
# Two deterministic checks:
#   1. duplicate lesson ids — LESSON-011 was used twice, which silently breaks
#      number-based citation for the whole log.
#   2. unrouted entries — a ratchet. The baseline is existing debt; a NEW
#      lesson with no citation outside LESSONS.md fails the gate.
#
# Usage:
#   ./scripts/check-lesson-distillation.sh            # verify against baseline
#   ./scripts/check-lesson-distillation.sh --update   # rewrite baseline
#   ./scripts/check-lesson-distillation.sh --list     # show unrouted ids

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

readonly LOG="agent_docs/LESSONS.md"
readonly DEFAULT_BASELINE="scripts/lesson-distillation-baseline.txt"
BASELINE_FILE="$DEFAULT_BASELINE"
UPDATE=0
LIST=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --update) UPDATE=1; shift ;;
    --list)   LIST=1; shift ;;
    --baseline) BASELINE_FILE="${2:?baseline file required}"; shift 2 ;;
    -h|--help) sed -n '2,19p' "$0"; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ ! -r "$LOG" ]]; then
  echo "ERROR: cannot read '$LOG'" >&2
  exit 2
fi

ids="$(grep -oE '^## LESSON-[0-9]{3,4}' "$LOG" | sed 's/^## //' | sort)"
dupes="$(printf '%s\n' "$ids" | uniq -d || true)"
if [[ -n "$dupes" ]]; then
  echo "HARNESS VIOLATION: lesson-distillation — duplicate lesson ids in $LOG:" >&2
  printf '  %s\n' $dupes >&2
  echo "Renumber the later entry to the next free id; a shared id makes every" >&2
  echo "citation of it ambiguous." >&2
  exit 1
fi

# Tracked files other than the log itself that mention a lesson by number.
routed() {
  local id
  while read -r id; do
    [[ -z "$id" ]] && continue
    # A mention in CHANGELOG.md is a release record, not a routing target, so it
    # must not count — otherwise writing about a lesson looks like wiring it in.
    if git grep -l -e "$id" -- ':!agent_docs/LESSONS.md' ':!CHANGELOG.md' >/dev/null 2>&1; then
      printf '%s\n' "$id"
    fi
  done <<< "$(printf '%s\n' "$ids")"
}

unrouted="$(LC_ALL=C comm -23 <(printf '%s\n' "$ids") <(routed | sort) || true)"
COUNT=0
if [[ -n "$unrouted" ]]; then
  COUNT=$(printf '%s\n' "$unrouted" | wc -l)
fi

if (( LIST )); then
  if [[ -n "$unrouted" ]]; then
    printf '%s\n' "$unrouted"
  else
    echo "(all lessons routed)"
  fi
  echo "unrouted=$COUNT"
  exit 0
fi

if (( UPDATE )); then
  {
    echo "# Lessons in agent_docs/LESSONS.md with no citation outside that file"
    echo "#   for scripts/check-lesson-distillation.sh — ratchet ceiling, never raise it"
    echo "# Regenerate with: ./scripts/check-lesson-distillation.sh --update"
    echo "$COUNT"
  } > "$BASELINE_FILE"
  echo "Baseline updated: $BASELINE_FILE = $COUNT"
  exit 0
fi

if [[ ! -f "$BASELINE_FILE" ]]; then
  echo "ERROR: baseline file '$BASELINE_FILE' not found; run with --update" >&2
  exit 1
fi

BASELINE=$(grep -vE '^[[:space:]]*#' "$BASELINE_FILE" | tr -d '[:space:]')
if [[ ! "$BASELINE" =~ ^[0-9]+$ ]]; then
  echo "ERROR: invalid baseline in '$BASELINE_FILE' (expected an integer)" >&2
  exit 1
fi

echo "lessons=$(printf '%s\n' "$ids" | wc -l) unrouted=$COUNT baseline=$BASELINE"

if (( COUNT > BASELINE )); then
  echo "HARNESS VIOLATION: lesson-distillation — $COUNT unrouted lessons exceed baseline $BASELINE" >&2
  echo "Unrouted ids:" >&2
  printf '  %s\n' $unrouted >&2
  echo "Cite the lesson from the file that carries the rule (AGENTS.md, a skill," >&2
  echo "HARNESS.md, or the script that enforces it)." >&2
  exit 1
fi

if (( COUNT < BASELINE )); then
  echo "NOTE: $((BASELINE - COUNT)) fewer unrouted lessons than baseline; run --update to ratchet down." >&2
fi

echo "OK: no lesson ids duplicated, unrouted count within baseline"
