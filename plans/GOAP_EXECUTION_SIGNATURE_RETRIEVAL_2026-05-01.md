# GOAP Plan: Execution-Signature Retrieval Prototype

**Goal**: Ship a CPU-local, behavior-aware retrieval tier that answers "implement like" queries using execution signatures captured from real test runs, prototyped against `memory-core`.
**Constraint**: No regressions to existing cascade, ≥90% coverage on new modules, ≤500 LOC per file, capture overhead ≤2.5× baseline test wall time, postcard serialization, no API calls in the default capture or match paths.
**Date**: 2026-05-01
**ADR Reference**: ADR-054 (this plan), ADR-051, ADR-050, ADR-044, WG-131
**Status**: 🟡 Proposed — awaiting maintainer green-light to enter Phase 1.

---

## World State (current)

| Predicate | Value | Evidence |
|-----------|-------|----------|
| `cascade_retriever_exists` | true | `memory-core/src/retrieval/cascade.rs` |
| `external_signal_provider_exists` | true | ADR-051, `memory-core/src/reward/external/` |
| `agentfs_artifact_store_available` | true | ADR-050 |
| `behavioral_retrieval_tier_exists` | **false** | only lexical/structural/semantic today |
| `execution_trace_type_exists` | **false** | no module under `memory-core/src/execution/` |
| `pattern_extractor_runs_tests` | **false** | extractor reads completed episodes only |
| `retrieval_mode_enum_exists` | **false** | `CascadeConfig` has no mode discriminator |
| `signature_match_algorithm_exists` | **false** | new module needed |
| `mcp_exposes_retrieval_mode` | **false** | tools accept query+filters only |
| `cli_supports_implement_like` | **false** | no `--mode` flag on `do-memory search` |
| `coverage_gate` | ≥90% | `./scripts/quality-gates.sh` |
| `ci_path_gating_in_place` | true | GOAP CI 2026-04-28, ADR-026 |

## Goal State (target)

| Predicate | Value |
|-----------|-------|
| `execution_trace_type_exists` | true, postcard round-trip stable, `schema_version = 1`, re-exported from `lib.rs` |
| `behavioral_retrieval_tier_exists` | true, behind `execution-signature` feature, integrated into cascade |
| `pattern_extractor_runs_tests` | true (gated), captures traces on episode promotion for `memory-core` |
| `retrieval_mode_enum_exists` | true with `Default`, `ImplementLike`, `DebugLike` placeholder |
| `signature_match_algorithm_exists` | true with structural+behavioral+semantic fusion + WL hash + Hamming |
| `mcp_exposes_retrieval_mode` | true, additive `retrieval_mode` parameter, backward-compatible |
| `cli_supports_implement_like` | true via `do-memory search --mode implement-like` |
| `prototype_quality_validated` | top-3 hit-rate ≥80% on 30-query bench, overhead ≤2.5× baseline |
| `coverage_gate` | ≥90% on new modules |

---

## Action Decomposition

Each action lists preconditions, effects, owner module, estimated cost (engineering hours), and the verification command.

### Phase 0 — Foundation (sequential)

#### A0.1 — Carve out `memory-core/src/execution/` module skeleton
- **Pre**: `execution_trace_type_exists == false`
- **Effect**: empty module compiles, gated behind `execution-signature` feature flag
- **Files**: `memory-core/src/execution/mod.rs`, `memory-core/src/execution/types.rs`, `memory-core/Cargo.toml` (feature)
- **Cost**: 1 h
- **Verify**: `cargo build -p do-memory-core --features execution-signature`

#### A0.2 — Add `ExecutionTrace`, `CallGraph`, `BranchCoverage`, `TimingFingerprint`, `ExecutionSignature`
- **Pre**: A0.1
- **Effect**: types implement `Serialize + Deserialize` (postcard), `schema_version = 1`, re-exported from `lib.rs`
- **Files**: `memory-core/src/execution/types.rs` (≤500 LOC), `memory-core/src/execution/signature.rs`, `memory-core/src/lib.rs`
- **Cost**: 4 h
- **Verify**: `cargo nextest run -p do-memory-core execution::types::tests` (round-trip + schema), `cargo test --doc`

