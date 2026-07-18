#!/usr/bin/env bash
# validate-skills.sh — K3.3c / D3.1a skill integrity and release-policy checks
#
# Usage:
#   ./scripts/validate-skills.sh              # default scan (links soft, fences hard)
#   ./scripts/validate-skills.sh --fixtures   # structure fixtures
#   ./scripts/validate-skills.sh --release-policy
#   ./scripts/validate-skills.sh --all

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SKILLS_DIR=".agents/skills"
MODE="${1:-}"

fail() {
  echo "HARNESS VIOLATION: skills — $1" >&2
  exit 1
}

warn() {
  echo "WARN: skills — $1" >&2
}

# --- --fixtures: structural integrity ---------------------------------------
check_fixtures() {
  [[ -d "$SKILLS_DIR" ]] || fail "missing $SKILLS_DIR"

  local empty_count=0 missing_skill=0

  # No empty skill directories (ignore non-directory artifacts)
  while IFS= read -r -d '' dir; do
    local name
    name=$(basename "$dir")
    # Skip known non-skill files/dirs at top level
    case "$name" in
      skill-rules.json|skill-catalog.generated.json) continue ;;
    esac
    if [[ -d "$dir" ]] && [[ -z "$(find "$dir" -mindepth 1 -maxdepth 1 2>/dev/null | head -1)" ]]; then
      warn "empty skill directory: $name"
      empty_count=$((empty_count + 1))
    fi
  done < <(find "$SKILLS_DIR" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)

  # Every skill directory with content must have SKILL.md
  while IFS= read -r -d '' dir; do
    local name
    name=$(basename "$dir")
    case "$name" in
      skill-rules.json|skill-catalog.generated.json) continue ;;
    esac
    if [[ ! -f "$dir/SKILL.md" ]]; then
      echo "FAIL: missing SKILL.md in $name" >&2
      missing_skill=$((missing_skill + 1))
    fi
  done < <(find "$SKILLS_DIR" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)

  [[ "$empty_count" -eq 0 ]] || fail "found $empty_count empty skill director(y/ies)"
  [[ "$missing_skill" -eq 0 ]] || fail "found $missing_skill skill director(y/ies) without SKILL.md"

  # At least one skill present
  local count
  count=$(find "$SKILLS_DIR" -mindepth 2 -maxdepth 2 -name 'SKILL.md' | wc -l)
  [[ "$count" -ge 1 ]] || fail "no SKILL.md files found under $SKILLS_DIR"

  echo "OK: skill structure fixtures ($count skills)"
}

# --- --release-policy: release-guard sole authority -------------------------
check_release_policy() {
  local skill="$SKILLS_DIR/release-guard/SKILL.md"
  [[ -f "$skill" ]] || fail "missing $skill"

  rg -q 'NEVER' "$skill" \
    || fail "release-guard SKILL.md missing NEVER (manual release forbid)"
  rg -q 'release-manager\.sh ship --execute' "$skill" \
    || fail "release-guard SKILL.md missing 'release-manager.sh ship --execute'"

  echo "OK: release-policy (NEVER + ship --execute present)"
}

# --- default scan: links (soft), fences (hard), empty skills (hard) ---------
check_default() {
  [[ -d "$SKILLS_DIR" ]] || fail "missing $SKILLS_DIR"

  local hard_fail=0 soft_warn=0
  local skill_count=0

  while IFS= read -r skill_md; do
    skill_count=$((skill_count + 1))
    local skill_dir skill_name
    skill_dir=$(dirname "$skill_md")
    skill_name=$(basename "$skill_dir")

    # Empty SKILL.md is a hard failure
    if [[ ! -s "$skill_md" ]]; then
      echo "FAIL: empty skill file: $skill_name/SKILL.md" >&2
      hard_fail=$((hard_fail + 1))
      continue
    fi

    # Unbalanced ``` fences: odd count is hard fail
    local fence_count
    fence_count=$(grep -c '```' "$skill_md" 2>/dev/null || echo 0)
    # grep -c returns 0 with exit 1 when no match under some greps; normalize
    fence_count=${fence_count//[^0-9]/}
    fence_count=${fence_count:-0}
    if (( fence_count % 2 != 0 )); then
      echo "FAIL: unbalanced \`\`\` fences in $skill_name/SKILL.md (count=$fence_count)" >&2
      hard_fail=$((hard_fail + 1))
    fi

    # Relative markdown links to missing files: soft warn
    # Match [text](path) but skip http(s), mailto, anchors-only, absolute /
    while IFS= read -r target; do
      [[ -z "$target" ]] && continue
      # Strip optional title after space/quote
      target="${target%% *}"
      target="${target%%\"*}"
      target="${target%%\'*}"
      # Skip external / anchors / absolute-from-root style
      case "$target" in
        http://*|https://*|mailto:*|\#*|/*) continue ;;
      esac
      # Drop fragment
      local path_only="${target%%#*}"
      [[ -z "$path_only" ]] && continue
      local resolved="$skill_dir/$path_only"
      if [[ ! -e "$resolved" ]]; then
        warn "broken relative link in $skill_name/SKILL.md -> $path_only"
        soft_warn=$((soft_warn + 1))
      fi
    done < <(grep -oE '\[[^]]+\]\(([^)]+)\)' "$skill_md" 2>/dev/null \
      | sed -E 's/.*\]\(([^)]+)\).*/\1/' || true)

  done < <(find "$SKILLS_DIR" -mindepth 2 -maxdepth 2 -name 'SKILL.md' | sort)

  [[ "$skill_count" -ge 1 ]] || fail "no skills scanned"

  if [[ "$hard_fail" -gt 0 ]]; then
    fail "default scan: $hard_fail hard failure(s), $soft_warn soft warning(s) across $skill_count skills"
  fi

  echo "OK: skill default scan ($skill_count skills, $soft_warn soft link warnings)"
}

usage() {
  sed -n '2,12p' "$0"
}

case "$MODE" in
  --fixtures)
    check_fixtures
    ;;
  --release-policy)
    check_release_policy
    ;;
  --all)
    check_fixtures
    check_release_policy
    check_default
    ;;
  "" )
    check_default
    ;;
  -h|--help)
    usage
    exit 0
    ;;
  *)
    echo "Unknown mode: $MODE" >&2
    usage >&2
    exit 2
    ;;
esac
