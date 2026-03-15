# ADR-045: Publishing Best Practices for crates.io and npm (2026)

- **Status**: Implemented
- **Date**: 2026-03-15
- **Deciders**: Project maintainers
- **Supersedes**: Extends ADR-034 (Release Engineering Modernization)
- **Related**: ADR-029 (GitHub Actions Modernization), ADR-033 (Modern Testing Strategy)

## Context

The rust-self-learning-memory project is approaching stability for public publishing. As the project matures toward v1.0, we need to establish comprehensive best practices for:

1. **Publishing Rust crates** to crates.io for ecosystem consumption
2. **Publishing npm packages** (potentially via WASM builds) for JavaScript/TypeScript users
3. **Cross-publishing** workflows that maintain version parity and quality

The publishing landscape has evolved significantly in 2025-2026 with new security requirements, provenance attestations, and automation tools.

### Current State

| Aspect | Status | Notes |
|--------|--------|-------|
| Workspace version | v0.1.20 | Single version for all crates |
| Cargo.toml metadata | Partial | Core has description, keywords, categories |
| CI/CD for publishing | None | Manual releases only |
| crates.io presence | None | Not published yet |
| npm presence | None | No WASM builds yet |
| Security attestations | None | No provenance, SBOM |

### Project Crates Analysis

| Crate | Publish Priority | Reason |
|-------|------------------|--------|
| `memory-core` | HIGH | Stable public API, core abstractions |
| `memory-storage-turso` | HIGH | Reference storage backend |
| `memory-storage-redb` | HIGH | Pure-Rust embedded backend |
| `memory-mcp` | MEDIUM | MCP server implementation |
| `memory-cli` | LOW | CLI tool, distributed as binary |
| `test-utils` | NEVER | Internal testing only |

---

## Decision

### Part 1: crates.io Publishing Best Practices (2026)

#### 1.1 Version Management and Semver Compliance

**Semantic Versioning Policy**:

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]

- MAJOR: Breaking API changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible
- PRERELEASE: alpha, beta, rc (e.g., 1.0.0-beta.1)
```

**Workspace Version Strategy**:

For this project, we use a **single version for all workspace crates** with version inheritance:

```toml
# workspace Cargo.toml
[workspace.package]
version = "0.2.0"  # All crates inherit

# Individual crate Cargo.toml
[package]
version.workspace = true
```

**Version Bumping Rules**:

| Change Type | Bump | Example |
|-------------|------|---------|
| Breaking API change | MAJOR | 0.1.x → 0.2.0 (pre-1.0) or 1.x → 2.0.0 |
| New feature | MINOR | 0.1.20 → 0.2.0 (pre-1.0) or 1.0.0 → 1.1.0 |
| Bug fix | PATCH | 0.1.19 → 0.1.20 |
| Pre-release | PRERELEASE | 1.0.0-alpha.1 → 1.0.0-alpha.2 |

**Pre-1.0 Semver Note**: Before 1.0, breaking changes may be shipped in MINOR versions per Cargo convention. However, we will use `cargo-semver-checks` to detect and flag breaking changes.

#### 1.2 Cargo.toml Metadata Requirements

**Required Fields** (for crates.io acceptance):

```toml
[package]
name = "memory-core"
version = "0.2.0"
description = "Core episodic learning system for AI agents with pattern extraction"
license = "MIT"
repository = "https://github.com/d-o-hub/rust-self-learning-memory"
edition = "2024"
```

**Strongly Recommended Fields**:

```toml
[package]
# Documentation
documentation = "https://docs.rs/memory-core"
readme = "README.md"

# Discovery
keywords = ["ai", "memory", "learning", "episodic", "patterns"]
categories = ["database", "development-tools", "science"]

# Links
homepage = "https://github.com/d-o-hub/rust-self-learning-memory"

# Package optimization
exclude = [
    "tests/*",
    "benches/*",
    "examples/*",
    ".github/*",
    ".*",
]
include = [
    "src/**/*.rs",
    "Cargo.toml",
    "README.md",
    "LICENSE",
]
```

**Metadata Verification Script**:

```bash
#!/bin/bash
# scripts/verify-crate-metadata.sh
# Verifies all required metadata before publishing

for crate in memory-core memory-storage-turso memory-storage-redb memory-mcp; do
    echo "Checking $crate..."
    cargo metadata --format-version 1 --no-deps | jq -r ".packages[] | select(.name == \"$crate\") | {
        name: .name,
        version: .version,
        description: .description // \"MISSING\",
        license: .license // \"MISSING\",
        repository: .repository // \"MISSING\",
        documentation: .documentation // \"MISSING\",
        readme: .readme // \"MISSING\",
        keywords: .keywords | length,
        categories: .categories | length
    }"
