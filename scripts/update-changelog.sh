#!/usr/bin/env bash
# update-changelog.sh — write one release section into CHANGELOG.md, idempotently.
#
# Two writers can produce a `## [X.Y.Z]` heading: the release-preparation PR
# (curated prose) and this generator (git-cliff render). An unbounded render
# clobbered curated content (#1178); the bounded `--prepend` added in #1186 got
# the commit range right but still inserts unconditionally, so a tag push that
# follows a preparation PR duplicates the heading (#1188) and parse-changelog
# then rejects the entire file, which is what leaves a GitHub Release without
# notes.
#
# Output contract: one heading per version, target version present and
# non-empty, file parseable.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

CHANGELOG="CHANGELOG.md"
CONFIG="cliff.toml"
TAG=""
CHECK_ONLY=false

usage() {
  cat <<'EOF'
Usage: scripts/update-changelog.sh [--tag vX.Y.Z] [--check-only]

  --tag         Release tag to write the section for (default: latest release tag).
  --check-only  Validate CHANGELOG.md without generating anything.

Exit 0 = changelog correct (generated, or already present so generation skipped).
Exit 1 = the file would break cargo-dist release notes. Exit 2 = bad usage.
EOF
}

die() {
  local message="$1" code="${2:-1}"
  echo "error: $message" >&2
  exit "$code"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      [[ $# -ge 2 ]] || die "--tag needs a value" 2
      TAG="$2"
      shift 2
      ;;
    --check-only) CHECK_ONLY=true; shift ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; die "unknown argument: $1" 2 ;;
  esac
done

# Same shape guard as the workflow's "Resolve release tag" step: a non-release
# ref must never be rendered as a version section.
if [[ -z "$TAG" ]]; then
  TAG="$(git tag --list 'v[0-9]*.[0-9]*.[0-9]*' --sort=-v:refname | head -1 || true)"
fi
[[ -n "$TAG" ]] || die "no release tag given and no vMAJOR.MINOR.PATCH tag found" 2
printf '%s' "$TAG" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$' \
  || die "not a release tag: '$TAG' (expected vMAJOR.MINOR.PATCH)" 2

VERSION="${TAG#v}"

section_body_lines() {
  awk -v heading="## [$VERSION]" '
    index($0, heading) == 1 { in_section = 1; next }
    in_section && /^## / { exit }
    in_section { print }
  ' "$CHANGELOG" | grep -cE '^(### |- |\*\*)' || true
}

count_headings() {
  grep -cE "^## \[$VERSION\]" "$CHANGELOG" || true
}

check_integrity() {
  local failures=0

  [[ -f "$CHANGELOG" ]] || { echo "error: $CHANGELOG not found" >&2; return 1; }

  local duplicates
  duplicates="$(grep -oE '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' "$CHANGELOG" | sort | uniq -d || true)"
  if [[ -n "$duplicates" ]]; then
    echo "error: duplicate version headings — parse-changelog rejects the whole file:" >&2
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      echo "  $line" >&2
    done <<<"$duplicates"
    failures=$((failures + 1))
  fi

  local hits
  hits="$(count_headings)"
  if [[ "$hits" -eq 0 ]]; then
    echo "error: $CHANGELOG has no section for $VERSION" >&2
    failures=$((failures + 1))
  elif [[ "$hits" -gt 1 ]]; then
    echo "error: $CHANGELOG has $hits sections for $VERSION" >&2
    failures=$((failures + 1))
  fi

  local body_lines
  body_lines="$(section_body_lines)"
  if [[ "$body_lines" -eq 0 ]]; then
    echo "error: the $VERSION section is empty — release notes would render blank" >&2
    failures=$((failures + 1))
  fi

  # Cross-check with the parser cargo-dist actually uses when it is available.
  # The structural checks above already fail the duplicate case, so an absent
  # binary is a warning rather than a hole.
  if command -v parse-changelog >/dev/null 2>&1; then
    local parsed
    if ! parsed="$(parse-changelog "$CHANGELOG" "$VERSION" 2>&1)" || [[ -z "$parsed" ]]; then
      echo "error: parse-changelog cannot render $VERSION" >&2
      printf '%s\n' "$parsed" | head -3 >&2
      failures=$((failures + 1))
    fi
  else
    echo "warning: parse-changelog not installed; structural checks only" >&2
  fi

  if [[ "$failures" -gt 0 ]]; then
    echo "error: $failures CHANGELOG.md integrity failure(s)" >&2
    return 1
  fi
  echo "ok: $CHANGELOG is parseable and $VERSION has exactly one non-empty section"
}

if [[ "$CHECK_ONLY" == "true" ]]; then
  check_integrity
  exit 0
fi

if grep -qE "^## \[$VERSION\]" "$CHANGELOG"; then
  echo "skip: $CHANGELOG already documents $VERSION (the release-preparation PR owns the curated section)"
  check_integrity
  exit 0
fi

command -v git-cliff >/dev/null 2>&1 \
  || die "git-cliff is not installed; refusing to report success while writing nothing (install: cargo install --locked git-cliff --version 2.14.2)"

git-cliff --config "$CONFIG" --latest --tag "$TAG" --prepend "$CHANGELOG"
check_integrity
echo "generated: wrote the $VERSION section for commits since the previous tag"
