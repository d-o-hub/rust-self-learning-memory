#!/usr/bin/env bash
# release-version.sh — release-version derivation and preparation primitives.
#
# Sourceable library (no side effects at load time) shared by
# scripts/release-manager.sh (`next`, `prepare`) and
# scripts/test-release-version.sh (the hermetic self-test), so the bump policy
# and the manifest rewrites have one implementation: a copy of the rules in a
# test cannot fail when the rules change.
#
# Ported from the do-harness release manager, which derives the next version
# from the conventional commits since the last release tag instead of trusting
# whoever remembers to bump the workspace by hand (LESSON-057: a workspace
# version ahead of the latest tag is a development bump, not a release).
#
# Differences from the do-harness implementation, all deliberate:
#   * no npm manifests and no generated man page — this repo ships neither, so
#     `--no-lock` is the only skip flag;
#   * the derived version is only *proposed* here; tagging stays behind
#     scripts/release-manager.sh ship and its blocker gate;
#   * this repo's tracked release docs (ROADMAP_ACTIVE.md, STATUS/CURRENT.md)
#     are rewritten to a release-preparation marker, because their provenance
#     parenthetical describes a publication that has not happened yet.
#
# The rules are order-sensitive; see docs/releasing.md for the policy table.

# C locale keeps `[0-9]` byte-oriented in regex matches (a UTF-8 locale also
# accepts ١ and １, which Cargo rejects) and makes glob/sort deterministic.
export LC_ALL=C

readonly RE_VERSION='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'
readonly RE_BREAKING='^[a-zA-Z]+(\([^)]*\))?!:'
readonly RE_FEAT='^feat(\([^)]*\))?:'
readonly RE_FIX='^(fix|perf)(\([^)]*\))?:'
readonly RE_CONVENTIONAL='^[a-zA-Z]+(\([^)]*\))?!?:'

release_die() {
    printf 'release-manager: %s\n' "$1" >&2
    exit 1
}

release_require_version() {
    local version="$1" origin="$2"
    if [[ ! "$version" =~ $RE_VERSION ]]; then
        release_die "'$version' is not a MAJOR.MINOR.PATCH version (from $origin)"
    fi
}

# release_version_lt <a> <b> — true when a sorts before b by component order.
# Hand-written because `sort -V` is unavailable on macOS.
release_version_lt() {
    local a="$1" b="$2" i
    local -a ap=() bp=()
    IFS=. read -r -a ap <<<"$a"
    IFS=. read -r -a bp <<<"$b"
    for i in 0 1 2; do
        if [ "${ap[i]:-0}" -lt "${bp[i]:-0}" ]; then return 0; fi
        if [ "${ap[i]:-0}" -gt "${bp[i]:-0}" ]; then return 1; fi
    done
    return 1
}

# release_workspace_version <root> — the [workspace.package] version, verbatim
# (a malformed value must reach release_require_version, not be silently
# normalized away).
release_workspace_version() {
    local root="$1"
    sed -n '/^\[workspace.package\]/,/^\[/ s/^version *= *"\([^"]*\)".*/\1/p' \
        "$root/Cargo.toml" | head -1
}

# release_latest_tag <root> <ref> — newest release tag reachable from ref.
# Reachability and the strict shape filter both matter: `--sort=-v:refname`
# alone would let a stray `v1.0.0-notes` tag win, and an unreachable tag would
# derive a bump from a range that is not an ancestor of the commit being shipped.
release_latest_tag() {
    local root="$1" ref="$2"
    git -C "$root" tag --merged "$ref" --list 'v[0-9]*' --sort=-v:refname 2>/dev/null \
        | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | head -1 || true
}

