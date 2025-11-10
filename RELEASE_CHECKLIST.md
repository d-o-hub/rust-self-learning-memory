# Release Checklist for rust-self-learning-memory

This checklist ensures all steps are completed before publishing crates to crates.io.

## Pre-Release Preparation

### 1. Version Management
- [ ] Update version in `Cargo.toml` (workspace.package.version)
- [ ] Verify all package versions match workspace version
- [ ] Update CHANGELOG.md with release notes
- [ ] Update version in documentation and README files

### 2. Code Quality & Testing
- [ ] Run `cargo fmt --all` - Code formatting
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings` - Strict linting (zero warnings)
- [ ] Run `cargo test --workspace` - All tests passing
- [ ] Run `cargo test -p tests --test quality_gates` - Quality gates passing
- [ ] Run `cargo build --release` - Release build successful
- [ ] Run `cargo bench` - Benchmarks complete without regressions

### 3. Security & Dependencies
- [ ] Run `cargo audit` - No known vulnerabilities
- [ ] Run `cargo deny check` - Security advisories, licenses, and bans passing
- [ ] Review dependency updates with `cargo outdated`
- [ ] Ensure no hardcoded secrets or credentials

### 4. Documentation
- [ ] Update README.md with current features and examples
- [ ] Verify all package README.md files are current
- [ ] Run `cargo doc --no-deps --all` - Documentation builds cleanly
- [ ] Review and update API documentation comments
- [ ] Update ROADMAP.md with completion status
- [ ] Update plans/ folder with implementation results

### 5. Publication Metadata (Required for crates.io)
- [ ] All packages have `license` or `license-file`
- [ ] All packages have `description`
- [ ] All packages have `homepage`
- [ ] All packages have `repository`
- [ ] All packages have `readme` field pointing to README.md
- [ ] All README.md files exist and are comprehensive
- [ ] Keywords and categories are accurate
- [ ] Documentation URLs are correct

### 6. Package Verification
- [ ] Run `cargo package --list -p memory-core` - Verify package contents
- [ ] Run `cargo package --list -p memory-storage-turso` - Verify package contents
- [ ] Run `cargo package --list -p memory-storage-redb` - Verify package contents
- [ ] Run `cargo package --list -p memory-mcp` - Verify package contents
- [ ] Verify no large files are included (check .crate file size < 10MB)
- [ ] Verify no sensitive files are included (.env, credentials, etc.)

### 7. Dry Run Publication
- [ ] Run `cargo publish --dry-run --allow-dirty -p memory-core`
- [ ] Review any warnings or errors from dry-run
- [ ] Fix any publication blockers

## Git & Version Control

### 8. Clean Repository
- [ ] All changes committed to git
- [ ] Working directory is clean (`git status`)
- [ ] All tests passing in CI/CD pipeline
- [ ] No pending code review comments

### 9. Tagging & Release Notes
- [ ] Create git tag for version (e.g., `v0.1.0`)
- [ ] Push tag to repository: `git push origin v0.1.0`
- [ ] Create GitHub Release with changelog
- [ ] Attach any release artifacts if needed

## Publication (Actual Release)

### 10. Publish to crates.io (IN ORDER)

**IMPORTANT**: Packages must be published in dependency order:

1. **memory-core** (no dependencies on local packages)
   ```bash
   cargo publish -p memory-core
   ```
   - [ ] Wait for crates.io to process (check https://crates.io/crates/memory-core)
   - [ ] Verify version appears on crates.io

2. **memory-storage-turso** (depends on memory-core)
   ```bash
   cargo publish -p memory-storage-turso
   ```
   - [ ] Wait for crates.io to process
   - [ ] Verify version appears on crates.io

3. **memory-storage-redb** (depends on memory-core)
   ```bash
   cargo publish -p memory-storage-redb
   ```
   - [ ] Wait for crates.io to process
   - [ ] Verify version appears on crates.io

4. **memory-mcp** (depends on memory-core)
   ```bash
   cargo publish -p memory-mcp
   ```
   - [ ] Wait for crates.io to process
   - [ ] Verify version appears on crates.io

### 11. Post-Publication Verification
- [ ] Verify all crates appear on crates.io
- [ ] Verify documentation appears on docs.rs
- [ ] Test installation: `cargo add memory-core memory-storage-turso memory-storage-redb`
- [ ] Verify examples work with published versions
- [ ] Check docs.rs builds complete successfully

## Announcement & Communication

### 12. Community Updates
- [ ] Announce release on project repository (GitHub Release)
- [ ] Update project website/homepage if applicable
- [ ] Post to relevant Rust forums/communities if appropriate
- [ ] Update any external documentation or tutorials

## Troubleshooting

### Common Publication Issues

**Issue: "no matching package named X found"**
- **Solution**: Publish dependencies first (follow order above)

**Issue: "file size exceeds 10MB limit"**
- **Solution**: Add large files to `.cargo/cargo.toml` exclude list

**Issue: "uncommitted changes detected"**
- **Solution**: Commit changes or use `--allow-dirty` flag (dry-run only)

**Issue: "API token required"**
- **Solution**: Run `cargo login` with your crates.io API token

**Issue: "package already exists"**
- **Solution**: Increment version number

### Rollback Procedure

If a release needs to be rolled back:

1. **Yank the problematic version** (does not delete, just prevents new dependencies):
   ```bash
   cargo yank --vers 0.1.0 memory-core
   ```

2. **Fix issues** and increment patch version

3. **Re-publish** with new version

4. **Announce** the issue and recommend users upgrade

## Notes

- **Never delete crates** from crates.io - use `cargo yank` instead
- **Semantic Versioning**: Follow SemVer (MAJOR.MINOR.PATCH)
- **Yanking** only prevents new projects from depending on that version
- **Documentation** on docs.rs builds automatically after publication
- **Publication is permanent** - version numbers cannot be reused

## 2025 Best Practices

Based on 2025 Rust ecosystem standards:

- ✅ Use `cargo clippy -- -D warnings` for strict linting
- ✅ Ensure cargo-deny checks pass for supply chain security
- ✅ Use workspace inheritance for metadata where possible
- ✅ Provide comprehensive README with examples
- ✅ Document all public APIs
- ✅ Include performance characteristics in documentation
- ✅ Use GitHub Actions with Trusted Publishing (OIDC) if available
- ✅ Fast-fail CI pattern: fmt → clippy → test → build

## Version History

- **v0.1.0** - Initial release
  - Core episodic learning system
  - Dual storage (Turso + redb)
  - Pattern extraction and learning
  - MCP integration with sandbox
  - Comprehensive testing and security

---

**Last Updated**: 2025-11-10
**For Questions**: See CONTRIBUTING.md or open a GitHub issue
