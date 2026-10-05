#!/usr/bin/env bash
# test-install-hooks.sh — fixtures for the local sensor hooks.
#
# Guards the defect found on 2026-10-05: .pre-commit-config.yaml declares 12 sensors, but a
# fresh clone installs none of them, and the header's own bootstrap
# (`pre-commit install`) still leaves the commitlint hook inert because that hook lives in the
# commit-msg stage. Net effect: commit-message and format feedback only arrive as a failed CI
# run. This fixture proves plain `pre-commit install` is insufficient and ./scripts/install-hooks.sh
# is sufficient.
#
# Usage:
#   ./scripts/test-install-hooks.sh --fixtures

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---fixtures}"

fail() {
  echo "HARNESS VIOLATION: install-hooks — $1" >&2
  exit 1
}

[[ "$MODE" == "--fixtures" ]] || fail "unknown mode: $MODE (expected --fixtures)"

command -v pre-commit >/dev/null || fail "pre-commit required to run these fixtures"
command -v git >/dev/null || fail "git required"

INSTALLER="$ROOT/scripts/install-hooks.sh"
[[ -f "$INSTALLER" ]] || fail "scripts/install-hooks.sh missing"

# Only language:system hooks, so the fixtures never touch the network to build hook envs.
make_fixture_repo() {
  local dir="$1"
  rm -rf "$dir"
  mkdir -p "$dir/scripts"
  git -C "$dir" init -q
  cat > "$dir/.pre-commit-config.yaml" <<'YAML'
repos:
  - repo: local
    hooks:
      - id: fmt-like
        name: cheap pre-commit stage sensor
        entry: test -n "$(git rev-parse HEAD)"
        language: system
        pass_filenames: false
      - id: commitlint-like
        name: commit message sensor
        entry: 'true'
        language: system
        pass_filenames: false
        stages: [commit-msg]
YAML
  cp "$INSTALLER" "$dir/scripts/install-hooks.sh"
  chmod +x "$dir/scripts/install-hooks.sh"
  printf 'x\n' > "$dir/README.md"
  git -C "$dir" add -A
  git -C "$dir" -c user.email=t@example.com -c user.name=t commit -qm fixture
}

hooks_dir() {
  local dir="$1"
  local hd
  hd="$(git -C "$dir" rev-parse --git-path hooks)"
  case "$hd" in
    /*) printf '%s\n' "$hd" ;;
    *) printf '%s/%s\n' "$dir" "$hd" ;;
  esac
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# F1 — the documented bootstrap is not enough: plain `pre-commit install` leaves commit-msg inert.
NAIVE="$WORK/naive"
make_fixture_repo "$NAIVE"
(cd "$NAIVE" && pre-commit install >/dev/null)
naive_hooks="$(hooks_dir "$NAIVE")"
[[ -x "$naive_hooks/pre-commit" ]] || fail "expected pre-commit install to create the pre-commit hook"
if [[ -e "$naive_hooks/commit-msg" ]]; then
  fail "plain 'pre-commit install' DID create commit-msg, so the premise of this fix is wrong"
fi
echo "OK: plain 'pre-commit install' creates pre-commit but NOT commit-msg (the inert sensor)"

# --verify must flag exactly that absence and fail, so the gap is visible instead of silent.
verify_out="$(cd "$NAIVE" && ./scripts/install-hooks.sh --verify 2>&1 || true)"
printf '%s\n' "$verify_out" | grep -q "INERT: commit-msg" \
  || fail "--verify did not report the missing commit-msg hook: $verify_out"
printf '%s\n' "$verify_out" | grep -q "installed: pre-commit" \
  || fail "--verify mis-reported the installed pre-commit hook: $verify_out"
if (cd "$NAIVE" && ./scripts/install-hooks.sh --verify >/dev/null 2>&1); then
  fail "--verify exited 0 while the commit-msg hook was missing"
fi
echo "OK: --verify reports the inert commit-msg hook and exits non-zero"

# F2 — the fix: install-hooks.sh activates both stages and then verifies clean.
FIXED="$WORK/fixed"
make_fixture_repo "$FIXED"
(cd "$FIXED" && ./scripts/install-hooks.sh >/dev/null)
fixed_hooks="$(hooks_dir "$FIXED")"
[[ -x "$fixed_hooks/pre-commit" ]] || fail "install did not create the pre-commit hook"
[[ -x "$fixed_hooks/commit-msg" ]] || fail "install did not create the commit-msg hook"
(cd "$FIXED" && ./scripts/install-hooks.sh --verify) || fail "--verify still fails after installing"
echo "OK: install-hooks.sh activates both stages and --verify then passes"

# F3 — idempotent re-install must not corrupt or duplicate the hooks.
(cd "$FIXED" && ./scripts/install-hooks.sh >/dev/null)
(cd "$FIXED" && ./scripts/install-hooks.sh --verify >/dev/null)
echo "OK: re-running the installer stays green"

echo "OK: install-hooks fixtures"