# release_classify <root> <range> — sets COMMITS, RELEASE_WORTHY, BREAKING,
# FEATURE, FIX, NONCONVENTIONAL from the commits in range.
release_classify() {
    local root="$1" range="$2" records record subject body line lowered worthy

    COMMITS=0
    if ! COMMITS="$(git -C "$root" rev-list --count "$range" 2>/dev/null)"; then
        release_die "cannot count commits in range '$range' in $root"
    fi

    if ! records="$(git -C "$root" log --no-merges \
        --format='%s%x1f%b%x1e' "$range" 2>/dev/null)"; then
        release_die "cannot read commit subjects and bodies in range '$range' in $root"
    fi

    RELEASE_WORTHY=0
    BREAKING=false
    FEATURE=false
    FIX=false
    NONCONVENTIONAL=false

    while IFS= read -r -d $'\x1e' record; do
        [ -n "$record" ] || continue
        record="${record#$'\n'}"
        subject="${record%%$'\x1f'*}"
        body="${record#*$'\x1f'}"
        lowered="$(printf '%s' "$subject" | tr '[:upper:]' '[:lower:]')"
        worthy=false

        if [[ "$lowered" =~ $RE_BREAKING ]]; then
            BREAKING=true
            worthy=true
        elif [[ "$lowered" =~ $RE_FEAT ]]; then
            FEATURE=true
            worthy=true
        elif [[ "$lowered" =~ $RE_FIX ]]; then
            FIX=true
            worthy=true
        fi

        # Footer tokens are case-sensitive by spec, so they are not lowercased.
        while IFS= read -r line; do
            case "$line" in
                'BREAKING CHANGE:'* | 'BREAKING-CHANGE:'*)
                    BREAKING=true
                    worthy=true
                    ;;
            esac
        done <<<"$body"

        if [[ ! "$lowered" =~ $RE_CONVENTIONAL ]]; then
            NONCONVENTIONAL=true
            worthy=true
        fi

        if [ "$worthy" = true ]; then
            RELEASE_WORTHY=$((RELEASE_WORTHY + 1))
        fi
    done <<<"$records"
}

# release_decide_bump <current> — prints major|minor|patch|none.
# First match wins; `0.0.x` is checked before breaking so the step to 0.1.0
# stays a decision rather than a side effect.
release_decide_bump() {
    local current="$1" major minor
    major="${current%%.*}"
    minor="${current#*.}"
    minor="${minor%%.*}"

    if [ "$major" -eq 0 ] && [ "$minor" -eq 0 ]; then
        printf 'patch'
        return 0
    fi
    if [ "$BREAKING" = true ]; then
        if [ "$major" -ge 1 ]; then printf 'major'; else printf 'minor'; fi
        return 0
    fi
    if [ "$FEATURE" = true ]; then
        printf 'minor'
        return 0
    fi
    if [ "$FIX" = true ] || [ "$NONCONVENTIONAL" = true ]; then
        printf 'patch'
        return 0
    fi
    printf 'none'
}

# release_next_version <current> <bump> — non-bump classes leave the version
# unchanged, which is what makes `pending` mean "already bumped, tag it".
release_next_version() {
    local current="$1" bump="$2" major minor patch
    IFS=. read -r major minor patch <<<"$current"
    case "$bump" in
        major) printf '%s.0.0' "$((major + 1))" ;;
        minor) printf '%s.%s.0' "$major" "$((minor + 1))" ;;
        patch) printf '%s.%s.%s' "$major" "$minor" "$((patch + 1))" ;;
        *) printf '%s' "$current" ;;
    esac
}

release_list_has() {
    local list="$1" item="$2" entry
    local IFS=,
    for entry in $list; do
        [ "$entry" = "$item" ] && return 0
    done
    return 1
}

