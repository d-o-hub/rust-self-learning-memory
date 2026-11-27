# Releasing

How a version of this workspace is derived, prepared, and shipped. The bump
derivation and manifest preparation are ported from the `do-harness` release
manager; the tag/publish half stays this repo's own, because the two projects
distribute different artifacts (this one publishes crates and a GitHub Release,
`do-harness` also ships binaries and npm packages).

## The model

```text
next        derive the bump from conventional commits since the last tag   (read-only)
  ↓
prepare     apply the version to every managed manifest   → reviewed PR → merge
  ↓
ship        blocker gate + validate + ci-check + tag + push
  ↓
release.yml draft-first release + crates.io publish + attestations
```

The version never depends on someone remembering to bump it (LESSON-057: a
workspace version ahead of the latest tag is a development bump, not a release),
and tagging never happens from a dirty or unverified tree.

## Bump policy

First match wins, applied to the commits in `<last-tag>..HEAD` (`--no-merges`):

| Signal | Bump |
|---|---|
| current version is `0.0.x` | `patch` (checked before breaking) |
| breaking (`type!:`, or a `BREAKING CHANGE:` / `BREAKING-CHANGE:` footer) | `major` at ≥1.0, `minor` at `0.y.z` |
| `feat` | `minor` |
| `fix`, `perf` | `patch` |
| non-conventional subject | `patch` |
| other conventional types (`docs`, `chore`, `test`, `refactor`, `style`, `ci`, `build`) | `none` |

Commit types are case-insensitive; a footer breaking change makes the commit
breaking even when its type is `fix`. Merge commits never classify.

Two non-bump results exist because a version can be present without being
released:

| `bump` | Meaning | Action |
|---|---|---|
| `pending` | the workspace version has no matching tag — it is already bumped | tag it with `ship` |
| `pending-release` | the basis tag has no published GitHub Release, so nothing may be derived on top of it | publish the held release first |
| `none` | nothing release-worthy in range | nothing to do |

`pending-release` is decided by `--published`, an explicit input because the
answer needs the network: absent means "not told" (gate inert), an empty value
means "nothing has published" (blocks every bump).

## Commands

```bash
# What would the next version be?
./scripts/release-manager.sh next
./scripts/release-manager.sh next --format json
./scripts/release-manager.sh next --ref <rev> --published v0.1.50,v0.1.49

# Apply a version to every managed manifest (dry-run by default)
./scripts/release-manager.sh prepare 0.2.0
./scripts/release-manager.sh prepare 0.2.0 --execute
./scripts/release-manager.sh prepare 0.2.0 --execute --no-lock

# Ship: blocker gate + validate + ci-check + tag + push
./scripts/release-manager.sh ship --execute

# Hermetic self-test for the derivation rules and the rewrites
./scripts/release-manager.sh --self-test
```

`prepare <version>` rewrites, in one pass:

- `Cargo.toml` `[workspace.package].version`
- intra-workspace version pins (`do-memory-* = { path = …, version = "…" }`) —
  a minor or major bump without them fails `cargo metadata --locked`
- `AGENTS.md` workspace-version line
- `CHANGELOG.md` — `## [Unreleased]` becomes `## [<version>] - <date>`, with a
  fresh empty `Unreleased` above it (checked by `check-changelog-headings.sh`)
- `plans/ROADMAPS/ROADMAP_ACTIVE.md` and `plans/STATUS/CURRENT.md` release lines,
  replaced with a release-preparation marker: the provenance parenthetical they
  carried described the *previous* publication, so it cannot be carried across
- `Cargo.lock` and `fuzz/Cargo.lock`, regenerated through `cargo metadata` (the
  lock is cargo's artifact, and the excluded `fuzz/` workspace has its own)

It is deliberately dry-run unless `--execute`, matching the rest of this repo's
destructive tooling. Partial failure is bounded: each rewrite is a
`mktemp → chmod → mv` of one file, so no manifest is ever truncated, and a
failure mid-run leaves earlier files bumped — review `git diff` and rerun, which
is idempotent.

### Divergences from `do-harness`

| `do-harness` | here |
|---|---|
| `prepare` writes immediately | dry-run unless `--execute` |
| `prepare` also rewrites npm manifests and a generated man page | no npm packages, no man page — `--no-lock` is the only skip |
| `ship` only tags; a workflow opens the bump PR | `ship` runs the whole gated pipeline and tags |
| automated proposal PR (`auto-release.yml`) | the bump PR is opened by hand; adding the workflow is a separate policy decision |
| `next` also detects shipped-surface renames (`held-rename`) | not ported — this repo has no CLI-surface rename contract to guard |

## Release checklist

1. `./scripts/release-manager.sh next` — confirm the derived bump.
2. `./scripts/release-manager.sh prepare <version> --execute`, review the diff,
   open a `chore(release):` PR, and merge it through the normal gate.
3. `./scripts/check-release-blockers.sh` — the tracker must be empty (the
   auto-maintained `release-drift` issue is excluded by label).
4. `do-harness verify --set verification --strict --record` on the release commit.
5. `./scripts/release-manager.sh ship --execute` — tags `v<version>`, triggers
   `.github/workflows/release.yml` (draft-first, with attestations).
6. `./scripts/release-manager.sh wait-release` — then verify the published
   artifacts.
7. Post-release: bump the workspace in a follow-up PR, so version drift stays
   `version_not_advanced` rather than an untagged release candidate.

## Related

- `.agents/skills/release-guard/SKILL.md` — the agent-facing release path
- `.agents/skills/release-cadence-manager/SKILL.md` — drift detection and cadence
- `scripts/check-release-blockers.sh`, `scripts/verify-release-state.sh`
- `scripts/lib/release-version.sh` — the derivation and rewrite primitives
- `scripts/test-release-version.sh` — the hermetic self-test (`release-version` sensor)
