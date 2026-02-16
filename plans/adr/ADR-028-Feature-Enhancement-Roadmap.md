# ADR-028: Feature Enhancement Roadmap — Architectural Decisions

**Status**: Accepted
**Date**: 2026-02-14
**Context**: After comprehensive codebase analysis on 2026-02-14, fourteen enhancement areas have been identified spanning near-term fixes, medium-term features, and long-term vision. This ADR documents the architectural decisions for each, providing a unified roadmap with clear dependencies and phasing.

**Decision**: Adopt a three-horizon roadmap (Near-term → Medium-term → Long-term) with explicit architectural guidance per feature area, prioritized by impact and dependency order.

---

## Alternatives Considered

### 1. Individual ADRs Per Feature
- **Pros**: Each decision fully self-contained, independent review
- **Cons**: 14 ADRs creates fragmentation, cross-cutting concerns invisible, dependency tracking scattered
- **REJECTED**: Too many small ADRs; roadmap coherence lost

### 2. Single Monolithic Implementation Plan
- **Pros**: Everything in one place, clear big picture
- **Cons**: Not an architectural decision record, mixes planning with architecture, hard to update
- **REJECTED**: ADRs should capture decisions, not project plans

### 3. Grouped Roadmap ADR with Per-Feature Decisions (Chosen)
- **Pros**: Single coherent document, each feature gets architectural decision, cross-cutting concerns visible, phasing explicit
- **Cons**: Large document, some features may evolve independently
- **ACCEPTED**: Best balance of coherence and architectural rigor

---

## Near-Term Decisions (v0.1.15 — Q1 2026)

### 1. MCP Token Optimization

**Problem**: MCP tool responses include full schemas and unused fields, consuming excessive tokens (~57% overhead measured in analysis).

**Decision**: Implement lazy tool registration with on-demand schema loading and field projection.

- Tools registered with minimal metadata at server startup (name + description only)
- Full JSON Schema loaded on first invocation per tool
- Response field projection: callers specify which fields they need; server omits the rest
- Cache loaded schemas in-process (no cross-request overhead)

**Architecture**:
```
MCP Server Start → Register tool stubs (name, description)
Tool Invocation  → Load full schema on demand → Cache
Response         → Apply field projection → Return minimal payload
```

**Files Affected**: `memory-mcp/src/server/tools/`, `memory-mcp/src/server/mod.rs`

**Rationale**: Aligns with ADR-024 (MCP Lazy Tool Loading). Token reduction directly improves agent efficiency and reduces API costs. Lazy loading keeps startup fast. Field projection is additive — clients that don't request projection get full responses (backward compatible).

**Risks**: Schema caching adds memory pressure (mitigated: schemas are small). Field projection parsing adds latency (mitigated: projection is a simple field filter, not a query language).

---

### 2. Batch Module Rehabilitation

**Problem**: MCP batch operations (`memory-mcp/src/server/tools/batch.rs`) are disabled due to dependency on non-existent `jsonrpsee`/`ServerState` types. The rest of the MCP server uses native JSON-RPC handling.

**Decision**: Replace `jsonrpsee` dependency with native JSON-RPC handling, consistent with existing MCP server patterns.

- Remove `jsonrpsee` references from batch module
- Implement batch operations using the same `serde_json`-based request/response pattern used by other MCP tools
- Re-enable batch module compilation and tests
- Add integration tests covering batch create, batch search, and batch delete

**Architecture**:
```
Batch Request (JSON-RPC) → Deserialize with serde_json
  → Fan out to individual tool handlers
  → Collect results
  → Serialize batch response
```

**Files Affected**: `memory-mcp/src/server/tools/batch.rs`, `memory-mcp/Cargo.toml`

**Rationale**: Native JSON-RPC handling is already proven in the codebase. Adding `jsonrpsee` as a dependency for one module creates unnecessary coupling and version management burden. Consistency across the MCP server simplifies maintenance.

**Risks**: Batch fan-out to individual handlers may surface concurrency issues (mitigated: use `tokio::join!` or `futures::join_all` for parallelism with proper error collection).

---

### 3. File Size Compliance

**Problem**: 29 files exceed the 500 LOC limit established in project conventions. Large files reduce readability and increase merge conflict probability.

**Decision**: Split oversized files into submodules following the `memory-storage-turso` precedent.

- Each file >500 LOC gets a corresponding directory module (`foo.rs` → `foo/mod.rs` + `foo/submodule.rs`)
- Split along logical boundaries: types, operations, helpers, tests
- Re-export public API from `mod.rs` to maintain backward compatibility
- Prioritize files >800 LOC first, then 500-800 LOC files