done
```

#### 1.3 CI/CD Integration for Automated Publishing

**Trusted Publishing with OIDC** (recommended approach):

```yaml
# .github/workflows/publish-crates.yml
name: Publish to crates.io

on:
  release:
    types: [published]
  workflow_dispatch:
    inputs:
      dry-run:
        description: 'Dry run (no actual publish)'
        required: false
        default: 'true'
        type: boolean

permissions:
  contents: read
  id-token: write  # Required for OIDC

jobs:
  publish:
    name: Publish ${{ matrix.crate }}
    runs-on: ubuntu-latest
    strategy:
      matrix:
        crate:
          - memory-core
          - memory-storage-turso
          - memory-storage-redb
          - memory-mcp
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-semver-checks
        run: cargo install --locked cargo-semver-checks

      - name: Semver Check
        run: cargo semver-checks --package ${{ matrix.crate }}

      - name: Verify metadata
        run: |
          cargo metadata --format-version 1 --no-deps | \
          jq -e ".packages[] | select(.name == \"${{ matrix.crate }}\") | .description" > /dev/null

      - name: Dry Run Publish
        if: ${{ inputs.dry-run }}
        run: cargo publish --package ${{ matrix.crate }} --dry-run

      - name: Publish to crates.io
        if: ${{ !inputs.dry-run }}
        uses: rust-lang/crates-io-auth-action@v1
        with:
          crate: ${{ matrix.crate }}
```

**Alternative: Token-based Publishing**:

```yaml
# For projects without OIDC support
- name: Publish to crates.io
  if: ${{ !inputs.dry-run }}
  run: cargo publish --package ${{ matrix.crate }} --token ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

#### 1.4 Release Automation Tools

**Recommended: cargo-release**

`cargo-release` provides the most comprehensive release workflow:

```toml
# release.toml (workspace root)
[workspace]
# Version management
allow-branch = ["main"]
consolidate-commits = true
consolidate-pushes = true

# Git settings
sign-commit = false
sign-tag = false
push = true

# Publishing (disabled until ready)
publish = false
registry = "crates-io"

# Release commit message
pre-release-commit-message = "release({{crate_name}}): v{{version}}"
tag-message = "{{crate_name}} v{{version}}"
tag-name = "{{crate_name}}-v{{version}}"

# Changelog integration
pre-release-hook = ["./scripts/update-changelog.sh"]

[[package.pre-release-replacements]]
file = "CHANGELOG.md"
search = "## \\[Unreleased\\]"
replace = "## [Unreleased]\n\n## [{{version}}] - {{date}}"

# Individual crate settings
[package.memory-core]
publish = true

[package.memory-storage-turso]
publish = true

[package.memory-storage-redb]
publish = true

[package.memory-mcp]
publish = true

[package.memory-cli]
publish = false  # Distributed as binary

[package.test-utils]
publish = false  # Internal use only
```

**Alternative: release-plz**

For GitHub-native release automation:

```toml
# release-plz.toml
[[releaser]]
name = "memory-core"
[[releaser]]
name = "memory-storage-turso"
[[releaser]]
name = "memory-storage-redb"
[[releaser]]
name = "memory-mcp"

[workspace]
changelog_update = true
```

```yaml
# .github/workflows/release-plz.yml
name: Release Plz

on:
  push:
    branches: [main]

permissions:
  contents: write
  pull-requests: write

jobs:
  release-plz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0

      - name: Run release-plz
        uses: MarcoIeni/release-plz-action@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

#### 1.5 Registry Verification and Security

**Supply Chain Security Checks**:

```yaml
# .github/workflows/supply-chain.yml
name: Supply Chain Security

on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 6 * * *'  # Daily at 6 UTC

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Security Audit
        run: cargo install --locked cargo-audit && cargo audit

      - name: License Check
        run: cargo install --locked cargo-deny && cargo deny check licenses

      - name: Advisory Check
        run: cargo deny check advisories

      - name: Bans and Restrictions
        run: cargo deny check bans sources

  sbom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Generate SBOM
        run: |
          cargo install --locked cargo-cyclonedx
          cargo cyclonedx --all --output-pattern crate

      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: "*.cdx.json"
```

**cargo-deny Configuration**:

```toml
# deny.toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"
notice = "warn"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "MPL-2.0"]
deny = ["GPL-3.0", "AGPL-3.0"]
copyleft = "warn"
allow-osi-fsf-free = "both"
default = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"
highlight = "all"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

#### 1.6 Documentation Hosting (docs.rs)

**Optimizing for docs.rs**:

```toml
# Cargo.toml
[package]
documentation = "https://docs.rs/memory-core"

[package.metadata.docs.rs]
# All features for complete documentation
all-features = true
# Document private items (optional)
rustdoc-args = ["--cfg", "docsrs"]

# Feature-specific documentation
[package.metadata.docs.rs.features]
default = []
full = ["embeddings-full", "turso", "redb"]
```

