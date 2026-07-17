#!/usr/bin/env bash
# release-manager.sh - Unified release operations wrapper.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly PROJECT_ROOT

ACTION="${1:-help}"
DRY_RUN="true"
TAG=""

usage() {
  cat <<EOF
Usage: $(basename "$0") <action> [options]

Actions:
  validate                Run release validation sequence
  prepare                 Run validation + create the release tag
  publish                 Verify and push the prepared release tag
  rollback                Rollback local release tag/commit state
  full                    validate + prepare + publish

Options:
  --tag <tag>                   Tag for rollback (required for rollback)
  --execute                     Execute commands (default is dry run)
  -h, --help                    Show this help

Examples:
  $(basename "$0") validate
  $(basename "$0") prepare
  $(basename "$0") publish --execute
  $(basename "$0") rollback --tag v0.1.17 --execute
  $(basename "$0") full --execute
EOF
}

shift || true
while [[ $# -gt 0 ]]; do
  case "$1" in
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
  ./scripts/verify-release-state.sh --check-unreleased
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

workspace_version() {
  sed -nE 's/^version[[:space:]]*=[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/p' Cargo.toml | head -1
}

ensure_release_context() {
  local branch
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "Release requires a clean worktree" >&2
    exit 1
  fi
  branch=$(git branch --show-current)
  if [[ "$branch" != "main" ]]; then
    echo "Release tags must be created from main (current branch: $branch)" >&2
    exit 1
  fi
  git fetch origin main
  if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
    echo "Local main must exactly match origin/main before release" >&2
    exit 1
  fi
}

do_prepare() {
  do_validate
  local version
  version=$(workspace_version)
  if [[ -z "$version" ]]; then
    echo "Unable to determine the workspace version" >&2
    exit 1
  fi
  if [[ "$DRY_RUN" == "false" ]]; then
    ensure_release_context
  fi
  run_cmd "git tag -a v${version} -m 'Release v${version}'"
}

do_publish() {
  local tag tagged_version version
  version=$(workspace_version)
  tag="v${version}"
  if [[ "$DRY_RUN" == "true" ]]; then
    echo "[dry-run] ./scripts/verify-release-state.sh --check-tag"
  else
    ensure_release_context
    ./scripts/verify-release-state.sh --check-tag
    if [[ "$(git rev-parse "${tag}^{commit}")" != "$(git rev-parse HEAD)" ]]; then
      echo "$tag must point to the current main HEAD" >&2
      exit 1
    fi
    tagged_version=$(git show "${tag}:Cargo.toml" | sed -nE 's/^version[[:space:]]*=[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/p' | head -1)
    if [[ "$tagged_version" != "$version" ]]; then
      echo "$tag contains Cargo.toml version $tagged_version (expected $version)" >&2
      exit 1
    fi
  fi
  run_cmd "git push origin refs/tags/${tag}"
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
  help|-h|--help|"")
    usage
    ;;
  *)
    echo "Unknown action: $ACTION" >&2
    usage >&2
    exit 1
    ;;
esac