# release_derive <root> <ref> [published] [published_known]
# Sets CURRENT BASIS COMMITS RELEASE_WORTHY BREAKING FEATURE FIX
# NONCONVENTIONAL BUMP NEXT. Every git read fails closed: a bogus range must
# never fall back to an empty history, because that renders as `bump=none`.
release_derive() {
    local root="$1" ref="$2" published="${3:-}" published_known="${4:-false}"
    local basis_version range

    if ! git -C "$root" rev-parse --verify --quiet "$ref^{commit}" >/dev/null; then
        release_die "cannot resolve ref '$ref' as a commit in $root"
    fi

    CURRENT="$(release_workspace_version "$root")"
    [ -n "$CURRENT" ] || release_die "cannot read [workspace.package].version from $root/Cargo.toml"
    release_require_version "$CURRENT" "[workspace.package].version in $root/Cargo.toml"

    BASIS="$(release_latest_tag "$root" "$ref")"
    if [ -n "$BASIS" ]; then
        range="$BASIS..$ref"
    else
        range="$ref"
    fi

    release_classify "$root" "$range"

    if [ -n "$BASIS" ]; then
        basis_version="${BASIS#v}"
        if release_version_lt "$CURRENT" "$basis_version"; then
            release_die "workspace version $CURRENT is behind the newest release tag $BASIS; apply the bump first"
        fi
    fi

    if git -C "$root" rev-parse -q --verify "refs/tags/v$CURRENT" >/dev/null; then
        BUMP="$(release_decide_bump "$CURRENT")"
    else
        BUMP="pending"
    fi
    NEXT="$(release_next_version "$CURRENT" "$BUMP")"

    # A basis tag with no published Release is intent, not a finished release:
    # refuse to derive a bump on top of it.
    if [ -n "$BASIS" ] && [ "$published_known" = true ] \
        && ! release_list_has "$published" "$BASIS"; then
        case "$BUMP" in
            major | minor | patch)
                BUMP="pending-release"
                NEXT="$CURRENT"
                ;;
        esac
    fi

    release_require_version "$NEXT" "the derived next version (current=$CURRENT bump=$BUMP)"
}

release_print_plain() {
    printf 'current=%s\nbasis=%s\ncommits=%s\nrelease_worthy=%s\nbreaking=%s\nfeature=%s\nfix=%s\nnonconventional=%s\nbump=%s\nnext=%s\n' \
        "$CURRENT" "${BASIS:-none}" "$COMMITS" "$RELEASE_WORTHY" "$BREAKING" \
        "$FEATURE" "$FIX" "$NONCONVENTIONAL" "$BUMP" "$NEXT"
}

release_print_json() {
    printf '{"current":"%s","basis":"%s","commits":%s,"release_worthy":%s,"breaking":%s,"feature":%s,"fix":%s,"nonconventional":%s,"bump":"%s","next":"%s"}\n' \
        "$CURRENT" "${BASIS:-none}" "$COMMITS" "$RELEASE_WORTHY" "$BREAKING" \
        "$FEATURE" "$FIX" "$NONCONVENTIONAL" "$BUMP" "$NEXT"
}

# release_substitute <expression> <file> — atomic in-place rewrite that keeps
# the file mode. Not `sed -i`: GNU and BSD take different arguments, and a
# non-atomic write can truncate a manifest.
release_substitute() {
    local expression="$1" file="$2" tmp mode
    if mode="$(stat -c '%a' "$file" 2>/dev/null)"; then
        :
    else
        mode="$(stat -f '%Lp' "$file")"
    fi
    tmp="$(mktemp "${file}.XXXXXX")"
    sed -e "$expression" "$file" >"$tmp"
    chmod "$mode" "$tmp"
    mv "$tmp" "$file"
}

# release_rewrite_released_block <file> <version> — replace the
# `**Released Version**` line and its wrapped continuation with a truthful
# release-preparation marker. The provenance parenthetical on these lines
# describes a publication, so it cannot be carried across a bump.
release_rewrite_released_block() {
    local file="$1" version="$2" tmp mode
    [ -f "$file" ] || return 0
    if mode="$(stat -c '%a' "$file" 2>/dev/null)"; then
        :
    else
        mode="$(stat -f '%Lp' "$file")"
    fi
    tmp="$(mktemp "${file}.XXXXXX")"
    awk -v replacement="**Released Version**: v${version} (release preparation; tag and publication pending green main CI)" '
        index($0, "**Released Version**:") == 1 {
            print replacement
            skipping = 1
            next
        }
        # Continuation lines of the old value: anything that does not start a
        # new bold field or a blank line.
        skipping && $0 !~ /^\*\*/ && $0 != "" { next }
        { skipping = 0; print }
    ' "$file" >"$tmp"
    chmod "$mode" "$tmp"
    mv "$tmp" "$file"
}

