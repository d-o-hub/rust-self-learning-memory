# ADR-040: Comprehensive Gap Analysis & GOAP Sprint Plan (v0.1.19)

- **Status**: Accepted
- **Date**: 2026-03-13
- **Deciders**: Project maintainers
- **Related**: ADR-028 (Feature Enhancement Roadmap), ADR-039 (Plans Consolidation), ADR-034 (Release Engineering)

## Context

A full GitHub Actions audit and codebase scan on 2026-03-13 revealed multiple categories of gaps between the implemented codebase, CI infrastructure, and ADR-documented goals. This ADR catalogs all findings and establishes a prioritized GOAP execution plan.

### CI/CD Audit Summary (2026-03-13)

| Workflow | Latest Status | Root Cause |
|----------|--------------|------------|
| CI | ✅ SUCCESS | — |
| Quick Check | ✅ SUCCESS | — |
| Coverage | ✅ SUCCESS | — |
| Security | ✅ SUCCESS | — |
| File Structure Validation | ✅ SUCCESS | — |
| Performance Benchmarks | ✅ SUCCESS | — |
| Release | ✅ SUCCESS | — |
| PR Check Anchor | ✅ SUCCESS | — |
| **Nightly Full Tests** | ❌ FAILURE | Turso integration tests panic (libsql env not available in CI) |
| **Changelog** | ❌ FAILURE | `git-cliff` install via `taiki-e/install-action@v2` fails; notify-failure job missing checkout step |
| **ci-old.yml** | ⚠️ ORPHAN | Listed by GH API but file deleted from repo; ghost workflow |

### Nightly Test Failures (Detail)

Tests panicking in CI due to missing Turso database environment:

| Test File | Failing Tests | Root Cause |
|-----------|--------------|------------|
| `compression_integration_test.rs:37` | 3 tests | `TURSO_DATABASE_URL` not set |
| `keepalive_pool_integration_test.rs:29` | 4 tests | `TURSO_DATABASE_URL` not set |
| `phase1_optimization_test.rs:58-292` | 4 tests | `TURSO_DATABASE_URL` not set |

**Issue**: These tests have `#[ignore]` but the nightly workflow runs them with `--run-ignored all` without providing Turso credentials. The exclusion filter in nightly-tests.yml does not cover these test names.

### Swatinem/rust-cache Node.js 20 Deprecation

All 10 workflow references to `Swatinem/rust-cache@v2.8.2` use Node.js 20, which GitHub Actions will force to Node.js 24 starting June 2, 2026. Requires upgrade to v2.9+ or replacement.

## Decision

### Gap Categories

#### G1: CI Failures (P0 — Blocking Nightly)

| ID | Gap | Action | Effort |
|----|-----|--------|--------|
| G1.1 | Nightly tests panic on Turso tests | Add missing test names to nightly exclusion filter | S |
| G1.2 | Changelog workflow `git-cliff` install fails | Pin `git-cliff` version explicitly or use cargo install fallback | S |
| G1.3 | Changelog notify-failure missing checkout | Add `actions/checkout@v6` step before `gh issue create` | S |
| G1.4 | `ci-old.yml` ghost workflow | Disable via GH API or create stub file | S |

#### G2: CI Maintenance (P1 — Pre-June 2026)

| ID | Gap | Action | Effort |
|----|-----|--------|--------|
| G2.1 | `Swatinem/rust-cache@v2.8.2` Node.js 20 deprecation | Upgrade to v2.9+ across all 10 workflow references | S |
| G2.2 | Mutation testing cancelled (2h timeout) | Reduce scope or increase timeout; add `continue-on-error` already present but timeout too low for full run | M |

#### G3: Unimplemented Production Features (P1 — From ADR-028/ADR-039)

| ID | Feature | Evidence | Status |
|----|---------|----------|--------|
| G3.1 | MCP OAuth token handling | `types.rs:22` — `TODO: Implement missing token handling` | Stub only |
| G3.2 | MCP Completion protocol | `types.rs:81` — `TODO: Implement completion support` | Types defined, no handler |
| G3.3 | MCP Elicitation protocol | `types.rs:138` — `TODO: Implement elicitation support` | Types defined, no handler |
| G3.4 | MCP Rate Limiting production wiring | `types.rs:332` — `TODO: Implement rate limiting in production` | Module exists but `#[allow(dead_code)]` |
| G3.5 | MCP Embedding config production wiring | `types.rs:315` — `TODO: Implement embedding config in production` | Types exist, not wired |
| G3.6 | WASM sandbox `execute_agent_code` | `handlers.rs:86` — always returns error | Registered but disabled |

