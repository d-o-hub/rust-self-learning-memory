#!/usr/bin/env bash
# check-changelog-headings.sh — the changelog must stay parseable by parse-changelog
#
# A duplicate `## [x.y.z]` heading makes parse-changelog reject the whole file,
# which is how a release-notes generator produced an empty release body while
# reporting success (LESSON-066, issue #1188). `update-changelog.sh` checks this
# only while it runs, so an edit made by hand or by a rebotted PR is unchecked:
# this guard runs the same invariant at the harness boundary.
#
# Checks: no duplicate version headings; exactly one `## [Unreleased]`; released
# headings in descending order.
#
# Usage: ./scripts/check-changelog-headings.sh [path/to/CHANGELOG.md]

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CHANGELOG="${1:-CHANGELOG.md}"
if [[ ! -r "$CHANGELOG" ]]; then
  echo "ERROR: cannot read '$CHANGELOG'" >&2
  exit 2
fi

status=0

dupes="$(grep -oE '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' "$CHANGELOG" | sort | uniq -d || true)"
if [[ -n "$dupes" ]]; then
  echo "HARNESS VIOLATION: changelog-headings — duplicate version headings:" >&2
  printf '%s\n' "$dupes" >&2
  echo "parse-changelog rejects the file, so the release body comes out empty." >&2
  status=1
fi

unreleased="$(grep -cE '^## \[Unreleased\]' "$CHANGELOG" || true)"
if [[ "$unreleased" -ne 1 ]]; then
  echo "HARNESS VIOLATION: changelog-headings — expected exactly one '## [Unreleased]', found $unreleased" >&2
  status=1
fi

versions="$(grep -oE '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' "$CHANGELOG" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)"
if [[ -n "$versions" ]]; then
  ordered="$(printf '%s\n' "$versions" | sort -t. -k1,1nr -k2,2nr -k3,3nr)"
  if [[ "$ordered" != "$versions" ]]; then
    echo "HARNESS VIOLATION: changelog-headings — released sections are not in descending order:" >&2
    printf 'file:    %s\nexpected: %s\n' \
      "$(printf '%s ' "$versions")" "$(printf '%s ' "$ordered")" >&2
    status=1
  fi
fi

if (( status )); then
  exit "$status"
fi
echo "OK: changelog headings unique, ordered, one Unreleased section"
