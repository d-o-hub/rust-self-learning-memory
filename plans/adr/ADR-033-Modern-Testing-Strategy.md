# ADR-033: Modern Testing Strategy (2026)

- **Status**: Proposed
- **Date**: 2026-02-21
- **Deciders**: Project maintainers
- **Supersedes**: ADR-030 (Test Optimization - extends, not replaces)
- **Related**: ADR-027 (Ignored Tests), ADR-025 (Non-Deterministic Tests)

## Context

Current testing approach has several gaps when measured against 2025/2026 Rust best practices:

1. **Inconsistent test runner**: CI mixes `cargo test` and `cargo nextest` arbitrarily
2. **No mutation testing**: No way to verify test suite actually catches bugs
3. **No property testing**: Only example-based tests; no fuzz-like coverage
4. **No snapshot testing**: Complex output comparisons done manually
5. **Coverage measures reachability**, not effectiveness
6. **No test categorization** beyond `#[ignore]`

### Current State

| Aspect | Status |
|--------|--------|
| Unit tests | ✅ Present, good coverage |
| Integration tests | ✅ Present in `tests/` crate |
| cargo-nextest | ⚠️ Partial (release.yml only) |
| Mutation testing | ❌ Missing |
| Property testing | ❌ Missing |
| Snapshot testing | ❌ Missing |
| Test timing | ⚠️ Basic timeouts only |

## Decision

### 1. Standardize on cargo-nextest (Immediate)

Replace all `cargo test` invocations in CI with `cargo nextest run`:

**Rationale**: nextest runs each test in its own process, provides up to 3x speedup, better output, retries, and JUnit XML output for CI integration.

```yaml
# .config/nextest.toml
[profile.default]
retries = 0
slow-timeout = { period = "60s", terminate-after = 2 }
fail-fast = false

[profile.ci]
retries = 2
slow-timeout = { period = "30s", terminate-after = 3 }
failure-output = "immediate-final"
junit.path = "target/nextest/ci/junit.xml"

[profile.ci.junit]
store-success-output = false
store-failure-output = true
```

**Exception**: `cargo test --doc` still required for doctests (nextest limitation on stable Rust).

### 2. Introduce Mutation Testing with cargo-mutants (Short-term)

Add periodic mutation testing to validate test effectiveness:

```yaml
# Nightly CI job or manual trigger
- name: Mutation Testing
  run: |
    cargo install --locked cargo-mutants
    cargo mutants --timeout 120 --jobs 4 -- --lib
```

**Scope**: Start with `memory-core` (highest-value crate), expand incrementally.

**Acceptance criteria**: <20% "missed" mutants in core business logic modules.

### 3. Introduce Property Testing with proptest (Medium-term)

Add property tests for key invariants:

- Episode lifecycle state machine transitions
- Serialization round-trip (postcard encode/decode)
- Embedding vector normalization
- Pattern extraction determinism
- Storage consistency (write → read = identity)

```toml
# memory-core/Cargo.toml [dev-dependencies]
proptest = "1.6"
```

### 4. Introduce Snapshot Testing with insta (Medium-term)

Add snapshot tests for:

- MCP tool response schemas
- CLI output formatting
- Error message formatting
- Configuration serialization

```toml
# Relevant crate Cargo.toml [dev-dependencies]
insta = { version = "1.42", features = ["json", "yaml"] }
```

### 5. Test Categorization with nextest Filtersets

Use nextest's filterset DSL for test selection:

```bash
# Run only fast unit tests
cargo nextest run -E 'not test(integration) and not test(slow)'

# Run storage tests only
cargo nextest run -E 'package(memory-storage-*)'

# Run everything except ignored
cargo nextest run --run-ignored default
```

## Implementation Plan

| Phase | Action | Timeline | ADR Ref |
|-------|--------|----------|---------|
| 1 | Standardize nextest everywhere | Week 1-2 | This |
| 2 | Add `.config/nextest.toml` profiles | Week 1 | This |
| 3 | Add JUnit XML to CI for test reporting | Week 2 | This |
| 4 | Pilot cargo-mutants on memory-core | Week 3-4 | This |
| 5 | Add proptest for serialization roundtrips | Week 4-6 | This |
| 6 | Add insta for MCP/CLI output | Week 6-8 | This |
| 7 | Nightly mutation testing CI job | Week 8 | This |

## Consequences

### Positive
- **Higher confidence** in test suite effectiveness (mutation testing)
- **Broader input coverage** (property testing finds edge cases humans miss)
- **Faster CI** (nextest 3x speedup)
- **Better regression detection** (snapshot tests catch output drift)
- **JUnit XML** enables CI dashboard integration

### Negative
- Additional dev-dependencies (`proptest`, `insta`, `cargo-mutants`)
- Mutation testing is slow (nightly/manual only)
- Snapshot files need to be committed and reviewed
- Learning curve for proptest strategies

### Risks
- Flaky property tests if generators produce pathological inputs
- Snapshot test churn if output changes frequently
- Mutation testing false positives in generated/boilerplate code

## References

- [cargo-nextest](https://nexte.st/) — Up to 3x faster test runner
- [cargo-mutants](https://mutants.rs/) — Mutation testing for Rust
- [proptest](https://docs.rs/proptest/) — Property testing framework
- [insta](https://insta.rs/) — Snapshot testing for Rust
- [Rust Testing in 2026](https://medium.com/@blogs-world/rust-in-2026) — Ecosystem trends