**Split Strategy**:
| Pattern | Example |
|---------|---------|
| Types + Impl | `types.rs` + `operations.rs` |
| Core + Helpers | `core.rs` + `helpers.rs` |
| Sync + Async | `sync.rs` + `async_ops.rs` |
| Logic + Tests | `mod.rs` + `tests.rs` (for `#[cfg(test)]` blocks) |

**Files Affected**: All crates — prioritized by file size descending

**Rationale**: The `memory-storage-turso` crate successfully completed this pattern. Consistent module structure across crates aids navigation. Re-exports ensure no public API breakage.

**Risks**: Circular dependency between submodules (mitigated: enforce single-direction dependency within a module). Churn in `use` statements across codebase (mitigated: re-exports from `mod.rs`).

---

### 4. Error Handling Improvement

**Problem**: 651 `unwrap()`/`expect()` calls in production code. Any of these can panic and crash the process.

**Decision**: Introduce crate-level error enums using `thiserror`, systematically replace unwraps with proper error propagation.

- Each crate defines a `crate::Error` enum in `src/error.rs` using `thiserror::Error`
- Public functions return `Result<T, crate::Error>` instead of panicking
- Internal helpers use `?` operator for propagation
- `expect()` is permitted only for programmer invariants with descriptive messages (e.g., mutex poisoning, static initialization)
- Target: ≤20 `unwrap`/`expect` calls in production code (down from 651)

**Error Enum Pattern**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("serialization error: {0}")]
    Serialization(#[from] postcard::Error),
    #[error("embedding error: {0}")]
    Embedding(String),
    // ...
}
```

**Files Affected**: All crates — `src/error.rs` (new per crate), all files with `unwrap()`/`expect()`

**Rationale**: `thiserror` is already a workspace dependency. Crate-level error enums follow Rust ecosystem conventions. Proper error handling is prerequisite for production reliability and library consumers.

**Risks**: Large diff across many files (mitigated: phased rollout crate-by-crate, starting with `memory-core`). Public API breaking changes (mitigated: error types are additive; existing `Result` return types gain more specific error variants).

---

### 5. Ignored Test Rehabilitation

**Problem**: 63 tests are marked `#[ignore]` across the test suite. Ignored tests hide regressions and inflate false confidence in coverage metrics.

**Decision**: Triage all ignored tests into three categories: fix, delete, or convert to integration tests.

| Category | Criteria | Action |
|----------|----------|--------|
| **Fix** | Test logic is valid, failure is environmental or fixable | Fix the test, remove `#[ignore]` |
| **Delete** | Test is obsolete, duplicated, or tests removed functionality | Delete the test |
| **Convert** | Test requires external services (Turso, API keys) | Move to `tests/` integration directory, run in CI with service containers |

- Each ignored test gets a tracking comment during triage: `// TRIAGE: fix|delete|convert — reason`
- Target: ≤10 ignored tests remaining (down from 63)
- Ignored tests that require external services use `#[ignore]` with a comment explaining the requirement

**Files Affected**: Test files across all crates, `tests/` integration test directory

**Rationale**: Aligns with ADR-027 (Ignored Tests Strategy). Ignored tests should be a temporary state, not a permanent fixture. Triage ensures each test gets a deliberate decision.

**Risks**: Deleting tests may remove coverage for edge cases (mitigated: review each test before deletion, ensure equivalent coverage exists). Converting to integration tests increases CI time (mitigated: integration tests run in separate CI job with longer timeout).

---

## Medium-Term Decisions (v0.1.16–v0.2.0 — Q2 2026)

### 6. Adaptive TTL Phase 2

**Problem**: Storage TTL is currently static. Episode access patterns vary — frequently accessed episodes should have longer TTLs, cold episodes should expire faster.

**Decision**: Implement access-frequency-based TTL adjustment in the redb cache layer.

- Track access count and last-access timestamp per cached entry
- On cache read, increment access counter and update timestamp
- TTL calculation: `base_ttl * min(access_count, max_multiplier)`
- Configurable via `StorageConfig`: `base_ttl`, `max_ttl`, `access_multiplier`
- Background task (Tokio interval) periodically evicts expired entries

**Architecture**:
```
Cache Read → Update access_count, last_access
Cache Write → Set initial TTL = base_ttl
Eviction Task → Every 60s, scan entries where now > last_access + computed_ttl → Evict
```

**Files Affected**: `memory-storage-redb/src/`, `memory-core/src/config.rs`

