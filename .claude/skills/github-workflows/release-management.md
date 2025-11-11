# Release Management for Rust Projects

Comprehensive guide for automating releases with GitHub Actions, including versioning, changelogs, and crates.io publishing.

## Before Creating Release Workflows

**ALWAYS verify the current repository context first:**

```bash
# Get repo owner and name
REPO_INFO=$(gh repo view --json nameWithOwner,owner,name)
REPO_OWNER=$(echo $REPO_INFO | jq -r '.owner')
REPO_NAME=$(echo $REPO_INFO | jq -r '.name')

echo "Repository: $REPO_OWNER/$REPO_NAME"

# Check for existing release workflows
gh workflow list | grep -i release

# Check existing releases
gh release list --limit 10

# Check for release-related files
ls -la CHANGELOG.md VERSION.txt .version 2>/dev/null
```

## Complete Release Workflow

### 1. Automated Release on Tag Push

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'  # Semantic versioning tags (v1.2.3)

permissions:
  contents: write  # Required for creating releases
  actions: read    # Required for downloading artifacts

env:
  CARGO_TERM_COLOR: always

jobs:
  # Verify tag format and version
  validate:
    name: Validate Release Tag
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.get_version.outputs.version }}
    steps:
      - uses: actions/checkout@v5

      - name: Get version from tag
        id: get_version
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          echo "version=$VERSION" >> $GITHUB_OUTPUT
          echo "Releasing version: $VERSION"

      - name: Validate semantic version
        run: |
          VERSION=${{ steps.get_version.outputs.version }}
          if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
            echo "Error: Invalid semantic version format: $VERSION"
            exit 1
          fi

  # Build release binaries for multiple platforms
  build-release:
    name: Build ${{ matrix.target }}
    needs: validate
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            binary-suffix: ''
            archive-suffix: tar.gz
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            binary-suffix: ''
            archive-suffix: tar.gz
          - os: macos-latest
            target: x86_64-apple-darwin
            binary-suffix: ''
            archive-suffix: tar.gz
          - os: macos-latest
            target: aarch64-apple-darwin
            binary-suffix: ''
            archive-suffix: tar.gz
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            binary-suffix: .exe
            archive-suffix: zip

    steps:
      - uses: actions/checkout@v5

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install musl tools (Linux musl only)
        if: matrix.target == 'x86_64-unknown-linux-musl'
        run: sudo apt-get update && sudo apt-get install -y musl-tools

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}-release

      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }} --all

      - name: Package binaries (Unix)
        if: matrix.os != 'windows-latest'
        run: |
          mkdir -p artifacts
          cd target/${{ matrix.target }}/release
          for bin in *; do
            if [ -f "$bin" ] && [ -x "$bin" ] && [ ! -d "$bin" ]; then
              tar czf "../../../artifacts/${bin}-${{ needs.validate.outputs.version }}-${{ matrix.target }}.tar.gz" "$bin"
            fi
          done

      - name: Package binaries (Windows)
        if: matrix.os == 'windows-latest'
        shell: pwsh
        run: |
          New-Item -ItemType Directory -Force -Path artifacts
          cd target/${{ matrix.target }}/release
          Get-ChildItem -File -Filter "*.exe" | ForEach-Object {
            Compress-Archive -Path $_.Name -DestinationPath "../../../artifacts/$($_.BaseName)-${{ needs.validate.outputs.version }}-${{ matrix.target }}.zip"
          }

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: binaries-${{ matrix.target }}
          path: artifacts/*
          if-no-files-found: warn
          retention-days: 7

  # Generate changelog from commits
  changelog:
    name: Generate Changelog
    needs: validate
    runs-on: ubuntu-latest
    outputs:
      changelog: ${{ steps.generate.outputs.changelog }}
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0  # Need full history for changelog

      - name: Generate changelog
        id: generate
        run: |
          # Get the previous tag
          PREV_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")

          if [ -z "$PREV_TAG" ]; then
            echo "No previous tag found, using all commits"
            COMMITS=$(git log --oneline --no-merges)
          else
            echo "Generating changelog from $PREV_TAG to ${{ github.ref_name }}"
            COMMITS=$(git log $PREV_TAG..HEAD --oneline --no-merges)
          fi

          # Categorize commits
          FEATURES=$(echo "$COMMITS" | grep -i "^[a-f0-9]* feat" || true)
          FIXES=$(echo "$COMMITS" | grep -i "^[a-f0-9]* fix" || true)
          DOCS=$(echo "$COMMITS" | grep -i "^[a-f0-9]* docs" || true)
          PERF=$(echo "$COMMITS" | grep -i "^[a-f0-9]* perf" || true)
          BREAKING=$(echo "$COMMITS" | grep -i "^[a-f0-9]* \(breaking\|BREAKING\)" || true)

          # Build changelog
          CHANGELOG="## What's Changed\n\n"

          if [ -n "$BREAKING" ]; then
            CHANGELOG="${CHANGELOG}### ⚠️ Breaking Changes\n${BREAKING}\n\n"
          fi

          if [ -n "$FEATURES" ]; then
            CHANGELOG="${CHANGELOG}### ✨ Features\n${FEATURES}\n\n"
          fi

          if [ -n "$FIXES" ]; then
            CHANGELOG="${CHANGELOG}### 🐛 Bug Fixes\n${FIXES}\n\n"
          fi

          if [ -n "$PERF" ]; then
            CHANGELOG="${CHANGELOG}### ⚡ Performance\n${PERF}\n\n"
          fi

          if [ -n "$DOCS" ]; then
            CHANGELOG="${CHANGELOG}### 📚 Documentation\n${DOCS}\n\n"
          fi

          # Save to file and output
          echo -e "$CHANGELOG" > /tmp/changelog.md
          echo "changelog<<EOF" >> $GITHUB_OUTPUT
          cat /tmp/changelog.md >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT

      - name: Upload changelog
        uses: actions/upload-artifact@v4
        with:
          name: changelog
          path: /tmp/changelog.md
          retention-days: 7

  # Create GitHub release with all artifacts
  create-release:
    name: Create GitHub Release
    needs: [validate, build-release, changelog]
    runs-on: ubuntu-latest
    permissions:
      contents: write
      actions: read
    steps:
      - uses: actions/checkout@v5

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Organize release artifacts
        run: |
          mkdir -p release-assets
          find artifacts/binaries-* -type f -exec cp {} release-assets/ \;
          ls -lh release-assets/

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          name: Release v${{ needs.validate.outputs.version }}
          body: ${{ needs.changelog.outputs.changelog }}
          files: release-assets/*
          draft: false
          prerelease: ${{ contains(needs.validate.outputs.version, '-') }}
          fail_on_unmatched_files: false
          generate_release_notes: true  # GitHub's automatic release notes
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  # Publish to crates.io (Rust-specific)
  publish-crates:
    name: Publish to crates.io
    needs: [validate, create-release]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Verify crate versions match tag
        run: |
          TAG_VERSION="${{ needs.validate.outputs.version }}"

          # Check each crate's version
          for crate_toml in $(find . -name "Cargo.toml" -not -path "*/target/*"); do
            CRATE_VERSION=$(grep "^version" "$crate_toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
            CRATE_NAME=$(grep "^name" "$crate_toml" | head -1 | sed 's/name = "\(.*\)"/\1/')

            echo "Checking $CRATE_NAME: $CRATE_VERSION vs $TAG_VERSION"

            if [ "$CRATE_VERSION" != "$TAG_VERSION" ]; then
              echo "Error: Version mismatch for $CRATE_NAME"
              echo "  Cargo.toml: $CRATE_VERSION"
              echo "  Git tag: $TAG_VERSION"
              exit 1
            fi
          done

      - name: Publish to crates.io
        run: cargo publish --all-features --token ${{ secrets.CARGO_REGISTRY_TOKEN }}
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Verify publication
        run: |
          sleep 10  # Wait for crates.io to update
          cargo search ${{ github.event.repository.name }} --limit 1
```

## Release Management with gh CLI

### Create a Release Manually

```bash
# Get repo info
REPO_INFO=$(gh repo view --json nameWithOwner -q .nameWithOwner)

# Create a tag
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3

# Create release with gh CLI
gh release create v1.2.3 \
  --title "Release v1.2.3" \
  --notes "Release notes here" \
  --verify-tag

# Add binaries to existing release
gh release upload v1.2.3 ./target/release/binary
```

### List and View Releases

```bash
# List all releases
gh release list

# View specific release
gh release view v1.2.3

# Download release assets
gh release download v1.2.3
```

## Version Bumping Strategies

### 1. Manual Version Bump Script

```bash
#!/bin/bash
# bump-version.sh

CURRENT_VERSION=$(grep "^version" Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $CURRENT_VERSION"

read -p "New version: " NEW_VERSION

# Update all Cargo.toml files
find . -name "Cargo.toml" -not -path "*/target/*" -exec sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" {} \;

echo "Updated to $NEW_VERSION"
```

### 2. Automated with cargo-release

```yaml
- name: Install cargo-release
  run: cargo install cargo-release

- name: Bump version and create release
  run: |
    cargo release patch --execute --no-confirm
    # Or: cargo release minor --execute --no-confirm
    # Or: cargo release major --execute --no-confirm
```

## Changelog Management

### Option 1: git-cliff (Automated)

```yaml
- name: Install git-cliff
  run: cargo install git-cliff

- name: Generate changelog
  run: git-cliff --latest --output CHANGELOG.md

- name: Commit changelog
  run: |
    git add CHANGELOG.md
    git commit -m "docs: update changelog for ${{ github.ref_name }}"
    git push
```

### Option 2: Manual CHANGELOG.md

Keep a CHANGELOG.md file following [Keep a Changelog](https://keepachangelog.com/):

```markdown
# Changelog

## [Unreleased]

## [1.2.3] - 2025-11-11

### Added
- New feature X

### Changed
- Modified behavior of Y

### Fixed
- Bug in Z

### Security
- Fixed vulnerability CVE-XXXX
```

## Pre-release and Beta Releases

### Create Pre-release

```yaml
on:
  push:
    tags:
      - 'v*.*.*-beta*'
      - 'v*.*.*-rc*'
      - 'v*.*.*-alpha*'

jobs:
  release:
    steps:
      - name: Create Pre-release
        uses: softprops/action-gh-release@v2
        with:
          prerelease: true
          name: Pre-release ${{ github.ref_name }}
```

### Using gh CLI

```bash
# Create beta release
gh release create v1.2.3-beta.1 \
  --title "Beta Release v1.2.3-beta.1" \
  --notes "Beta release for testing" \
  --prerelease

# Promote pre-release to stable
gh release edit v1.2.3 --draft=false --prerelease=false
```

## Release Branch Strategy

### Git Flow Releases

```yaml
on:
  push:
    branches:
      - 'release/*'

jobs:
  prepare-release:
    steps:
      - name: Extract version from branch
        run: |
          VERSION=${GITHUB_REF#refs/heads/release/}
          echo "VERSION=$VERSION" >> $GITHUB_ENV

      - name: Update version in Cargo.toml
        run: |
          find . -name "Cargo.toml" -not -path "*/target/*" \
            -exec sed -i "s/^version = .*/version = \"$VERSION\"/" {} \;

      - name: Create PR to main
        run: |
          gh pr create \
            --title "Release v$VERSION" \
            --body "Automated release PR for v$VERSION" \
            --base main \
            --head release/$VERSION
