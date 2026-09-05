#!/usr/bin/env bash
# bns-cargo.sh - Run workspace cargo builds on the persistent Bunnyshell builder.
# Mirrors scripts/build-rust.sh modes but executes remotely via `bns exec`,
# keeping local CPU/disk free. The builder component stays alive running
# `sleep infinity` with warm /usr/local/cargo/registry + /app/target volumes.
#
# Prerequisites:
#   export BUNNYSHELL_TOKEN=<org-scoped PAT>
#   bns environments create --from-path bunnyshell.yaml \
#     --name rust-memory-builder --project <PROJECT_ID> --k8s <CLUSTER_ID>
#   bns environments deploy --id <ENV_ID> --no-wait
#
# Usage:
#   BNS_ENV_ID=<env> ./scripts/bns-cargo.sh check [-- <extra cargo args>]
#   BNS_COMPONENT_ID=<comp> ./scripts/bns-cargo.sh dev do-memory-core
#   ./scripts/bns-cargo.sh shell        # interactive remote shell
#   ./scripts/bns-cargo.sh sync         # git pull latest on the builder
#   ./scripts/bns-cargo.sh status       # env + component status (read-only)
#
# Env overrides:
#   BNS_ENV_ID        environment id (or pass --env <id>)
#   BNS_COMPONENT_ID  component id (skips lookup; or pass --component <id>)
#   BNS_CONTAINER     container name (default: rust-builder)
#   BNS_BRANCH        branch to checkout on `sync` (default: main)

set -euo pipefail

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly PROJECT_ROOT
cd "$PROJECT_ROOT"

BNS_ENV_ID="${BNS_ENV_ID:-}"
BNS_COMPONENT_ID="${BNS_COMPONENT_ID:-}"
BNS_CONTAINER="${BNS_CONTAINER:-rust-builder}"
BNS_BRANCH="${BNS_BRANCH:-main}"

# Bunnyshell clones gitRepo only as the Docker build context; the running
# container holds no checkout. The repo lives at REPO_DIR on the container's
# (ephemeral) root FS and is (re-)cloned on demand by ensure_repo. Cargo
# caches stay warm on persistent volumes regardless.
REPO_DIR="${BNS_REPO_DIR:-/app/repo}"
REPO_URL="${BNS_REPO_URL:-https://github.com/d-o-hub/rust-self-learning-memory.git}"

log_info() { echo -e "${GREEN}[bns-cargo]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[bns-cargo]${NC} $*" >&2; }
log_error() { echo -e "${RED}[bns-cargo]${NC} $*" >&2; }

usage() {
  cat <<'EOF'
Usage: bns-cargo.sh [--env <id>] [--component <id>] [--container <name>] <mode> [crate] [-- <extra args>]

Modes (mirror build-rust.sh, executed remotely):
  dev       cargo build --workspace (or --package <crate>)
  release   cargo build --release --workspace (or --package <crate>)
  check     cargo check --workspace (or --package <crate>)
  clippy    cargo clippy --workspace -- -D warnings
  nextest   cargo nextest run --workspace (add profile args after --)
  clean     cargo clean (use with care: wipes warm remote target cache)
  shell     interactive remote shell (bns exec --tty --stdin)
  sync      git fetch + checkout <branch> + pull on the builder
  status    show environment + component status (read-only, no build)

Examples:
  BNS_ENV_ID=abc123 ./scripts/bns-cargo.sh check
  ./scripts/bns-cargo.sh --env abc123 dev do-memory-core
  ./scripts/bns-cargo.sh --env abc123 nextest -- --profile ci
  ./scripts/bns-cargo.sh --env abc123 sync
EOF
  exit 1
}

require_bns() {
  if ! command -v bns >/dev/null 2>&1; then
    log_error "bns CLI not found. Install: brew install bunnyshell/tap/bunnyshell-cli"
    log_error "or https://github.com/bunnyshell/cli/releases"
    return 1
  fi
  if [[ -z "${BUNNYSHELL_TOKEN:-}" ]]; then
    log_error "BUNNYSHELL_TOKEN is unset. Export an org-scoped PAT first."
    return 1
  fi
}

resolve_component() {
  if [[ -n "$BNS_COMPONENT_ID" ]]; then
    return 0
  fi
  if [[ -z "$BNS_ENV_ID" ]]; then
    log_error "Set BNS_ENV_ID or pass --env <id> (or --component <id> to skip lookup)."
    return 1
  fi
  log_info "Resolving component 'rust-builder' in env $BNS_ENV_ID..."
  BNS_COMPONENT_ID="$(bns components list --environment "$BNS_ENV_ID" --output json \
    | jq -r '._embedded.item[] | select(.name == "rust-builder") | .id' | head -n1)"
  if [[ -z "$BNS_COMPONENT_ID" || "$BNS_COMPONENT_ID" == "null" ]]; then
    log_error "Component 'rust-builder' not found. Deploy bunnyshell.yaml first."
    return 1
  fi
  log_info "Using component $BNS_COMPONENT_ID"
}