### Phase 1 — Capture pipeline (mostly sequential, A1.3 parallelizable with A1.4)

#### A1.1 — Test discovery via `cargo_metadata`
- **Pre**: A0.2
- **Effect**: `ExecutionCapture::discover_tests(crate)` returns `Vec<TestId>`
- **Files**: `memory-core/src/execution/discovery.rs`
- **Dependencies added**: `cargo_metadata` (workspace-pinned)
- **Cost**: 3 h
- **Verify**: unit test against `memory-core` itself returns >100 tests

#### A1.2 — Branch coverage capture via `cargo llvm-cov --json --branch`
- **Pre**: A1.1
- **Effect**: `BranchCoverage` populated per `TestId` from llvm-cov export JSON
- **Files**: `memory-core/src/execution/coverage.rs`
- **External tool**: `cargo-llvm-cov` (document install in `agent_docs/running_tests.md`)
- **Cost**: 6 h
- **Verify**: integration test runs llvm-cov on a fixture crate, asserts hit bitmap ≠ all-zero

#### A1.3 — Lightweight `tracing` instrumentation for call graph + timing
- **Pre**: A1.1
- **Effect**: a `tracing-subscriber` layer collects span enter/exit events, builds `CallGraph` and per-node cumulative ns
- **Files**: `memory-core/src/execution/tracer.rs` (≤500 LOC)
- **Dependencies added**: `tracing-tree` or custom layer; existing `tracing` workspace dep
- **Cost**: 6 h
- **Verify**: unit test on a fixture with two nested spans asserts edges + bucketed timing

#### A1.4 — Timing normalization (log2 bucketing, jitter clamp)
- **Pre**: A0.2
- **Effect**: `TimingFingerprint::from_raw(&[u64]) -> Self` is deterministic for inputs differing by < bucket width
- **Files**: `memory-core/src/execution/timing.rs`
- **Cost**: 2 h
- **Verify**: property test (proptest) — same input ±5% jitter ⇒ same buckets

#### A1.5 — Wire `PatternExtractor` to call `ExecutionCapture` on episode completion (gated)
- **Pre**: A1.2 + A1.3 + A1.4
- **Effect**: when episode contains `cargo test`-style steps and `execution-signature` feature is on, capture runs and traces are attached
- **Files**: `memory-core/src/extraction/extractor.rs`, `memory-core/src/extraction/extractors/execution.rs`
- **Cost**: 4 h
- **Verify**: `tests/execution_capture_integration.rs` end-to-end on `memory-core`

#### A1.6 — Persist traces (Turso row + redb cache + AgentFS blob for raw `CallGraph`)
- **Pre**: A1.5
- **Effect**: `ExecutionSignature` (compact 32 B + optional HDC) lives in storage; raw `CallGraph` in AgentFS, fetched on demand
- **Files**: `memory-storage-turso/src/execution_traces.rs`, `memory-storage-redb/src/execution_traces.rs`, migrations
- **Cost**: 6 h
- **Verify**: `cargo nextest run -p do-memory-storage-turso execution_traces`, p95 storage size ≤200 KB on memory-core corpus

### Phase 2 — Matching algorithm (parallelizable A2.1/A2.2/A2.3, then A2.4 fuses)

#### A2.1 — Structural similarity reuse (Jaccard on AST n-grams)
- **Pre**: A0.2
- **Effect**: `StructuralScorer` returns `f32` from existing extraction n-grams; no new extractor needed
- **Files**: `memory-core/src/retrieval/signature/structural.rs`
- **Cost**: 2 h
- **Verify**: unit test on synthetic n-gram sets

