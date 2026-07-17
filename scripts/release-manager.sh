#!/usr/bin/env bash
# release-manager.sh — Single canonical release CLI for this workspace.
#
# One path for humans and agents (see .agents/skills/release-guard/SKILL.md):
#
#   1. Land version bump + CHANGELOG on main (PR)
#   2. Wait for main CI green on HEAD
#   3. ./scripts/release-manager.sh ship --execute
#      → validate docs/version + local quality
#      → re-check main CI green
#      → git tag -a vX.Y.Z at origin/main HEAD
#      → git push origin refs/tags/vX.Y.Z
#   4. GitHub Actions release.yml builds artifacts + creates the GitHub Release
#
# NEVER: gh release create (manual). NEVER: tag from a non-main branch.
# NEVER: --admin merge. NEVER: tag when Cargo.toml version ≠ tag.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly PROJECT_ROOT

ACTION="${1:-help}"
DRY_RUN="true"
TAG=""
SKIP_LOCAL_TESTS="false"
SKIP_CI_CHECK="false"

usage() {
  cat <<EOF
Usage: $(basename "$0") <action> [options]

Canonical ship path (use this every time):
  $(basename "$0") ship --execute

Actions:
  status                  Show version, latest tag, sync, and main CI summary
  validate                Version/docs check + local quality gates
  ci-check                Fail if origin/main HEAD has incomplete/failed CI
  prepare                 validate + create annotated tag v\${version} (local)
  publish                 Verify tag on HEAD, push tag only (triggers release.yml)
  ship | full             validate + ci-check + prepare + publish
  wait-release            Poll until Release workflow for the tag completes
  rollback                Delete local tag only (--tag required)

Options:
  --tag <tag>             Tag for rollback (e.g. v0.1.35)
  --execute               Perform real git tag/push (default is dry-run)
  --skip-local-tests      Skip nextest/clippy/quality-gates (docs-only emergency)
  --skip-ci-check         Skip GitHub Actions green check (NOT for production)
  -h, --help              Show this help

Examples:
  $(basename "$0") status
  $(basename "$0") validate
  $(basename "$0") ship              # dry-run
  $(basename "$0") ship --execute    # real tag + push
  $(basename "$0") wait-release
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
    --skip-local-tests)
      SKIP_LOCAL_TESTS="true"
      shift
      ;;
    --skip-ci-check)
      SKIP_CI_CHECK="true"
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

workspace_version() {
  sed -nE 's/^version[[:space:]]*=[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/p' Cargo.toml | head -1
}

latest_release_tag() {
  git tag -l 'v*' --sort=-v:refname | head -1
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
  git fetch origin main --tags --quiet
  if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
    echo "Local main must exactly match origin/main before release" >&2
    echo "  HEAD:        $(git rev-parse --short HEAD)" >&2
    echo "  origin/main: $(git rev-parse --short origin/main)" >&2
    exit 1
  fi
}

# Fail if any workflow run for origin/main HEAD is still pending or failed.
# Treat success + skipped as OK. Neutral is OK. Missing runs: warn only if zero runs.
do_ci_check() {
  echo "═══ Phase CI: origin/main HEAD must be green ═══"
  if [[ "$SKIP_CI_CHECK" == "true" ]]; then
    echo "  ⚠️  --skip-ci-check set; skipping GitHub Actions verification"
    return 0
  fi
  if ! command -v gh >/dev/null 2>&1; then
    echo "  ❌ gh CLI required for ci-check" >&2
    exit 1
  fi
  git fetch origin main --quiet
  local head
  head=$(git rev-parse origin/main)
  echo "  origin/main: $(git rev-parse --short "$head")"

  local runs_json
  runs_json=$(gh run list --branch main --commit "$head" --limit 30 \
    --json status,conclusion,name,databaseId 2>/dev/null || echo '[]')

  local total pending failed
  total=$(echo "$runs_json" | jq 'length')
  pending=$(echo "$runs_json" | jq '[.[] | select(.status != "completed")] | length')
  failed=$(echo "$runs_json" | jq '[.[] | select(
      .status == "completed" and
      (.conclusion != "success") and
      (.conclusion != "skipped") and
      (.conclusion != "neutral") and
      (.conclusion != null)
    )] | length')

  echo "  workflow runs on HEAD: total=$total pending=$pending failed=$failed"
  echo "$runs_json" | jq -r '.[] | "    \(.status)/\(.conclusion // "-")  \(.name)"' 2>/dev/null || true

  if [[ "$total" -eq 0 ]]; then
    echo "  ❌ No GitHub Actions runs found for origin/main HEAD yet" >&2
    echo "     Wait for push-triggered CI, then re-run." >&2
    exit 1
  fi
  if [[ "$pending" -gt 0 ]]; then
    echo "  ❌ Main CI still has $pending incomplete run(s)" >&2
    exit 1
  fi
  if [[ "$failed" -gt 0 ]]; then
    echo "  ❌ Main CI has $failed failed run(s) on HEAD" >&2
    exit 1
  fi
  echo "  ✅ Main CI green on origin/main HEAD"
}

