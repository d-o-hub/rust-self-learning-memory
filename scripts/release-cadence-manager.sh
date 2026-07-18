#!/usr/bin/env bash
# Release Cadence Manager CLI
# Monitor release cadence, detect drift, and coordinate resolution

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="${SCRIPT_DIR}/.."

usage() {
  cat <<EOF
Usage: $(basename "$0") <command> [options]

Commands:
  detect              Detect release drift
  resolve --pr <n>    Resolve drift for PR
  validate            Validate resolution
  status              Show release status
  help                Show this help

Options:
  --pr <n>            PR number (required for resolve)
  --severity <s>      Drift severity (optional)
  --reason <r>        Drift reason (optional)

Examples:
  $(basename "$0") detect
  $(basename "$0") resolve --pr 870
  $(basename "$0") validate
  $(basename "$0") status
EOF
}

detect_drift() {
  echo "=== Release Drift Detection ==="
  echo ""
  
  # Run drift detection script
  if [[ -f "$PROJECT_ROOT/scripts/check-release-drift.sh" ]]; then
    "$PROJECT_ROOT/scripts/check-release-drift.sh"
  else
    echo "ERROR: check-release-drift.sh not found" >&2
    exit 1
  fi
}

resolve_drift() {
  local pr_number="${1:?PR number required}"
  local severity="${2:-}"
  local reason="${3:-}"
  
  echo "=== Resolving Release Drift for PR #${pr_number} ==="
  echo ""
  
  # Check if PR exists
  if ! gh pr view "$pr_number" --json number,title,state &>/dev/null; then
    echo "ERROR: PR #${pr_number} not found" >&2
    exit 1
  fi
  
  # Check if PR is open
  local pr_state
  pr_state=$(gh pr view "$pr_number" --json state --jq '.state')
  if [[ "$pr_state" != "OPEN" ]]; then
    echo "ERROR: PR #${pr_number} is not open (state: $pr_state)" >&2
    exit 1
  fi
  
  # Check if label exists
  if ! gh label list --search "release-preparation" --json name --jq '.[].name' | grep -q "release-preparation"; then
    echo "Creating release-preparation label..."
    gh label create "release-preparation" \
      --color "0E8A16" \
      --description "Trusted-collaborator release preparation PR" \
      2>/dev/null || true
  fi
  
  # Add label to PR
  echo "Adding release-preparation label to PR #${pr_number}..."
  gh pr edit "$pr_number" --add-label "release-preparation"
  
  # Verify label was added
  local labels
  labels=$(gh pr view "$pr_number" --json labels --jq '.labels[].name')
  if echo "$labels" | grep -q "release-preparation"; then
    echo "✓ Label added successfully"
  else
    echo "ERROR: Failed to add label" >&2
    exit 1
  fi
  
  echo ""
  echo "=== Drift Resolution Complete ==="
  echo "PR #${pr_number} now has the release-preparation label."
  echo "The release cadence check should now pass."
}

validate_resolution() {
  echo "=== Validating Release Resolution ==="
  echo ""
  
  # Check if verify-release-state.sh exists
  if [[ -f "$PROJECT_ROOT/scripts/verify-release-state.sh" ]]; then
    echo "Running verify-release-state.sh..."
    "$PROJECT_ROOT/scripts/verify-release-state.sh" --check-unreleased
  else
    echo "WARNING: verify-release-state.sh not found" >&2
  fi
  
  echo ""
  echo "=== Validation Complete ==="
}

show_status() {
  echo "=== Release Status ==="
  echo ""
  
  # Check if release-manager.sh exists
  if [[ -f "$PROJECT_ROOT/scripts/release-manager.sh" ]]; then
    "$PROJECT_ROOT/scripts/release-manager.sh" status
  else
    echo "WARNING: release-manager.sh not found" >&2
  fi
  
  echo ""
  echo "=== Status Complete ==="
}

main() {
  local command="${1:-help}"
  shift || true
  
  local pr_number=""
  local severity=""
  local reason=""
  
  # Parse options
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --pr)
        pr_number="$2"
        shift 2
        ;;
      --severity)
        severity="$2"
        shift 2
        ;;
      --reason)
        reason="$2"
        shift 2
        ;;
      *)
        echo "ERROR: Unknown option: $1" >&2
        usage
        exit 1
        ;;
    esac
  done
  
  case "$command" in
    detect)
      detect_drift
      ;;
    resolve)
      if [[ -z "$pr_number" ]]; then
        echo "ERROR: --pr <n> is required for resolve command" >&2
        usage
        exit 1
      fi
      resolve_drift "$pr_number" "$severity" "$reason"
      ;;
    validate)
      validate_resolution
      ;;
    status)
      show_status
      ;;
    help|--help|-h)
      usage
      ;;
    *)
      echo "ERROR: Unknown command: $command" >&2
      usage
      exit 1
      ;;
  esac
}

main "$@"
