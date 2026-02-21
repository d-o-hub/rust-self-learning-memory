# ADR-034: Release Engineering Modernization

- **Status**: Proposed
- **Date**: 2026-02-21
- **Deciders**: Project maintainers
- **Supersedes**: None (extends current release.yml)
- **Related**: ADR-029 (GitHub Actions Modernization), ADR-031 (Cargo Lock Integrity)

## Context

Current release process is manual and fragile:

1. **Manual version bumping** in workspace `Cargo.toml` (currently v0.1.15)
2. **Manual git tagging** triggers `release.yml`
3. **No changelog automation** — release notes rely on GitHub's `generate_release_notes`
4. **No semver verification** — breaking changes can ship as patch versions
5. **No installer generation** — only raw binaries uploaded
6. **No crate publishing** — workspace crates not published to crates.io
7. **Binary packaging** is ad-hoc (no tarballs, checksums, or signatures)

### Current Release Flow

```
Manual version edit → git commit → git tag v0.1.15 → push tag
  → CI: pre-release tests → build 4 targets → upload raw binaries → GitHub Release
```

### 2025/2026 Best Practices

The Rust ecosystem has converged on:
- **cargo-release** for version management and publish workflow
- **cargo-dist** for binary distribution (installers, tarballs, checksums)
- **cargo-semver-checks** for API compatibility verification
- **release-plz** or **cargo-release** for changelog automation
- **Conventional commits** → automated changelogs

## Decision

### Phase 1: cargo-semver-checks Integration (Immediate)

Add semver verification to CI to prevent accidental breaking changes:

```yaml
# In ci.yml essential checks
- name: Semver Check
  run: |
    cargo install --locked cargo-semver-checks
    cargo semver-checks check-release --workspace
```

**Rationale**: This is a safety net, not a release blocker. It flags potential semver violations early.

### Phase 2: cargo-release Workflow (Short-term)

Adopt `cargo-release` for version management:

```toml
# release.toml (workspace root)
[workspace]
allow-branch = ["main"]
sign-commit = false
sign-tag = false
push = true
publish = false           # Not publishing to crates.io yet
pre-release-commit-message = "release: v{{version}}"
tag-message = "v{{version}}"
tag-prefix = "v"
tag-name = "v{{version}}"

[[package.pre-release-replacements]]
file = "CHANGELOG.md"
search = "## \\[Unreleased\\]"
replace = "## [Unreleased]\n\n## [{{version}}] - {{date}}"
```

**Release workflow becomes**:
```bash
# 1. Update CHANGELOG.md "Unreleased" section
# 2. Run release (handles version bump, commit, tag, push)
cargo release patch  # or minor/major
```

### Phase 3: cargo-dist Binary Distribution (Medium-term)

Replace custom `release.yml` build matrix with cargo-dist:

```bash
cargo dist init
```

This generates:
- Platform-specific tarballs with checksums (SHA256)
- Shell installer script (`curl | sh`)
- PowerShell installer for Windows
- Homebrew formula (optional)
- Self-updating release.yml workflow

**Benefits over current approach**:
- Reproducible builds with locked dependencies
- Checksums and optional signing
- Installer scripts for end users
- Automatic artifact naming conventions

### Phase 4: Changelog Automation (Medium-term)

Adopt conventional commits format for automated changelog generation:

```
feat(core): add episode relationship queries
fix(storage): prevent race in concurrent writes
perf(redb): optimize cache invalidation
docs(mcp): update tool registry API docs
```

Configure `cargo-release` or `git-cliff` to generate changelogs from commits.

### Phase 5: Crates.io Publishing (Future)

When public API stabilizes (post-1.0):
- Enable `publish = true` in `release.toml`
- Add `cargo-semver-checks` as release gate
- Configure `cargo-release` publish order (core → storage → mcp → cli)

## Implementation Plan

| Phase | Action | Timeline | Blocking? |
|-------|--------|----------|-----------|
| 1 | Add cargo-semver-checks to CI | Week 1 | No |
| 2 | Create release.toml, adopt cargo-release | Week 2-3 | No |
| 3 | cargo-dist init + replace release.yml | Week 4-6 | No |
| 4 | Conventional commits + git-cliff | Week 6-8 | No |
| 5 | Crates.io publishing | Post-1.0 | No |

## Consequences

### Positive
- **Automated, repeatable releases** — eliminates manual version editing
- **Semver safety** — breaking changes caught before release
- **Professional distribution** — installers, checksums, proper packaging
- **Changelog automation** — reduces release friction
- **Foundation for crates.io publishing** when ready

### Negative
- Additional tooling to learn (cargo-release, cargo-dist)
- Conventional commit discipline required
- cargo-dist generates its own release.yml (replaces current custom one)

### Risks
- cargo-dist is opinionated; may not fit all edge cases
- Conventional commit enforcement may slow down quick fixes
- cargo-semver-checks may produce false positives on internal APIs

## References

- [cargo-dist](https://axodotdev.github.io/cargo-dist/) — Shippable application packaging
- [cargo-release](https://github.com/crate-ci/cargo-release) — Release workflow automation
- [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) — Semver linting
- [git-cliff](https://git-cliff.org/) — Changelog generator from conventional commits
- [Rust Project Goals: cargo-semver-checks into cargo](https://rust-lang.github.io/rust-project-goals/2025h2/goals.html)
