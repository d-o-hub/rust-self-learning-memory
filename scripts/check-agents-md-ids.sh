#!/usr/bin/env bash
# check-agents-md-ids.sh — keep bare numeric ids out of AGENTS.md
#
# The required Codacy context flags a bare 10-15 digit run in AGENTS.md as a
# possible phone number and parks the check at `action_required`, which blocks
# the merge while every other row is green. The same run ids in
# agent_docs/LESSONS.md are not flagged: the analyzer reports on the *changed*
# file, and AGENTS.md is the file it reads (LESSON-076).
#
# Policy: cite a CI run by its short run number in AGENTS.md; keep raw API ids
# in LESSONS.md. Adding a `.codacy.yml` to silence this is worse than the
# finding — its presence disables the repo-wide UI ignore list.
#
# Usage: ./scripts/check-agents-md-ids.sh [path/to/AGENTS.md]

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGET="${1:-AGENTS.md}"
if [[ ! -r "$TARGET" ]]; then
  echo "ERROR: cannot read '$TARGET'" >&2
  exit 2
fi

# A digit run of 10-15 that is not part of a longer alphanumeric token, so SHAs
# (hex, letters present) and dotted versions are not matched.
hits="$(grep -nE '(^|[^0-9A-Za-z._-])[0-9]{10,15}([^0-9A-Za-z._-]|$)' "$TARGET" || true)"

if [[ -n "$hits" ]]; then
  echo "HARNESS VIOLATION: agents-md-ids — bare numeric runs in $TARGET read as PII by the required Codacy context:" >&2
  printf '%s\n' "$hits" >&2
  echo "Replace each with the CI run number, or move the id to agent_docs/LESSONS.md." >&2
  echo "Verify with: gh api repos/{owner}/{repo}/commits/<sha>/check-runs -q '.total_count'" >&2
  exit 1
fi

echo "OK: no bare numeric runs in $TARGET"
