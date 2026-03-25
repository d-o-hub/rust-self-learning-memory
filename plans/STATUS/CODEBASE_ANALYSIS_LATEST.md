# GOAP Codebase Analysis — 2026-03-09

**Status**: Revalidated against current workspace code
**Methodology**: GOAP (Analyze → Decompose → Strategize → Synthesize) with ADR traceability and direct code verification
**Branch**: `main` (commit `e6a55b9`)
**Last Updated**: 2026-03-09

---

## Phase 1: ANALYZE — Current Codebase Reality

### Baseline Metrics

| Metric | Current (2026-03-09) | Notes |
|--------|----------------------|-------|
| Workspace version | `0.1.16` | From workspace `Cargo.toml` |
| Rust edition | `2024` | Workspace-wide |
| Workspace members | 9 | `memory-core`, `memory-storage-turso`, `memory-storage-redb`, `memory-mcp`, `memory-cli`, `test-utils`, `benches`, `tests`, `examples` |
| Rust files | **867** | `rg --files -g '*.rs'` |
| Rust LOC | **207,679** | `wc -l` across Rust files |
| Duplicate dependency roots | **134** | `cargo tree -d` |
| Ignored tests | **121** | `#[ignore]` count across Rust tests |
| Snapshot files | **65** | `*.snap` files |
| Property-test files | **7** | `proptest!` present in 7 Rust files |
| Docs integrity broken links | **204** | From `./scripts/check-docs-integrity.sh` |

### File Size Compliance

| Scope | Result | Notes |
|-------|--------|-------|
| All Rust files | 45 files >500 LOC | Mostly test, e2e, bench, or internal test modules |
| Production source files | **0 files >500 LOC** | Excluding `tests/`, `benches/`, `examples/`, and `*_test*.rs`/`tests.rs` files |
| Largest oversize src modules | `memory-core/src/episode/relationship_manager_tests.rs` (860), `memory-mcp/src/patterns/statistical/bocpd_tests.rs` (660) | Internal test modules under `src/` |

### Error Handling Baseline

Two baselines are useful:

| Baseline | unwrap | expect | Total | Notes |
|----------|--------|--------|-------|-------|
| Pragmatic non-test-named `src` baseline | **402** | **70** | **472** | Excludes `tests/`, benches, examples, and files named `*test*.rs` |
| Broad `src` baseline | **1096** | **118** | **1214** | Includes internal test modules under `src/` |

Pragmatic non-test-named `src` breakdown:

| Crate | unwrap | expect |
|-------|--------|--------|
| `memory-core` | 140 | 25 |
| `memory-storage-turso` | 98 | 15 |
| `memory-storage-redb` | 26 | 21 |
| `memory-mcp` | 90 | 3 |
| `memory-cli` | 48 | 6 |

---

## Phase 2: DECOMPOSE — ADR Progress vs Code

### ADR Status Matrix

| ADR | Status | Current Reality |
|-----|--------|-----------------|
| ADR-024 MCP Lazy Tool Loading | ✅ Implemented | `tools/list lazy=true`, `tools/describe`, and `tools/describe_batch` all exist and have dedicated tests |
| ADR-028 Feature Enhancement Roadmap | 🟡 Mixed | Some near-term items shipped, several remain partial or disabled |
| ADR-029 GitHub Actions Modernization | 🟡 Mostly Done | Roadmap says quick wins shipped; no contrary code evidence found in this pass |
| ADR-033 Modern Testing Strategy | 🟡 Further along than docs imply | `cargo nextest`, `proptest`, and `insta` are all present |
| ADR-035 Rust 2024 Edition | ✅ Complete | Workspace still on edition 2024 |
| ADR-036 Dependency Deduplication | 🟡 Monitoring only | 134 duplicate roots, worse than the March 6 snapshot |

### ADR-028 Item-by-Item Revalidation

