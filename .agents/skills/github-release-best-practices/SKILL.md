---
name: github-release-best-practices
description: "Background reference for release prep (SemVer, changelog categories). For actual releases, ALWAYS use the release-guard skill and release-manager.sh ship — never manual gh release create."
audience: maintainers
workflow: github
---

# GitHub Release Best Practices (reference only)

> **Canonical procedure:** [release-guard](../release-guard/SKILL.md)  
> **Canonical CLI:** `./scripts/release-manager.sh ship --execute`  
> **GitHub Release creator:** `.github/workflows/release.yml` (tag push only)

Do **not** invent a second release path. This file is supporting context only.

## Prep before ship (via PR to main)

1. Bump workspace `version` in root `Cargo.toml` (all members follow workspace).
2. Update `CHANGELOG.md` with **exactly one** `## [X.Y.Z] - YYYY-MM-DD` (unique version headers — cargo-dist/parse-changelog fails silently on duplicates → empty GitHub Release body).
3. Set **Released Version** in `plans/ROADMAPS/ROADMAP_ACTIVE.md` and `plans/STATUS/CURRENT.md` to `vX.Y.Z`.
4. Preflight the ship state: `./scripts/verify-release-state.sh --check-unreleased` (validates items 1–3 against `workspace_version` and prints per-file mismatches).
5. Merge PR (version + changelog + version docs land as **one** release-prep PR); wait for main CI green.
6. Run: `./scripts/release-manager.sh ship --execute`
7. Confirm notes: `gh release view vX.Y.Z` shows `## Release Notes` content (v0.1.34/v0.1.35 format).

### A bumped version is not a shippable repo

Tagging requires 1 + 2 + 3 above **on main**. `⚠️ parse-changelog not installed` in the preflight output is also a
blocker — install it (`cargo install --locked parse-changelog`; the crate is **`parse-changelog`**, not
`parse-changelog-cli`, which does not exist) or release notes silently fall back to an empty body.
Mismatch output looks like this:

```
$ ./scripts/verify-release-state.sh --check-unreleased
❌ ROADMAP_ACTIVE.md says v0.1.45 (expected v0.1.46)
❌ STATUS/CURRENT.md references v0.1.45
❌ CHANGELOG.md missing entry for v0.1.46
⚠️  parse-changelog not installed (release notes will fall back)
```

## Semver (0.x)

| Change | Bump |
|--------|------|
| Breaking | minor or major (team call) |
| Feature | minor preferred |
| Fix | patch |

## Changelog categories

Added · Changed · Deprecated · Removed · Fixed · Security

## crates.io reality check

The publish chain (`.github/workflows/publish-crates.yml`, `environment: crates.io`, OIDC Trusted Publishing) owns
**exactly four** crates: core → redb → turso → mcp.

- Publish acceptance = `index("X.Y.Z") != null` for those four **plus** a green `Verify publish landed` step.
- A **green** workflow whose publish step conclusion was `skipped` (`conclusion=skipped;duration_ms=0`) published
  **nothing** — check step conclusions, not just the workflow conclusion, before calling a version released.
- `do-memory-cli` returning HTTP 404 on crates.io is **correct** (it ships as a GitHub Release asset), and
  `do-memory-test-utils` is `publish = false` — neither is a publish failure.
- Query crates.io with a custom UA (`curl -A <UA>`); the default curl UA is rejected with `FETCH_FAILED`-shaped noise.

## Forbidden

```bash
# DO NOT — bypasses cargo-dist and release.yml preflight
gh release create vX.Y.Z ...
```

## References

- ADR-034 Release Engineering
- `.agents/skills/release-guard/SKILL.md`
- `./scripts/release-manager.sh`
- `./scripts/verify-release-state.sh` (ship preflight)
- `.github/workflows/publish-crates.yml` (crates.io chain)
