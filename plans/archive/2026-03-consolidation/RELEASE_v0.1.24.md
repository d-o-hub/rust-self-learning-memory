# Release Plan — v0.1.24

- **Date**: 2026-03-31
- **Previous Release**: v0.1.23 (2026-03-25)
- **Branch**: `main`
- **Workspace Version**: 0.1.23 → bump to 0.1.24
- **GitHub Issue**: #401 (Dispatch discoverability — external, low priority)

---

## Pre-Release: Already Merged to `main`

| PR | Description | Status |
|----|-------------|--------|
| #404 | fix: v0.1.24 test stability (DBSCAN budget, quality gate timeout) | ✅ Merged |
| #402 | ci(deps): actions/checkout v6, codecov v6, wait-on-check v1.6 | ✅ Merged |
| #403 | chore(deps): 9 Rust patch-minor crate bumps | ✅ Merged |
| #397 | chore(deps): toml bump | ✅ Merged |

---

## Release Tasks (Ordered)

### Task 1: Version Bump (REQUIRED)

- [ ] Update `workspace.package.version` in root `Cargo.toml`: `0.1.23` → `0.1.24`
- [ ] Run `cargo check` to propagate
- [ ] Commit: `chore: bump version to 0.1.24`

### Task 2: CHANGELOG.md Update (REQUIRED)

**Gap**: CHANGELOG.md stops at v0.1.19. Missing entries for v0.1.20, v0.1.21, v0.1.22, v0.1.23, and v0.1.24.

- [ ] Add `## [0.1.24] - 2026-03-31` section with:
  - Fixed: DBSCAN benchmark time budget (60s→120s)
  - Fixed: quality_gate_performance_regression timeout (added `#[ignore]`)
  - Updated: actions/checkout v4→v6, codecov/codecov-action v5→v6
  - Updated: 9 Rust dependency patch-minor bumps
  - Updated: plans/ folder synced to v0.1.23/v0.1.24 reality
- [ ] Add `## [0.1.23] - 2026-03-25` section with:
  - Added: Durable recommendation attribution (Turso/redb storage, WG-051)
  - Added: Durable checkpoint/handoff persistence (WG-052)
  - Fixed: MCP batch tool contract aligned (WG-053)
  - Fixed: CI test scope expanded from `--lib` to workspace (WG-055)
  - Fixed: Coverage enforcement ≥90% threshold (WG-056)
  - Updated: Docs/CLI/API truth source refresh (WG-054)
  - Updated: Agent guidance parity (WG-058)
  - Updated: Disk hygiene automation (WG-057)
- [ ] Add `## [0.1.22] - 2026-03-20` section with:
  - Added: Actionable Playbooks (26 tests)
  - Added: Recommendation Attribution (8 tests)
  - Added: Episode Checkpoints/Handoff (6 tests)
  - Added: Recommendation Feedback (3 tests)
  - Fixed: 2 failing doctests
  - Fixed: Test timeout (quality gate)
  - Fixed: 3 files >500 LOC split
  - Added: 80 snapshot tests, 16 property test files
  - Added: git-cliff changelog automation
- [ ] Add `## [0.1.21] - 2026-03-15` section with:
  - Added: Publishing infrastructure (ADR-045)
  - Added: Supply chain security workflow
  - Added: Claude Code configuration improvements (ADR-046)
- [ ] Add `## [0.1.20] - 2026-03-15` section with:
  - Fixed: do-memory-storage-redb compilation errors
  - Fixed: Stale `#[ignore]` reasons
  - Added: Ignored-test ceiling CI guard
  - Added: Code coverage improvements (ADR-042)
- [ ] Commit: `docs: add changelog entries for v0.1.20–v0.1.24`

### Task 3: ROADMAP_ACTIVE.md Sync (REQUIRED)

- [ ] Update `Released Version: v0.1.22` → `Released Version: v0.1.24`
- [ ] Add v0.1.24 sprint section (test stability + dependency updates)
- [ ] Update release history table

### Task 4: Issue #401 — Dispatch Discoverability (OPTIONAL, LOW EFFORT)

External request to add a description to the `github-workflows` skill for Dispatch.visionairy.biz discoverability.

- [ ] Close as `wontfix` (external tool, not actionable) OR add 1-line description to `.agents/skills/github-workflows/SKILL.md`

### Task 5: Quality Gates (REQUIRED — pre-tag)

- [ ] `./scripts/code-quality.sh fmt`
- [ ] `./scripts/code-quality.sh clippy --workspace`
- [ ] `./scripts/build-rust.sh check`
- [ ] `cargo nextest run --all` → 0 failures, 0 timeouts
- [ ] `cargo test --doc` → 0 failures
- [ ] `./scripts/quality-gates.sh`

### Task 6: Tag & Release (REQUIRED)

- [ ] `git tag v0.1.24`
- [ ] `git push origin v0.1.24` → triggers `release.yml` (cargo-dist builds)
- [ ] Verify GitHub Release created with multi-platform binaries
- [ ] Verify `publish-crates.yml` triggers on release event

---

## Gaps Identified (Not Blocking v0.1.24, Tracked for v0.1.25)

### Gap 1: GitHub Pages Deployment (Broken — was active until Feb 2026)

**Current state**: Pages was previously configured and had 10+ deployments (last: 2026-02-12). Now returns 404 — Pages site has been removed/deconfigured. No `pages.yml` workflow exists. No `book/` directory.