**Documentation Best Practices**:

```rust
//! Crate-level documentation (lib.rs)
//!
//! # memory-core
//!
//! Core episodic learning system for AI agents with pattern extraction,
//! reward scoring, and dual storage backend support.
//!
//! ## Features
//!
//! - Episode creation and lifecycle management
//! - Pattern extraction with BOCPD changepoint detection
//! - Dual storage backends (Turso and redb)
//! - Embedding providers (OpenAI, Mistral, local)
//!
//! ## Quick Start
//!
//! ```rust
//! use memory_core::{EpisodeStorage, EpisodeConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = EpisodeConfig::default();
//! // ... usage example
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! - `openai` - Enable OpenAI embedding provider
//! - `mistral` - Enable Mistral AI embedding provider
//! - `local-embeddings` - Enable local ONNX embedding models
//! - `embeddings-full` - Enable all embedding providers

/// Module-level documentation
///
/// Provides the core episode abstraction and lifecycle management.
pub mod episode;

/// Storage traits and implementations.
///
/// This module defines the [`EpisodeStorage`] trait and provides
/// reference implementations for Turso and redb backends.
pub mod storage;
```

**Documentation Quality Checklist**:

- [ ] Crate-level documentation with overview and examples
- [ ] All public items have `///` documentation
- [ ] Examples use `#![doc(html_playground_url)]` for runnable examples
- [ ] Feature flags documented in crate root
- [ ] Safety documentation for `unsafe` code
- [ ] Error conditions documented in `# Errors` sections
- [ ] Panic conditions documented in `# Panics` sections

#### 1.7 Feature Flags for Published Crates

**Feature Design Principles**:

1. **Additive only**: Features should only add functionality, never remove
2. **No breaking changes**: Disabling a feature should not break compilation
3. **Sensible defaults**: `default` features should work out-of-the-box
4. **Clear naming**: Use descriptive names like `openai` not `feat1`

**Example Feature Configuration**:

```toml
# memory-core/Cargo.toml
[features]
default = []

# Embedding providers
openai = ["reqwest"]
mistral = ["reqwest"]
embeddings-full = ["openai", "mistral"]
local-embeddings = ["ort", "tokenizers", "ndarray", "reqwest/stream"]

# Storage backends
turso = ["libsql"]
redb-backend = ["redb"]
storage-full = ["turso", "redb-backend"]

# Search features
hybrid-search = []

# Testing utilities
proptest-arbitrary = ["proptest"]

# Full feature set
full = ["embeddings-full", "storage-full", "hybrid-search"]
```

**Feature Documentation**:

```rust
/// Configuration for embedding providers.
///
/// # Features
///
/// - `openai`: Enables OpenAI embedding provider. Requires `OPENAI_API_KEY`.
/// - `mistral`: Enables Mistral AI embedding provider. Requires `MISTRAL_API_KEY`.
/// - `local-embeddings`: Enables local ONNX models. Requires model files.
/// - `embeddings-full`: Enables all embedding providers.
///
/// # Example
///
/// ```rust,ignore
/// #[cfg(feature = "openai")]
/// let provider = OpenAIEmbeddings::new()?;
/// ```
pub struct EmbeddingConfig {
    // ...
}
```

---

### Part 2: npm Publishing Best Practices (2026)

#### 2.1 Package.json Metadata Standards

**Essential Fields**:

```json
{
  "name": "@d-o/memory-core",
  "version": "0.2.0",
  "description": "Episodic learning system for AI agents - WASM bindings",
  "license": "MIT",
  "author": "Self-Learning Memory Contributors",
  "repository": {
    "type": "git",
    "url": "https://github.com/d-o-hub/rust-self-learning-memory.git"
  },
  "bugs": {
    "url": "https://github.com/d-o-hub/rust-self-learning-memory/issues"
  },
  "homepage": "https://github.com/d-o-hub/rust-self-learning-memory#readme",
  "keywords": ["ai", "memory", "wasm", "learning", "episodic"],
  "type": "module",
  "main": "./dist/index.js",
  "module": "./dist/index.mjs",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.js",
      "types": "./dist/index.d.ts"
    },
    "./wasm": {
      "import": "./dist/memory_core_bg.wasm",
      "types": "./dist/memory_core_bg.d.ts"
    }
  },
  "files": [
    "dist/**/*",
    "README.md",
    "LICENSE"
  ],
  "sideEffects": false,
  "engines": {
    "node": ">=18.0.0"
  },
  "publishConfig": {
    "access": "public",
    "provenance": true
  }
}
```

**Field Explanations**:

| Field | Purpose | Notes |
|-------|---------|-------|
| `name` | Package identifier | Scoped for organization |
| `exports` | Module resolution | ESM + CJS compatibility |
| `sideEffects` | Tree-shaking | `false` enables aggressive optimization |
| `engines` | Runtime requirements | Prevents wrong Node versions |
| `publishConfig.provenance` | Security attestation | Required for npm provenance |

#### 2.2 Version Management

**npm version commands**:

```bash
# Patch release (0.2.0 → 0.2.1)
npm version patch -m "chore(release): %s"