| Item | Feature | Status | Evidence |
|------|---------|--------|----------|
| #1 | MCP token optimization | ✅ Complete | `memory-mcp/src/protocol/handlers.rs`, `memory-mcp/src/bin/server_impl/core.rs`, `memory-mcp/tests/adr024_lazy_loading_tests.rs` |
| #2 | Batch module rehabilitation | 🟡 Partial | Generic `batch/execute` exists, but named batch handlers are still commented out in `memory-mcp/src/bin/server_impl/handlers.rs` |
| #3 | File size compliance | ✅ Complete for production code | No non-test production source files exceed 500 LOC |
| #4 | Error handling improvement | 🔴 Incomplete | 472 pragmatic non-test-named `unwrap`/`expect` calls remain |
| #5 | Ignored test rehabilitation | 🔴 Regressed | 121 ignored tests remain, concentrated in Turso integration tests |
| #6 | Adaptive TTL Phase 2 | 🟡 Partial foundation only | `AdaptiveCache` exists, but `RedbStorage` still constructs `LRUCache` by default |
| #7 | Embeddings integration completion | 🟡 Partial/mostly complete | MCP has generate/search/status/configure/test; CLI has management/test commands, not the roadmap’s full interactive UX |
| #8 | Transport compression | 🟡 Partial foundation only | Compression modules and wrapper exist, but no production wiring to a concrete Turso transport path was found |
| #9-#14 | Long-term roadmap items | ⚪ Not started in this pass | No concrete implementation evidence found during targeted verification |

### ADR-033 Revalidation

The March 6 analysis understates current test infrastructure maturity:

- `proptest!` appears in 7 files across `memory-core`, `memory-cli`, `memory-storage-redb`, `memory-storage-turso`, and `memory-mcp`.
- Snapshot testing is established in 3 test harnesses with 65 committed `.snap` files.
- The remaining gap is not tool adoption; it is reducing ignores/flakiness and keeping docs in sync with the actual test estate.

---

## Phase 3: SYNTHESIZE — Confirmed Features, Gaps, and Issues

### Confirmed Implemented Features

1. **ADR-024 is fully implemented and tested**
   - Lazy tool stubs, single-tool schema fetch, and batch schema fetch are all present.

2. **Embedding support in MCP is broader than the March 6 report claimed**
   - `configure_embeddings`
   - `test_embeddings`
   - `generate_embedding`
   - `search_by_embedding`
   - `embedding_provider_status`

3. **File-size compliance for production code is materially better than status docs suggest**
   - Remaining oversize files are now test-heavy modules, not core production logic.

4. **Adaptive cache and transport-compression subsystems exist**
   - They should now be treated as integration work, not greenfield implementation.

### Confirmed Missing or Partial Implementation

1. **Batch-specific MCP analytics tools are intentionally deferred at tool level (WG-053)**
   - `batch_query_episodes`, `batch_pattern_analysis`, and `batch_compare_episodes` are intentionally absent from MCP `tools/list`.
   - Runtime dispatch in `memory-mcp/src/bin/server_impl/handlers.rs` does not expose handlers for these names.

2. **CLI workflow coverage exposes real missing UX/features**
   - `tests/e2e/cli_workflows.rs` still ignores pattern discovery and episode search/filter workflows because expected commands/flags do not exist.

3. **Adaptive TTL is not the default runtime path**
   - `memory-storage-redb/src/cache/adaptive/*` is implemented.
   - `memory-storage-redb/src/lib.rs` still creates `LRUCache`, not `AdaptiveCache`, in the main `RedbStorage` flow.

4. **Transport compression is not wired into the concrete Turso runtime path**
   - `memory-storage-turso/src/transport/wrapper.rs` implements `CompressedTransport`.
   - Search did not find production construction of `CompressedTransport::new(...)`; current evidence points to tests and helper modules only.

### Issue Register

#### P1: Ignored test concentration is now the largest visible quality debt

- 121 ignored tests total.
- 70 ignored tests are in `memory-storage-turso/tests`.
- Most Turso ignores cite the same libsql native memory-corruption reason and still reference a placeholder issue URL (`issues/XXX` in file headers).

#### P1: Status-document drift is significant

The active and unified status documents under-report or misstate current reality:

