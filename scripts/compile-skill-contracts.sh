#!/usr/bin/env bash
# compile-skill-contracts.sh — F4.4 agent skill contract compiler
#
# Generates a versioned skill catalog from frontmatter + validates routes/evals.
#
# Usage:
#   ./scripts/compile-skill-contracts.sh
#   ./scripts/compile-skill-contracts.sh --check   # fail if generated catalog would change
#   ./scripts/compile-skill-contracts.sh --out path.json

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CHECK=0
OUT=".agents/skills/skill-catalog.generated.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check) CHECK=1; shift ;;
    --out) OUT="$2"; shift 2 ;;
    -h|--help) sed -n '2,14p' "$0"; exit 0 ;;
    *) echo "Unknown: $1" >&2; exit 2 ;;
  esac
done

command -v jq >/dev/null || { echo "jq required" >&2; exit 1; }

SKILLS_DIR=".agents/skills"
TMP=$(mktemp)

{
  echo '{'
  echo '  "schema_version": 1,'
  echo "  \"generated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
  echo '  "skills": ['
  first=1
  while IFS= read -r skill; do
    skill_md="$SKILLS_DIR/$skill/SKILL.md"
    [[ -f "$skill_md" ]] || continue
    name=$(sed -n 's/^name:[[:space:]]*//p' "$skill_md" | head -1 | tr -d '"' | tr -d "'")
    [[ -z "$name" ]] && name="$skill"
    desc=$(sed -n 's/^description:[[:space:]]*//p' "$skill_md" | head -1 | sed 's/^"//;s/"$//')
    has_evals=false
    [[ -f "$SKILLS_DIR/$skill/evals/evals.json" ]] && has_evals=true
    if [[ $first -eq 0 ]]; then echo ','; fi
    first=0
    jq -nc \
      --arg id "$skill" \
      --arg name "$name" \
      --arg desc "$desc" \
      --argjson evals "$has_evals" \
      '{id:$id, name:$name, description:$desc, has_evals:$evals}'
  done < <(find "$SKILLS_DIR" -mindepth 2 -maxdepth 2 -name SKILL.md | sed "s|$SKILLS_DIR/||;s|/SKILL.md||" | sort)
  echo
  echo '  ]'
  echo '}'
} > "$TMP"

# Stable pretty JSON
jq -S . "$TMP" > "${TMP}.pretty"
mv "${TMP}.pretty" "$TMP"

if [[ "$CHECK" -eq 1 ]]; then
  if [[ -f "$OUT" ]]; then
    if ! diff -q "$OUT" "$TMP" >/dev/null; then
      echo "HARNESS VIOLATION: skill catalog drift — run ./scripts/compile-skill-contracts.sh" >&2
      diff -u "$OUT" "$TMP" | head -40 >&2 || true
      rm -f "$TMP"
      exit 1
    fi
    echo "OK: skill catalog up to date ($OUT)"
  else
    echo "HARNESS VIOLATION: missing $OUT — run ./scripts/compile-skill-contracts.sh" >&2
    rm -f "$TMP"
    exit 1
  fi
else
  mkdir -p "$(dirname "$OUT")"
  mv "$TMP" "$OUT"
  echo "Wrote $OUT ($(jq '.skills|length' "$OUT") skills)"
fi

# Contract checks (must pass for GO)
./scripts/generate-skill-inventory.sh --check >/dev/null
./scripts/validate-skill-routes.sh
./scripts/run-evals.sh --fixtures >/dev/null

echo "OK: skill contract compile complete"