# Minor release (0.2.1 → 0.3.0)
npm version minor -m "chore(release): %s"

# Major release (0.3.0 → 1.0.0)
npm version major -m "chore(release): %s"

# Pre-release (1.0.0 → 1.0.1-beta.0)
npm version prerelease --preid beta

# With git tag signing
npm version patch -m "chore(release): %s" --sign-git-tag
```

**Lifecycle Hooks**:

```json
{
  "scripts": {
    "preversion": "npm test && npm run lint",
    "version": "npm run build && git add dist/",
    "postversion": "git push --follow-tags"
  }
}
```

#### 2.3 CI/CD Workflows for npm Publishing

**Trusted Publishing with OIDC**:

```yaml
# .github/workflows/publish-npm.yml
name: Publish to npm

on:
  release:
    types: [published]
  workflow_dispatch:

permissions:
  contents: read
  id-token: write  # Required for npm OIDC

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '22'
          registry-url: 'https://registry.npmjs.org'

      - name: Install dependencies
        run: npm ci

      - name: Build WASM
        run: |
          cargo install wasm-pack
          wasm-pack build --release --target web --out-dir dist

      - name: Verify package
        run: |
          npm install -g publint @arethetypeswrong/cli
          publint
          attw pack --profile=minimal

      - name: Publish to npm
        run: npm publish --provenance --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

#### 2.4 npm Provenance and Security Attestations

**Provenance Statement** (2025+ requirement):

npm provenance links published packages to their source code and build process:

```json
{
  "publishConfig": {
    "access": "public",
    "provenance": true
  }
}
```

**What Provenance Provides**:

1. **Build transparency**: Links package to source commit
2. **Supply chain integrity**: Verifies build environment
3. **Reproducibility**: Documents exact build process
4. **Sigstore integration**: Cryptographic signatures

**Verifying Provenance**:

```bash
# View provenance for a published package
npm view @d-o/memory-core provenance

# Verify provenance during install
npm install @d-o/memory-core --verify-provenance
```

**SBOM Generation**:

```yaml
# In CI workflow
- name: Generate SBOM
  run: |
    npm install -g @cyclonedx/cyclonedx-npm
    cyclonedx-npm --output-file sbom.json

- name: Upload SBOM
  uses: actions/upload-artifact@v4
  with:
    name: npm-sbom
    path: sbom.json
```

#### 2.5 Scoped vs Unscoped Packages

**Recommendation: Use Scoped Packages**

```json
// Scoped package (recommended)
{
  "name": "@d-o/memory-core"
}

// Unscoped package (not recommended for orgs)
{
  "name": "memory-core"
}
```

**Benefits of Scoping**:

| Benefit | Description |
|---------|-------------|
| Namespace collision | Avoids conflicts with similarly-named packages |
| Organization | Groups all packages under `@d-o/` |
| Permissions | Easier to manage team publish access |
| Branding | Clear ownership and source |

#### 2.6 TypeScript Declarations

**Generating Type Declarations**:

```json
{
  "scripts": {
    "build": "wasm-pack build --release --target web && npm run build:types",
    "build:types": "tsc --emitDeclarationOnly --declaration --outDir dist"
  },
  "devDependencies": {
    "typescript": "^5.7.0"
  }
}
```

**TypeScript Configuration**:

```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts"],
  "exclude": ["node_modules", "dist"]
}
```

**Type Declaration Example**:

```typescript
// dist/index.d.ts
export class EpisodeStorage {
  constructor(config: EpisodeConfig);
  
  /**
   * Creates a new episode in storage.
   * @param context - The episode context
   * @returns Promise resolving to the episode ID
   */
  createEpisode(context: EpisodeContext): Promise<string>;
  
  /**
   * Retrieves an episode by ID.
   * @param id - The episode ID
   * @returns Promise resolving to the episode, or null if not found
   */
  getEpisode(id: string): Promise<Episode | null>;
}

export interface EpisodeConfig {
  maxPatterns?: number;
  rewardThreshold?: number;
}

export interface Episode {
  id: string;
  context: EpisodeContext;
  steps: EpisodeStep[];
  patterns: Pattern[];
  reward: number;
}
```

#### 2.7 Dual Publishing (ESM + CJS)

