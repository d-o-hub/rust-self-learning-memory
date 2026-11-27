#!/usr/bin/env bash
# test-release-version.sh — hermetic self-test for scripts/lib/release-version.sh
#
# Sensor: `release-version` (do-harness verification set), invoked by
# `scripts/release-manager.sh --self-test`.
#
# Everything runs in a throwaway git repo: no network, no real cargo, no writes
# outside $WORK. A check that cannot fail is decoration, so each case asserts a
# concrete value (bump AND next), not merely "it ran".

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly PROJECT_ROOT

# shellcheck source=scripts/lib/release-version.sh
source "$PROJECT_ROOT/scripts/lib/release-version.sh"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
repo="$WORK/repo"
failures=0

pass() { printf 'ok   %s\n' "$1"; }
fail() {
    printf 'FAIL %s\n' "$1" >&2
    failures=$((failures + 1))
}

commit() { git -C "$repo" commit -q --allow-empty -m "$1"; }
tag() { git -C "$repo" tag "v$1"; }

# release <version> — land the bump and tag it, so the next case measures a
# fresh baseline instead of re-deriving from the same current version.
release() {
    set_workspace_version "$1"
    git -C "$repo" commit -qam "chore(release): v$1"
    tag "$1"
}

set_workspace_version() {
    release_substitute "s/^version *= *\"[^\"]*\"/version = \"$1\"/" "$repo/Cargo.toml"
}

# expect_derive <bump> <next> <description> [published] [published_known]
expect_derive() {
    local want_bump="$1" want_next="$2" description="$3"
    local published="${4:-}" known="${5:-false}"
    release_derive "$repo" HEAD "$published" "$known"
    if [ "$BUMP" = "$want_bump" ] && [ "$NEXT" = "$want_next" ]; then
        pass "$description (bump=$BUMP next=$NEXT)"
    else
        fail "$description: expected bump=$want_bump next=$want_next, got bump=$BUMP next=$NEXT"
    fi
}

# expect_die <message fragment> <description> <command...>
# Runs in a subshell, so the command's exit is observable. "It exited non-zero
# somehow" would also be satisfied by an unrelated crash, hence the fragment.
expect_die() {
    local fragment="$1" description="$2"
    shift 2
    local out
    if out="$("$@" 2>&1)"; then
        fail "$description: expected a failure, got success"
    elif [[ "$out" == *"$fragment"* ]]; then
        pass "$description"
    else
        fail "$description: expected message '$fragment', got: $out"
    fi
}

wrote() { # wrote <file> <fixed string> <description>
    if grep -qF -- "$2" "$repo/$1"; then
        pass "$3"
    else
        fail "$3: '$2' not found in $1"
    fi
}

absent() { # absent <file> <fixed string> <description>
    if grep -qF -- "$2" "$repo/$1"; then
        fail "$3: '$2' still present in $1"
    else
        pass "$3"
    fi
}

# ─── fixtures ───────────────────────────────────────────────────────────────
git init -q -b main "$repo"
git -C "$repo" config user.email selftest@example.invalid
git -C "$repo" config user.name selftest
git -C "$repo" config commit.gpgsign false
mkdir -p "$repo/memory-core" "$repo/memory-mcp" "$repo/plans/ROADMAPS" "$repo/plans/STATUS"

cat >"$repo/Cargo.toml" <<'EOF'
[workspace]
members = ["memory-core", "memory-mcp"]

[workspace.package]
version = "0.3.0"
edition = "2021"
EOF

# One rewritable intra-workspace pin (with a trailing feature list, which the
# rewrite must preserve) and one renamed dependency that must keep its own
# version.
cat >"$repo/memory-mcp/Cargo.toml" <<'EOF'
[package]
name = "do-memory-mcp"
version.workspace = true

[dependencies]
do-memory-core = { path = "../memory-core", version = "0.1.40", features = ["agentfs"] }
libsql = { package = "do-memory-libsql", version = "0.9.30" }
EOF

cat >"$repo/memory-core/Cargo.toml" <<'EOF'
[package]
name = "do-memory-core"
version.workspace = true
EOF

cat >"$repo/AGENTS.md" <<'EOF'
**Workspace version**: `v0.3.0` (development) · **Latest released tag**: `v0.2.0` (`abc12345`, 2026-01-01).
EOF

cat >"$repo/CHANGELOG.md" <<'EOF'
# Changelog

## [Unreleased]

### Added

- Something new.