- ADR-024 is already implemented and tested.
- MCP embedding tools are not pending; they exist.
- Production file-size compliance is better than reported.
- ADR-033 adoption is more advanced than reported.
- Ignored-test and duplicate-dependency counts are worse than reported.

#### P1: Pre-existing documentation integrity failures are substantial

`./scripts/check-docs-integrity.sh` currently reports **204 broken markdown links**. This is pre-existing repository debt, not introduced by this update.

Highest-count files in the current failure set:

| File | Broken links |
|------|--------------|
| `plans/ROADMAPS/ROADMAP_ACTIVE.md` | 37 |
| `.opencode/agents/documentation.md` | 20 |
| `plans/archive/2026-02-completed/skills-cli-invocation-best-practices.md` | 12 |
| `plans/STATUS/PROJECT_STATUS_UNIFIED.md` | 10 |
| `plans/README.md` | 9 |
| `plans/ROADMAPS/ROADMAP_VERSION_HISTORY.md` | 9 |

Important planning/status docs already affected before this pass:

- `plans/ROADMAPS/ROADMAP_ACTIVE.md`: 37 broken links
- `plans/STATUS/PROJECT_STATUS_UNIFIED.md`: 10 broken links
- `plans/STATUS/IMPLEMENTATION_STATUS.md`: 3 broken links

#### P2: Documentation drift in redb serialization comments

`memory-storage-redb/src/lib.rs` still states that redb uses bincode serialization, while the crate implementation is using postcard in production code paths. This is an architecture/documentation mismatch against project conventions.

---

## Phase 4: Progress Update for Active Roadmap

### Rebased Priorities for v0.1.17+

1. **Re-enable or remove disabled batch-specific handlers**
   - Close the gap between published schemas/tests and runtime dispatch.

2. **Triage the Turso ignored-test cluster**
   - Either isolate libsql-native failures behind dedicated workflows/features or replace the placeholder issue trail with a real tracked blocker.

3. **Finish runtime integration for already-built subsystems**
   - Wire `AdaptiveCache` into real redb paths.
   - Wire transport compression into the actual Turso client path, or downgrade roadmap claims.

4. **Reduce practical panic surface**
   - Use the 472-call pragmatic baseline, not the historical inflated numbers, as the next error-handling target.

5. **Refresh stale status documents**
   - Keep `ROADMAP_ACTIVE.md` as the canonical live summary.
   - Follow up with a dedicated docs-integrity cleanup pass; current broken-link debt is too large to ignore.

### Recommended Next Sprint Focus

| Priority | Work Item | Why |
|----------|-----------|-----|
| P0 | Batch handler rehabilitation | Implemented surface area is still partially unreachable |
| P0 | Ignored-test reduction in Turso | Largest visible regression vs roadmap intent |
| P1 | Error-handling reduction | Still substantial in non-test-named source |
| P1 | Adaptive cache wiring | Feature exists but is not on the default runtime path |
| P1 | Transport compression wiring | Same pattern as adaptive cache: subsystem exists, integration missing |
| P1 | Docs integrity remediation | 204 pre-existing broken markdown links undermine plan/status reliability |
| P2 | CLI workflow parity | Remove ignored e2e tests by aligning commands/flags with expected workflows |

---

## Phase 5: New Opportunities Not Yet Captured Cleanly in the Plan

These are improvement features that emerged from the codebase audit but are not yet represented well as standalone roadmap items.

### Prioritized Backlog

| ID | Opportunity | Priority | Impact | Effort | Recommended Target |
|----|-------------|----------|--------|--------|--------------------|
| O1 | MCP tool contract parity checker | P0 | High | Small-Medium | v0.1.17 |
| O5 | Runtime feature wiring verification suite | P0 | High | Medium | v0.1.17 |
| O3 | Docs integrity ownership report | P1 | High | Small | v0.1.17 |
| O7 | Structured technical-debt registry | P1 | Medium-High | Small-Medium | v0.1.17 |
| O6 | CLI workflow parity generator | P1 | Medium-High | Medium | v0.1.18 |
| O4 | Architecture drift linter for comments and docs | P2 | Medium | Small-Medium | v0.1.18 |
| O2 | Single-source MCP schema generation | P2 | High | Medium | v0.1.18+ |