**Depends On**: Near-term Phase 3 (File Size Compliance) — redb storage files need splitting before adding TTL complexity.

---

### 7. Embeddings Integration Completion

**Problem**: Embedding generation is available in `memory-core` but not exposed through CLI or MCP tools. Users cannot generate or inspect embeddings interactively.

**Decision**: Add embedding-related commands to CLI and MCP tool registry.

- CLI commands: `memory embed <text>`, `memory embed-search <query>`, `memory embed-status`
- MCP tools: `generate_embedding`, `search_by_embedding`, `embedding_provider_status`
- Use existing `EmbeddingProvider` trait — no new embedding logic needed
- Feature-gated: `openai`, `local-embeddings` flags control which providers are available

**Files Affected**: `memory-cli/src/commands/`, `memory-mcp/src/server/tools/`

**Depends On**: Near-term Phase 1 (MCP Token Optimization) — embedding tools should use lazy loading from day one.

---

### 8. Transport Compression

**Problem**: Turso wire protocol transmits uncompressed data. For bulk operations (batch sync, large episode retrieval), bandwidth is a bottleneck.

**Decision**: Add optional Zstd compression for Turso client communication.

- Compress request/response bodies using `zstd` crate at compression level 3 (fast)
- Content negotiation: client sends `Accept-Encoding: zstd` header; server responds compressed if supported
- Fallback: if Turso server doesn't support compression, transmit uncompressed (no failure)
- Feature-gated under `turso` feature flag — no impact on non-Turso builds

**Files Affected**: `memory-storage-turso/src/client.rs`, `Cargo.toml` (workspace dependency)

**Depends On**: Near-term Phase 2 (Batch Module Rehabilitation) — compression benefits are largest for batch operations.

---

## Long-Term Vision (Q3–Q4 2026)

### 9. Distributed Memory Synchronization

**Problem**: Multiple instances of the memory system cannot share or synchronize their episode stores. Each instance is an island.

**Decision**: CRDT-based (Conflict-free Replicated Data Types) multi-instance synchronization.

- Use operation-based CRDTs for episode metadata (LWW-Register for fields, G-Counter for access counts)
- Content (episode steps, embeddings) synchronized via hash-based deduplication
- Sync protocol: pull-based with Merkle tree comparison for efficient delta detection
- Transport: gRPC between instances (tonic crate)
- Conflict resolution: last-writer-wins for metadata, union for steps/tags

**Architecture**:
```
Instance A ←→ Sync Protocol (gRPC) ←→ Instance B
         ↕                                    ↕
    Local Storage                      Local Storage
    (Turso/redb)                       (Turso/redb)
```

**Rationale**: CRDTs provide eventual consistency without coordination — ideal for episodic memory where strict ordering is not required. Merkle trees minimize sync bandwidth.

**Risks**: CRDT overhead for high-cardinality data. Embedding vectors are large and expensive to sync (mitigated: sync embedding hashes, regenerate on demand). Requires careful schema design for CRDT-compatible types.

---

### 10. Observability Stack

**Problem**: No structured observability beyond `tracing` log output. Production debugging relies on log grepping.

**Decision**: Prometheus metrics + OpenTelemetry distributed tracing.

- **Metrics** (Prometheus): `metrics` crate with `metrics-exporter-prometheus`
  - Counters: episodes created, steps logged, searches performed, cache hits/misses
  - Histograms: operation latency (episode creation, search, embedding generation)
  - Gauges: active episodes, cache size, connection pool utilization
- **Tracing** (OpenTelemetry): `tracing-opentelemetry` crate
  - Span per MCP tool invocation, storage operation, embedding generation
  - Trace context propagation across async boundaries
  - Export to OTLP-compatible backend (Jaeger, Grafana Tempo)
- Feature-gated: `observability` feature flag — zero overhead when disabled

**Files Affected**: New `memory-core/src/observability/` module, instrumentation across all crates

**Rationale**: Production systems need metrics and tracing. Feature gating ensures zero cost for development/testing. `metrics` + `tracing-opentelemetry` are the Rust ecosystem standards.

---

### 11. Multi-Tenancy & RBAC

**Problem**: The memory system is single-tenant. Multiple users or agents sharing an instance have no isolation.

**Decision**: Tenant isolation with role-based access control at the storage layer.

- Tenant ID column added to all database tables (episodes, steps, patterns)
- All queries scoped by tenant ID — enforced at the storage trait level
- RBAC roles: `admin` (full access), `writer` (create/update own episodes), `reader` (search only)
- Authentication: API key per tenant (for MCP server), with optional JWT support
- Authorization enforced in MCP server middleware, before tool handler invocation