## [0.2.0] - 2026-01-01

### Added

- Previous release.
EOF

cat >"$repo/plans/ROADMAPS/ROADMAP_ACTIVE.md" <<'EOF'
# Roadmap

**Released Version**: v0.2.0 (`abc12345`, GitHub Release 2026-01-01T00:00:00Z; all four
registered crates verified at `0.2.0` on crates.io)
**Workspace Version**: 0.3.0 (post-release development bump)
**Branch**: main
EOF

cat >"$repo/plans/STATUS/CURRENT.md" <<'EOF'
# Status

**Released Version**: v0.2.0 (`abc12345`, GitHub Release 2026-01-01T00:00:00Z with 37
assets; all four registered crates verified at `0.2.0` on crates.io)
**Workspace Version**: 0.3.0 (post-release development bump)
EOF

git -C "$repo" add -A
git -C "$repo" commit -qm "chore: seed"
tag 0.3.0

# ─── bump derivation ────────────────────────────────────────────────────────
commit "docs: tidy the readme"
commit "test(cli): cover the parser"
commit "refactor(core): drop a helper"
expect_derive none 0.3.0 "docs/test/refactor release nothing"

commit "fix(db): stop dropping the last row"
expect_derive patch 0.3.1 "fix is a patch"
expect_derive patch 0.3.1 "--published lists the basis tag" "v0.3.0" true
expect_derive pending-release 0.3.0 "--published omits the basis tag" "v0.2.0" true
expect_derive pending-release 0.3.0 "--published present but empty blocks the bump" "" true
expect_derive patch 0.3.1 "--published absent means not told" "" false

commit "feat(cli): add a checkpoint column"
expect_derive minor 0.4.0 "feat is a minor"
release 0.4.0

commit "perf(core): cut one allocation"
expect_derive patch 0.4.1 "perf is a patch"
release 0.4.1

commit 'Revert "feat(cli): add a checkpoint column"'
expect_derive patch 0.4.2 "non-conventional subject is a patch"
release 0.4.2

commit "FIX(core): mixed-case type"
expect_derive patch 0.4.3 "commit type is case-insensitive"
release 0.4.3

commit "refactor(core)!: drop the v1 signature"
expect_derive minor 0.5.0 "breaking at 0.x is a minor"
release 0.5.0

commit "fix(core): guard the empty case

BREAKING CHANGE: callers must pass Some(session)"
expect_derive minor 0.6.0 "BREAKING CHANGE footer at 0.x is a minor"
release 0.6.0

commit "feat(core)!: remove the legacy field"
expect_derive minor 0.7.0 "feat + ! at 0.x is still a minor"
release 0.7.0

# Already bumped, not yet tagged → pending, not a fresh bump.
set_workspace_version 0.8.0
git -C "$repo" commit -qam "chore(release): bump to 0.8.0"
expect_derive pending 0.8.0 "untagged workspace version derives pending"

# 0.0.x stays on patches, and is checked before breaking.
set_workspace_version 0.0.4
git -C "$repo" commit -qam "chore: reset to 0.0.4 for the rule check"
if [ "$(release_decide_bump 0.0.4)" = "patch" ]; then
    pass "0.0.x stays on patch even with breaking set"
else
    fail "0.0.x must stay on patch"
fi
BREAKING=true FEATURE=true
if [ "$(release_decide_bump 0.0.4)" = "patch" ]; then
    pass "0.0.x patch rule wins over breaking (BREAKING/FEATURE set)"
else
    fail "0.0.x patch rule must win over breaking"
fi
BREAKING=false FEATURE=false
set_workspace_version 1.2.3
git -C "$repo" commit -qam "chore: reset to 1.2.3"
tag 1.2.3
commit "feat(core): a new API"
expect_derive minor 1.3.0 "feat at >=1.0 is a minor"
release 1.3.0
commit "feat(core)!: replace the API"
expect_derive major 2.0.0 "breaking at >=1.0 is a major"

# ─── fail-closed guards ─────────────────────────────────────────────────────
set_workspace_version 1.0.0
git -C "$repo" commit -qam "chore: go behind the newest tag"
expect_die "behind the newest release tag" "a workspace behind the newest tag refuses" \
    release_derive "$repo" HEAD "" false

set_workspace_version 1.2.3-beta
git -C "$repo" commit -qam "chore: malformed version"
expect_die "is not a MAJOR.MINOR.PATCH version" "malformed manifest version refuses" \
    release_derive "$repo" HEAD "" false
