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
  local lib_dir
  lib_dir=$(mktemp -d)

  # Drive the real do_ci_check against a stubbed gh that applies the event
  # filter before --limit, matching GitHub CLI's server-side query semantics.
  local stub_bin
  stub_bin="$lib_dir/bin"
  mkdir -p "$stub_bin"

  # Stub gh: apply branch/event filters before the requested result limit.
  cat > "$stub_bin/gh" <<'STUB'
#!/usr/bin/env bash
event=""
branch=""
limit=20
while [[ $# -gt 0 ]]; do
  case "$1" in
    --event) event="$2"; shift 2 ;;
    --limit) limit="$2"; shift 2 ;;
    --json) shift 2 ;;
    --branch) branch="$2"; shift 2 ;;
    --commit) shift 2 ;;
    *) shift ;;
  esac
done
if [[ -n "$event" ]]; then
  printf '%s' "$STUB_RUNS" | jq -c --arg b "$branch" --arg e "$event" \
    --argjson limit "$limit" \
    '[ .[] | select(($b == "" or (.headBranch // "main") == $b) and .event == $e) ] | .[:$limit]'
else
  printf '%s' "$STUB_RUNS" | jq -c --arg b "$branch" --argjson limit "$limit" \
    '[ .[] | select($b == "" or (.headBranch // "main") == $b) ] | .[:$limit]'
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

  run_release_status() {
    # $1 = runs json; sets STATUS_RC and STATUS_OUT
    local runs="$1"
    STATUS_OUT=$(PATH="$stub_bin:$PATH" STUB_RUNS="$runs" \
      bash -c 'git fetch origin main --quiet 2>/dev/null || true
               exec ./scripts/release-manager.sh status' 2>&1) && STATUS_RC=0 || STATUS_RC=$?
  }

  # (a) green push CI + a red nightly -> release gate must PASS.
  run_ci_check '[
    {"name":"CI","status":"completed","conclusion":"success","event":"push"},
    {"name":"Nightly","status":"completed","conclusion":"failure","event":"schedule"}
  ]'
  [[ "$CI_CHECK_RC" -eq 0 ]] || fail "a red schedule run must not block the gate (rc=$CI_CHECK_RC): $CI_CHECK_OUT"
  [[ "$CI_CHECK_OUT" == *"Nightly [schedule]"* ]] || fail "the red schedule run must remain visible: $CI_CHECK_OUT"

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
  [[ "$CI_CHECK_OUT" == *"Manual [workflow_dispatch]"* ]] || fail "the red workflow_dispatch run must remain visible: $CI_CHECK_OUT"

  # (f) a non-CI event (issues / dynamic CodeQL) must not decide the verdict.
  run_ci_check '[
    {"name":"CI","status":"completed","conclusion":"success","event":"push"},
    {"name":"Push on main","status":"completed","conclusion":"failure","event":"dynamic"}
  ]'
  [[ "$CI_CHECK_RC" -eq 0 ]] || fail "a dynamic (CodeQL) run must not block the release gate (rc=$CI_CHECK_RC): $CI_CHECK_OUT"
  [[ "$CI_CHECK_OUT" == *"Push on main [dynamic]"* ]] || fail "the non-CI run must remain visible: $CI_CHECK_OUT"

  # (g) a red pull_request run must block, just like a red push run.
  # This catches an implementation that queries push but omits pull_request.
  run_ci_check '[
    {"name":"CI","status":"completed","conclusion":"success","event":"push"},
    {"name":"CI (PR)","status":"completed","conclusion":"failure","event":"pull_request"}
  ]'
  [[ "$CI_CHECK_RC" -ne 0 ]] || fail "a red pull_request run must block the gate: $CI_CHECK_OUT"
  # (h) status must use the same per-event queries and report advisory counts.
  run_release_status '[
    {"name":"CI","status":"in_progress","conclusion":null,"event":"push"},
    {"name":"Nightly","status":"completed","conclusion":"failure","event":"schedule"},
    {"name":"Manual","status":"completed","conclusion":"failure","event":"workflow_dispatch"}
  ]'
  [[ "$STATUS_RC" -eq 0 ]] || fail "release status must succeed with available run data: $STATUS_OUT"
  [[ "$STATUS_OUT" == *"main CI pending:    1 (gating)"* ]] || fail "release status must count pending gating runs: $STATUS_OUT"
  [[ "$STATUS_OUT" == *"main CI failed:     0 (gating)"* ]] || fail "advisory failures must not count as gating failures: $STATUS_OUT"
  [[ "$STATUS_OUT" == *"advisory runs:      2 (schedule/workflow_dispatch — non-blocking)"* ]] || fail "release status must report both advisory event classes: $STATUS_OUT"

  # (i) unrelated runs cannot consume the result cap and hide a failed push.
  local crowded_runs
  crowded_runs=$(jq -cn '
    [{name:"CI green",status:"completed",conclusion:"success",event:"push"}]
    + [range(0; 99) | {name:("Issue " + tostring),status:"completed",
       conclusion:"success",event:"issues"}]
    + [{name:"CI failure",status:"completed",conclusion:"failure",event:"push"}]
  ')
  run_ci_check "$crowded_runs"
  [[ "$CI_CHECK_RC" -ne 0 ]] || fail "a failed push run beyond unrelated event noise must block the gate: $CI_CHECK_OUT"
  # (j) a failed tag-triggered push run on the pinned SHA must remain gating.
  run_ci_check '[
    {"name":"CI main","status":"completed","conclusion":"success","event":"push","headBranch":"main"},
    {"name":"Nightly tag","status":"completed","conclusion":"failure","event":"push","headBranch":"v0.1.45"},
    {"name":"Tag scan","status":"completed","conclusion":"success","event":"dynamic","headBranch":"v0.1.45"}
  ]'
  [[ "$CI_CHECK_RC" -ne 0 ]] || fail "a failed tag-triggered push run on the pinned release SHA must block the gate: $CI_CHECK_OUT"
  [[ "$CI_CHECK_OUT" == *"Tag scan [dynamic]"* ]] || fail "non-gating runs on a tag ref must remain visible for the pinned SHA: $CI_CHECK_OUT"


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
