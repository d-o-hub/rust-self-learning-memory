# GOAP Execution Plan — v0.1.23 Remediation Sprint

- **Date**: 2026-03-24
- **Branch**: `main`
- **Scope**: Close audit findings from the 2026-03-24 verification run
- **Strategy**: Sequential + parallel hybrid (truth-source reset first, then parallel feature/documentation/validation streams)
- **Primary ADRs**: ADR-022 (GOAP), ADR-032 (Disk Space), ADR-033 (Testing), ADR-038 (CI parity), ADR-044 (High-Impact Features)

## Goals

1. Restore single-source-of-truth status docs and roadmap (truth-source reset)
2. Make ADR-044 features (recommendation attribution + checkpoints/handoffs) fully durable end-to-end
3. Align MCP + CLI contracts and documentation with actual capabilities (including batch tool decisions)
4. Expand required CI validation + coverage enforcement to match documented expectations
5. Reduce local disk footprint and document artifact hygiene aligned with ADR-032

## Quality Gates

- `git status --short --branch`
- `./scripts/code-quality.sh fmt`
- `./scripts/code-quality.sh clippy --workspace`
- `cargo test --doc`
- `cargo nextest run --all`
- `cargo llvm-cov nextest --all --summary-only`
- `./scripts/check-docs-integrity.sh`
- `cargo run -p do-memory-cli -- --help`
- `cargo test -p do-memory-mcp tool_contract_parity -- --nocapture`
- `du -sh target node_modules`

## Phase Plan

### Phase 1 — Truth Source Reset (Sequential)
- Update `plans/STATUS/*`, `plans/ROADMAPS/ROADMAP_ACTIVE.md`, `plans/README.md`
- Capture audit evidence (git status, du output, MCP/CLI findings)
- Owners: goap-agent + documentation skillset

### Phase 2 — Durable Learning Signals (Parallelizable once Phase 1 complete)
- **WG-051** (✅ Complete — 2026-03-24): Persist recommendation sessions + feedback via storage traits + Turso schema
  - Added Turso SQL + `do-memory-storage-turso/src/storage/recommendations.rs` and redb postcard-backed `do-memory-storage-redb/src/recommendations.rs`
  - Extended storage trait impls/wrappers/resilient cache plus `tests/attribution_integration_test.rs` persistence coverage
  - Commands run: `./scripts/code-quality.sh fmt`, `cargo nextest run --test attribution_integration`
- **WG-052**: Persist checkpoint/handoff metadata in Turso + redb read paths
- Owners: feature-implementer + architecture

### Phase 3 — Contract & Docs Integrity
- **WG-053** (✅ Complete — 2026-03-24): Keep tool-level batch MCP analytics names intentionally deferred/absent; harden parity tests and align active docs/plans to contract truth
- **WG-054**: Regenerate API + CLI + playbook/checkpoint docs + README references
- Owners: do-memory-mcp specialist + documentation skillset

### Phase 4 — Validation & Coverage Parity
- **WG-055**: Expand PR-required CI jobs beyond `--lib` smoke subset
- **WG-056**: Enforce ≥90% coverage threshold in scripts/tests
- Owners: github-workflows + test-runner + quality-unit-testing

### Phase 5 — Disk & Developer Experience Hygiene
- **WG-057**: Document + automate local artifact cleanup (target/32G, node_modules/130M)
- **WG-058**: Align AGENTS/agent_docs/.agents/skills with script-first workflows + disk guidance
- Owners: performance + documentation skillset

## Dependencies & Sequencing

1. Phase 1 establishes accurate state references used by every downstream phase.
2. Phases 2–3 depend on updated docs to avoid additional drift; they can run in parallel per workstream once truth sources are updated.
3. Phase 4 relies on Phase 2 data models minimally (CI still runnable), but final validation messaging should reference updated docs.
4. Phase 5 can begin once truth sources are in place; shell scripts may depend on CI changes from Phase 4.

## Contingencies

- If attribution persistence requires schema migrations, coordinate release windows via ADR-044 change log and block on storage migration scripts.
- If CI expansion exceeds GitHub minutes budgets, introduce split workflows with concurrency groups; document tradeoffs in ADR-033 appendix.
- If batch tool decision remains blocked, explicitly mark `STATUS/` docs with “Deferred” and create WG-053 follow-up issue before sprint end.

## Exit Criteria

- All status/roadmap files reflect 2026-03-24 audit findings and active sprint goals.
- Storage + MCP code implements ADR-044 durability requirements with integration tests.
- MCP API reference, CLI docs, README, and plans agree on feature names and availability.
- Required CI workflows exercise full workspace tests (or documented filtersets) and coverage gate enforces ≥90%.
- Target directory + node_modules usage is either reduced or documented with automated cleanup instructions.