### Prioritization Notes

- **P0** items close correctness gaps where the system can advertise or contain functionality that is not actually wired or executable.
- **P1** items reduce maintenance drag and make the existing debt measurable enough to manage.
- **P2** items are high-leverage cleanup/architecture work, but they are safer after the current runtime and validation gaps are closed.

### O1: MCP Tool Contract Parity Checker

**Problem**: Tool definitions, tool registry visibility, and runtime dispatch can drift apart.

**Current evidence**:
- Batch tool schemas exist in `memory-mcp/src/server/tool_definitions_extended.rs`
- Runtime dispatch still leaves batch-specific handlers commented out in `memory-mcp/src/bin/server_impl/handlers.rs`

**Proposal**:
- Add a test/validation layer that verifies every advertised MCP tool is either:
  - callable through runtime dispatch, or
  - explicitly marked internal/disabled and excluded from public listing

**Value**:
- Prevents “tool is listed but not executable” regressions
- Tightens the contract between protocol layer and server implementation

**Effort**: Small to Medium

**Files likely affected**:
- `memory-mcp/src/bin/server_impl/handlers.rs`
- `memory-mcp/src/server/tool_definitions.rs`
- `memory-mcp/src/server/tool_definitions_extended.rs`
- `memory-mcp/tests/`

### O2: Single-Source MCP Schema Generation

**Problem**: MCP tool metadata is spread across multiple layers, increasing drift risk.

**Current evidence**:
- Tool definitions exist in multiple places
- Separate parameter-schema helpers also exist in `memory-mcp/src/server/tool_params.rs`

**Proposal**:
- Generate public MCP schemas from a single tool-definition source
- Derive runtime registration, `tools/list`, and parameter-schema exposure from the same model

**Value**:
- Reduces maintenance overhead
- Makes ADR-024 lazy-loading safer and easier to extend
- Simplifies future tool additions

**Effort**: Medium

**Files likely affected**:
- `memory-mcp/src/server/tool_definitions.rs`
- `memory-mcp/src/server/tool_definitions_extended.rs`
- `memory-mcp/src/server/tool_params.rs`
- `memory-mcp/src/server/tools/registry/`

### O3: Docs Integrity Ownership Report

**Problem**: The repository already has a docs integrity checker, but it does not produce a durable ownership-oriented remediation view.

**Current evidence**:
- `./scripts/check-docs-integrity.sh` reports 204 broken links
- Failures are concentrated in a small set of planning and documentation files

**Proposal**:
- Add a generated report summarizing:
  - broken-link totals
  - top offending files
  - categories such as `plans/`, `.opencode/`, `archive/`, product docs
- Optionally store the latest report under `plans/STATUS/` for trend tracking

**Value**:
- Turns docs-integrity failure from noisy output into actionable maintenance work
- Makes planning/status docs auditable over time

**Effort**: Small

**Files likely affected**:
- `scripts/check-docs-integrity.sh`
- `plans/STATUS/`

### O4: Architecture Drift Linter for Comments and Docs

**Problem**: Some code comments and documentation now contradict implementation reality.

**Current evidence**:
- `memory-storage-redb/src/lib.rs` still references bincode-oriented wording while production serialization is postcard-based

**Proposal**:
- Add a lightweight lint/check for known architecture invariants:
  - postcard, not bincode
  - Tokio async patterns
  - file-size / module conventions where relevant
- Start with a curated denylist of outdated phrases

**Value**:
- Prevents internal docs from becoming misleading
- Helps keep ADR decisions reflected in the codebase narrative

**Effort**: Small to Medium

**Files likely affected**:
- `memory-storage-redb/src/lib.rs`
- `memory-core/src/types/constants.rs`
- `scripts/`
- `plans/adr/`

### O5: Runtime Feature Wiring Verification Suite

