#!/usr/bin/env bash
# check-commit-ci-tokens.sh — no CI control token may appear in a commit message
#
# GitHub scans the entire last commit message for skip-CI control tokens and
# prose, backticks or quotes do not neutralize them. A branch whose head
# message carries one gets zero pull_request runs, so the aggregate required
# context can never report and the merge stays BLOCKED with nothing pending.
# Squash-merge then copies the body onto main, suppressing push CI for exactly
# the commit the release gate counts (LESSON-065).
#
# Usage:
#   ./scripts/check-commit-ci-tokens.sh                # check HEAD's message
#   ./scripts/check-commit-ci-tokens.sh --ref <rev>    # check one commit
#   ./scripts/check-commit-ci-tokens.sh --file <path>  # commit-msg hook input

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="head"
TARGET="HEAD"
MESSAGE_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ref)
      MODE="ref"
      TARGET="${2:?revision required}"
      shift 2
      ;;
    --file)
      MODE="file"
      MESSAGE_FILE="${2:?message file required}"
      shift 2
      ;;
    -h|--help)
      sed -n '2,15p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      exit 2
      ;;
  esac
done

# The bracket class in `sk[i]p` keeps this file's own text from containing the
# literal token spelling that GitHub's matcher looks for.
PATTERN='\[(sk[i]p[-_ ]ci|ci[-_ ]skip|no[-_ ]ci)\]'

case "$MODE" in
  head|ref)
    if ! git rev-parse --verify --quiet "$TARGET" >/dev/null; then
      echo "ERROR: '$TARGET' is not a revision" >&2
      exit 2
    fi
    MESSAGE="$(git log -1 --format=%B "$TARGET")"
    SOURCE="$TARGET"
    ;;
  file)
    if [[ ! -r "$MESSAGE_FILE" ]]; then
      echo "ERROR: cannot read message file '$MESSAGE_FILE'" >&2
      exit 2
    fi
    # Comment lines are stripped by git before the message is stored.
    MESSAGE="$(grep -v '^#' "$MESSAGE_FILE" || true)"
    SOURCE="$MESSAGE_FILE"
    ;;
esac

if printf '%s\n' "$MESSAGE" | grep -qniE "$PATTERN"; then
  echo "HARNESS VIOLATION: commit-ci-tokens — a CI control token in $SOURCE:" >&2
  printf '%s\n' "$MESSAGE" | grep -niE "$PATTERN" >&2
  echo "GitHub's matcher reads the whole message, prose context included." >&2
  echo "Reword in prose (*skip-ci marker*), never the bracketed form:" >&2
  echo "  git commit --amend -F <file> && git push --force-with-lease" >&2
  echo "Prove the rewrite was message-only:" >&2
  echo "  git rev-parse 'HEAD^{tree}'   # identical before and after" >&2
  exit 1
fi

echo "OK: no CI control tokens in $SOURCE"
