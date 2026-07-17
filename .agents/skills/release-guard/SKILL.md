---
name: release-guard
description: "Canonical release workflow for this repo. One path every time: main green → release-manager ship → tag vX.Y.Z → release.yml. Use when the user says release, tag, publish, deploy, version, or cut a GitHub release."
---

# Release Guard — Canonical Release Skill

**One workflow. Same for humans and agents. No alternate paths.**

## Golden path (memorize this)

```text
1. Version + CHANGELOG already on main (via PR)
2. origin/main CI fully green
3. ./scripts/release-manager.sh ship --execute
4. release.yml builds artifacts + creates GitHub Release
5. Optional: ./scripts/release-manager.sh wait-release
```

| Step | Who | Tool |
|------|-----|------|
| Bump `Cargo.toml` version + CHANGELOG | Human/agent via **PR** | PR to main |
| Docs: `Released Version` = workspace version | Same PR | ROADMAP + STATUS |
| Wait for main CI | Agent | `gh run list --branch main --commit $(git rev-parse origin/main)` |
| Tag + push **only** | Agent/human | `./scripts/release-manager.sh ship --execute` |
| Build + GitHub Release | **GitHub Actions** | `.github/workflows/release.yml` (on tag push) |
| crates.io (if needed) | **GitHub Actions** | `publish-crates.yml` (separate) |

## NEVER

| Forbidden | Why |
|-----------|-----|
| `gh release create` by hand | Bypasses cargo-dist / preflight / artifacts |
| Tag from non-`main` | Policy + dist target |
| Tag when `Cargo.toml` ≠ tag (`v0.1.35` ↔ `0.1.35`) | release.yml preflight fails |
| `--admin` / force merge | Branch protection exists for a reason |
| Ship while main CI pending/failed | Broken release |
| Multiple competing “release procedures” | This skill + `release-manager.sh` only |

## Agent checklist (every release)

```bash
# 0) Status
./scripts/release-manager.sh status

# 1) On clean main, synced
git checkout main && git pull --ff-only origin main
test -z "$(git status --porcelain)"

# 2) Version consistency (docs + crates)
./scripts/verify-release-state.sh --check-unreleased   # exit 0 (warnings OK)

# 3) Main CI green (also enforced by ship)
./scripts/release-manager.sh ci-check

# 4) Ship (local quality + CI re-check + tag + push tag)
./scripts/release-manager.sh ship --execute

# 5) Watch release.yml
./scripts/release-manager.sh wait-release
# or: gh run list --workflow=release.yml --limit 3
```

Dry-run first if unsure:

```bash
./scripts/release-manager.sh ship          # no --execute → dry-run
```

## Version rules

- **Workspace** `Cargo.toml` `version = "X.Y.Z"` is the source of truth.
- **Git tag** is always `vX.Y.Z` (leading `v`).
- **release.yml** rejects tag/version mismatch.
- **ROADMAP_ACTIVE** and **STATUS/CURRENT** first `Released Version` line must equal `X.Y.Z` before ship (verify-release-state).
- CHANGELOG must have `## [X.Y.Z]` section.

Semver for this 0.x line: prefer **patch** for fixes, **minor** for features (team convention).

## What `ship` does

1. `verify-release-state.sh --check-unreleased`
2. Local: fmt, clippy, build check, nextest, doctest, quality-gates  
   (skip with `--skip-local-tests` only in documented emergencies)
3. `ci-check` on `origin/main` HEAD (all runs completed success/skipped)
4. `git tag -a vX.Y.Z -m "Release vX.Y.Z"` on that HEAD
5. `git push origin refs/tags/vX.Y.Z` only (does **not** push commits)
6. Prints how to monitor `release.yml`

## After ship

- Confirm: `gh release view vX.Y.Z`
- Drift issue (#849-style) should close when tag matches workspace version
- Bump workspace to next patch for development in a **follow-up PR** (optional)

## Failure playbook

| Symptom | Fix |
|---------|-----|
| verify-release-state fails docs | PR: set `Released Version: vX.Y.Z` in ROADMAP + STATUS |
| ci-check pending | Wait; re-run `ci-check` |
| ci-check failed | Fix main CI first |
| Tag already on remote | Do not retag; inspect `gh release view` |
| release.yml failed preflight | Version/tag mismatch — delete bad tag only after review |
| Want crates.io | Use publish workflow / team process after GitHub Release |

## Relationship to other skills

| Skill | Use for |
|-------|---------|
| **release-guard** (this) | **All** release/tag/publish/deploy requests |
| github-release-best-practices | Background only; defers to this skill |
| pr-readiness | Merging the version-bump PR before ship |
| release-drift workflow | Alerts when main outruns tags — does not ship |

## Commands reference

```bash
./scripts/release-manager.sh status
./scripts/release-manager.sh validate
./scripts/release-manager.sh ci-check
./scripts/release-manager.sh ship              # dry-run
./scripts/release-manager.sh ship --execute    # production
./scripts/release-manager.sh wait-release
./scripts/release-manager.sh rollback --tag vX.Y.Z --execute   # local tag only
```

## Progressive disclosure

- CI wait patterns: [ci-reference.md](ci-reference.md)
- Workflow definition: `.github/workflows/release.yml`
- Version consistency: `./scripts/verify-release-state.sh`
