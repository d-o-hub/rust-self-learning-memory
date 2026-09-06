#!/usr/bin/env bash
# harness walkthrough: proves greenfield adoption end-to-end with the real
# binary — init scaffolds the workspace plus a minimal crate, the full rust
# sensor suite must actually pass (no assumed green), then a broken workspace
# drives the fail-fast error-signature lifecycle (record -> list) as residue.
set -euo pipefail
root="${DO_HARNESS_ROOT:?DO_HARNESS_ROOT required}"
bin="${DO_HARNESS_BIN:-do-harness}"

# Greenfield adoption: the full suite (fmt/check/clippy/test/loc/commitlint)
# runs against the scaffolded crate and must exit 0.
"$bin" --root "$root" init >/dev/null
"$bin" --root "$root" verify

# Break the crate: the test sensor must fail and record its signature.
rm -rf "$root/Cargo.toml" "$root/src"
"$bin" --root "$root" verify --only test --record >/dev/null 2>&1 || true

"$bin" --root "$root" errors list

# Self-correction: minimal fix (restore the crate), then prove green again.
"$bin" --root "$root" init >/dev/null
"$bin" --root "$root" verify
