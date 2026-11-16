# Validation Latest — 2026-11-16 (v0.1.50 release + 0.1.51 bump)

**Workspace**: `0.1.51` (post-release development bump) · **Released tag**: `v0.1.50`
(`0fe1b89d`) · **Main baseline**: `0fe1b89d`
**Scope**: `v0.1.50` release, crates.io publication, released-CLI dogfood, `0.1.51` bump.
**Previous full record**: [`archive/2026/VALIDATION_2026-11-01.md`](archive/2026/VALIDATION_2026-11-01.md)

## Release evidence — v0.1.50

| Artifact | Evidence |
|----------|----------|
| Tag | `v0.1.50` → `0fe1b89d`, pushed by `release-manager.sh ship --execute` |
| Release workflow | run `43829111481` — preflight, plan, all five build targets, host, SBOMs, attestations and publish all succeeded |
| GitHub Release | published 2026-11-16T08:45:18Z, **37 assets**, not a draft, not a prerelease; body contains `## Release Notes` from `parse-changelog` |
| crates.io | `do-memory-core`, `do-memory-storage-redb`, `do-memory-storage-turso`, `do-memory-mcp` all report `0.1.50` in version membership |
| Publish workflow | run `43833847807` green; `Verify publish landed` in the publishing jobs |
| Released-CLI dogfood | asset `do-memory-cli-x86_64-unknown-linux-gnu.tar.xz` digest `ea2da8eddec492f04211cf598449a0dfdda64646f06880cb6495999344d4087d` matched `sha256sum`; `--version` = `0.1.50` |
| Dogfood database | fresh `/tmp/opencode/dogfood-0.1.50/memory.redb`; episode `e982decc-1618-4965-ba74-0e7b547bc975`, 3 successful steps, outcome `success` |
| Cross-process parity | fresh processes: `episode list` = 1, `pattern list` = 2, `pattern search "release dogfood"` = 2, `storage stats.patterns.total_count` = 2 → `2 == 2` |

No mismatch blocked closeout. The release notes were generated, not empty.

## Post-release bump checks

| Check | Command | Result |
|-------|---------|--------|
| Workspace version | `cargo metadata --format-version 1` | ✅ all 9 packages at `0.1.51` |
| Lockfiles | `./scripts/check-lockfile-sync.sh` | ✅ root and fuzz locks satisfy their manifests; `libsql` single-version holds in both |
| Release drift | `./scripts/check-release-drift.sh` | ✅ workspace `0.1.51` ahead of tag `v0.1.50`; `severity=clean` |
| Plan hygiene | `./scripts/validate-plans.sh --all` | ✅ `dated_root=0`, `cargo=0.1.51 tag=0.1.50` |
| Docs integrity | `./scripts/check-docs-integrity.sh` | ✅ links, scripts, version consistency, fail-closed contract |
| Changelog headings | `./scripts/check-changelog-headings.sh` | ✅ unique, ordered, one `Unreleased` section |
| Harness | `do-harness verify --set verification --strict --record` | ✅ 23/23 sensors; `status --set verification` green |

## Earlier verification (2026-11-15)

Plans compaction, the single agent contract, the harness `0.4.6` doctrine and the R-F
backlog truth were verified in that change: `validate-plans.sh --all`,
`check-docs-integrity.sh`, `validate-gate-contract.sh --ci-parity`, 31 harness contract
cases, 17 spike artifacts, and 23/23 verification sensors. Full history:
[`archive/2026/`](archive/2026/).

## Method

- Release claims come from the tag, the workflow run, the GitHub Release object and the
  crates.io version-membership API — not from the release job's green conclusion alone.
- Dogfood uses the published asset from this tag, never a local build.
- A version bump is not a release: `0.1.51` stays a development version until its own
  release-prep PR and `ship`.
