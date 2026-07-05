---
title: "ADR-061: VERSION File Adoption as Co-Canonical Version Source"
date: 2026-07-05
status: Accepted
issue: "https://github.com/d-o-hub/rust-self-learning-memory/issues/653"
---

# ADR-061: VERSION File Adoption as Co-Canonical Version Source

## Status

Accepted

## Context

The Rust 2026 template uses a root `VERSION` file as a single plain-text source of truth
for tooling (shell scripts, CI steps, Docker builds) that cannot easily parse TOML.
Its CI enforces that `VERSION == workspace.package.version` in `Cargo.toml`.

This repository already has a robust version-consistency story:

- `Cargo.toml` `[workspace.package] version` is the authoritative source for `cargo`
  tooling, `cargo release`, and `cargo-dist`.
- `scripts/verify-release-state.sh` uses `cargo metadata` (with a grep fallback) to
  extract the workspace version and cross-checks ROADMAP, STATUS docs, git tags, and
  CHANGELOG.
- `.github/workflows/release-drift.yml` used a bare `grep` against `Cargo.toml` — fragile
  against whitespace variants and in-comment version strings.
- `agent_docs/git_workflow.md` instructed bumping the version in `Cargo.toml` directly.

**Problem**: Shell scripts that `grep` `Cargo.toml` are brittle (multiple `version =`
keys exist in the file, comment lines may contain version-like strings, leading whitespace
varies). A plain-text `VERSION` file makes the workspace version trivially accessible to
any shell step without spawning `cargo metadata` or writing a precise regex.

**Tradeoff**: A second version file requires synchronization. If someone bumps `Cargo.toml`
without updating `VERSION` (or vice versa), they diverge silently unless a CI gate catches
it.

## Decision

**Adopt a root `VERSION` file as a co-canonical version source alongside `Cargo.toml`.**

`Cargo.toml` remains the **primary** source of truth for `cargo` tooling. `VERSION` is a
read-friendly companion for shell, CI, and documentation tooling. The rule is:

> `VERSION` must equal `[workspace.package] version` in `Cargo.toml` at every commit on
> `main` and in every PR. CI enforces this.

### What this means concretely

| Canonical role | Tool | Value |
|---|---|---|
| Primary (cargo) | `Cargo.toml` `[workspace.package] version` | `0.1.33` |
| Shell/CI companion | `VERSION` (root) | `0.1.33` |

When bumping the version for a release:

1. Update `Cargo.toml` workspace version (via `cargo release` or manually).
2. Update `VERSION` to the same value.
3. CI's VERSION-sync check will fail the PR if they diverge.

### Rejected alternative: VERSION only (remove Cargo.toml reliance)

This would break `cargo release`, `cargo-dist`, `cargo semver-checks`, and the publish
workflow — all of which read `Cargo.toml`. Not viable for a multi-crate Rust workspace.

### Rejected alternative: Keep grep-only approach, harden scripts

Script hardening alone (`cargo metadata` everywhere) is achievable but does not provide
the side benefit of a trivially shell-readable version file. The `VERSION` file is a
low-cost addition that pays dividends for both human readability and CI simplicity.

## Consequences

### Positive

- Release drift CI no longer greps `Cargo.toml` with fragile regexes; it reads `VERSION`
  directly.
- Any external tooling (Dockerfile, shell provisioners, docs generators) can `cat VERSION`
  instead of spawning `cargo metadata`.
- The ADR creates a documented, reviewable decision trail.

### Negative / Mitigations

- One more file to keep in sync → mitigated by CI enforcement (`file-structure.yml` step
  and `verify-release-state.sh` check).
- Developers must remember to update both files → mitigated by `scripts/release-manager.sh`
  guidance and this ADR being referenced from `agent_docs/git_workflow.md`.

## Enforcement

1. `scripts/verify-release-state.sh` checks `VERSION == WORKSPACE_VERSION`.
2. `.github/workflows/file-structure.yml` has a dedicated `version-sync` job that fails
   the PR if `VERSION` diverges from `Cargo.toml`.
3. `.github/workflows/release-drift.yml` reads `VERSION` directly instead of grepping
   `Cargo.toml`.

## References

- Issue: <https://github.com/d-o-hub/rust-self-learning-memory/issues/653>
- ADR-034: Release Engineering Modernization
- ADR-058: CI Health / Gitleaks / Release Drift
- `agent_docs/git_workflow.md` — updated to reflect dual-source bump requirement
- `scripts/verify-release-state.sh` — updated to check VERSION
- `.github/workflows/file-structure.yml` — updated with version-sync job
- `.github/workflows/release-drift.yml` — updated to read VERSION directly