**Package Exports Configuration**:

```json
{
  "type": "module",
  "main": "./dist/index.cjs",
  "module": "./dist/index.mjs",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      },
      "require": {
        "types": "./dist/index.d.cts",
        "default": "./dist/index.cjs"
      },
      "default": "./dist/index.mjs"
    },
    "./package.json": "./package.json"
  }
}
```

**Build Script for Dual Output**:

```json
{
  "scripts": {
    "build:esm": "tsc --module ESNext --outDir dist/esm",
    "build:cjs": "tsc --module CommonJS --outDir dist/cjs",
    "build": "npm run build:esm && npm run build:cjs && node scripts/fix-exports.js"
  }
}
```

#### 2.8 Package Verification

**publint** - Package linter:

```bash
npm install -g publint
publint

# Common issues it catches:
# - Missing README
# - Missing LICENSE
# - Invalid exports field
# - Missing type definitions
# - Incorrect file paths
```

**@arethetypeswrong/cli** - Type checking:

```bash
npm install -g @arethetypeswrong/cli
attw pack

# Checks:
# - ESM/CJS type compatibility
# - Correct type declaration paths
# - Missing type exports
```

**Package Size Analysis**:

```bash
# Check package size before publishing
npm install -g pkg-size
pkg-size --publish

# Alternative: bundlephobia CLI
npx bundlephobia memory-core-wasm
```

---

### Part 3: Cross-Publishing Considerations

#### 3.1 WASM Builds for npm

**wasm-pack Configuration**:

```toml
# Cargo.toml (WASM crate)
[package]
name = "memory-core-wasm"
version.workspace = true
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2.100"
js-sys = "0.3.77"
serde-wasm-bindgen = "0.6.5"
wasm-bindgen-futures = "0.4.50"

[dependencies.web-sys]
version = "0.3.77"
features = [
  "console",
]

[profile.release]
opt-level = "s"
lto = true
```

**wasm-bindgen API Design**:

```rust
// src/lib.rs
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;

/// Main entry point for the WASM module
#[wasm_bindgen]
pub fn init() {
    // Initialize panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Episode storage WASM wrapper
#[wasm_bindgen]
pub struct WasmEpisodeStorage {
    inner: EpisodeStorage,
}

#[wasm_bindgen]
impl WasmEpisodeStorage {
    /// Creates a new episode storage instance
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WasmEpisodeStorage, JsValue> {
        let config: EpisodeConfig = serde_wasm_bindgen::from_value(config)?;
        let inner = EpisodeStorage::new(config)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self { inner })
    }

    /// Creates an episode
    pub async fn create_episode(&self, context: JsValue) -> Result<String, JsValue> {
        let context: EpisodeContext = serde_wasm_bindgen::from_value(context)?;
        self.inner.create_episode(context)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

**Build Script**:

```bash
#!/bin/bash
# scripts/build-wasm.sh

set -e

# Build for web target
wasm-pack build \
    --release \
    --target web \
    --out-dir npm/dist \
    --scope d-o \
    memory-core-wasm

# Optimize WASM binary
wasm-opt -Oz \
    npm/dist/memory_core_wasm_bg.wasm \
    -o npm/dist/memory_core_wasm_bg.wasm

# Generate TypeScript declarations
# (handled by wasm-pack automatically)
```

#### 3.2 Version Synchronization

**Single Source of Truth Strategy**:

```toml
# workspace Cargo.toml
[workspace.package]
version = "0.2.0"
```

```json
// package.json - synced from Cargo.toml
{
  "version": "0.2.0"
}
```

**Automated Version Sync Script**:

```bash
#!/bin/bash
# scripts/sync-versions.sh

# Extract version from workspace Cargo.toml
CARGO_VERSION=$(grep -m1 'version = "' Cargo.toml | cut -d'"' -f2)

# Update package.json
jq --arg v "$CARGO_VERSION" '.version = $v' npm/package.json > npm/package.json.tmp
mv npm/package.json.tmp npm/package.json

echo "Synchronized versions to $CARGO_VERSION"
```

**CI Version Check**:

```yaml
# .github/workflows/version-check.yml
name: Version Sync Check

on:
  pull_request:
    paths:
      - 'Cargo.toml'
      - 'npm/package.json'

jobs:
  check-versions:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Check version sync
        run: |
          CARGO_VERSION=$(grep -m1 'version = "' Cargo.toml | cut -d'"' -f2)
          NPM_VERSION=$(jq -r '.version' npm/package.json)
          
          if [ "$CARGO_VERSION" != "$NPM_VERSION" ]; then
            echo "Version mismatch: Cargo=$CARGO_VERSION, npm=$NPM_VERSION"
            exit 1
          fi
          echo "Versions synchronized: $CARGO_VERSION"
