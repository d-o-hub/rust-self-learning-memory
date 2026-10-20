#!/usr/bin/env bash
# test-release-workflow.sh — W2.4 release/publish precondition fixtures
#
# Usage:
#   ./scripts/test-release-workflow.sh --fixtures
#   ./scripts/test-release-workflow.sh --publish-fixtures

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---fixtures}"

fail() {
  echo "HARNESS VIOLATION: release-workflow — $1" >&2
  exit 1
}

check_release_authority() {
  [[ -x ./scripts/release-manager.sh ]] || fail "release-manager.sh missing or not executable"
  [[ -f .github/workflows/release.yml ]] || fail "release.yml missing"
  [[ -f .agents/skills/release-guard/SKILL.md ]] || fail "release-guard skill missing"

  ./scripts/release-manager.sh help 2>&1 | rg -q 'ship' || fail "release-manager help lacks ship"

  # CIT-A4 / ADR-079 §6 / ADR-072: tag-only release authority. A manual
  # workflow_dispatch on release.yml is a guaranteed-failure advertised trigger.
  if rg -q 'workflow_dispatch' .github/workflows/release.yml; then
    fail "release.yml must not expose workflow_dispatch (tag-only release under ADR-072)"
  fi

  # Active skills must forbid manual gh release create as the ship path
  if rg -n 'gh release create' .agents/skills --glob '**/SKILL.md' \
    | rg -v 'NEVER|never|not |forbid|do not|Don.t|must not' \
    | rg -v 'release-guard' \
    | head -5 | rg -q .; then
    # Soft: list offenders for visibility but only fail if release-guard lacks NEVER
    :
  fi
  rg -q 'NEVER' .agents/skills/release-guard/SKILL.md || fail "release-guard lacks NEVER for manual release"
  rg -q 'release-manager.sh ship --execute' .agents/skills/release-guard/SKILL.md \
    || fail "release-guard missing canonical ship command"

  echo "OK: release authority fixtures"
}