**Problem**: Some subsystems appear implemented but are not obviously wired into default runtime paths.

**Current evidence**:
- Adaptive cache code exists, but main redb storage still constructs `LRUCache`
- Transport compression code exists, but the concrete Turso runtime path does not clearly instantiate compressed transport wiring

**Proposal**:
- Add integration checks for “implemented and reachable” behavior in default configurations
- Explicitly verify active wiring for:
  - adaptive cache
  - transport compression
  - optional MCP tool pathways

**Value**:
- Distinguishes shipped capability from dormant subsystem code
- Reduces roadmap overstatement risk

**Effort**: Medium

**Files likely affected**:
- `memory-storage-redb/src/lib.rs`
- `memory-storage-redb/tests/`
- `memory-storage-turso/src/transport/`
- `memory-storage-turso/tests/`

### O6: CLI Workflow Parity Generator

**Problem**: Some e2e CLI tests are ignored because expected workflows no longer match the actual command surface.

**Current evidence**:
- Ignored workflow tests in `tests/e2e/cli_workflows.rs` still reference unsupported commands/flags

**Proposal**:
- Add a command-surface parity test generated from the Clap model
- Validate that documented/expected workflows only use real commands and flags

**Value**:
- Prevents CLI UX drift
- Keeps end-to-end workflow tests aligned with the actual interface

**Effort**: Medium

**Files likely affected**:
- `memory-cli/src/commands/`
- `tests/e2e/cli_workflows.rs`
- CLI test helpers

### O7: Structured Technical-Debt Registry

**Problem**: Known debt is currently scattered across `TODO`s, ignored tests, placeholder issue references, and plan files.

**Current evidence**:
- `issues/XXX` placeholders in Turso tests
- multiple `TODO: Re-enable when batch module is fixed` comments
- `#[allow(dead_code)]` comments carrying implementation debt

**Proposal**:
- Introduce a machine-readable debt registry file for:
  - ignored tests
  - placeholder issue references
  - disabled handlers
  - deferred feature wiring
- Link each item to owner file, reason, and intended exit condition

**Value**:
- Makes debt measurable and easier to report in status docs
- Reduces reliance on stale prose documents for live debt tracking

**Effort**: Small to Medium

**Files likely affected**:
- `plans/STATUS/`
- `tests/`
- `memory-mcp/src/bin/server_impl/handlers.rs`
- `memory-storage-turso/tests/`

### Suggested Execution Order

1. **O1: MCP tool contract parity checker**
   - Cheapest high-signal guardrail
   - Likely to expose concrete batch/tool registration mismatches immediately

2. **O5: Runtime feature wiring verification suite**
   - Validates whether adaptive cache and transport compression are truly shipped capabilities

3. **O3: Docs integrity ownership report**
   - Converts the existing 204-link failure into tractable work packets

4. **O7: Structured technical-debt registry**
   - Gives ignored tests, TODOs, and placeholders a durable tracking surface

5. **O6: CLI workflow parity generator**
   - Best tackled after debt visibility and tool/runtime parity checks are in place

6. **O4: Architecture drift linter**
   - Useful once the highest-noise debt is cataloged

7. **O2: Single-source MCP schema generation**
   - Strong longer-term simplification, but riskier than the guardrail-oriented work above

### Detailed Task Breakdown for Immediate Candidates

#### O1 Implementation Track: MCP Tool Contract Parity Checker

**Goal**:
- Ensure every tool exposed through MCP listing has matching runtime execution support, or is explicitly excluded from public exposure.

**Scope**:
- Validate parity between:
  - tool definitions
  - tool registry exposure
  - JSON-RPC runtime dispatch
  - public `tools/list` visibility

**Concrete Tasks**:
1. Add a test that collects all publicly listed tool names from the MCP server.
2. Add a dispatchability check for each listed tool against `tools/call` routing.
3. Detect and fail on any tool that is:
   - listed but not dispatchable, or
   - dispatchable but not intentionally registered
4. Decide a policy for disabled tools:
   - either remove them from public listing, or
   - fully re-enable their handlers
