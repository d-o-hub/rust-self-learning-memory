#!/usr/bin/env bash
# release-gate-events.sh — classify GitHub Actions workflow runs for release gating.
#
# ADR-083: `release-manager.sh ci-check` must distinguish the runs that prove
# the release commit is green from out-of-band runs that merely happen to sit
# on the same SHA.
#
# Gating   : push, pull_request  — the integration control plane for the commit.
# Advisory : schedule, workflow_dispatch — nightlies, manual dispatch, periodic
#            jobs. They re-attach to whatever is HEAD, so a red one must not
#            hold a release whose own CI is green (LESSON-051, ADR-083).
#
# Everything else (issues, dynamic, release, …) is *neither* advisory nor a
# per-commit integration gate: `dynamic` runs are CodeQL's own analysis of the
# repository, and `issues` runs react to issue events. Both are checked by
# their own workflows; they are not evidence about this commit's CI, so they
# must not be counted as a failure of the release commit.
#
# The classification lives here — a sourceable library, not an inline jq
# expression — so `scripts/test-release-workflow.sh` can exercise the *same*
# code the gate uses instead of re-implementing it (a copy test cannot fail).

# Events that do not gate a release.
RELEASE_GATE_ADVISORY_EVENTS=("schedule" "workflow_dispatch")

# Events whose runs are evidence about the commit's own CI.
RELEASE_GATE_GATING_EVENTS=("push" "pull_request")

# release_gate_is_advisory_event EVENT -> 0 when advisory, 1 otherwise.
release_gate_is_advisory_event() {
  local event="$1" candidate
  for candidate in "${RELEASE_GATE_ADVISORY_EVENTS[@]}"; do
    [[ "$event" == "$candidate" ]] && return 0
  done
  return 1
}

# release_gate_is_gating_event EVENT -> 0 when gating, 1 otherwise.
release_gate_is_gating_event() {
  local event="$1" candidate
  for candidate in "${RELEASE_GATE_GATING_EVENTS[@]}"; do
    [[ "$event" == "$candidate" ]] && return 0
  done
  return 1
}

# release_gate_gating_failures GATING_JSON -> count of completed gating runs
# that are not success/skipped/neutral.
release_gate_gating_failures() {
  printf '%s' "$1" | jq '[ .[] | select(
      .status == "completed" and
      (.conclusion != "success") and
      (.conclusion != "skipped") and
      (.conclusion != "neutral") and
      (.conclusion != null)
    ) ] | length'
}

# release_gate_gating_pending GATING_JSON -> count of gating runs still running.
release_gate_gating_pending() {
  printf '%s' "$1" | jq '[ .[] | select(.status != "completed") ] | length'
}

# release_gate_gating_total GATING_JSON -> number of gating runs.
release_gate_gating_total() {
  printf '%s' "$1" | jq 'length'
}