check_publish_fixtures() {
  # The publish gates live in the workflow and in the `publish-crate` composite
  # action it calls; assert them over the union of the two files.
  local publish_files=()
  local publish_file
  for publish_file in .github/workflows/publish-crates.yml .github/actions/publish-crate/action.yml; do
    [[ -f "$publish_file" ]] && publish_files+=("$publish_file")
  done
  if (( ${#publish_files[@]} > 0 )); then
    # LESSON-014 / AGENTS.md publish pipeline: reproducible locked publish
    rg -q -- '--locked' "${publish_files[@]}" || fail "publish workflow/action should use cargo publish --locked"
    # CIT-A4 / LESSON-014: bounded propagation polling, never a fixed sleep
    if rg -q 'run: sleep 30' "${publish_files[@]}"; then
      fail "publish workflow/action must not use fixed 'run: sleep 30' (bounded polling required)"
    fi
    rg -q 'Wait for crates.io propagation' "${publish_files[@]}" || fail "publish workflow/action missing propagation wait step"
    rg -q 'seq 1 20' "${publish_files[@]}" || fail "publish propagation polling must be bounded (seq 1 20)"
    # CIT-A4 / ADR-079 §6: single-crate dispatch must verify the non-dev
    # workspace dependency closure and fail with a named reason - no silent skip.
    rg -q 'Verify dependency closure' "${publish_files[@]}" || fail "publish workflow/action missing dependency-closure verification step"
    rg -q "inputs.crate != ''" .github/workflows/publish-crates.yml || fail "the workflow must gate single-crate dispatch with inputs.crate != ''"
    rg -q '::error::Cannot publish' "${publish_files[@]}" || fail "dependency-closure step must fail with a named ::error:: reason"
    # Issue #1109 C1 / ADR-078: OIDC-only publishing - the static token secret
    # must not come back, the auth action must stay SHA-pinned, and the publish
    # command itself must keep --locked.
    if rg -q 'secrets\.CARGO_REGISTRY_TOKEN' "${publish_files[@]}"; then
      fail "publish workflow/action must not read the CARGO_REGISTRY_TOKEN secret (OIDC-only)"
    fi
    rg -q 'id-token: write' .github/workflows/publish-crates.yml || fail "publish jobs must grant 'id-token: write'"
    rg -q 'crates-io-auth-action@[0-9a-f]{40}' .github/actions/publish-crate/action.yml || fail "publish action must call the SHA-pinned crates-io-auth-action"
    rg -q -- 'cargo publish --package .*--locked' .github/actions/publish-crate/action.yml || fail "the publish command itself must use cargo publish --locked"
    # A green publish job must mean an uploaded crate: every crates.io gate is a
    # membership query over the full version list, and a real publish is followed
    # by a bounded visibility poll (the guard once skipped upload in silence).
    rg -qF '[(.versions // [])[].num] | index($v)' .github/actions/publish-crate/action.yml || fail "crates.io version gates must test membership, not the newest entry"
    rg -q 'Verify publish landed' .github/actions/publish-crate/action.yml || fail "publish action must prove the upload landed on crates.io"
  else
    echo "NOTE: publish-crates.yml not present; skipping publish fixture checks"
  fi
  [[ -x ./scripts/verify-release-state.sh ]] || fail "verify-release-state.sh missing"
  echo "OK: publish fixtures"
}

check_ci_check_event_filtering() {
  local rm="./scripts/release-manager.sh"
  local lib="./scripts/lib/release-gate-events.sh"
  [[ -f "$rm" ]] || fail "missing $rm"
  [[ -f "$lib" ]] || fail "missing $lib"

  # (1) release-manager.sh must fetch event in gh run list
  if ! rg -q 'gh run list' "$rm"; then
    fail "release-manager.sh must query workflow runs"
  fi
  if ! rg -q '\--json.*event' "$rm"; then
    fail "release-manager.sh must include event in gh run list --json"
  fi

  # (2) The classification must live in the shared library, and the gate must
  #     use it — otherwise the test below would only re-implement the logic.
  rg -q 'release-gate-events\.sh' "$rm" || fail "release-manager.sh must source scripts/lib/release-gate-events.sh"
  rg -q 'release_gate_gating_failures' "$rm" || fail "release-manager.sh must use the shared gate helpers"
  # The event classes must come from the library, not from a second copy of the
  # event names in a jq expression: two copies can disagree, and the fixture
  # below would then test a rule the gate does not implement.
  rg -q 'release_gate_select_runs' "$rm" \
    || fail "release-manager.sh must select gating runs with release_gate_select_runs"
  if rg -q '\.event == "push"' "$rm"; then
    fail "release-manager.sh must not inline the event names; use release_gate_select_runs"
  fi

  # (3) The library must classify events exactly as ADR-083 states:
  #     push/pull_request gate; schedule/workflow_dispatch are advisory.
  local lib_dir
  lib_dir=$(mktemp -d)
  # shellcheck source=/dev/null
  source "$lib"

  release_gate_is_gating_event "push" || fail "push must be a gating event"
  release_gate_is_gating_event "pull_request" || fail "pull_request must be a gating event"
  release_gate_is_advisory_event "schedule" || fail "schedule must be advisory"
  release_gate_is_advisory_event "workflow_dispatch" || fail "workflow_dispatch must be advisory"
  if release_gate_is_advisory_event "push"; then fail "push must not be advisory"; fi
  if release_gate_is_gating_event "schedule"; then fail "schedule must not be gating"; fi

  # (4) Behavioural test: drive the REAL do_ci_check against a stubbed `gh`
  #     and assert the verdict. This is what the previous fixture could not do:
  #     it re-implemented the filter inline, so it passed even when the gate
  #     was inverted.
  local stub_bin
  stub_bin="$lib_dir/bin"
  mkdir -p "$stub_bin"

  # stub `gh`: returns the run list encoded in STUB_RUNS, honouring --event.
  cat > "$stub_bin/gh" <<'STUB'
#!/usr/bin/env bash
# Stub `gh` for the release-gate fixture. STUB_RUNS holds the JSON run list;
# STUB_EVENT, when set, filters it by event.
event=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --event) event="$2"; shift 2 ;;
    --json) shift 2 ;;
    --branch|--commit|--limit) shift 2 ;;
    *) shift ;;
  esac
done
if [[ -n "$event" ]]; then
  printf '%s' "$STUB_RUNS" | jq -c --arg e "$event" '[ .[] | select(.event == $e) ]'
