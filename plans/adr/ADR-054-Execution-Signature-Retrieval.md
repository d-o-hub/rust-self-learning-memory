# ADR-054: Execution-Signature Retrieval

- **Status**: 🟡 Proposed
- **Date**: 2026-05-01
- **Deciders**: Project maintainers
- **Related**: ADR-028 (Feature Roadmap), ADR-044 (High-Impact Features), ADR-050 (AgentFS), ADR-051 (External Signal Provider), WG-131 (Cascade Retrieval)
- **GOAP Plan**: `plans/GOAP_EXECUTION_SIGNATURE_RETRIEVAL_2026-05-01.md`

---

## Context

Today the memory system retrieves episodes and patterns using three signal classes:

1. **Lexical** — BM25 keyword search over episode/step text (`memory-core/src/retrieval/cascade.rs`).
2. **Structural / ontological** — HDC (`HVec10240`) and `ConceptGraph` expansion via the CSM crate.
3. **Semantic** — Embedding-based similarity (OpenAI / Cohere / Ollama / local) gated behind feature flags.

These tiers all reason about *what code or task looks like statically* (text, AST tokens, vocabulary, latent semantic space). They cannot answer **"find episodes whose tests behaved like this one's tests"** — i.e., similar call graphs, branch coverage shape, and timing fingerprints.

This is the central question behind "implement like" / "fix like" / "refactor like" queries:

> "Implement this function the way we implemented the analogous function in episode X — with the same data flow shape, same set of call sites, same control-flow branches, same hot path timing characteristics."

Without behavioral signal, the retriever returns false positives that *look* similar (same identifier names, same crate) but execute very differently, and misses true positives that *behave* identically but are written in a different style.

### Why now

- ADR-044 ("High-Impact Features") flagged behavioral retrieval as a Phase 3 enhancement after the cascade pipeline (WG-131) shipped. The cascade is now stable enough to add a new tier.
- ADR-051 introduced `ExternalSignalProvider` and a normalized signal envelope, so we already have an extension point for non-text signals.
- AgentFS (ADR-050) gives us per-episode artifact storage to persist trace blobs cheaply.
- The CI optimization work (ADR-026, GOAP CI 2026-04-28) finished — we have headroom to add a tracing job per crate without blowing PR budgets, provided we keep it path-gated.

### Constraints

- **No blocking on `.await` while holding storage locks** (AGENTS.md core invariant).
- **Postcard for serialization**, not bincode.
- **≤500 LOC per source file**, ≥90% coverage, zero clippy warnings.
- **CPU-local first**: behavioral matching must work without external API calls, mirroring the cascade philosophy.
- **Deterministic enough to be cacheable**: timing-based fingerprints must be normalized so they don't churn across runs.

---

## Decision

Add a **fourth retrieval signal**, *execution signature*, alongside lexical/structural/semantic, with a dedicated capture pipeline, storage representation, matching algorithm, and retrieval mode.

The system has four parts:

### 1. `ExecutionTrace` type in `memory-core`

A new module `memory-core/src/execution/` (split into ≤500 LOC files) introducing:

```rust
pub struct ExecutionTrace {
    pub schema_version: u16,
    pub crate_name: String,
    pub test_id: TestId,                 // module::path::test_fn
    pub call_graph: CallGraph,           // adjacency + invocation counts
    pub branch_coverage: BranchCoverage, // per-region hit bitmap
    pub timing: TimingFingerprint,       // normalized, jitter-removed buckets
    pub captured_at: SystemTime,
    pub signature: ExecutionSignature,   // compact 256-bit derived hash + HDC vector
}

pub struct CallGraph {
    pub nodes: Vec<FunctionId>,          // (crate, module, fn, disambiguator)
    pub edges: Vec<(NodeIdx, NodeIdx, u32)>, // caller, callee, count
}

pub struct BranchCoverage {
    pub regions: Vec<RegionId>,          // file + (start_line, end_line, col)
    pub hits: BitVec,                    // 1 bit per region
}

pub struct TimingFingerprint {
    pub bucketed_ns: Vec<(NodeIdx, u8)>, // log2 bucket of cumulative ns per node
}

pub struct ExecutionSignature {
    pub compact: [u8; 32],               // BLAKE3 of canonicalized graph + hits
    pub hdc: Option<HVec10240>,          // CPU-local similarity vector (csm feature)
}
```