**Reference**: `chaotic_semantic_memory` has 3 active deployment environments:
- **github-pages** — mdBook + `cargo doc --no-deps --all-features` via `actions/deploy-pages@v4`
- **crates.io** — `cargo publish` with idempotent version check
- **npm** — `wasm-pack` + npm publish with provenance

`rust-self-learning-memory` currently has **0 active deployment environments** (only historical github-pages artifacts from Feb 2026).

**Recommendation for v0.1.25** (match `chaotic_semantic_memory` pattern):
1. Create `book/` with mdBook structure (getting started, architecture, API reference)
2. Add `.github/workflows/pages.yml`:
   - Trigger: push to main (on `book/**` or `docs/**` changes)
   - Build: `mdbook build` + `cargo doc --no-deps --all-features` → copy to `book/build/api/`
   - Deploy: `actions/configure-pages@v5` → `actions/upload-pages-artifact@v4` → `actions/deploy-pages@v4`
   - Concurrency: `cancel-in-progress: false` (never cancel mid-deploy)
3. Re-enable Pages in repo settings (source: GitHub Actions workflow)
4. Expose at `https://d-o-hub.github.io/rust-self-learning-memory/`

### Gap 2: No `llms.txt` / LLM Context File

**Current state**: No `llms.txt` or `llms-full.txt` at repo root.

**Reference**: `chaotic_semantic_memory` ships LLM-context files regenerated on every doc sync — emerging convention for AI agent discoverability.

**Recommendation for v0.1.25**: Generate `llms.txt` with crate overview, key types, MCP tool inventory, and CLI commands. Low effort, high discoverability for AI agents consuming this project.

### Gap 3: No Version Sync Script

**Current state**: Version is manually maintained in `Cargo.toml` only.

**Reference**: `chaotic_semantic_memory` has `scripts/verify-version-sync.sh` enforced in CI — ensures Cargo.toml, README, docs, and package.json all match.

**Recommendation for v0.1.25**: Add `scripts/verify-version-sync.sh` checking Cargo.toml version matches README badges, docs references, and STATUS/CURRENT.md.

### Gap 4: No Doc Sync Check in CI

**Current state**: `docs/API_REFERENCE.md` and `docs/PLAYBOOKS_AND_CHECKPOINTS.md` still reference v0.1.22. No CI enforcement.

**Reference**: `chaotic_semantic_memory` has `scripts/sync-docs.sh --check` in CI that fails if docs version drifts.

**Recommendation for v0.1.25**: Add `scripts/sync-docs-version.sh` + wire into `quick-check.yml`.

### Gap 5: Cargo Publish Not Yet Executed

**Current state**: `publish-crates.yml` exists but crates have never been published. Name `do-memory-core` is taken on crates.io. `release.toml` has `publish = false` at workspace level.

**Decision: Use `do-memory-*` namespace** (verified available 2026-03-31):

| Current Crate | Publish Name (crates.io) | npm Name | crates.io | npm |
|---------------|--------------------------|----------|-----------|-----|
| `do-memory-core` | `do-memory-core` | `do-memory-core` | ✅ Available | ✅ Available |
| `do-memory-storage-turso` | `do-memory-storage-turso` | `do-memory-storage-turso` | ✅ Available | ✅ Available |
| `do-memory-storage-redb` | `do-memory-storage-redb` | `do-memory-storage-redb` | ✅ Available | ✅ Available |
| `do-memory-mcp` | `do-memory-mcp` | `do-memory-mcp` | ✅ Available | ✅ Available |

**Implementation steps for v0.1.25**:
1. Add `[package] name = "do-memory-core"` (publish name) to each crate's `Cargo.toml`
2. First manual `cargo publish -p do-memory-core` to establish ownership
3. Flip `publish = true` in `release.toml`
4. Update `publish-crates.yml` matrix with new names
5. Add idempotent publish check (curl crates.io API, skip if version exists — pattern from `chaotic_semantic_memory`)

### Gap 6: Patterns from `chaotic_semantic_memory` to Adopt

High-impact patterns identified for future sprints:

| Pattern | Impact | Effort | Target |
|---------|--------|--------|--------|
| **TTL/expiry on episodes** (`singularity_ttl.rs`) | High — enables GC | 2-3 days | v0.1.25 WG-075 |
| **`MemoryEvent` broadcast channel** | Medium — reactive pipelines | 1-2 days | v0.1.25+ |
| **`select_nth_unstable_by` for top-k** | Medium — O(n) vs O(n log n) retrieval | 1 day | v0.1.25 |
| **SIMD-accelerated similarity** | Low — marginal perf gain | 2-3 days | Backlog |
| **Version-retained persistence** | Medium — track concept drift | 3-4 days | v0.1.26+ |
| **`BundleAccumulator` sliding window** | Medium — recency-weighted context | 2-3 days | v0.1.26+ |
| **Idempotent cargo publish with crates.io check** | High — release safety | 1 day | v0.1.25 |
| **500 LOC CI gate (bash loop)** | Already have script | 0 | ✅ Already enforced |

---

## Cross-References

- **ADR-048**: [v0.1.24 Stability Sprint](adr/ADR-048-v0.1.24-Stability-Sprint.md)
- **ADR-049**: [Comprehensive Analysis v0.1.25](adr/ADR-049-Comprehensive-Analysis-v0.1.25.md)
- **Execution plan**: [GOAP_EXECUTION_PLAN_v0.1.24.md](GOAP_EXECUTION_PLAN_v0.1.24.md)
- **Reference repo**: [d-o-hub/chaotic_semantic_memory](https://github.com/d-o-hub/chaotic_semantic_memory)