remote_exec() {
  # Always pass -c: multi-container pods hang on an interactive picker without it.
  # Run inside the repo checkout; exit 3 signals a missing checkout (see ensure_repo).
  # Note: dir goes in $1 (shifted away) since $0 is not part of "$@" and cannot be shifted.
  bns exec "$BNS_COMPONENT_ID" -c "$BNS_CONTAINER" -- \
    sh -c 'cd "$1" || exit 3; shift; exec "$@"' sh "$REPO_DIR" "$@"
}

ensure_repo() {
  if remote_exec test -d .git 2>/dev/null; then
    return 0
  fi
  log_info "Cloning $REPO_URL (branch $BNS_BRANCH) to $REPO_DIR..."
  bns exec "$BNS_COMPONENT_ID" -c "$BNS_CONTAINER" -- \
    git clone --branch "$BNS_BRANCH" "$REPO_URL" "$REPO_DIR"
}

validate_crate() {
  local crate="$1"
  if [[ ! "$crate" =~ ^[a-z0-9_-]+$ ]]; then
    log_error "Invalid crate name: $crate"
    return 1
  fi
}

# ---- arg parsing: flags, then mode, optional crate, then -- <extra> ----
EXTRA_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --env) BNS_ENV_ID="${2:-}"; shift 2 ;;
    --component) BNS_COMPONENT_ID="${2:-}"; shift 2 ;;
    --container) BNS_CONTAINER="${2:-}"; shift 2 ;;
    --) shift; while [[ $# -gt 0 ]]; do EXTRA_ARGS+=("$1"); shift; done; break ;;
    -h|--help) usage ;;
    *) break ;;
  esac
done

[[ $# -lt 1 ]] && usage
MODE="$1"; shift || true
CRATE="${1:-}"
if [[ -n "$CRATE" && "$CRATE" != -* ]]; then
  shift || true
else
  CRATE=""
fi
# Allow trailing args without explicit -- for convenience.
while [[ $# -gt 0 ]]; do EXTRA_ARGS+=("$1"); shift; done

require_bns || exit 1

case "$MODE" in
  status)
    if [[ -z "$BNS_ENV_ID" ]]; then
      log_error "status needs BNS_ENV_ID / --env."
      exit 1
    fi
    bns environments show --id "$BNS_ENV_ID" --output json | jq '{id, name, operationStatus, namespace}'
    bns components list --environment "$BNS_ENV_ID" --output json \
      | jq '._embedded.item[] | {id, name, operationStatus}'
    ;;
  shell)
    resolve_component || exit 1
    log_info "Opening interactive shell on $BNS_COMPONENT_ID..."
    bns exec "$BNS_COMPONENT_ID" -c "$BNS_CONTAINER" --tty --stdin -- /bin/bash
    ;;
  sync)
    resolve_component || exit 1
    ensure_repo || exit 1
    log_info "Syncing builder to branch $BNS_BRANCH..."
    remote_exec git fetch origin
    remote_exec git checkout "$BNS_BRANCH"
    remote_exec git pull --ff-only origin "$BNS_BRANCH"
    ;;
  dev|release|check|clean|clippy|nextest)
    resolve_component || exit 1
    ensure_repo || exit 1
    if [[ -n "$CRATE" ]]; then
      validate_crate "$CRATE" || exit 1
    fi
    cmd=()
    case "$MODE" in
      dev)
        cmd=(cargo build)
        if [[ -n "$CRATE" ]]; then cmd+=(--package "$CRATE"); else cmd+=(--workspace); fi ;;
      release)
        cmd=(cargo build --release)
        if [[ -n "$CRATE" ]]; then cmd+=(--package "$CRATE"); else cmd+=(--workspace); fi ;;
      check)
        cmd=(cargo check)
        if [[ -n "$CRATE" ]]; then cmd+=(--package "$CRATE"); else cmd+=(--workspace); fi ;;
      clean)
        cmd=(cargo clean)
        if [[ -n "$CRATE" ]]; then cmd+=(--package "$CRATE"); fi
        if [[ -z "$CRATE" ]]; then
          log_warn "cargo clean wipes the warm remote cache; next build will be cold."
        fi ;;
      clippy)
        cmd=(cargo clippy --workspace --all-targets --)
        if [[ ${#EXTRA_ARGS[@]} -eq 0 ]]; then cmd+=(-D warnings); fi ;;
      nextest)
        cmd=(cargo nextest run --workspace) ;;
    esac
    if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
      cmd+=("${EXTRA_ARGS[@]}")
    fi
    remote_exec "${cmd[@]}"
    log_info "Remote $MODE build finished."
    ;;
  *)
    log_error "Unknown mode: $MODE"
    usage
    ;;
esac