do_status() {
  local version tag head_short remote_short
  version=$(workspace_version)
  git fetch origin main --tags --quiet 2>/dev/null || true
  tag=$(latest_release_tag)
  head_short=$(git rev-parse --short HEAD 2>/dev/null || echo "?")
  remote_short=$(git rev-parse --short origin/main 2>/dev/null || echo "?")
  echo "Release status"
  echo "  workspace version:  $version"
  echo "  expected tag:       v${version}"
  echo "  latest local tag:   ${tag:-none}"
  echo "  local HEAD:         $head_short ($(git branch --show-current 2>/dev/null || echo detached))"
  echo "  origin/main:        $remote_short"
  if [[ -n "$(git status --porcelain 2>/dev/null || true)" ]]; then
    echo "  worktree:           DIRTY"
  else
    echo "  worktree:           clean"
  fi
  if command -v gh >/dev/null 2>&1 && git rev-parse origin/main >/dev/null 2>&1; then
    local head runs_json pending failed
    head=$(git rev-parse origin/main)
    runs_json=$(gh run list --branch main --commit "$head" --limit 20 \
      --json status,conclusion,name 2>/dev/null || echo '[]')
    pending=$(echo "$runs_json" | jq '[.[] | select(.status != "completed")] | length')
    failed=$(echo "$runs_json" | jq '[.[] | select(
        .status == "completed" and
        (.conclusion != "success") and
        (.conclusion != "skipped") and
        (.conclusion != "neutral") and
        (.conclusion != null)
      )] | length')
    echo "  main CI pending:    $pending"
    echo "  main CI failed:     $failed"
  fi
  echo ""
  echo "Ship when version docs match and main CI is green:"
  echo "  ./scripts/release-manager.sh ship --execute"
}

do_validate() {
  echo "═══ Phase 0: Version state verification ═══"
  # Always execute version check (read-only)
  ./scripts/verify-release-state.sh --check-unreleased
  echo ""
  if [[ "$SKIP_LOCAL_TESTS" == "true" ]]; then
    echo "═══ Phase 1: Code quality (SKIPPED via --skip-local-tests) ═══"
    return 0
  fi
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
  local version
  version=$(workspace_version)
  if [[ -z "$version" ]]; then
    echo "Unable to determine the workspace version" >&2
    exit 1
  fi
  local tag="v${version}"
  if git rev-parse "$tag" >/dev/null 2>&1; then
    echo "Tag $tag already exists locally" >&2
    if [[ "$DRY_RUN" == "false" ]]; then
      if [[ "$(git rev-parse "${tag}^{commit}")" != "$(git rev-parse HEAD)" ]]; then
        echo "  and it does not point at current HEAD — aborting" >&2
        exit 1
      fi
      echo "  (points at HEAD; will reuse for publish)"
    fi
  else
    if [[ "$DRY_RUN" == "false" ]]; then
      ensure_release_context
    fi
    run_cmd "git tag -a ${tag} -m 'Release ${tag}'"
  fi
  echo "Tag ready: $tag → $(git rev-parse --short HEAD 2>/dev/null || echo HEAD)"
}

