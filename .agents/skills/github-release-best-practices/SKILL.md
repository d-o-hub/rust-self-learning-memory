---
name: github-release-best-practices
description: GitHub release preparation for Rust workspace projects. Use when preparing releases with proper versioning, changelog, and quality gates.
audience: maintainers
workflow: github
---
# GitHub Release Best Practices

Release management for Rust workspaces with versioning, changelog, and quality gates.

## Key Steps

1. **Quality Gates**: `./scripts/quality-gates.sh`
2. **Version Bump**: Update `Cargo.toml` workspace version
3. **Changelog**: Update `CHANGELOG.md` (Keep a Changelog format)
4. **Tag + Push**: `git tag -a v0.X.Y -m "Release v0.X.Y" && git push origin v0.X.Y`
5. **GitHub Release**: `gh release create v0.X.Y --title "v0.X.Y" --notes-file CHANGELOG.md`

## Semver Matrix

| Change Type | Bump | Example |
|-------------|------|---------|
| Breaking | MAJOR | 0.1.7 → 1.0.0 |
| New Feature | MINOR | 0.1.7 → 0.2.0 |
| Bug Fix | PATCH | 0.1.7 → 0.1.8 |

For 0.x: MINOR for most changes, PATCH for critical bug fixes.

## Changelog Categories

1. Added - New features
2. Changed - Modified functionality
3. Deprecated - Planned removals
4. Removed - Deleted features
5. Fixed - Bug fixes
6. Security - Vulnerability patches

## Modern Tooling (ADR-034)

```bash
# API compatibility check
cargo semver-checks check-release --workspace

# Version bump automation
cargo release patch  # handles version + commit + tag

# Binary distribution
cargo dist init      # generates release.yml workflow
```

**CRITICAL**: Cargo.toml version must match git tag (without 'v'). Use `cargo release` to ensure atomicity.

## Release Notes Template

```markdown
## Summary
Brief description of changes.

## Fixed / Changed / Added
- Item 1
- Item 2

## Performance / Security (if applicable)
- Metrics, improvements

## Full Changelog
https://github.com/d-o-hub/rust-self-learning-memory/compare/v0.1.7...v0.1.8
```

## References

- ADR-034: Release Engineering Modernization
- ADR-031: Cargo Lock Integrity