```

#### 3.3 Changelog Management

**Keep a Changelog Format**:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Placeholder for upcoming features

## [0.2.0] - 2026-03-15

### Added
- WASM bindings for JavaScript/TypeScript consumers
- npm package with TypeScript declarations
- Cross-platform binary releases via cargo-dist

### Changed
- Updated to Rust 2024 edition
- Upgraded all dependencies to latest versions

### Fixed
- Race condition in concurrent episode creation

## [0.1.20] - 2026-02-21

### Added
- Hybrid search with FTS5 support

### Security
- Updated dependencies per RUSTSEC advisories
```

**Automated Changelog Generation**:

```yaml
# .github/workflows/changelog.yml
name: Generate Changelog

on:
  push:
    tags:
      - 'v*'

jobs:
  changelog:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0

      - name: Generate Changelog
        uses: orhun/git-cliff-action@v4
        with:
          config: cliff.toml
          args: --verbose --tag ${{ github.ref_name }}
        env:
          OUTPUT: CHANGELOG.md

      - name: Commit Changelog
        run: |
          git config user.name 'github-actions[bot]'
          git config user.email 'github-actions[bot]@users.noreply.github.com'
          git add CHANGELOG.md
          git commit -m "docs: update changelog for ${{ github.ref_name }}"
          git push
```

**git-cliff Configuration**:

```toml
# cliff.toml
[changelog]
header = """# Changelog\n\n"""
body = """
{% if version %}\
    ## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else %}\
    ## [Unreleased]
{% endif %}\
{% for commit in commits %}
    * {{ commit.message | upper_first }}{% if commit.breaking %} (BREAKING){% endif %}
{% endfor %}

"""
trim = true

[git]
conventional_commits = true
filter_unconventional = false
split_commits = true
commit_preprocessors = []
commit_parsers = [
    { message = "^feat", group = "Added" },
    { message = "^fix", group = "Fixed" },
    { message = "^doc", group = "Documentation" },
    { message = "^perf", group = "Performance" },
    { message = "^refactor", group = "Changed" },
    { message = "^style", group = "Styling" },
    { message = "^test", group = "Testing" },
    { message = "^chore\\(release\\):", skip = true },
    { message = "^chore", group = "Miscellaneous" },
]
```

---

### Part 4: Security Best Practices

#### 4.1 Token Management

**GitHub Secrets Configuration**:

| Secret Name | Purpose | Rotation |
|-------------|---------|----------|
| `CARGO_REGISTRY_TOKEN` | crates.io publish token | Every 90 days |
| `NPM_TOKEN` | npm publish token | Every 90 days |
| `CARGO_PUBLISH_KEY` | Optional: GPG signing key | Annually |

**Token Scoping**:

```yaml
# Use environment-specific secrets
jobs:
  publish-crates:
    environment: crates-io
    steps:
      - name: Publish
        run: cargo publish --token ${{ secrets.CARGO_REGISTRY_TOKEN }}

  publish-npm:
    environment: npm
    steps:
      - name: Publish
        run: npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

#### 4.2 Trusted Publishing (OIDC)

**crates.io OIDC Setup**:

```yaml
# No API token needed - uses GitHub OIDC
permissions:
  id-token: write
  contents: read

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - name: Publish with OIDC
        uses: rust-lang/crates-io-auth-action@v1
        with:
          crate: memory-core
```

**npm OIDC Setup**:

```yaml
- name: Setup Node.js for OIDC
  uses: actions/setup-node@v4
  with:
    node-version: '22'
    registry-url: 'https://registry.npmjs.org'

- name: Publish with provenance
  run: npm publish --provenance
```

#### 4.3 Supply Chain Security

**Security Gates Before Publishing**:

```yaml
# .github/workflows/security-gates.yml
name: Security Gates

on:
  push:
    branches: [main]
  pull_request:

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Cargo Audit
        run: cargo install --locked cargo-audit && cargo audit

      - name: Cargo Deny
        run: cargo install --locked cargo-deny && cargo deny check all

      - name: Semver Check
        run: cargo install --locked cargo-semver-checks && cargo semver-checks

      - name: Check Dependencies
        run: cargo tree --duplicates
```

#### 4.4 SBOM Generation

**CycloneDX for Rust**:

```yaml
- name: Generate Rust SBOM
  run: |
    cargo install --locked cargo-cyclonedx
    cargo cyclonedx --all --format json --output-pattern crate

- name: Upload SBOM
  uses: actions/upload-artifact@v4
  with:
    name: rust-sbom
    path: "*.cdx.json"
```

**CycloneDX for npm**:

```yaml
- name: Generate npm SBOM
  run: |
    npm install -g @cyclonedx/cyclonedx-npm
    cyclonedx-npm --output-file sbom.json