set_workspace_version 2.0.0
git -C "$repo" commit -qam "chore: restore"

expect_die "cannot resolve ref" "unresolvable ref refuses" \
    release_derive "$repo" no-such-ref "" false
expect_die "cannot count commits in range" "bogus range refuses instead of deriving none" \
    release_classify "$repo" "v1.0.0..no-such-ref"

if release_version_lt 0.9.0 0.10.0 && ! release_version_lt 0.10.0 0.9.0 \
    && ! release_version_lt 1.2.3 1.2.3; then
    pass "version_lt compares components, not bytes"
else
    fail "version_lt must compare components"
fi

# ─── prepare: every managed file ────────────────────────────────────────────
release_prepare_version "$repo" 2.1.0 false >/dev/null
wrote Cargo.toml 'version = "2.1.0"' "prepare rewrites [workspace.package].version"
wrote memory-mcp/Cargo.toml 'do-memory-core = { path = "../memory-core", version = "2.1.0", features = ["agentfs"] }' \
    "prepare rewrites the intra-workspace pin and keeps its features"
wrote memory-mcp/Cargo.toml 'libsql = { package = "do-memory-libsql", version = "0.9.30" }' \
    "prepare leaves a renamed dependency's own version alone"
absent memory-mcp/Cargo.toml 'version = "0.1.40"' "the stale pin is gone"
wrote AGENTS.md "**Workspace version**: \`v2.1.0\`" "prepare rewrites the AGENTS.md version"
wrote CHANGELOG.md '## [2.1.0] - ' "prepare retitles Unreleased to the released version"
if [ "$(grep -cE '^## \[Unreleased\]' "$repo/CHANGELOG.md")" = "1" ]; then
    pass "exactly one Unreleased heading survives"
else
    fail "prepare must leave exactly one Unreleased heading"
fi
wrote plans/ROADMAPS/ROADMAP_ACTIVE.md '**Released Version**: v2.1.0 (release preparation' \
    "prepare marks the roadmap release-preparation"
absent plans/ROADMAPS/ROADMAP_ACTIVE.md 'assets; all four' "the old provenance text is gone (roadmap)"
wrote plans/STATUS/CURRENT.md '**Released Version**: v2.1.0 (release preparation' \
    "prepare marks the status release-preparation"
absent plans/STATUS/CURRENT.md 'assets; all four' "the old provenance text is gone (status)"

if out="$(release_prepare_version "$repo" 2.1.0 false)"; then
    if [[ "$out" == *"already at 2.1.0"* ]]; then
        pass "prepare is idempotent at the same version"
    else
        fail "prepare idempotency message: $out"
    fi
else
    fail "prepare at the same version must not fail"
fi

expect_die "does not supersede" "prepare refuses a lower version" \
    release_prepare_version "$repo" 2.0.0 false
for bad in 1.2 1.2.3.4 1.2.x 1.2.3-rc1 1.2.3+build 01.2.3 1.2.08; do
    expect_die "is not a MAJOR.MINOR.PATCH version" "prepare refuses '$bad'" \
        release_prepare_version "$repo" "$bad" false
done
expect_die "prepare needs a version" "prepare refuses an empty version" \
    release_prepare_version "$repo" "" false

# The changelog gate the release relies on must accept what prepare wrote.
if [ -x "$PROJECT_ROOT/scripts/check-changelog-headings.sh" ]; then
    if (cd "$repo" && "$PROJECT_ROOT/scripts/check-changelog-headings.sh" CHANGELOG.md) >/dev/null 2>&1; then
        pass "the rewritten CHANGELOG passes check-changelog-headings.sh"
    else
        fail "the rewritten CHANGELOG fails check-changelog-headings.sh"
    fi
fi

# ─── output contract ────────────────────────────────────────────────────────
release_derive "$repo" HEAD "" false
plain="$(release_print_plain)"
json="$(release_print_json)"
for key in current basis commits release_worthy breaking feature fix nonconventional bump next; do
    [[ "$plain" == *"$key="* ]] || fail "plain output is missing '$key'"
    [[ "$json" == *"\"$key\":"* ]] || fail "json output is missing '$key'"
done
[ "$failures" -eq 0 ] && pass "plain and json expose the same keys"

if [ "$failures" -ne 0 ]; then
    printf 'test-release-version: %s failure(s)\n' "$failures" >&2
    exit 1
fi
printf 'test-release-version: all checks passed\n'
