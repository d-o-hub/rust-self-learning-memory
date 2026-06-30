# ADR-059: LOC Boundary Proactive Splits & Unwrap Hygiene Strategy

- **Status**: 🟢 Accepted
- **Date**: 2026-06-30
- **Deciders**: Project maintainers
- **Related**: ADR-055 (Comprehensive Analysis v0.1.32), ADR-043 (Comprehensive
  Analysis v0.1.20), `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-30.md`

## Context

The project enforces a hard quality gate: **≤500 LOC per production source file**
(defined in `scripts/quality-gates.sh`). This analysis identified:

1. **2 files at exactly 500 LOC** — any addition will break the gate
2. **8 files at 490-499 LOC** — one feature change away from breach
3. **~90 production `.unwrap()` calls** — potential panics in non-test paths

### Files at the Boundary (500 LOC)

| File | LOC | Module Purpose |
|------|-----|----------------|
| `memory-core/src/retrieval/cascade/mod.rs` | 500 | Cascading retrieval pipeline |
| `memory-cli/src/commands/tag/core.rs` | 500 | Tag command implementation |

### High-Risk Files (490-499 LOC)

| File | LOC |
|------|-----|
| `memory-core/src/embeddings/local.rs` | 497 |
| `memory-storage-turso/src/resilient.rs` | 494 |
| `memory-core/src/storage/mod.rs` | 494 |
| `memory-core/src/security/audit/mod.rs` | 493 |
| `memory-storage-turso/src/pool/adaptive.rs` | 491 |
| `memory-core/src/indexing/spatiotemporal/mod.rs` | 491 |
| `memory-storage-turso/src/storage/batch/query_batch.rs` | 490 |
| `memory-cli/src/commands/episode/relationships/types.rs` | 489 |

### Unwrap Analysis

472 total `.unwrap()` calls in production files (including inline test modules).
Approximately 90 are in actual production paths:

- **Monitoring/metrics**: Lock acquisition, channel send
- **Config serialization**: `serde_json::to_string(&x).unwrap()` (technically safe for `Serialize` types but not idiomatic)
- **Storage operations**: Parse/lock unwraps that could panic under resource pressure

## Decision

### D1: Proactive LOC Splits

**Strategy**: Pre-split files before they hit 500 LOC rather than reactively splitting after gate failure.

**Threshold**: Files at 490+ LOC should be proactively split when:
- They are actively being modified (recent commits touch them)
- A planned feature will add lines to them
- They contain naturally separable concerns (tests, helpers, types)

**Split pattern** (established by prior refactors):
- `mod.rs` → `mod.rs` + `types.rs` (extract struct/enum definitions)
- `mod.rs` → `mod.rs` + `helpers.rs` (extract utility functions)
- `core.rs` → `core.rs` + `core_ops.rs` (split by operation group)

**Immediate action** (WG-185):
- `cascade/mod.rs`: Extract `CascadeConfig` and `CascadeResult` types + non-CSM fallback into `cascade/types.rs`
- `tag/core.rs`: Extract tag validation/formatting helpers into `tag/helpers.rs`

### D2: Unwrap Hygiene — Graduated Approach

**Strategy**: Don't attempt a single bulk conversion (425→0 would be noisy and risky). Instead:

1. **P1 — Critical paths** (WG-187): Convert unwraps in monitoring and storage code that could panic under load. Replace with `expect("reason")` or proper error propagation.

2. **P2 — Config/serialization**: Replace `serde_json::to_string(&x).unwrap()` with `expect("serialization of X should never fail")` for self-documenting code. These are technically safe but silent about intent.

3. **P3 — Future commits**: Add a clippy lint config to warn on new unwraps in production paths (consider `#![warn(clippy::unwrap_used)]` at crate level with `#[allow]` for existing code).

**Not in scope**: Unwraps in `#[cfg(test)]` blocks, doc examples, or test utilities.

### D3: Dependency Duplication Monitoring

The `thiserror` v1/v2 duplication is caused by transitive dependencies (`agentfs-sdk`, `argmin`) that haven't upgraded. This is:
- **Not actionable** without upstream PRs
- **Low impact** (separate proc-macro compilations add ~2-3s)
- **Should be monitored** — when agentfs-sdk or argmin release v2 versions, deduplicate

## Consequences

### Positive
- No surprise quality gate failures from boundary files
- Explicit documentation of acceptable unwrap patterns
- Lower risk of production panics from resource exhaustion
- Clear escalation path (P1→P2→P3) avoids churn

### Negative
- Pre-splitting adds modules that may feel premature for small files
- Adding `expect()` messages increases line count slightly (mitigated by the split)
- Clippy `unwrap_used` lint requires explicit `#[allow]` annotations on existing code

### Risks
- Over-splitting can make navigation harder — limit to files genuinely at risk
- `expect()` messages can become stale if logic changes — keep them generic

## Implementation Plan

| WG | Phase | Action | Files |
|----|-------|--------|-------|
| WG-185 | P1 | Split boundary files (500 LOC) | `cascade/mod.rs`, `tag/core.rs` |
| WG-186 | P2 | Preemptive splits (490+ LOC, top 4) | `local.rs`, `resilient.rs`, `storage/mod.rs`, `audit/mod.rs` |
| WG-187 | P1 | Convert critical-path unwraps | `monitoring/core.rs`, `storage/` |
| — | P3 | Add `clippy::unwrap_used` lint | Crate-level `lib.rs` |

## Metrics

| Metric | Before | After (target) |
|--------|--------|----------------|
| Files at 500 LOC | 2 | 0 |
| Files at 490-499 LOC | 8 | ≤4 |
| Production unwraps (non-test) | ~90 | ≤50 |
| Quality gate failures from LOC | Imminent | Prevented |