**Architecture**:
```
MCP Request → Auth Middleware (extract tenant + role)
  → Authorization Check (role vs. required permission)
  → Tool Handler (tenant-scoped storage access)
```

**Risks**: Tenant ID in every query adds complexity and minor performance overhead. Schema migration required for existing single-tenant data. RBAC adds authentication dependency.

---

### 12. Real-Time Pattern Learning

**Problem**: Pattern extraction currently happens at episode completion. Patterns are not refined based on ongoing usage or feedback.

**Decision**: Online pattern refinement using streaming updates.

- Maintain a sliding window of recent episodes (configurable, default 100)
- After each episode completion, run incremental pattern analysis against the window
- Pattern confidence scores updated using exponential moving average
- Low-confidence patterns pruned after configurable decay period
- New patterns surfaced proactively during context retrieval

**Architecture**:
```
Episode Complete → Add to sliding window
  → Incremental pattern analysis (async background task)
  → Update confidence scores (EMA)
  → Prune low-confidence patterns
  → Update pattern index for retrieval
```

**Files Affected**: `memory-core/src/patterns/`, new `memory-core/src/learning/` module

**Rationale**: Batch pattern extraction misses temporal patterns and delays learning. Online refinement enables the system to adapt to changing usage patterns without manual reprocessing.

---

### 13. Custom Embedding Models

**Problem**: Embedding providers are limited to API-based (OpenAI, Cohere) and pre-packaged local models. Users cannot bring their own fine-tuned models.

**Decision**: ONNX Runtime model loading for custom embedding models.

- Accept ONNX model files via configuration (`embedding.custom_model_path`)
- Use `ort` crate (already a dependency for `local-embeddings` feature) to load and run inference
- Model metadata (dimensions, tokenizer config) read from model file or companion JSON
- Custom models registered as an `EmbeddingProvider` implementation
- Feature-gated under `local-embeddings` flag

**Files Affected**: `memory-core/src/embeddings/`, new `custom.rs` provider

**Rationale**: `ort` is already in the dependency tree. ONNX is the standard interchange format — most PyTorch/TensorFlow models can be exported to ONNX. No new native dependencies required.

---

### 14. A/B Testing Framework

**Problem**: No way to compare pattern extraction strategies, embedding models, or retrieval algorithms in production.

**Decision**: Built-in experiment framework for comparing strategies.

