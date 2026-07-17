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
2. Update `CHANGELOG.md` with `## [X.Y.Z] - YYYY-MM-DD`.
3. Set **Released Version** in `plans/ROADMAPS/ROADMAP_ACTIVE.md` and `plans/STATUS/CURRENT.md` to `vX.Y.Z`.
4. Merge PR; wait for main CI green.
5. Run: `./scripts/release-manager.sh ship --execute`

## Semver (0.x)

| Change | Bump |
|--------|------|
| Breaking | minor or major (team call) |
| Feature | minor preferred |
| Fix | patch |

## Changelog categories

Added · Changed · Deprecated · Removed · Fixed · Security

## Forbidden

```bash
# DO NOT — bypasses cargo-dist and release.yml preflight
gh release create vX.Y.Z ...
```

## References

- ADR-034 Release Engineering
- `.agents/skills/release-guard/SKILL.md`
- `./scripts/release-manager.sh`
