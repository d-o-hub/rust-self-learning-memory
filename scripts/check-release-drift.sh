#!/usr/bin/env bash
# Calculate release cadence and version-state drift for CI.

set -euo pipefail

readonly PROJECT_ROOT="${1:-$(pwd)}"
readonly WARNING_COMMITS=20
readonly MAX_COMMITS=30
readonly WARNING_AGE_DAYS=10
readonly MAX_AGE_DAYS=14

cd "$PROJECT_ROOT"

workspace_version=$(sed -nE 's/^version[[:space:]]*=[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/p' Cargo.toml | head -1)
latest_tag=$(git tag --list 'v[0-9]*.[0-9]*.[0-9]*' --sort=-v:refname \
  | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | head -1 || true)

if [[ -z "$workspace_version" ]]; then
  echo "Unable to read the workspace version from Cargo.toml" >&2
  exit 2
fi

if [[ -z "$latest_tag" ]]; then
  cat <<EOF
workspace_version=$workspace_version
latest_tag=none
latest_version=none
total=0
feats=0
fixes=0
release_age_days=0
severity=critical
reason=no_release_tag
EOF
  exit 0
fi

latest_version=${latest_tag#v}
total=$(git rev-list --count "${latest_tag}..HEAD")
feats=$(git log --format=%s "${latest_tag}..HEAD" | grep -cE '^feat([(:]|$)' || true)
fixes=$(git log --format=%s "${latest_tag}..HEAD" | grep -cE '^fix([(:]|$)' || true)
tag_is_ancestor=true
if ! git merge-base --is-ancestor "$latest_tag" HEAD; then
  tag_is_ancestor=false
fi
tag_epoch=$(git log -1 --format=%ct "$latest_tag")
now_epoch=${RELEASE_DRIFT_NOW_EPOCH:-$(date +%s)}
release_age_days=$(( (now_epoch - tag_epoch) / 86400 ))
if (( release_age_days < 0 )); then
  release_age_days=0
fi

is_valid_next_version() {
  local current_major current_minor current_patch
  local next_major next_minor next_patch
  IFS=. read -r current_major current_minor current_patch <<< "$latest_version"
  IFS=. read -r next_major next_minor next_patch <<< "$workspace_version"

  (( next_major == current_major && next_minor == current_minor && next_patch == current_patch + 1 )) \
    || (( next_major == current_major && next_minor == current_minor + 1 && next_patch == 0 )) \
    || (( next_major == current_major + 1 && next_minor == 0 && next_patch == 0 ))
}

severity=clean
reason=within_cadence

if [[ "$tag_is_ancestor" == "false" ]]; then
  severity=critical
  reason=tag_not_ancestor
elif [[ "$workspace_version" == "$latest_version" ]]; then
  if (( total > 0 )); then
    severity=critical
    reason=version_not_advanced
  fi
elif ! is_valid_next_version; then
  severity=critical
  reason=invalid_next_version
elif (( total >= MAX_COMMITS )); then
  severity=critical
  reason=commit_limit
elif (( release_age_days >= MAX_AGE_DAYS )); then
  severity=critical
  reason=age_limit
elif (( total >= WARNING_COMMITS )); then
  severity=warning
  reason=commit_warning
elif (( release_age_days >= WARNING_AGE_DAYS )); then
  severity=warning
  reason=age_warning
fi

cat <<EOF
workspace_version=$workspace_version
latest_tag=$latest_tag
latest_version=$latest_version
total=$total
feats=$feats
fixes=$fixes
release_age_days=$release_age_days
severity=$severity
reason=$reason
EOF