- Experiment definition: name, variants (A/B/...), traffic split, metric to optimize
- Traffic routing: hash-based consistent assignment (episode ID → variant)
- Metrics collection: per-variant counters and histograms (uses observability stack from #10)
- Significance testing: simple z-test for proportions, t-test for means
- Results stored as episodes (dogfooding the memory system)

**Architecture**:
```
Experiment Config → Register variants
Episode Created → Assign variant (consistent hash)
  → Execute variant strategy
  → Record metrics
Experiment Complete → Statistical analysis → Winner determination
```

**Files Affected**: New `memory-core/src/experiments/` module

**Depends On**: #10 (Observability Stack) for metrics collection.

**Rationale**: Without experimentation, strategy improvements are based on intuition. A/B testing enables data-driven decisions. Consistent hashing ensures reproducible assignments.

---

## Decision Matrix

| # | Feature | Priority | Risk | Effort | Value | Dependencies |
|---|---------|----------|------|--------|-------|--------------|
| 1 | MCP Token Optimization | P1 | Low | Medium | High | ADR-024 |
| 2 | Batch Module Rehabilitation | P1 | Low | Low | Medium | ADR-025 Phase C |
| 3 | File Size Compliance | P1 | Low | Medium | Medium | None |
| 4 | Error Handling Improvement | P0 | Medium | High | Very High | None |
| 5 | Ignored Test Rehabilitation | P1 | Low | Medium | High | ADR-027 |
| 6 | Adaptive TTL Phase 2 | P2 | Medium | Medium | High | #3 |
| 7 | Embeddings Integration | P2 | Low | Medium | Medium | #1 |
| 8 | Transport Compression | P2 | Low | Low | Medium | #2 |
| 9 | Distributed Sync | P3 | High | Very High | High | #4, #6 |
| 10 | Observability Stack | P3 | Medium | High | High | #4 |
| 11 | Multi-Tenancy & RBAC | P3 | High | Very High | Medium | #4, #9 |
| 12 | Real-Time Pattern Learning | P3 | High | High | Very High | #6 |
| 13 | Custom Embedding Models | P3 | Medium | Medium | Medium | #7 |
| 14 | A/B Testing Framework | P3 | Medium | High | High | #10, #12 |

---

## Execution Order

```
Near-term (Q1 2026):
  #3 File Size ─┐
  #4 Error Handling ─┤─→ #1 MCP Token Opt ─→ #2 Batch Rehab
  #5 Ignored Tests ──┘

Medium-term (Q2 2026):
  #6 Adaptive TTL (after #3)
  #7 Embeddings Integration (after #1)
  #8 Transport Compression (after #2)

Long-term (Q3-Q4 2026):
  #10 Observability (after #4)
  #9 Distributed Sync (after #4, #6)
  #12 Real-Time Patterns (after #6)
  #11 Multi-Tenancy (after #4, #9)
  #13 Custom Embeddings (after #7)
  #14 A/B Testing (after #10, #12)
```

---

## Tradeoffs

### Positive
- Comprehensive roadmap provides visibility across all planned enhancements
- Per-feature architectural decisions enable independent implementation
- Phased execution ensures each feature builds on stable foundations
- Feature flags isolate optional functionality — no bloat for minimal deployments
- Long-term vision documented early, reducing future architectural surprise

### Negative
- 14 features across 3 horizons is ambitious — scope creep risk if not triaged
- Long-term features (#9–#14) are speculative and may change significantly by Q3
- Near-term debt remediation (#3, #4, #5) delays feature delivery
- Some long-term decisions (CRDTs, RBAC) may be over-engineered for current scale

---

## Consequences

- **Positive**: All enhancement areas have documented architectural direction — no ad-hoc decisions
- **Positive**: Dependency graph prevents building on unstable foundations
- **Positive**: Near-term focus on quality (#3, #4, #5) improves reliability before adding complexity
- **Positive**: Feature flags ensure opt-in complexity — minimal builds stay minimal
- **Positive**: Medium-term features (#6, #7, #8) complete existing functionality gaps
- **Positive**: Long-term vision (#9–#14) positions the system for multi-agent production use
- **Negative**: Large ADR document requires periodic review to stay current
- **Negative**: Long-term features may be superseded by ecosystem changes (e.g., MCP protocol evolution)
- **Negative**: 651→≤20 unwrap reduction (#4) is a significant cross-crate effort

---

## Implementation Status

🔄 **PARTIALLY COMPLETE** (2 of 14 features shipped)

| # | Feature | Status | Notes |
|---|---------|--------|-------|
| 1 | MCP Token Optimization | ✅ Complete | Shipped in v0.1.15 — `list_tool_names()` for 98% token reduction |
| 2 | Batch Module Rehabilitation | ⬚ Not Started | ADR-025 Phase C scope; planned for v0.1.16 Phase C |
| 3 | File Size Compliance | ✅ Complete | All source files ≤500 LOC (completed v0.1.14) |
| 4 | Error Handling Improvement | ⬚ Not Started | 561 unwrap() + 90 .expect() in prod; planned for v0.1.16 Phase B |
| 5 | Ignored Test Rehabilitation | ⬚ Not Started | ADR-027 accepted; 63 tests to triage; planned for v0.1.16 Phase B |
| 6 | Adaptive TTL Phase 2 | ⬚ Not Started | Unblocked (dependency #3 complete) |
| 7 | Embeddings Integration | ⬚ Not Started | Unblocked (dependency #1 complete); planned for v0.1.16 Phase C |
| 8 | Transport Compression | ⬚ Not Started | Blocked by #2 |
| 9-14 | Long-term features | ⬚ Not Started | Long-term |

---

## Related ADRs

- **ADR-022**: GOAP Agent System — orchestration methodology for multi-feature execution
- **ADR-024**: MCP Lazy Tool Loading — foundational decision for Feature #1
- **ADR-025**: Project Health Remediation — overlaps with Features #2, #4 (Phases C, D)
- **ADR-027**: Ignored Tests Strategy — foundational decision for Feature #5

---

## References

- Codebase analysis report (2026-02-14) — source of all metrics and file counts
- `plans/ROADMAPS/ROADMAP_ACTIVE.md` — active project roadmap
- `plans/research/MCP_TOKEN_OPTIMIZATION_RESEARCH.md` — MCP optimization research
- `agent_docs/code_conventions.md` — 500 LOC limit, postcard serialization, error handling conventions
- `agent_docs/service_architecture.md` — system architecture reference

---

**Individual ADR**: `plans/adr/ADR-028-Feature-Enhancement-Roadmap.md`
**Supersedes**: None
**Superseded By**: None