#### A2.2 — Behavioral similarity (Hamming on coverage + 1-WL hash on `CallGraph` + cosine on timing)
- **Pre**: A0.2
- **Effect**: `BehavioralScorer` returns fused `f32`, weights configurable
- **Files**: `memory-core/src/retrieval/signature/behavioral.rs` (≤500 LOC), `memory-core/src/retrieval/signature/wl_hash.rs`
- **Dependencies added**: `bitvec` (workspace), `blake3` (already transitive)
- **Cost**: 8 h
- **Verify**: unit tests — identical traces ⇒ score ≈ 1.0; disjoint coverage ⇒ < 0.1; permuted node ids ⇒ unchanged

#### A2.3 — Semantic similarity adapter (cosine on existing embeddings)
- **Pre**: A0.2
- **Effect**: `SemanticScorer` wraps existing embedding cosine path
- **Files**: `memory-core/src/retrieval/signature/semantic.rs`
- **Cost**: 1 h
- **Verify**: regression test that score equals current embedding similarity

#### A2.4 — `SignatureMatcher` trait + default fusion implementation
- **Pre**: A2.1 + A2.2 + A2.3
- **Effect**: `score = w_s·s + w_b·b + w_e·e`, BLAKE3 short-circuit on identical compact signatures, top-k via HDC pre-filter
- **Files**: `memory-core/src/retrieval/signature/mod.rs`, `memory-core/src/retrieval/signature/matcher.rs`
- **Cost**: 4 h
- **Verify**: unit + property tests (weights sum to 1, monotonic in each component)

### Phase 3 — Retrieval-mode integration (sequential after Phase 2)

#### A3.1 — Add `RetrievalMode` enum to `CascadeConfig`
- **Pre**: A2.4
- **Effect**: `Default` (current), `ImplementLike { require_trace, weights }`, `DebugLike` (stub)
- **Files**: `memory-core/src/retrieval/cascade.rs`
- **Cost**: 2 h
- **Verify**: serde round-trip, default = `Default`, no behavior change for existing callers

#### A3.2 — `ImplementLike` cascade ordering: signature → BM25/HDC re-rank → API fallback
- **Pre**: A3.1
- **Effect**: cascade routes through `SignatureMatcher` when mode is `ImplementLike`; API tier skipped if `require_trace == true`
- **Files**: `memory-core/src/retrieval/cascade.rs`
- **Cost**: 4 h
- **Verify**: integration test asserts API tier is not invoked when traces are present

#### A3.3 — MCP surface: additive `retrieval_mode` parameter on existing search tools
- **Pre**: A3.2
- **Effect**: backward-compatible JSON schema additions; lazy-loaded per ADR-024
- **Files**: `memory-mcp/src/tools/search.rs`
- **Cost**: 3 h
- **Verify**: MCP client integration test, schema snapshot test

#### A3.4 — CLI surface: `do-memory search --mode implement-like`
- **Pre**: A3.2
- **Effect**: new flag, defaults to `default`
- **Files**: `memory-cli/src/commands/search.rs`
- **Cost**: 2 h
- **Verify**: `cargo nextest run -p do-memory-cli`

### Phase 4 — Validation (parallelizable)

#### A4.1 — Benchmarks: `benches/execution_signature.rs`
- **Pre**: A2.4
- **Effect**: capture overhead, match latency, storage size measured against thresholds
- **Files**: `benches/execution_signature.rs`, `Cargo.toml` bench entry
- **Cost**: 3 h
- **Verify**: `cargo bench --bench execution_signature`; numbers logged to `benchmark_results/`

#### A4.2 — End-to-end retrieval quality benchmark (30-query labeled set)
- **Pre**: A3.2 + A1.6
- **Effect**: top-3 hit rate ≥80% on `memory-core` corpus
- **Files**: `tests/execution_signature_e2e.rs`, `tests/fixtures/implement_like_queries.json`
- **Cost**: 6 h (incl. labeling)
- **Verify**: test asserts hit-rate threshold, prints per-query breakdown