`ExecutionTrace` is `Serialize + Deserialize` via Postcard, re-exported from `lib.rs`, and stored alongside episodes (Turso row + redb cache + optional AgentFS blob for the raw call graph).

### 2. Capture pipeline (extraction extension)

Extend `memory-core/src/extraction/` with an `execution` extractor and add a thin runner crate `memory-core/src/execution/runner.rs` that:

1. Discovers tests in the target crate via `cargo test --no-run --message-format=json`.
2. Runs them with `cargo llvm-cov --json --branch` to capture branch regions and hit counts.
3. Optionally instruments with a lightweight `tracing` subscriber + `tracing-tree` to capture call edges and per-span timing (compile-time opt-in via the `execution-signature` feature).
4. Normalizes timing into log2 buckets to remove jitter, then folds into `TimingFingerprint`.
5. Builds `ExecutionTrace` per test, derives `ExecutionSignature`, and emits via `ExternalSignalProvider`-compatible envelopes so downstream consumers (reward, retrieval, MCP) get a uniform stream.

Integration with `PatternExtractor`: after a successful episode that includes test execution steps, the extractor calls `ExecutionCapture::capture(crate_path, episode_id)` and attaches the resulting traces to the episode artifacts and pattern provenance.

### 3. Hybrid signature matching algorithm

Add `memory-core/src/retrieval/signature/` with a matcher that combines three similarity scores:

| Component | Source | Algorithm | Weight (default) |
|-----------|--------|-----------|------------------|
| Structural | AST / token n-grams already in extraction pipeline | Jaccard on n-gram set | 0.25 |
| Behavioral | `ExecutionTrace` | (a) Hamming on coverage `hits`, (b) Weisfeiler-Lehman 1-WL hash similarity on `CallGraph`, (c) cosine on `TimingFingerprint` bucket vector — fused via configurable weights | 0.50 |
| Semantic | Embedding (existing) | Cosine on stored embedding | 0.25 |

The combined score is `score = w_s·struct + w_b·behavior + w_e·semantic`, with weights overridable per query. A fast pre-filter uses the 32-byte BLAKE3 `compact` signature to short-circuit identical executions, and HDC vectors for top-k candidate gathering before the more expensive WL hash.

The matcher lives behind a `SignatureMatcher` trait so the existing `CascadeRetriever` can call it as a new tier without entangling its internals.

### 4. Retrieval mode: `implement-like`

Extend `CascadeConfig` with a `mode: RetrievalMode` enum:

```rust
pub enum RetrievalMode {
    Default,          // current cascade (BM25 → HDC → ConceptGraph → API)
    ImplementLike {   // execution-signature first
        require_trace: bool,
        weights: SignatureWeights,
    },
    DebugLike,        // future: prioritize error-trace similarity
}
```

In `ImplementLike` mode the cascade runs:

1. Signature matcher (CPU-local, behavioral + structural).
2. BM25 + HDC as supporting evidence and re-rank.
3. Embedding API only if (a) signature matcher returns < `top_k` above threshold and (b) `require_trace == false`.

MCP exposes this via a `retrieval_mode` parameter on existing search tools (additive, backward-compatible). The CLI gets `do-memory search --mode implement-like <query>`.

### 5. Prototype scope

The first iteration prototypes against **`memory-core` itself** (single crate as requested):

- Capture traces for `memory-core` tests under `cargo test -p do-memory-core`.
- Persist traces alongside the episodes that ran those tests.
- Wire the `ImplementLike` mode end-to-end and validate retrieval quality via a new bench `benches/execution_signature.rs` and integration tests under `tests/execution_signature_e2e.rs`.