do_publish() {
  local tag version
  version=$(workspace_version)
  tag="v${version}"

  if [[ "$DRY_RUN" == "true" ]]; then
    echo "[dry-run] ./scripts/verify-release-state.sh --check-tag  # after tag exists"
    echo "[dry-run] git push origin refs/tags/${tag}"
    echo "[dry-run] # release.yml creates the GitHub Release (do not gh release create)"
    return 0
  fi

  ensure_release_context
  if ! git rev-parse "$tag" >/dev/null 2>&1; then
    echo "Missing local tag $tag — run prepare first" >&2
    exit 1
  fi
  if [[ "$(git rev-parse "${tag}^{commit}")" != "$(git rev-parse HEAD)" ]]; then
    echo "$tag must point to the current main HEAD" >&2
    exit 1
  fi
  local tagged_version
  tagged_version=$(git show "${tag}:Cargo.toml" | sed -nE 's/^version[[:space:]]*=[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/p' | head -1)
  if [[ "$tagged_version" != "$version" ]]; then
    echo "$tag contains Cargo.toml version $tagged_version (expected $version)" >&2
    exit 1
  fi
  # --check-tag expects the tag to already exist as latest; after first create it works
  ./scripts/verify-release-state.sh --check-tag || {
    echo "Note: --check-tag may warn until tag is the latest on remote; continuing if local checks passed" >&2
  }

  if git ls-remote --tags origin "refs/tags/${tag}" | grep -q .; then
    echo "Remote already has $tag — not re-pushing" >&2
    exit 1
  fi

  run_cmd "git push origin refs/tags/${tag}"
  echo ""
  echo "✅ Pushed $tag. GitHub Actions 'Release' workflow will create the release."
  echo "   Monitor: gh run list --workflow=release.yml --limit 5"
  echo "   Or:      ./scripts/release-manager.sh wait-release"
}

do_ship() {
  echo "════════════════════════════════════════════════════════"
  echo " Canonical release: ship v$(workspace_version)"
  echo " Dry-run: $DRY_RUN"
  echo "════════════════════════════════════════════════════════"
  do_validate
  do_ci_check
  do_prepare
  do_publish
}

do_wait_release() {
  local version tag
  version=$(workspace_version)
  tag="v${version}"
  if ! command -v gh >/dev/null 2>&1; then
    echo "gh CLI required" >&2
    exit 1
  fi
  echo "Waiting for Release workflow for $tag ..."
  local i status conclusion
  for i in $(seq 1 60); do
    local row
    row=$(gh run list --workflow=release.yml --limit 10 \
      --json databaseId,status,conclusion,displayTitle \
      --jq "[.[] | select(.displayTitle | contains(\"${tag}\"))] | .[0] // empty" 2>/dev/null || true)
    if [[ -z "$row" || "$row" == "null" ]]; then
      # fallback: newest release workflow run
      row=$(gh run list --workflow=release.yml --limit 3 \
        --json databaseId,status,conclusion,displayTitle \
        --jq '.[0] // empty' 2>/dev/null || true)
    fi
    if [[ -z "$row" || "$row" == "null" ]]; then
      echo "  poll $i: no release run found yet"
    else
      status=$(echo "$row" | jq -r '.status')
      conclusion=$(echo "$row" | jq -r '.conclusion // "-"')
      echo "  poll $i: $status/$conclusion $(echo "$row" | jq -r '.displayTitle')"
      if [[ "$status" == "completed" ]]; then
        if [[ "$conclusion" == "success" ]]; then
          echo "✅ Release workflow succeeded"
          gh release view "$tag" 2>/dev/null || gh release list --limit 3
          exit 0
        fi
        echo "❌ Release workflow finished with: $conclusion" >&2
        exit 1
      fi
    fi
    sleep 30
  done
  echo "Timeout waiting for release.yml" >&2
  exit 1
}

do_rollback() {
  if [[ -z "$TAG" ]]; then
    echo "--tag is required for rollback" >&2
    exit 1
  fi
  run_cmd "git tag -d ${TAG}"
  echo "Rollback note: remote tag deletion is intentional and manual after review:"
  echo "  git push origin :refs/tags/${TAG}"
}

case "$ACTION" in
  status)
    do_status
    ;;
  validate)
    do_validate
    ;;
  ci-check)
    do_ci_check
    ;;
  prepare)
    do_prepare
    ;;
  publish)
    do_publish
    ;;
  ship|full)
    do_ship
    ;;
  wait-release)
    do_wait_release
    ;;
  rollback)
    do_rollback
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