# release_prepare_version <root> <version> <do_lock>
release_prepare_version() {
    local root="$1" version="$2" do_lock="${3:-true}"
    local current file pin_files=0 date

    [ -n "$version" ] || release_die "prepare needs a version (try: release-manager.sh next)"
    release_require_version "$version" "the prepare argument"

    current="$(release_workspace_version "$root")"
    [ -n "$current" ] || release_die "cannot read [workspace.package].version from $root/Cargo.toml"
    release_require_version "$current" "[workspace.package].version in $root/Cargo.toml"

    if [ "$version" = "$current" ]; then
        printf 'workspace is already at %s\n' "$version"
    else
        release_version_lt "$current" "$version" \
            || release_die "'$version' does not supersede the current $current"

        release_substitute \
            "/^\[workspace.package\]/,/^\[/ s/^version *= *\"[^\"]*\"/version = \"$version\"/" \
            "$root/Cargo.toml"


        # Intra-workspace version pins. Cargo reads `version = "0.1.40"` on a
        # path dependency as `^0.1.40`, which excludes the next minor, so a
        # minor/major bump fails `cargo metadata --locked` without this. Only
        # `do-memory-*` keys are rewritten, never a renamed dependency.
        while IFS= read -r file; do
            [ -n "$file" ] || continue
            release_substitute \
                "s/^\(do-memory-[a-z0-9-]* *= *{[^}]*version *= *\)\"[^\"]*\"/\1\"$version\"/" \
                "$file"
            pin_files=$((pin_files + 1))
        done < <(grep -rlE '^do-memory-[a-z0-9-]+ *= *\{[^}]*version *= *"' \
            --include=Cargo.toml \
            "$root"/memory-core "$root"/memory-storage-turso "$root"/memory-storage-redb \
            "$root"/memory-mcp "$root"/memory-cli "$root"/test-utils "$root"/benches \
            "$root"/tests "$root"/examples 2>/dev/null || true)

        # AGENTS.md carries the workspace version next to the latest tag.
        if [ -f "$root/AGENTS.md" ]; then
            release_substitute \
                "s/^\*\*Workspace version\*\*: \`v[0-9.]*\`/**Workspace version**: \`v$version\`/" \
                "$root/AGENTS.md"
        fi

        # CHANGELOG: the accumulated Unreleased section becomes this release,
        # and a fresh empty Unreleased sits above it. One heading per version,
        # exactly one Unreleased (scripts/check-changelog-headings.sh).
        date="$(date -u +%F)"
        if [ -f "$root/CHANGELOG.md" ]; then
            release_substitute \
                "s/^## \[Unreleased\]\$/## [Unreleased]\\n\\n## [$version] - $date/" \
                "$root/CHANGELOG.md"
        fi

        # Tracker docs: version token plus a release-preparation marker, since
        # the provenance text they carried describes the previous publication.
        release_rewrite_released_block "$root/plans/ROADMAPS/ROADMAP_ACTIVE.md" "$version"
        release_rewrite_released_block "$root/plans/STATUS/CURRENT.md" "$version"
        for tracker in "$root/plans/ROADMAPS/ROADMAP_ACTIVE.md" "$root/plans/STATUS/CURRENT.md"; do
            [ -f "$tracker" ] || continue
            release_substitute \
                "s/^\*\*Workspace Version\*\*:.*/**Workspace Version**: $version (release preparation)/" \
                "$tracker"
        done

        printf 'applied %s (was %s; %s pin file(s) rewritten)\n' \
            "$version" "$current" "$pin_files"
    fi

    if [ "$do_lock" = true ]; then
        # Regenerate rather than sed: the lock is cargo's artifact. fuzz/ is an
        # excluded workspace with its own lock, and the lockfile-sync sensor
        # fails the release when it is left behind.
        (cd "$root" && cargo metadata --format-version 1 >/dev/null) \
            || release_die "cargo metadata failed in $root"
        if [ -f "$root/fuzz/Cargo.toml" ]; then
            (cd "$root/fuzz" && cargo metadata --format-version 1 >/dev/null) \
                || release_die "cargo metadata failed in $root/fuzz"
        fi
    fi

    return 0
}
