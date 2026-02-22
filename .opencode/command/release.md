---
description: Create a GitHub release with atomic commit, quality gates, and semver safety. Use: /release [patch|minor|major]
subtask: true
---

# GitHub Release Command (2026 Best Practices)

Create a professional GitHub release following 2026 best practices for Rust workspaces.

## Usage

```
/release [patch|minor|major]
```

**Examples:**
- `/release patch` - Bug fixes, backward compatible (default)
- `/release minor` - New features, backward compatible
- `/release major` - Breaking API changes

---

## Pre-Release Safety Gates (BLOCKING)

### 1. Branch Verification (REQUIRED)
```bash
# Must be on main branch
git branch --show-current
# Expected: main

# Working directory must be clean
git status --porcelain
# Expected: empty output
```

**FAIL Response:**
```
🚫 BLOCKED: Not on main or working directory not clean
Fix: git checkout main && git pull && [commit/stash changes]
Re-run: /release [patch|minor|major]
```

### 2. CI Status Check (REQUIRED)
```bash
# Verify all CI workflows passed
gh run list --branch main --limit 5 --json status,conclusion,name
# Expected: ALL status=completed, conclusion=success
```

**FAIL Response:**
```
🚫 BLOCKED: CI not green
Fix: Wait for CI to complete or fix failures
Check: gh run list --branch main
Re-run: /release [patch|minor|major]
```

### 3. Quality Gates (REQUIRED)
```bash
# Run full quality gate suite
./scripts/quality-gates.sh
# Expected: All gates PASSED
```

**Quality Gate Requirements:**
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all -- -D warnings` passes (zero warnings)
- [ ] `cargo build --all` succeeds
- [ ] `cargo nextest run --all` passes
- [ ] Test coverage ≥90%
- [ ] Security audit clean

**FAIL Response:**
```
🚫 BLOCKED: Quality gates failed
Fix: Run ./scripts/quality-gates.sh and resolve issues
Re-run: /release [patch|minor|major]
```

### 4. Semver Compatibility Check (REQUIRED for minor/major)
```bash
# Check for accidental breaking changes
cargo semver-checks check-release --workspace
# Expected: No semver violations
```

**FAIL Response:**
```
🚫 BLOCKED: Semver violations detected
Review: cargo semver-checks check-release --workspace
Options:
  1. Fix breaking changes (recommended)
  2. Use /release major if breaking change is intentional
Re-run: /release [patch|minor|major]
```

---

## Release Workflow

### Step 1: Atomic Commit (if changes pending)

If there are uncommitted changes in CHANGELOG.md or version-related files:

```bash
# Stage release preparation changes
git add CHANGELOG.md Cargo.toml Cargo.lock

# Atomic commit with conventional format
git commit -m "release: prepare v{VERSION}"
```

### Step 2: Version Bump & Tag (via cargo-release)

Use `cargo-release` for automated version management:

```bash
# Patch release (default)
cargo release patch --no-publish --execute

# Minor release
cargo release minor --no-publish --execute

# Major release
cargo release major --no-publish --execute
```

**What cargo-release does automatically:**
- Bumps version in workspace `Cargo.toml`
- Updates `Cargo.lock`
- Updates CHANGELOG.md `[Unreleased]` section
- Creates git commit: `release: v{VERSION}`
- Creates annotated git tag: `v{VERSION}`
- Pushes commit and tag to origin

### Step 3: Verify Tag & Push

```bash
# Confirm tag was created
git tag -l "v*"

# Verify tag pushed to remote
git ls-remote --tags origin | grep "v{VERSION}"
```

### Step 4: CI-Driven Release (cargo-dist)

The `.github/workflows/release.yml` (cargo-dist) automatically:
- Builds binaries for all platforms (Linux, macOS, Windows)
- Creates tarballs with SHA256 checksums
- Generates shell/PowerShell installers
- Creates GitHub Release with release notes
- Uploads all artifacts

**Monitor the release:**
```bash
# Watch the release workflow
gh run watch

# Or check specific workflow
gh run list --workflow=release.yml --limit 3
```

### Step 5: Verify GitHub Release

```bash
# Check release was created
gh release view v{VERSION} --json tagName,name,url,createdAt

# Verify assets uploaded
gh release view v{VERSION} --json assets --jq '.assets[].name'
```

---

## Changelog Management

### Update CHANGELOG.md Before Release

Ensure `[Unreleased]` section contains all changes:

```markdown
## [Unreleased]

### Added
- New feature description (closes #123)

### Changed
- Changed behavior description

### Fixed
- Bug fix description (fixes #456)

### Breaking
- Breaking change with migration guide
```

### Version Categories (Keep a Changelog)

| Category | Description |
|----------|-------------|
| **Added** | New features |
| **Changed** | Changes to existing functionality |
| **Deprecated** | Features planned for removal |
| **Removed** | Removed features |
| **Fixed** | Bug fixes |
| **Security** | Security vulnerability patches |

---

## Version Bump Decision Matrix

| Change Type | Bump | Example |
|-------------|------|---------|
| Bug fix, backward compatible | `patch` | 0.1.15 → 0.1.16 |
| New feature, backward compatible | `minor` | 0.1.15 → 0.2.0 |
| Breaking API change | `major` | 0.1.15 → 1.0.0 |
| CI/quality improvements | `patch` | 0.1.15 → 0.1.16 |
| Documentation only | `patch` | 0.1.15 → 0.1.16 |

### Special: 0.x Versions

During initial development (0.x):
- "Anything MAY change at any time" (SemVer spec)
- Most changes warrant `minor` bump
- Reserve `patch` for critical bug fixes only

---

## Rollback Procedure

If critical issue discovered post-release:

```bash
# 1. Yank from crates.io (if published)
cargo yank --version {VERSION}

# 2. Add warning to GitHub release
gh release edit v{VERSION} --notes-file - <<EOF
⚠️ **DEPRECATED** - Critical issue discovered

Issue: [description]
Fixed in: v{NEXT_VERSION}
EOF

# 3. Prepare and release fix
/release patch
```

---

## Integration with Skills

| Skill | Purpose |
|-------|---------|
| `release-guard` | Strict gatekeeping (PR merged, CI green, main branch) |
| `github-release-best-practices` | 2026 best practices, templates, automation |
| `code-quality` | Pre-release formatting, linting |
| `github-workflows` | CI/CD pipeline validation |

**Load before release:**
```
skill: release-guard, github-release-best-practices
```

---

## Quick Reference Commands

```bash
# Full pre-release check
./scripts/quality-gates.sh

# Semver validation
cargo semver-checks check-release --workspace

# Release (automated)
cargo release patch --no-publish --execute

# Monitor release workflow
gh run watch

# Verify release
gh release view v{VERSION}
```

---

## Configuration Files

| File | Purpose |
|------|---------|
| `release.toml` | cargo-release configuration |
| `Cargo.toml` | Workspace version (single source of truth) |
| `CHANGELOG.md` | Release notes history |
| `.github/workflows/release.yml` | cargo-dist automated release |

---

## References

- [ADR-034: Release Engineering Modernization](plans/adr/ADR-034-Release-Engineering-Modernization.md)
- [Skill: release-guard](.agents/skills/release-guard/SKILL.md)
- [Skill: github-release-best-practices](.agents/skills/github-release-best-practices/SKILL.md)
- [cargo-release](https://github.com/crate-ci/cargo-release)
- [cargo-dist](https://axodotdev.github.io/cargo-dist/)
- [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
- [Keep a Changelog](https://keepachangelog.com/)
