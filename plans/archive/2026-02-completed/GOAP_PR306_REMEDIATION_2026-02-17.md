# GOAP Plan: PR #306 CI Remediation (2026-02-17)

## ADR References
- `plans/adr/ADR-022-GOAP-Agent-System.md`
- `plans/adr/ADR-023-CI-CD-GitHub-Actions-Remediation.md`
- `plans/adr/ADR-029-GitHub-Actions-Modernization.md`

## ANALYZE
- PR `#306` fails CI in MCP builds, clippy/tests compile phase, and security audit.
- Root causes from failed logs:
1. `memory-mcp` missing declared features used by `#[cfg(feature = "...")]` (`oauth`, `embeddings`) with `-D warnings`.
2. `Cargo.lock` dependency graph inconsistent for `wasm-encoder/wasmparser 0.244.0` (cargo-audit panic).
3. `memory-cli` test/config fixtures add nonexistent `StorageConfig.quality_threshold` field.

## DECOMPOSE
1. Add missing feature declarations in `memory-mcp/Cargo.toml`.
2. Restore missing lockfile package nodes for `0.244.0`.
3. Remove invalid `quality_threshold` field initializers in `memory-cli` fixtures/tests.
4. Run local checks: fmt, check/build, clippy/tests targeted.
5. Push branch and verify PR checks.

## STRATEGY
- Keep patch minimal and scoped to known failing causes.
- Validate locally before pushing to minimize CI cycles.

## STATUS
- [x] Failure evidence collected from GitHub Actions logs.
- [x] Code patch applied.
- [x] Local validation complete for failing CI paths (`memory-mcp` strict builds, clippy/tests compile path, cargo-audit panic).
- [ ] PR checks green.

## Validation Notes
- `cargo test --all` still has unrelated pre-existing e2e failures in `e2e-tests/cli_workflows`:
  - `test_bulk_operations`
  - `test_episode_full_lifecycle`
  - `test_relationship_workflow`
  - `test_tag_workflow`
- These did not match the original failing checks from PR `#306` remediation scope.