```

## Rollback Strategy

### Revert a Release

```bash
# Delete tag locally and remotely
git tag -d v1.2.3
git push origin :refs/tags/v1.2.3

# Delete GitHub release
gh release delete v1.2.3 --yes

# Revert to previous version
gh release view v1.2.2
```

### Create Hotfix Release

```yaml
on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  hotfix:
    if: contains(github.ref, 'hotfix')
    steps:
      - name: Fast-track hotfix release
        run: |
          # Skip some checks for hotfix
          cargo build --release
          # Create release immediately
```

## Security Considerations

### Sign Releases

```yaml
- name: Import GPG key
  run: |
    echo "${{ secrets.GPG_PRIVATE_KEY }}" | gpg --import

- name: Sign release artifacts
  run: |
    for file in release-assets/*; do
      gpg --armor --detach-sign "$file"
    done

- name: Upload signatures
  uses: softprops/action-gh-release@v2
  with:
    files: release-assets/*.asc
```

### Generate SBOMs (Software Bill of Materials)

```yaml
- name: Install cargo-sbom
  run: cargo install cargo-sbom

- name: Generate SBOM
  run: cargo sbom --output-format spdx_json_2_3 > sbom.json

- name: Upload SBOM
  uses: actions/upload-artifact@v4
  with:
    name: sbom
    path: sbom.json
```

## Monitoring Releases

### Track Release Success

```bash
# Check latest release
gh release view --json tagName,publishedAt,assets

# Monitor release downloads
gh release view v1.2.3 --json assets \
  --jq '.assets[] | "\(.name): \(.download_count) downloads"'

# Check if release succeeded
gh run list --workflow=release.yml --limit 5
```

## Best Practices

1. **Always use semantic versioning** (MAJOR.MINOR.PATCH)
2. **Verify version consistency** across all Cargo.toml files
3. **Generate comprehensive changelogs** from git history
4. **Test release process** with pre-releases first
5. **Use gh CLI** to get actual repo owner/name
6. **Sign release artifacts** for security
7. **Include SBOMs** for supply chain transparency
8. **Automate crates.io publishing** only after GitHub release succeeds
9. **Support rollback** strategy for failed releases
10. **Monitor release metrics** (downloads, issues)

## Troubleshooting

### Release Creation Failed

```bash
# Check workflow run
gh run list --workflow=release.yml --limit 1
gh run view <run-id> --log

# Check permissions
gh api repos/{owner}/{repo}/actions/permissions

# Verify token has write access
gh auth status
```

### Version Mismatch

```bash
# Find all version declarations
rg "^version = " --type toml

# Update all versions consistently
find . -name "Cargo.toml" -not -path "*/target/*" \
  -exec sed -i 's/version = ".*"/version = "1.2.3"/' {} \;
```

### Crates.io Publish Failed

```bash
# Verify token
cargo login --token $CARGO_REGISTRY_TOKEN

# Dry run
cargo publish --dry-run

# Check if version already exists
cargo search <crate-name>
```