- name: Upload SBOM
  uses: actions/upload-artifact@v4
  with:
    name: npm-sbom
    path: sbom.json
```

#### 4.5 Audit Trails

**Release Audit Log**:

```yaml
# .github/workflows/release-audit.yml
name: Release Audit

on:
  release:
    types: [published]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - name: Record Release
        run: |
          echo "## Release Audit Log" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "- **Timestamp**: $(date -u +"%Y-%m-%dT%H:%M:%SZ")" >> $GITHUB_STEP_SUMMARY
          echo "- **Version**: ${{ github.event.release.tag_name }}" >> $GITHUB_STEP_SUMMARY
          echo "- **Publisher**: ${{ github.actor }}" >> $GITHUB_STEP_SUMMARY
          echo "- **Workflow**: ${{ github.workflow }}" >> $GITHUB_STEP_SUMMARY
          echo "- **Run ID**: ${{ github.run_id }}" >> $GITHUB_STEP_SUMMARY

      - name: Generate Attestation
        uses: actions/attest-build-provenance@v2
        with:
          subject-path: dist/*
          push-to-registry: true
```

---

### Part 5: Release Workflow

#### 5.1 Pre-Release Checklist

**Automated Pre-Release Checks**:

```yaml
# .github/workflows/pre-release.yml
name: Pre-Release Checks

on:
  workflow_dispatch:
    inputs:
      version-bump:
        description: 'Version bump type'
        required: true
        type: choice
        options:
          - patch
          - minor
          - major

jobs:
  pre-release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Quality Gates
        run: |
          echo "Running pre-release quality gates..."

          # Format check
          cargo fmt --all -- --check

          # Clippy with warnings as errors
          cargo clippy --all -- -D warnings

          # Build all crates
          cargo build --all

          # Run all tests
          cargo nextest run --all

          # Run doctests
          cargo test --doc --all

          echo "All quality gates passed!"

      - name: Security Audit
        run: |
          cargo install --locked cargo-audit
          cargo audit

      - name: Semver Check
        run: |
          cargo install --locked cargo-semver-checks
          cargo semver-checks

      - name: Verify Metadata
        run: ./scripts/verify-crate-metadata.sh

      - name: Check Uncommitted Changes
        run: |
          if [ -n "$(git status --porcelain)" ]; then
            echo "ERROR: Uncommitted changes detected"
            git status
            exit 1
          fi

      - name: Report Success
        run: |
          echo "## Pre-Release Checks Passed" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "Ready to proceed with release." >> $GITHUB_STEP_SUMMARY
```

#### 5.2 Automated Quality Gates

**Quality Gate Matrix**:

```yaml
# .github/workflows/quality-gates.yml
name: Quality Gates

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest
      - run: cargo nextest run --all
      - run: cargo test --doc --all

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - run: cargo install --locked cargo-audit && cargo audit
      - run: cargo install --locked cargo-deny && cargo deny check all

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-llvm-cov
      - run: cargo llvm-cov --workspace --ignore-filename-regex "(tests|benches)" --lcov --output-path lcov.info
      - name: Coverage Check
        run: |
          cargo install --locked cargo-llvm-cov
          COVERAGE=$(cargo llvm-cov --workspace --ignore-filename-regex "(tests|benches)" --summary-only 2>&1 | grep -oP '\d+\.\d+%' | head -1)
          echo "Coverage: $COVERAGE"
          if (( $(echo "$COVERAGE < 90.0" | bc -l) )); then
            echo "ERROR: Coverage below 90%"
            exit 1
          fi

  gates-passed:
    needs: [lint, test, security, coverage]
    runs-on: ubuntu-latest
    steps:
      - run: echo "All quality gates passed!"
```

#### 5.3 Post-Publish Verification

**Automated Verification**:

```yaml
# .github/workflows/post-publish.yml
name: Post-Publish Verification

on:
  release:
    types: [published]

jobs:
  verify-crates-io:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        crate:
          - memory-core
          - memory-storage-turso
          - memory-storage-redb
    steps:
      - name: Wait for registry propagation
        run: sleep 60

      - name: Verify crate on crates.io
        run: |
          cargo install ${{ matrix.crate }} --version ${{ github.event.release.tag_name }}
          cargo test --package ${{ matrix.crate }}

      - name: Verify docs.rs
        run: |
          curl -sSf "https://docs.rs/${{ matrix.crate }}/${{ github.event.release.tag_name }}" > /dev/null
          echo "docs.rs documentation available"

  verify-npm:
    runs-on: ubuntu-latest
    steps:
      - name: Wait for npm propagation
        run: sleep 30

      - name: Verify npm package
        run: |
          npm install @d-o/memory-core@${{ github.event.release.tag_name }}
          npm test

  notify:
    needs: [verify-crates-io, verify-npm]
    runs-on: ubuntu-latest
    steps:
      - name: Success Notification
        run: |
          echo "## Release Verified" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "All packages successfully published and verified." >> $GITHUB_STEP_SUMMARY
```

#### 5.4 Rollback Procedures

**Yanking Published Crates**:

```bash
# Yank a specific version (removes from index, existing users unaffected)
cargo yank --vers 0.2.0 memory-core

# Unyank if mistake was made
cargo yank --vers 0.2.0 --undo memory-core

# Note: Yanking doesn't delete, just prevents new installations
```

**npm Deprecation**:

```bash
# Deprecate a version (users see warning)
npm deprecate @d-o/memory-core@0.2.0 "Critical bug, please upgrade"

# Deprecate all versions below a threshold
npm deprecate @d-o/memory-core@"<0.2.1" "Security vulnerability, upgrade required"
```

**Emergency Release Process**:

```yaml
# .github/workflows/emergency-release.yml
name: Emergency Release

on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release'
        required: true

jobs:
  emergency-release:
    runs-on: ubuntu-latest
    environment: emergency-release
    steps:
      - uses: actions/checkout@v6

      - name: Quick Security Audit
        run: cargo audit

      - name: Build
        run: cargo build --release

      - name: Publish (no semver checks)
        run: cargo publish --token ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Notify Team
        run: |
          echo "Emergency release ${{ inputs.version }} published"
          # Integration with Slack/PagerDuty here
```

---

## Implementation Plan

| Phase | Action | Timeline | Dependencies |
|-------|--------|----------|--------------|
| 1 | Add Cargo.toml metadata to all publishable crates | Week 1 | None |
| 2 | Create verify-crate-metadata.sh script | Week 1 | Phase 1 |
| 3 | Add supply-chain.yml workflow | Week 1 | None |
| 4 | Configure cargo-deny with deny.toml | Week 2 | None |
| 5 | Add pre-release.yml workflow | Week 2 | Phase 3, 4 |
| 6 | Create release.toml for cargo-release | Week 2 | None |
| 7 | Add post-publish verification workflow | Week 3 | Phase 5 |
| 8 | Set up crates.io trusted publishing (OIDC) | Week 3 | None |
| 9 | First dry-run publish to crates.io | Week 3 | Phase 1-8 |
| 10 | Configure WASM build pipeline (optional) | Week 4-6 | Phase 9 |
| 11 | Set up npm trusted publishing (OIDC) | Week 4-6 | Phase 10 |
| 12 | First dry-run publish to npm | Week 4-6 | Phase 10, 11 |
| 13 | Document release process in agent_docs/ | Week 7 | Phase 1-12 |
| 14 | First production release (v0.2.0) | Week 7+ | All phases |

---

## Consequences

### Positive

1. **Supply Chain Security**: Provenance, SBOMs, and OIDC eliminate token-based attacks
2. **Reproducibility**: Automated versioning and changelog generation ensure consistency
3. **Trust**: Users can verify package integrity and source
4. **Discoverability**: Complete metadata improves crate/npm visibility
5. **Automation**: Reduced manual effort and human error in releases

### Negative

1. **Complexity**: Multiple CI workflows and tools to maintain
2. **CI Time**: Additional security and verification steps increase build time
3. **Learning Curve**: Team must learn cargo-release, wasm-pack, and provenance concepts
4. **Token Management**: Periodic rotation of secrets still required (for non-OIDC)

### Risks

1. **OIDC Availability**: GitHub Actions outage could block publishing
2. **Registry Propagation**: crates.io/npm may have delays affecting verification
3. **Feature Flag Complexity**: Many features increase testing burden
4. **WASM Size**: Large WASM binaries may deter npm users

---

## References

- [cargo-release Documentation](https://github.com/crate-ci/cargo-release)
- [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
- [crates.io Trusted Publishing](https://blog.rust-lang.org/2024/12/10/crates-io-trusted-publishing.html)
- [npm Provenance](https://docs.npmjs.com/generating-software-bill-of-materials)
- [wasm-pack Book](https://rustwasm.github.io/wasm-pack/)
- [git-cliff](https://git-cliff.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [publint](https://publint.dev/)
- [ARETHETYPESWRONG](https://github.com/arethetypeswrong/arethetypeswrong.github.io)
- [CycloneDX for Rust](https://github.com/CycloneDX/cyclonedx-rust-cargo)
- [GitHub Attestations](https://github.com/actions/attest-build-provenance)

---

## Related ADRs

- **ADR-029**: GitHub Actions Modernization
- **ADR-034**: Release Engineering Modernization
- **ADR-033**: Modern Testing Strategy
- **ADR-031**: Cargo Lock Integrity for Security Audit