5. Apply the same parity rule to lazy-loading paths so `tools/describe` cannot describe unreachable tools.

**Success Criteria**:
- No publicly listed MCP tool is unreachable through runtime dispatch.
- No disabled batch tool remains visible unless it has an explicit supported-disabled state.
- Parity checks run in test/CI without manual inspection.

**Primary Code Areas**:
- `memory-mcp/src/bin/server_impl/handlers.rs`
- `memory-mcp/src/bin/server_impl/core.rs`
- `memory-mcp/src/server/tool_definitions.rs`
- `memory-mcp/src/server/tool_definitions_extended.rs`
- `memory-mcp/src/server/tools/registry/`

**Recommended Tests**:
- Extend or add tests under:
  - `memory-mcp/tests/simple_integration_tests.rs`
  - `memory-mcp/tests/adr024_lazy_loading_tests.rs`
  - new parity-focused test file, e.g. `memory-mcp/tests/tool_contract_parity.rs`

**Historical First Failing Cases (now deferred by design)**:
- `batch_query_episodes`
- `batch_pattern_analysis`
- `batch_compare_episodes`

**Suggested Execution Sequence**:
1. Write the failing parity test first.
2. Use it to identify mismatched tools.
3. Remove or re-enable mismatched tools.
4. Re-run lazy-loading and tool-list integration coverage.

#### O5 Implementation Track: Runtime Feature Wiring Verification Suite

**Goal**:
- Verify that subsystems claimed as implemented are actually active or reachable in default runtime paths.

**Scope**:
- Focus first on:
  - adaptive cache in redb
  - transport compression in Turso
- Secondary scope:
  - optional MCP tool activation paths where implementation exists but runtime enablement may drift

**Concrete Tasks**:
1. Define what “wired” means for each subsystem:
   - adaptive cache: default or explicit production path instantiates and exercises adaptive behavior
   - transport compression: real transport path constructs compression-capable transport or explicitly documents why it does not
2. Add tests that assert runtime path selection, not just module-level helper behavior.
3. Add one negative test per feature to distinguish dormant code from active wiring.
4. Update roadmap/status wording if the subsystem is present but not yet truly wired.

**Success Criteria**:
- Tests can prove whether adaptive cache is used in the real storage path.
- Tests can prove whether transport compression is used in the real Turso path.
- Status docs no longer overstate subsystem readiness.

**Primary Code Areas**:
- `memory-storage-redb/src/lib.rs`
- `memory-storage-redb/src/cache/adaptive/`
- `memory-storage-turso/src/transport/mod.rs`
- `memory-storage-turso/src/transport/wrapper.rs`
- `memory-storage-turso/src/lib.rs`

**Recommended Tests**:
- Add or extend tests under:
  - `memory-storage-redb/tests/`
  - `memory-storage-turso/tests/`
  - feature-specific verification tests such as:
    - `memory-storage-redb/tests/runtime_wiring_adaptive_cache.rs`
    - `memory-storage-turso/tests/runtime_wiring_transport_compression.rs`

**Decision Gate**:
- If runtime wiring is absent but subsystem code is sound:
  - either wire it into default execution, or
  - move the status from “implemented” to “available but not default”

**Suggested Execution Sequence**:
1. Write wiring-verification tests against current default constructors.
2. Confirm current behavior rather than assuming intended behavior.
3. Decide whether to integrate, feature-gate more clearly, or downgrade claims.
4. Update docs and roadmap language to match verified runtime reality.

---

## Summary

The codebase is in better shape than the March 6 progress snapshot in some important areas:

- ADR-024 is done, not partial.
- MCP embeddings are implemented, not pending.
- Production source file-size compliance is effectively met.
- ADR-033 testing adoption is materially ahead of the previous write-up.

The main risks have shifted:

- ignored tests have grown into the dominant quality signal,
- batch tooling is still half-disabled,
- some roadmap items are now integration problems rather than greenfield features,
- and the plans/docs corpus carries significant pre-existing broken-link debt.
