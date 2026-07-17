#!/usr/bin/env bash
# Regression tests for check-release-drift.sh.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

new_repo() {
  local name=$1
  local version=$2
  local repo="$TMP_DIR/$name"
  mkdir -p "$repo"
  git -C "$repo" init -q -b main
  git -C "$repo" config user.email "release-drift-test@example.com"
  git -C "$repo" config user.name "Release Drift Test"
  printf '[workspace.package]\nversion = "%s"\n' "$version" > "$repo/Cargo.toml"
  git -C "$repo" add Cargo.toml
  GIT_AUTHOR_DATE='2026-07-01T00:00:00Z' GIT_COMMITTER_DATE='2026-07-01T00:00:00Z' \
    git -C "$repo" commit -q -m 'chore: initial release'
  git -C "$repo" tag v0.1.34
  echo "$repo"
}

set_version() {
  local repo=$1
  local version=$2
  printf '[workspace.package]\nversion = "%s"\n' "$version" > "$repo/Cargo.toml"
}

add_commits() {
  local repo=$1
  local count=$2
  local prefix=${3:-chore}
  local i
  for ((i = 1; i <= count; i++)); do
    printf '%s\n' "$i" >> "$repo/changes"
    git -C "$repo" add changes Cargo.toml
    git -C "$repo" commit -q -m "$prefix: change $i"
  done
}

assert_output() {
  local output=$1
  local expected=$2
  if ! grep -qx "$expected" <<< "$output"; then
    echo "Expected '$expected' in output:" >&2
    echo "$output" >&2
    exit 1
  fi
}

repo=$(new_repo clean 0.1.34)
output=$(RELEASE_DRIFT_NOW_EPOCH=1782864000 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=clean'
assert_output "$output" 'reason=within_cadence'

repo=$(new_repo no-tag 0.1.34)
git -C "$repo" tag -d v0.1.34 >/dev/null
output=$(RELEASE_DRIFT_NOW_EPOCH=1782864000 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=critical'
assert_output "$output" 'reason=no_release_tag'

repo=$(new_repo age-warning 0.1.34)
set_version "$repo" 0.1.35
add_commits "$repo" 1
output=$(RELEASE_DRIFT_NOW_EPOCH=1783728000 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=warning'
assert_output "$output" 'reason=age_warning'
assert_output "$output" 'release_age_days=10'

repo=$(new_repo warning 0.1.34)
set_version "$repo" 0.1.35
add_commits "$repo" 20 feat
output=$(RELEASE_DRIFT_NOW_EPOCH=1783296000 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=warning'
assert_output "$output" 'reason=commit_warning'
assert_output "$output" 'total=20'

repo=$(new_repo commit-limit 0.1.34)
set_version "$repo" 0.1.35
add_commits "$repo" 30 fix
output=$(RELEASE_DRIFT_NOW_EPOCH=1783296000 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=critical'
assert_output "$output" 'reason=commit_limit'

repo=$(new_repo age-limit 0.1.34)
set_version "$repo" 0.1.35
add_commits "$repo" 1
output=$(RELEASE_DRIFT_NOW_EPOCH=1784073600 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=critical'
assert_output "$output" 'reason=age_limit'
assert_output "$output" 'release_age_days=14'

repo=$(new_repo stale-version 0.1.34)
add_commits "$repo" 1 fix
output=$(RELEASE_DRIFT_NOW_EPOCH=1783296000 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=critical'
assert_output "$output" 'reason=version_not_advanced'

repo=$(new_repo skipped-version 0.1.34)
set_version "$repo" 0.1.36
add_commits "$repo" 1 feat
output=$(RELEASE_DRIFT_NOW_EPOCH=1783296000 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=critical'
assert_output "$output" 'reason=invalid_next_version'

repo=$(new_repo valid-minor 0.1.34)
set_version "$repo" 0.2.0
add_commits "$repo" 1 feat
output=$(RELEASE_DRIFT_NOW_EPOCH=1783296000 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=clean'
assert_output "$output" 'reason=within_cadence'

repo=$(new_repo divergent-tag 0.1.34)
git -C "$repo" checkout -q -b other-release
set_version "$repo" 0.1.35
add_commits "$repo" 1 fix
git -C "$repo" tag v0.1.35
git -C "$repo" checkout -q main
output=$(RELEASE_DRIFT_NOW_EPOCH=1783296000 "$SCRIPT_DIR/check-release-drift.sh" "$repo")
assert_output "$output" 'severity=critical'
assert_output "$output" 'reason=tag_not_ancestor'

echo 'All release drift tests passed.'
