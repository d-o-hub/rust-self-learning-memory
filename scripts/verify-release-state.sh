#!/usr/bin/env bash
# verify-release-state.sh - Verify all version references are consistent.
#
# Checks that Cargo.toml workspace version matches:
#   1. All workspace member Cargo.toml versions
#   2. ROADMAP_ACTIVE.md "Released Version" line
#   3. STATUS/CURRENT.md version references
#   4. Git tag (if --check-tag is passed)
#   5. No unreleased feat/fix commits (if --check-unreleased is passed)
#
# Exit 0 = all consistent, Exit 1 = mismatch found.

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CHECK_TAG=false
CHECK_UNRELEASED=false
FIX_MODE=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check-tag) CHECK_TAG=true; shift ;;
    --check-unreleased) CHECK_UNRELEASED=true; shift ;;
    --fix) FIX_MODE=true; shift ;;
    -h|--help)
      echo "Usage: $(basename "$0") [--check-tag] [--check-unreleased] [--fix]"
      echo ""
      echo "Options:"
      echo "  --check-tag         Verify latest git tag matches Cargo.toml version"
      echo "  --check-unreleased  Warn if unreleased feat/fix commits exist on main"
      echo "  --fix               Auto-fix version references in .md files"
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

cd "$PROJECT_ROOT"

failures=0
warnings=0

# ─── 1. Extract workspace version from Cargo.toml ───
WORKSPACE_VERSION=$(cargo metadata --format-version=1 --no-deps 2>/dev/null \
  | jq -r '.packages[0].version' 2>/dev/null || true)