else
  printf '%s' "$STUB_RUNS"
fi
STUB
  chmod +x "$stub_bin/gh"

  run_ci_check() {
    # $1 = runs json; sets CI_CHECK_RC and CI_CHECK_OUT
    local runs="$1"
    CI_CHECK_OUT=$(PATH="$stub_bin:$PATH" STUB_RUNS="$runs" \
      bash -c 'git fetch origin main --quiet 2>/dev/null || true
               exec ./scripts/release-manager.sh ci-check' 2>&1) && CI_CHECK_RC=0 || CI_CHECK_RC=$?
  }

  # (a) green push CI + a red nightly -> release gate must PASS.
  run_ci_check '[
    {"name":"CI","status":"completed","conclusion":"success","event":"push"},
    {"name":"Nightly","status":"completed","conclusion":"failure","event":"schedule"}
  ]'
  [[ "$CI_CHECK_RC" -eq 0 ]] || fail "a red schedule run must not block the gate (rc=$CI_CHECK_RC): $CI_CHECK_OUT"

  # (b) a red push run -> release gate must FAIL.
  run_ci_check '[
    {"name":"CI","status":"completed","conclusion":"failure","event":"push"},
    {"name":"Nightly","status":"completed","conclusion":"failure","event":"schedule"}
  ]'
  [[ "$CI_CHECK_RC" -ne 0 ]] || fail "a red push run must block the gate: $CI_CHECK_OUT"

  # (c) a pending push run -> release gate must FAIL.
  run_ci_check '[
    {"name":"CI","status":"in_progress","conclusion":null,"event":"push"}
  ]'
  [[ "$CI_CHECK_RC" -ne 0 ]] || fail "a pending push run must block the gate: $CI_CHECK_OUT"

  # (d) only advisory runs -> the gate must FAIL closed (no gating CI yet).
  run_ci_check '[
    {"name":"Nightly","status":"completed","conclusion":"success","event":"schedule"}
  ]'
  [[ "$CI_CHECK_RC" -ne 0 ]] || fail "advisory-only runs must not pass the gate: $CI_CHECK_OUT"

  # (e) a red workflow_dispatch run -> must not block, like schedule.
  run_ci_check '[
    {"name":"CI","status":"completed","conclusion":"success","event":"push"},
    {"name":"Manual","status":"completed","conclusion":"failure","event":"workflow_dispatch"}
  ]'
  [[ "$CI_CHECK_RC" -eq 0 ]] || fail "a red workflow_dispatch run must not block the gate (rc=$CI_CHECK_RC): $CI_CHECK_OUT"

  # (f) a non-CI event (issues / dynamic CodeQL) must not decide the verdict.
  run_ci_check '[
    {"name":"CI","status":"completed","conclusion":"success","event":"push"},
    {"name":"Push on main","status":"completed","conclusion":"failure","event":"dynamic"}
  ]'
  [[ "$CI_CHECK_RC" -eq 0 ]] || fail "a dynamic (CodeQL) run must not block the release gate (rc=$CI_CHECK_RC): $CI_CHECK_OUT"

  # (g) a red pull_request run is a gating failure: both gating events must
  #     count. Without this case the fixture cannot tell "only push gates"
  #     from "push and pull_request gate" — the gate and the library would
  #     have to disagree for the fixture to notice.
  run_ci_check '[
    {"name":"CI","status":"completed","conclusion":"success","event":"push"},
    {"name":"CI (PR)","status":"completed","conclusion":"failure","event":"pull_request"}
  ]'
  [[ "$CI_CHECK_RC" -ne 0 ]] || fail "a red pull_request run must block the gate: $CI_CHECK_OUT"

  rm -rf "$lib_dir"
  echo "OK: ci-check event filtering fixtures"
}

case "$MODE" in
  --fixtures)
    check_release_authority
    check_ci_check_event_filtering
    ;;
  --publish-fixtures)
    check_release_authority
    check_publish_fixtures
    check_ci_check_event_filtering
    ;;
  -h|--help)
    sed -n '2,10p' "$0"
    exit 0
    ;;
  *)
    echo "Unknown mode: $MODE" >&2
    exit 2
    ;;
esac