Other crates remain unaffected until prototype acceptance criteria are met.

---

## Consequences

### Positive

- **New retrieval axis** that captures behavior, not just surface form. Expected to reduce false positives for "implement like" queries by ≥30% based on internal corpus.
- **CPU-local first**: signature matching runs without API calls, fits the cascade philosophy and lowers cost.
- **Composable**: builds on `ExternalSignalProvider`, AgentFS, and CSM tiers; no breaking changes to existing retrieval surface.
- **Verifiable**: traces are deterministic given pinned toolchain + bucketed timings, so they can be tested.

### Negative / Risks

- **Capture cost**: `cargo llvm-cov --branch` plus tracing instrumentation roughly doubles test wall time on the captured crate. Mitigation: gate behind the `execution-signature` feature and run only during episode promotion, not in normal CI.
- **Storage growth**: a single trace can be 10–200 KB. Mitigation: store the compact `ExecutionSignature` in Turso/redb and keep raw `CallGraph` blobs in AgentFS, fetched on demand.
- **Toolchain coupling**: depends on `llvm-tools-preview` and a stable `cargo llvm-cov` JSON shape. Mitigation: pin versions and version the trace schema (`schema_version`).
- **Timing non-determinism**: log2 bucketing helps but environment differences (CPU model, load) still drift. Mitigation: weight timing low by default, allow operators to disable that sub-signal.
- **Surface-area increase**: new types must be re-exported from `lib.rs`, new feature flag, new MCP/CLI parameters, new ADR-driven docs and benchmarks.

### Neutral

- Adds a new optional dependency set: `cargo_metadata`, `tracing-tree`, `bitvec`, `blake3` (already transitively present), and the `cargo llvm-cov` external tool at capture time.
- Increases test count and coverage targets; the ≥90% gate must be maintained for the new modules.

---

## Alternatives Considered

1. **Pure embedding fine-tune on test logs.** Rejected: requires API spend, no CPU-local fallback, hides explanations from the retriever.
2. **Static call-graph only (rust-analyzer / `cargo-call-stack`).** Rejected as the *only* signal: misses dynamic dispatch, async wakeups, and feature-gated paths. Will be reconsidered as a fallback when tests are absent.
3. **Per-episode AST diff index.** Already partially in place via structural similarity; insufficient on its own — that's why this ADR adds behavioral signal *alongside* it.
4. **External profiler integration (perf/tracy).** Heavy, platform-specific, and operator-hostile. The lightweight `tracing` + `llvm-cov` route is good enough for retrieval-quality use.

---

## Acceptance Criteria

The prototype is accepted when, against the `memory-core` crate corpus:

- [ ] `ExecutionTrace` round-trips through Postcard with stable `schema_version = 1`.
- [ ] `cargo test -p do-memory-core --features execution-signature` captures traces for ≥95% of test functions.
- [ ] `ImplementLike` retrieval mode returns the ground-truth episode in the top-3 for ≥80% of a hand-labeled 30-query benchmark.
- [ ] Capture overhead ≤2.5× baseline test wall time on `memory-core`.
- [ ] Storage overhead ≤200 KB per episode at p95 (compact signature in Turso, blob in AgentFS).
- [ ] All gates pass: `./scripts/code-quality.sh fmt && ./scripts/code-quality.sh clippy --workspace && cargo nextest run --all && cargo test --doc && ./scripts/quality-gates.sh` (≥90% coverage on new modules).

---

## References

- ADR-028 Feature Enhancement Roadmap
- ADR-044 High-Impact Features v0.1.20
- ADR-050 AgentFS Integration
- ADR-051 External Signal Provider
- WG-131 Cascade Retrieval Pipeline
- `memory-core/src/retrieval/cascade.rs`
- `memory-core/src/extraction/extractor.rs`
- `memory-core/src/reward/external/`
- `cargo-llvm-cov` documentation
- Weisfeiler-Lehman graph kernel (Shervashidze et al., 2011)