#### G4: Integration Gaps (P2 — Built But Not Wired)

| ID | Subsystem | Location | Gap |
|----|-----------|----------|-----|
| G4.1 | Transport Compression → TursoStorage | `transport/wrapper.rs` | `CompressedTransport` test-only; config flag unused |
| G4.2 | Batch CLI workaround | `commands/mod.rs:356` | `TODO: batch commands should not need direct storage access` |

#### G5: Test Health (P1 — Ongoing)

| ID | Metric | Current | Target |
|----|--------|---------|--------|
| G5.1 | Ignored tests | 119 | ≤30 (non-Turso) |
| G5.2 | `#[allow(dead_code)]` in production src | 79 instances | ≤20 |

#### G6: Documentation & Hygiene (P2)

| ID | Gap | Action |
|----|-----|--------|
| G6.1 | 89 broken markdown links (archived) | Accept as-is per ADR-039 |
| G6.2 | GOAP_STATE.md needs v0.1.19 gap analysis update | This ADR |

### GOAP Execution Plan (v0.1.19 Sprint)

#### Phase 1: CI Stabilization (Parallel, ~1h)

| Task | Priority | Agent | Dependencies |
|------|----------|-------|--------------|
| G1.1 Fix nightly exclusion filter | P0 | ci-engineer | None |
| G1.2 Fix changelog git-cliff install | P0 | ci-engineer | None |
| G1.3 Fix changelog notify-failure | P0 | ci-engineer | G1.2 |
| G1.4 Disable ci-old ghost workflow | P0 | ci-engineer | None |
| G2.1 Upgrade rust-cache to v2.9+ | P1 | ci-engineer | None |

#### Phase 2: Feature Resolution (Sequential, ~4h)

| Task | Priority | Agent | Dependencies |
|------|----------|-------|--------------|
| G3.4 Wire rate limiter to production | P1 | mcp-developer | Phase 1 |
| G3.5 Wire embedding config to production | P1 | mcp-developer | Phase 1 |
| G5.2 Audit and remove dead_code attrs | P1 | code-quality | Phase 1 |
| G4.2 Fix batch CLI direct storage access | P2 | cli-developer | Phase 1 |

#### Phase 3: Protocol Stubs Documentation (P2, ~1h)

| Task | Priority | Agent | Dependencies |
|------|----------|-------|--------------|
| G3.1-G3.3 Document OAuth/Completion/Elicitation as future work | P2 | docs | Phase 2 |
| G3.6 Document WASM sandbox status | P2 | docs | Phase 2 |
| G4.1 Document transport compression integration plan | P2 | docs | Phase 2 |

### Deferred (Not This Sprint)

| Gap | Reason |
|-----|--------|
| G3.1 OAuth implementation | Requires MCP spec finalization |
| G3.2 Completion protocol | MCP spec still evolving |
| G3.3 Elicitation protocol | MCP spec still evolving |
| G3.6 WASM sandbox fix | Javy/Wasmtime compilation issue; low user impact |
| G5.1 Reduce ignored tests to ≤30 | Blocked by upstream libsql bug (ADR-027) |

## Consequences

### Positive

- Nightly CI will be green after Phase 1
- Changelog automation restored
- Node.js 20 deprecation addressed 3 months before deadline
- Rate limiter and embedding config move from dead code to production
- Clear deferral documentation for protocol stubs

### Negative

- OAuth, Completion, Elicitation remain stubs (accepted: specs not finalized)
- 119 ignored tests remain (accepted: upstream libsql blocker)

### Neutral

- Ghost workflow cleanup is cosmetic but reduces confusion
- Dead code audit may reveal additional integration gaps

## Compliance

- All changes are CI/config/documentation — no breaking API changes
- Quality gates: `./scripts/quality-gates.sh` must pass after Phase 1 & 2
- CI parity: `./scripts/code-quality.sh check` validates local matches CI
