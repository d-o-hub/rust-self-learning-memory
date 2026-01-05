---
description: create a GitHub release
subtask: true
---

create a GitHub release

## Release Process

Use this workflow for creating releases:

## Pre-Release Checklist

**1. Version Bump**
- Update version in `Cargo.toml` following SemVer (MAJOR.MINOR.PATCH)
- MAJOR: Breaking changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible

**2. Update Changelog**
- Add release notes to `CHANGELOG.md`
- Include section for "Added", "Changed", "Fixed", "Removed"
- Link to relevant issues/PRs

**3. Quality Gates**
- Run `cargo test --all` (must pass all tests)
- Run `cargo clippy --all -- -D warnings` (zero warnings)
- Run `cargo fmt --all -- --check` (formatted code)
- Verify test coverage >90%

**4. Documentation**
- Update README with new features/API changes
- Update relevant documentation files
- Ensure examples work with new version

**5. Create Git Tag**
```bash
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

## Creating the Release

**Using GitHub CLI:**
```bash
gh release create v0.2.0 \
  --title "v0.2.0" \
  --notes "See CHANGELOG.md for details"
```

**Manual on GitHub:**
1. Go to repository → Releases
2. Click "Draft a new release"
3. Choose/enter tag
4. Enter title (same as version)
5. Add release notes
6. Click "Publish release"

## Release Notes Structure

```
## [0.2.0] - 2025-01-05

### Added
- New feature description (closes #123)
- Another new feature

### Changed
- Breaking change description
- API modification details

### Fixed
- Bug fix description (fixes #456)
- Performance issue resolved

### Removed
- Deprecated feature removed (see migration guide)
```

## After Release

**1. Publish to crates.io** (if applicable)
```bash
cargo publish
```

**2. Verify Release**
- Check release page renders correctly
- Verify assets are downloadable
- Test install from release

**3. Notify Users**
- Announce on project communication channels
- Update version badge in README
- Post release announcement

## Key Principles

**1. Semantic Versioning**
- Never release breaking changes without MAJOR bump
- Document breaking changes prominently
- Maintain backward compatibility when possible

**2. Clear Release Notes**
- Group changes by type (Added, Changed, Fixed)
- Reference related issues/PRs
- Highlight breaking changes
- Include migration guide if needed

**3. Quality Assurance**
- All tests must pass before release
- No clippy warnings allowed
- Code must be properly formatted
- Documentation must be up to date

**4. Git Tags**
- Use annotated tags: `git tag -a`
- Tag name matches version: `v0.2.0`
- Push tags to remote
- Never retag a released version

## Release Assets

- Attach precompiled binaries if applicable
- Include checksums for binaries
- Provide installation instructions
- Link to documentation

## Rollback Plan

If critical issue is discovered:
1. Yank crates.io version: `cargo yank --version 0.2.0`
2. Add warning to GitHub release
3. Issue security advisory if vulnerability
4. Prepare patch release: `0.2.1`
5. Document the issue and fix

## Examples

Good release notes:
```
## [0.2.0] - 2025-01-05

### Added
- New MCP server tools for memory queries (closes #42)
- Support for local embeddings via Ollama (closes #38)
- Performance benchmark suite

### Changed
- Improved cache sync performance by 40%
- Refactored storage layer for better modularity

### Fixed
- Resolved redb cache corruption on concurrent writes (fixes #56)
- Fixed memory leak in pattern extraction (fixes #61)

### Breaking
- Renamed `MemoryManager::new()` to `MemoryManager::with_config()`
  See MIGRATION.md for upgrade guide
```
