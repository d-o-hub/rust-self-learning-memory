#!/usr/bin/env bash
# release-manager.sh - Unified release operations wrapper.

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

ACTION="${1:-help}"
RELEASE_LEVEL="patch"
DRY_RUN="true"
TAG=""

usage() {
  cat <<EOF
Usage: $(basename "$0") <action> [options]

Actions:
  validate                Run release validation sequence
  prepare                 Run validation + prepare release metadata
  publish                 Publish prepared release artifacts
  rollback                Rollback local release tag/commit state
  full                    validate + prepare + publish

Options:
  --level <patch|minor|major>   Release bump level (default: patch)
  --tag <tag>                   Tag for rollback (required for rollback)
  --execute                     Execute commands (default is dry run)
  -h, --help                    Show this help

Examples:
  $(basename "$0") validate
  $(basename "$0") prepare --level minor
  $(basename "$0") publish --execute
  $(basename "$0") rollback --tag v0.1.17 --execute
  $(basename "$0") full --level patch
EOF
}

shift || true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --level)
      RELEASE_LEVEL="${2:-}"
      shift 2
      ;;
    --tag)
      TAG="${2:-}"
      shift 2
      ;;
    --execute)
      DRY_RUN="false"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

cd "$PROJECT_ROOT"

if [[ "$RELEASE_LEVEL" != "patch" && "$RELEASE_LEVEL" != "minor" && "$RELEASE_LEVEL" != "major" ]]; then
  echo "Invalid --level value: $RELEASE_LEVEL" >&2
  exit 1
fi

run_cmd() {
  local cmd="$1"
  if [[ "$DRY_RUN" == "true" ]]; then
    echo "[dry-run] $cmd"
  else
    echo "[exec] $cmd"
    eval "$cmd"
  fi
}

do_validate() {
  echo "═══ Phase 0: Version state verification ═══"
  # Always execute version check (not dry-run) — it's read-only
  ./scripts/verify-release-state.sh --check-tag --check-unreleased
  echo ""
  echo "═══ Phase 1: Code quality ═══"
  run_cmd "./scripts/check-docs-integrity.sh"
  run_cmd "./scripts/code-quality.sh fmt"
  run_cmd "./scripts/code-quality.sh clippy"
  run_cmd "./scripts/build-rust.sh check"
  run_cmd "cargo nextest run --all"
  run_cmd "cargo test --doc"
  run_cmd "./scripts/quality-gates.sh"
}

do_prepare() {
  do_validate
  run_cmd "cargo release ${RELEASE_LEVEL}"
}

do_publish() {
  run_cmd "git push"
  run_cmd "git push --tags"
  run_cmd "gh release list --limit 5"
}

do_rollback() {
  if [[ -z "$TAG" ]]; then
    echo "--tag is required for rollback" >&2
    exit 1
  fi

  run_cmd "git tag -d ${TAG}"
  echo "Rollback note: remote tag deletion and commit-level rollback are intentionally manual."
  echo "Use explicit commands only after team review."
}

case "$ACTION" in
  validate)
    do_validate
    ;;
  prepare)
    do_prepare
    ;;
  publish)
    do_publish
    ;;
  rollback)
    do_rollback
    ;;
  full)
    do_prepare
    do_publish
    ;;
  help|"")
    usage
    ;;
  *)
    echo "Unknown action: $ACTION" >&2
    usage >&2
    exit 1
    ;;
esac
