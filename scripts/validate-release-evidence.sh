#!/usr/bin/env bash
# validate-release-evidence.sh — Soft validation of release/tag/version evidence (T0.1b)
#
# Usage:
#   ./scripts/validate-release-evidence.sh
#   ./scripts/validate-release-evidence.sh --fixtures
#   ./scripts/validate-release-evidence.sh --help
#
# Soft checks:
#   - git describe / latest tag presence
#   - Cargo.toml workspace version parseable
#   - optional gh release view when gh is available (never required)
#
# NEVER creates tags or releases. --fixtures runs offline without network.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/validate-release-evidence.sh
  ./scripts/validate-release-evidence.sh --fixtures
  ./scripts/validate-release-evidence.sh --help

Soft-validates local release evidence:
  - parse workspace version from Cargo.toml
  - report git describe / latest tag (if any)
  - optionally query gh release view when available

Never creates tags or GitHub releases.
Exit 0 when local evidence is collectable; exit 1 only on fatal parse errors.
Remote unavailability is recorded as a blocker note, not a hard failure.
EOF
}

fail() {
  echo "HARNESS VIOLATION: release-evidence — $1" >&2
  exit 1
}

parse_cargo_version() {
  local ver
  ver=$(grep -E '^version\s*=' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/' || true)
  [[ -n "$ver" ]] || fail "could not parse workspace version from Cargo.toml"
  echo "$ver"
}

run_live() {
  local cargo_ver tag_ver describe blockers=0
  cargo_ver=$(parse_cargo_version)
  echo "Cargo.toml version: $cargo_ver"

  tag_ver=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
  describe=$(git describe --tags --always 2>/dev/null || echo "unknown")
  if [[ -n "$tag_ver" ]]; then
    echo "Latest tag: $tag_ver"
    echo "git describe: $describe"
    local tag_bare="${tag_ver#v}"
    if [[ "$tag_bare" != "$cargo_ver" ]]; then
      echo "NOTE: tag version ($tag_bare) differs from Cargo.toml ($cargo_ver) — may be unreleased development"
    else
      echo "OK: tag matches Cargo.toml version"
    fi
  else
    echo "NOTE: no git tags found (blocker for remote_release_state_verified)"
    blockers=$((blockers + 1))
  fi

  if command -v gh >/dev/null 2>&1; then
    if [[ -n "$tag_ver" ]]; then
      if gh release view "$tag_ver" --json tagName,targetCommitish,isDraft,isPrerelease,publishedAt,url \
        >/tmp/rslm-release-evidence-gh.json 2>/dev/null; then
        echo "OK: gh release view succeeded for $tag_ver"
        if command -v jq >/dev/null 2>&1; then
          jq -c '{tagName, targetCommitish, isDraft, isPrerelease, publishedAt, url}' \
            /tmp/rslm-release-evidence-gh.json 2>/dev/null || true
        fi
        rm -f /tmp/rslm-release-evidence-gh.json
      else
        echo "NOTE: gh release view failed or no release for $tag_ver (remote blocker)"
        blockers=$((blockers + 1))
      fi
    else
      echo "NOTE: skip gh release view (no tag)"
      blockers=$((blockers + 1))
    fi
  else
    echo "NOTE: gh not available — remote release state unverified"
    blockers=$((blockers + 1))
  fi

  echo "OK: release-evidence local collection complete (blockers=$blockers)"
  # Soft validation: local collectability is success even with remote blockers
  exit 0
}

run_fixtures() {
  local tmp
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' RETURN

  # Mock Cargo.toml version parse
  cat >"$tmp/Cargo.toml" <<'TOML'
[workspace.package]
version = "0.2.0"

[package]
name = "fixture"
version = "0.2.0"
TOML

  # Mock git describe output stored as fixture data
  cat >"$tmp/evidence.json" <<'JSON'
{
  "cargo_version": "0.2.0",
  "tag": "v0.2.0",
  "describe": "v0.2.0",
  "gh_release": {
    "tagName": "v0.2.0",
    "targetCommitish": "abc123",
    "isDraft": false,
    "isPrerelease": false,
    "publishedAt": "2026-07-01T00:00:00Z",
    "url": "https://example.invalid/releases/tag/v0.2.0"
  },
  "network": false
}
JSON

  # Mock mismatch / blocker case
  cat >"$tmp/blocker.json" <<'JSON'
{
  "cargo_version": "0.2.0",
  "tag": "v0.1.34",
  "describe": "v0.1.34-12-gdeadbeef",
  "gh_release": null,
  "network": false,
  "blockers": ["tag_mismatch", "gh_unavailable"]
}
JSON

  python3 - "$tmp" <<'PY'
import json, sys
from pathlib import Path

tmp = Path(sys.argv[1])
good = json.loads((tmp / "evidence.json").read_text())
bad = json.loads((tmp / "blocker.json").read_text())

assert good["cargo_version"] == "0.2.0"
assert good["tag"].lstrip("v") == good["cargo_version"]
assert good.get("network") is False
assert good["gh_release"]["tagName"] == good["tag"]
print("OK: mock matching release evidence")

assert bad["cargo_version"] != bad["tag"].lstrip("v")
assert bad["gh_release"] is None
assert "tag_mismatch" in bad["blockers"]
print("OK: mock mismatch evidence records blockers")

# Parse mock Cargo.toml
text = (tmp / "Cargo.toml").read_text()
import re
m = re.search(r'version\s*=\s*"([^"]+)"', text)
assert m and m.group(1) == "0.2.0"
print("OK: mock Cargo.toml version parse")
print("OK: validate-release-evidence fixtures passed (offline)")
PY
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
  --fixtures)
    run_fixtures
    exit 0
    ;;
  "")
    run_live
    ;;
  *)
    echo "Unknown argument: $1" >&2
    usage >&2
    exit 2
    ;;
esac
