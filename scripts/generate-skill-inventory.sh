#!/usr/bin/env bash
# generate-skill-inventory.sh — K3.3a: list canonical skills from frontmatter
#
# Usage:
#   ./scripts/generate-skill-inventory.sh
#   ./scripts/generate-skill-inventory.sh --check

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SKILLS_DIR=".agents/skills"
CHECK=0
[[ "${1:-}" == "--check" ]] && CHECK=1

if [[ ! -d "$SKILLS_DIR" ]]; then
  echo "ERROR: missing $SKILLS_DIR" >&2
  exit 1
fi

mapfile -t SKILLS < <(
  find "$SKILLS_DIR" -mindepth 2 -maxdepth 2 -name 'SKILL.md' | sed "s|$SKILLS_DIR/||;s|/SKILL.md||" | sort
)

COUNT=${#SKILLS[@]}
if [[ "$COUNT" -lt 1 ]]; then
  echo "ERROR: no skills found" >&2
  exit 1
fi

echo "skill_count=$COUNT"
printf '%s\n' "${SKILLS[@]}"

# Sanity: required high-risk skills present
REQUIRED=(release-guard pr-readiness commit ci-fix code-quality test-runner goap-agent)
for s in "${REQUIRED[@]}"; do
  printf '%s\n' "${SKILLS[@]}" | rg -qx "$s" || {
    echo "ERROR: required skill missing: $s" >&2
    exit 1
  }
done

if [[ "$CHECK" -eq 1 ]]; then
  echo "OK: inventory check passed ($COUNT skills)"
fi