#### A4.3 — Quality gates + docs
- **Pre**: all earlier
- **Effect**: zero clippy warnings, ≥90% coverage on new modules, doc URLs angle-wrapped, `agent_docs/` updated
- **Files**: `agent_docs/execution_signature.md` (new), `agent_docs/running_tests.md` (llvm-cov install note)
- **Cost**: 3 h
- **Verify**: `./scripts/code-quality.sh fmt && ./scripts/code-quality.sh clippy --workspace && cargo nextest run --all && cargo test --doc && cargo doc --no-deps --document-private-items && ./scripts/quality-gates.sh`

---

## Execution Strategy

```diagram
╭─────────────╮     ╭────────────────────────────╮
│ Phase 0     │────▶│ Phase 1 (capture)          │
│ (skeleton)  │     │  A1.1→A1.2                 │
╰─────────────╯     │  A1.1→A1.3                 │
                    │  A1.4 ║ A1.3               │
                    │  A1.5 (joins 1.2/1.3/1.4)  │
                    │  A1.6                      │
                    ╰──────────────┬─────────────╯
                                   │
                                   ▼
              ╭────────────────────────────────────╮
              │ Phase 2 (matching, A2.1/2.2/2.3 ║) │
              │              ↓ join                │
              │             A2.4                   │
              ╰─────────────────┬──────────────────╯
                                ▼
              ╭────────────────────────────────────╮
              │ Phase 3 (cascade + MCP + CLI seq)  │
              ╰─────────────────┬──────────────────╯
                                ▼
              ╭────────────────────────────────────╮
              │ Phase 4 (bench ║ e2e ║ docs/gates) │
              ╰────────────────────────────────────╯
```

- **Parallelization candidates**: A1.3 ║ A1.4, A2.1 ║ A2.2 ║ A2.3, A4.1 ║ A4.2 ║ A4.3.
- **Recommended subagents** (per `agent-coordination` skill): one per parallel branch in Phase 2 and Phase 4.
- **Atomic commits** per AGENTS.md: one action = one commit, message format `feat(execution): A1.2 capture branch coverage via llvm-cov`.

## Total Cost Estimate

| Phase | Hours |
|-------|-------|
| 0     | 5     |
| 1     | 27    |
| 2     | 15    |
| 3     | 11    |
| 4     | 12    |
| **Total** | **70 h** (~2 focused weeks for one engineer; ~5 days with subagent parallelism in Phases 2 and 4) |

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `cargo-llvm-cov` JSON shape changes | Medium | High | Pin version, snapshot-test parser, gate by `schema_version` |
| Tracing overhead pushes capture > 2.5× | Medium | Medium | Sample-based instrumentation, per-test toggle, drop async-instrumented spans below threshold |
| WL hash collisions on small graphs | Low | Medium | Combine with coverage Hamming; require ≥2 of 3 sub-signals to agree |
| Storage explosion | Low | Medium | Compact signature in DB, raw graph in AgentFS, GC by episode TTL |
| Timing non-determinism across machines | High | Low | Bucket via log2, weight low (0.1) by default, allow disable |
| Coverage gate breakage on new modules | Medium | High | Write tests alongside each action; track coverage delta in PR |
| Feature-flag combinatorics blow up | Medium | Medium | Add `execution-signature` to the existing combinations matrix in CI, path-gate the job |

## Rollback Plan

All changes are additive and behind the `execution-signature` cargo feature plus the `RetrievalMode::ImplementLike` enum variant. Rollback = disable the feature flag, default `RetrievalMode::Default` continues to behave exactly as today. Storage migrations are version-tagged (`schema_version = 1`) and reversible by skipping the new tables/columns at read time.

## Definition of Done

- [ ] All actions A0.1–A4.3 merged on individual atomic commits
- [ ] Acceptance criteria from ADR-054 all green
- [ ] `plans/STATUS/CURRENT.md` and `plans/ROADMAPS/ROADMAP_ACTIVE.md` updated
- [ ] `agent_docs/LESSONS.md` records non-obvious findings (e.g., llvm-cov quirks, WL hash tuning)
- [ ] ADR-054 status flipped to ✅ Accepted
- [ ] CHANGELOG.md entry under next minor version
