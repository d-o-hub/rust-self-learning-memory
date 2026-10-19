#!/usr/bin/env bash
# Regression tests for update-changelog.sh.
#
# Guards the #1188 failure mode: the generator prepending a second `## [X.Y.Z]`
# section for a version the release-preparation PR already wrote, which makes
# parse-changelog reject CHANGELOG.md and ships a release with no notes.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

failures=0

pass() { echo "ok - $1"; }
fail() {
  echo "not ok - $1" >&2
  [[ -z "${2:-}" ]] || echo "    $2" >&2
  failures=$((failures + 1))
}

# A toolchain with git + coreutils but deliberately WITHOUT git-cliff and
# parse-changelog, so generator-absence and parser-absence paths are
# deterministic on every machine.
MINIMAL_BIN="$TMP_DIR/bin"
mkdir -p "$MINIMAL_BIN"
for tool in awk bash cat chmod cp dirname git grep head ln mkdir mktemp printf sed sha256sum sort uniq wc; do
  bin="$(command -v "$tool" || true)"
  [[ -n "$bin" ]] && ln -s "$bin" "$MINIMAL_BIN/$tool"
done
readonly MINIMAL_BIN

new_repo() {
  local name=$1
  local repo="$TMP_DIR/$name"
  mkdir -p "$repo/scripts"
  git -C "$repo" init -q -b main
  git -C "$repo" config user.email "changelog-test@example.com"
  git -C "$repo" config user.name "Changelog Test"
  cp "$SCRIPT_DIR/update-changelog.sh" "$repo/scripts/update-changelog.sh"
  chmod +x "$repo/scripts/update-changelog.sh"
  printf '# Changelog\n\n(tag_pattern guard)\n' > "$repo/cliff.toml"
  echo "$repo"
}

write_changelog() {
  local repo=$1
  shift
  printf '%s\n' "$@" > "$repo/CHANGELOG.md"
}

run_generator() {
  local repo=$1
  shift
  env -i PATH="$MINIMAL_BIN" HOME="$HOME" \
    bash "$repo/scripts/update-changelog.sh" "$@" 2>&1
}

assert_rc() {
  local label=$1 expected=$2 actual=$3
  if [[ "$actual" -eq "$expected" ]]; then
    pass "$label (rc=$actual)"
  else
    fail "$label" "expected rc=$expected, got rc=$actual; output: $(printf '%s' "$LAST_OUTPUT" | head -3 | tr '\n' ' ')"
  fi
}

assert_match() {
  local label=$1 pattern=$2 output=$3
  if printf '%s' "$output" | grep -qE "$pattern"; then
    pass "$label"
  else
    fail "$label" "no match for $pattern in: $(printf '%s' "$output" | head -5 | tr '\n' ' ')"
  fi
}

run_and_capture() {
  local repo=$1
  shift
  set +e
  local out
  out="$(run_generator "$repo" "$@")"
  local rc=$?
  set -e
  LAST_OUTPUT="$out"
  LAST_RC="$rc"
}

# 1. The #1188 payload: two sections for one version must be rejected.
repo=$(new_repo duplicate-heading)
write_changelog "$repo" \
  '# Changelog' '' \
  '## [Unreleased]' '' \
  '## [0.1.46] - 2026-10-18' '' \
  '### Fixed' '' \
  '- second render of the same release' '' \
  '## [0.1.46] - 2026-10-17' '' \
  '### Fixed' '' \
  '- curated release-preparation section'
git -C "$repo" add -A && git -C "$repo" commit -q -m 'chore: duplicate changelog'
git -C "$repo" tag v0.1.46
run_and_capture "$repo" --tag v0.1.46
assert_rc "duplicate headings rejected" 1 "$LAST_RC"
assert_match "names the duplicated version" '## \[0\.1\.46\]' "$LAST_OUTPUT"

# 2. Already-documented version: skip, and do not touch the file.
repo=$(new_repo idempotent-skip)
write_changelog "$repo" \
  '# Changelog' '' \
  '## [Unreleased]' '' \
  '## [0.1.46] - 2026-10-17' '' \
  '### Fixed' '' \
  '- curated release-preparation section'
git -C "$repo" add -A && git -C "$repo" commit -q -m 'chore: prepared changelog'
git -C "$repo" tag v0.1.46
before=$(sha256sum "$repo/CHANGELOG.md" | cut -d' ' -f1)
run_and_capture "$repo" --tag v0.1.46
assert_rc "existing section skips generation" 0 "$LAST_RC"
assert_match "reports the skip" '^skip: ' "$LAST_OUTPUT"
after=$(sha256sum "$repo/CHANGELOG.md" | cut -d' ' -f1)
if [[ "$before" == "$after" ]]; then
  pass "CHANGELOG.md byte-identical after skip"
else
  fail "CHANGELOG.md byte-identical after skip" "hash changed"
fi

# 3. A heading with no content would render blank release notes.
repo=$(new_repo empty-section)
write_changelog "$repo" '# Changelog' '' '## [0.1.46] - 2026-10-17' ''
git -C "$repo" add -A && git -C "$repo" commit -q -m 'chore: empty section'
git -C "$repo" tag v0.1.46
run_and_capture "$repo" --tag v0.1.46
assert_rc "empty section rejected" 1 "$LAST_RC"
assert_match "explains the empty section" 'is empty' "$LAST_OUTPUT"

# 4. No generator binary and no section to write: must fail, never silently succeed.
repo=$(new_repo missing-generator)
write_changelog "$repo" '# Changelog' '' '## [0.1.45] - 2026-10-14' '' '### Fixed' '' '- older release'
git -C "$repo" add -A && git -C "$repo" commit -q -m 'chore: previous release only'
git -C "$repo" tag v0.1.45
git -C "$repo" tag v0.1.46
run_and_capture "$repo" --tag v0.1.46
assert_rc "missing git-cliff refuses to no-op green" 1 "$LAST_RC"
assert_match "names the missing tool" 'git-cliff is not installed' "$LAST_OUTPUT"

# 5. Usage guards: a non-release ref must never become a version section, and
#    default tag resolution must ignore non-release refs entirely.
repo=$(new_repo bad-tag)
write_changelog "$repo" '# Changelog' '' '## [0.1.46] - 2026-10-17' '' '### Fixed' '' '- section'
git -C "$repo" add -A && git -C "$repo" commit -q -m 'chore: valid changelog'
git -C "$repo" tag -a 'backup/pr-1171-pre-squash' -m 'newer than the release tag by creatordate'
git -C "$repo" tag v0.1.46
run_and_capture "$repo" --tag backup/pr-1171-pre-squash
assert_rc "non-release --tag rejected" 2 "$LAST_RC"
assert_match "rejects the bad tag shape" 'not a release tag' "$LAST_OUTPUT"

run_and_capture "$repo" --check-only
assert_rc "--check-only resolves the release tag, not backup/*" 0 "$LAST_RC"
assert_match "reports parseability" 'exactly one non-empty section' "$LAST_OUTPUT"

echo ""
if [[ "$failures" -gt 0 ]]; then
  echo "FAILED: $failures update-changelog test(s)" >&2
  exit 1
fi
echo 'All update-changelog tests passed.'
