#!/usr/bin/env bash
# validate-skill-routes.sh — K3.3b skill routing fixtures
#
# Usage:
#   ./scripts/validate-skill-routes.sh
#   ./scripts/validate-skill-routes.sh --fixtures

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RULES=".agents/skills/skill-rules.json"
SKILLS_DIR=".agents/skills"

fail() {
  echo "HARNESS VIOLATION: skill-routes — $1" >&2
  exit 1
}

[[ -f "$RULES" ]] || fail "missing $RULES"
command -v jq >/dev/null || fail "jq required"

# Every rule.skill must exist
mapfile -t RULE_SKILLS < <(jq -r '.rules[].skill' "$RULES" | sort -u)
for s in "${RULE_SKILLS[@]}"; do
  [[ -f "$SKILLS_DIR/$s/SKILL.md" ]] || fail "rule references missing skill: $s"
done

# Required high-frequency skills must have at least one route
REQUIRED=(release-guard pr-readiness commit ci-fix code-quality test-runner goap-agent build-rust)
for s in "${REQUIRED[@]}"; do
  printf '%s\n' "${RULE_SKILLS[@]}" | rg -qx "$s" \
    || fail "high-frequency skill '$s' has no skill-rules route"
done

# Negative: no empty keyword lists for high priority
while IFS= read -r line; do
  skill=$(echo "$line" | jq -r '.skill')
  pri=$(echo "$line" | jq -r '.priority')
  kw=$(echo "$line" | jq -r '.triggers.keywords | length')
  if [[ "$pri" == "high" && "$kw" -eq 0 ]]; then
    fail "high-priority rule for $skill has zero keywords"
  fi
done < <(jq -c '.rules[]' "$RULES")

echo "OK: skill routes validated (${#RULE_SKILLS[@]} unique skills routed)"