if [[ -z "$WORKSPACE_VERSION" || "$WORKSPACE_VERSION" == "null" ]]; then
  # Fallback to grep
  WORKSPACE_VERSION=$(grep -E '^version\s*=' Cargo.toml | head -1 \
    | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+[^"]*)".*/\1/')
fi

if [[ -z "$WORKSPACE_VERSION" ]]; then
  echo "❌ FATAL: Could not determine workspace version from Cargo.toml"
  exit 1
fi

echo "📦 Workspace version: $WORKSPACE_VERSION"

# ─── 1.5. Enforce 0.1.x patch release line ───
MAJOR_MINOR=$(echo "$WORKSPACE_VERSION" | grep -oP '^\d+\.\d+')
if [[ "$MAJOR_MINOR" != "0.1" ]]; then
  echo ""
  echo "🚫 VERSION POLICY VIOLATION: Workspace version is $WORKSPACE_VERSION"
  echo "   The project MUST stay on the 0.1.x patch release line."
  echo "   Minor/major bumps require EXPLICIT human approval."
  echo "   Fix: Change version in Cargo.toml to 0.1.x (next patch after latest release)"
  echo ""
  failures=$((failures + 1))
fi
echo ""

# ─── 2. Check all workspace member versions match ───
echo "── Checking workspace member versions ──"
MEMBERS=$(cargo metadata --format-version=1 --no-deps 2>/dev/null \
  | jq -r '.packages[] | "\(.name)=\(.version)"' 2>/dev/null || true)

if [[ -n "$MEMBERS" ]]; then
  while IFS='=' read -r name version; do
    if [[ "$version" != "$WORKSPACE_VERSION" ]]; then
      echo "  ❌ $name has version $version (expected $WORKSPACE_VERSION)"
      failures=$((failures + 1))
    else
      echo "  ✅ $name = $version"
    fi
  done <<< "$MEMBERS"
else
  echo "  ⚠️  Could not read workspace members (cargo metadata failed)"
  warnings=$((warnings + 1))
fi

# ─── 3. Check ROADMAP_ACTIVE.md ───
echo ""
echo "── Checking ROADMAP_ACTIVE.md ──"
ROADMAP="plans/ROADMAPS/ROADMAP_ACTIVE.md"
if [[ -f "$ROADMAP" ]]; then
  # Check "Released Version" line
  ROADMAP_VERSION=$(grep -oP 'Released Version.*?v?\K[0-9]+\.[0-9]+\.[0-9]+' "$ROADMAP" | head -1 || true)
  if [[ -z "$ROADMAP_VERSION" ]]; then
    echo "  ⚠️  No 'Released Version' found in $ROADMAP"
    warnings=$((warnings + 1))
  elif [[ "$ROADMAP_VERSION" != "$WORKSPACE_VERSION" ]]; then
    echo "  ❌ $ROADMAP says v$ROADMAP_VERSION (expected v$WORKSPACE_VERSION)"
    failures=$((failures + 1))
  else
    echo "  ✅ Released Version = v$ROADMAP_VERSION"
  fi

  # Check "Unreleased on main" — should not exist after a proper release
  if grep -q "Unreleased on main" "$ROADMAP"; then
    echo "  ⚠️  $ROADMAP still references unreleased features on main"
    warnings=$((warnings + 1))
  fi
else
  echo "  ⚠️  $ROADMAP not found"
  warnings=$((warnings + 1))
fi

# ─── 4. Check STATUS/CURRENT.md ───
echo ""
echo "── Checking STATUS/CURRENT.md ──"
STATUS="plans/STATUS/CURRENT.md"
if [[ -f "$STATUS" ]]; then
  STATUS_VERSION=$(grep -oP 'Released Version.*?v?\K[0-9]+\.[0-9]+\.[0-9]+' "$STATUS" | head -1 || true)
  if [[ -z "$STATUS_VERSION" ]]; then
    # Try alternate pattern: "v0.1.26 (crates.io)"
    STATUS_VERSION=$(grep -oP 'v\K[0-9]+\.[0-9]+\.[0-9]+' "$STATUS" | head -1 || true)
  fi
  if [[ -z "$STATUS_VERSION" ]]; then
    echo "  ⚠️  No version reference found in $STATUS"
    warnings=$((warnings + 1))
  elif [[ "$STATUS_VERSION" != "$WORKSPACE_VERSION" ]]; then
    echo "  ❌ $STATUS references v$STATUS_VERSION (expected v$WORKSPACE_VERSION)"
    failures=$((failures + 1))
  else
    echo "  ✅ Status version = v$STATUS_VERSION"
  fi
else
  echo "  ⚠️  $STATUS not found"
  warnings=$((warnings + 1))
fi

# ─── 5. Check git tag (optional) ───
if [[ "$CHECK_TAG" == "true" ]]; then
  echo ""
  echo "── Checking git tag ──"
  LATEST_TAG=$(git tag --sort=-creatordate | head -1 || true)
  EXPECTED_TAG="v$WORKSPACE_VERSION"

  if [[ -z "$LATEST_TAG" ]]; then
    echo "  ❌ No git tags found"
    failures=$((failures + 1))
  elif [[ "$LATEST_TAG" != "$EXPECTED_TAG" ]]; then
    echo "  ❌ Latest tag is $LATEST_TAG (expected $EXPECTED_TAG)"
    COMMITS_SINCE=$(git rev-list --count "${LATEST_TAG}..HEAD" 2>/dev/null || echo "?")
    echo "     $COMMITS_SINCE commits since $LATEST_TAG"
    failures=$((failures + 1))
  else
    echo "  ✅ Latest tag = $LATEST_TAG"
  fi
fi

# ─── 6. Check for unreleased changes (optional) ───
if [[ "$CHECK_UNRELEASED" == "true" ]]; then
  echo ""
  echo "── Checking for unreleased changes ──"
  LATEST_TAG=$(git tag --sort=-creatordate | head -1 || true)
  if [[ -n "$LATEST_TAG" ]]; then
    FEAT_COUNT=$(git log "${LATEST_TAG}..HEAD" --oneline --grep="^feat" 2>/dev/null | wc -l || echo 0)
    FIX_COUNT=$(git log "${LATEST_TAG}..HEAD" --oneline --grep="^fix" 2>/dev/null | wc -l || echo 0)
    TOTAL_COUNT=$(git rev-list --count "${LATEST_TAG}..HEAD" 2>/dev/null || echo 0)

    if [[ "$FEAT_COUNT" -gt 0 || "$FIX_COUNT" -gt 0 ]]; then
      echo "  ⚠️  Unreleased since $LATEST_TAG: $FEAT_COUNT feat, $FIX_COUNT fix ($TOTAL_COUNT total commits)"
      warnings=$((warnings + 1))
    else
      echo "  ✅ No unreleased feat/fix commits"
    fi
  else
    echo "  ⚠️  No tags found, cannot check unreleased changes"
    warnings=$((warnings + 1))
  fi
fi

# ─── 7. Check CHANGELOG.md has entry for current version ───
echo ""
echo "── Checking CHANGELOG.md ──"
if [[ -f "CHANGELOG.md" ]]; then
  if grep -q "## \[${WORKSPACE_VERSION}\]\|## v${WORKSPACE_VERSION}\|## ${WORKSPACE_VERSION}" CHANGELOG.md; then
    echo "  ✅ CHANGELOG.md has entry for v$WORKSPACE_VERSION"
  else
    echo "  ❌ CHANGELOG.md missing entry for v$WORKSPACE_VERSION"
    failures=$((failures + 1))
  fi
else
  echo "  ⚠️  CHANGELOG.md not found"
  warnings=$((warnings + 1))
fi

# ─── Summary ───
echo ""
echo "════════════════════════════════════════"
if [[ $failures -gt 0 ]]; then
  echo "❌ FAILED: $failures version mismatches, $warnings warnings"
  echo ""
  echo "Before releasing, fix all mismatches:"
  echo "  1. Bump Cargo.toml: cargo release patch|minor|major"
  echo "  2. Update ROADMAP_ACTIVE.md 'Released Version' line"
  echo "  3. Update STATUS/CURRENT.md version references"
  echo "  4. Update CHANGELOG.md with release entry"
  echo "  5. Commit all changes, then tag"
  exit 1
elif [[ $warnings -gt 0 ]]; then
  echo "⚠️  PASSED with $warnings warnings"
  exit 0
else
  echo "✅ ALL CHECKS PASSED — ready to release v$WORKSPACE_VERSION"
  exit 0
fi
