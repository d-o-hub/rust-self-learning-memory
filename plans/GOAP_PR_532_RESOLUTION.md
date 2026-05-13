# GOAP Plan: PR 532 Resolution

## Goal
Resolve all failing GitHub Actions and address all reviewer comments in PR 532 ("Add DuckDB as optional storage backend").

## Success Criteria
- [ ] All GitHub Actions checks pass (CI, Quick Check, YAML Lint, Security, Performance, Coverage).
- [ ] All CodeRabbit reviewer comments are addressed and resolved.
- [ ] Code quality standards (fmt, clippy, audit) are met.
- [ ] Test coverage meets the 90% threshold.

## Tasks

### Phase 1: Infrastructure & Infrastructure-adjacent Fixes
- **T1.1: Fix `scripts/code-quality.sh`**
  - Fix CLI parsing (remove unconditional `shift`, handle flags properly).
  - Update error handling to capture exit codes from `cargo fmt`, `clippy`, and `audit` even if they fail.
- **T1.2: Update `Cargo.toml`**
  - Add dependency debug override in `[profile.dev.package."*"]`.
  - Set `debug = "line-tables-only"` for overall dev profile.
- **T1.3: Fix Markdown Linting (MD022)**
  - Ensure headings are surrounded by blank lines in:
    - `plans/adr/ADR-054-CloudEvents-Integration.md`
    - `plans/GOAP_CLOUDEVENTS_INTEGRATION.md`
    - `plans/GOAP_CLOUDEVENTS_STABILIZATION.md`
- **T1.4: Fix `memory-core/src/retrieval/cascade/mod.rs`**
  - Set `api_calls: 1` in the Tier 4 fallback branch.

### Phase 2: Storage Backend Refinements
- **T2.1: Refactor `memory-storage-duckdb/src/storage/patterns.rs`**
  - Sanitize raw JSON in `Error::Storage` messages (don't leak sensitive data).
  - Handle `i64::try_from` error explicitly for `sample_size`.
  - Use `match &pattern` to avoid moving bindings.
- **T2.2: Refactor `memory-storage-turso/src/cache/query_cache_types.rs`**
  - Update `from_query` to strip SQL comments and normalize whitespace before dependency detection.

### Phase 3: Test Suite Optimization
- **T3.1: Split `memory-mcp/tests/persistent_storage_tests.rs`**
  - Break the file down into smaller, focused test files (≤ 500 lines).
  - Create a shared helper for setup if needed.
- **T3.2: Update `memory-storage-turso/src/cache/query_cache_tests.rs`**
  - Add regression tests for multiline and commented SQL in dependency detection.

### Phase 4: Validation
- **T4.1: Run Formatting & Linting**
  - `./scripts/code-quality.sh fmt`
  - `./scripts/code-quality.sh clippy --workspace`
- **T4.2: Execute Tests**
  - `cargo nextest run --all`
- **T4.3: Quality Gates**
  - `./scripts/quality-gates.sh`

## Execution Strategy
- Use parallel agents for Tasks in Phase 1 and Phase 2.
- Phase 3 can be handled by a specialized test-fix agent.
- Phase 4 will be the final validation step.
