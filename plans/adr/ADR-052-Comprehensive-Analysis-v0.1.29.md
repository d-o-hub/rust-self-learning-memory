# ADR-052: Comprehensive Codebase Analysis & v0.1.29 Sprint Plan

- **Status**: Proposed
- **Date**: 2026-04-04
- **Deciders**: Project maintainers
- **Related**: ADR-040, ADR-049, ADR-050, ADR-051
- **Workspace Version**: 0.1.26 (released), v0.1.27+v0.1.28 features merged to main (unreleased)
- **Method**: GOAP orchestration + Analysis Swarm (RYAN/FLASH/SOCRATES)

## Context

A GOAP-orchestrated analysis on 2026-04-04 covered the entire codebase, all ADRs, roadmap, implementation state, and 2026 industry research. Three key decisions emerged requiring detailed analysis.

---

## Section A: Version Numbering Issue

### Problem

| Item | Value |
|------|-------|
| Latest git tag | `v0.1.26` |
| `Cargo.toml` workspace version | `0.1.26` |
| Commits since v0.1.26 | 23 |
| Roadmap references | v0.1.27 sprint (complete), v0.1.28 sprint (complete) |
| crates.io published | v0.1.26 |

**Finding:** v0.1.27 and v0.1.28 are "virtual sprints" — features were merged to `main` but **no git tags were created** and **no releases were published**. The workspace version in `Cargo.toml` was never bumped past `0.1.26`.

### Recommendation

The next release should be **v0.1.29** (acknowledging v0.1.27 + v0.1.28 as completed sprint labels). Before tagging:
1. Bump `Cargo.toml` workspace version to `0.1.29`
2. Update CHANGELOG.md with v0.1.27 + v0.1.28 + v0.1.29 entries
3. Tag and release

---

## Section B: WASM Sandbox — Analysis Swarm Decision

### Decision: **REMOVE** (Swarm vote: 2-1)

### Evidence

| Fact | Data |
|------|------|
| WASM sandbox LOC | 1,899 lines across 11 files |
| Cross-codebase references | 127 (imports, tests, configs) |
| Root cause of failure | Javy plugin = 9-byte placeholder file |
| Wasmtime version | v43.0.0 (current) |
| Feature status | Permanently disabled since v0.1.17+ |
| User demand | Zero issues, zero PRs, zero requests |
| Dependencies removed | `wasmtime` (43.0.0), `wasmtime-wasi` (43.0.0), `rquickjs` (0.11.0) |

### 2026 Industry Research

| Source | Finding | Implication |
|--------|---------|-------------|
| MCP-SandboxScan (arXiv:2601.01241, Jan 2026) | WASM/WASI validated as security primitive for MCP tools | Concept is valid, but execution requires proper WASI P2 implementation |
| MCP 2026 Roadmap (New Stack, Mar 2026) | No standard code execution tool in MCP spec; focus is on Tasks, auth, transport | Code execution is not a core MCP feature |
| Reddit: WASM runtimes in MCP Rust SDK | Community interest in WASM for MCP | Validates future opportunity, not current need |
| WorkOS MCP Guide (Mar 2026) | No major MCP server ships code execution | Feature has no market precedent |

### Swarm Analysis

**RYAN (Fix):** WASM sandbox is a differentiator; the research validates the approach. But concedes that without Javy, the tool can only accept pre-compiled `.wasm` — impractical for AI agents that generate JS/TS.

**FLASH (Remove):** 1,899 LOC of never-enabled code. Zero users. Removing saves compile time (wasmtime + rquickjs are heavy). If demand emerges, build fresh with WASI Preview 2 + Component Model.

**SOCRATES (Remove):** Even "fixed" (Option A), the tool accepts `.wasm` binaries only. AI agents produce JS/TS, not WASM. The gap between "sandbox works" and "agents can execute code" remains unbridged. Future reimplementation with WASI P2 would be superior.

### Removal Scope

```
DELETE:
  memory-mcp/src/wasm_sandbox/         (6 files)
  memory-mcp/src/unified_sandbox/      (4 files)
  memory-mcp/src/wasmtime_sandbox.rs   (1 file)
  memory-mcp/src/server/tools/code.rs  (1 file)
  memory-mcp/src/server/sandbox.rs     (1 file)
  memory-mcp/data/javy-plugin.wasm     (placeholder)

UPDATE:
  memory-mcp/Cargo.toml               (remove wasmtime, wasmtime-wasi, rquickjs deps)
  Cargo.toml                          (remove workspace wasmtime, wasmtime-wasi, rquickjs)
  memory-mcp/src/server/mod.rs        (remove sandbox module)
  memory-mcp/src/server/tool_definitions.rs (remove execute_agent_code)
  memory-mcp/src/bin/server_impl/handlers.rs (remove handler)
  ~20 test files                      (remove sandbox test references)
```

### Future Path

If code execution demand materializes (post v0.3.0):
- Create `do-memory-wasm` crate with WASI Preview 2 + Component Model
- Target WASM Component Model for polyglot execution (not Javy-specific)
- Follow MCP-SandboxScan architecture for security audit integration

---

## Section C: Turso Native Vector Search — Detailed Analysis

### Current State (Broken)

The "native" search path (`find_similar_episodes_native()`) is labeled native but performs **brute-force in application code**:

1. Joins episodes with embedding table via SQL
2. Fetches each embedding row individually (N+1 queries)
3. Computes `cosine_similarity()` in Rust
4. Sorts results in memory and truncates

**Complexity:** O(n) per query where n = total episodes. At 10K episodes: ~10,001 SQL roundtrips.

### What Turso 2026 Supports (Confirmed)

Turso docs confirm `vector_top_k()` table-valued function with DiskANN:

```sql
-- Single query, DiskANN-accelerated O(log n):
SELECT e.episode_id, e.task_type, e.task_description, ...
FROM vector_top_k('idx_embeddings_384_vector', vector32(?), ?)
JOIN episodes e ON e.rowid = id;
```

**Key capabilities:**
- `vector_top_k(idx_name, query_vector, k)` — ANN search using DiskANN index
- `vector_distance_cos(v1, v2)` — native cosine distance
- `vector32(?)` — convert JSON array to F32_BLOB
- Supports cosine, L2, dot product, Jaccard distance metrics
- DiskANN auto-updates on INSERT/UPDATE/DELETE
- Sparse vector support with inverted index (Turso 0.3.0+)

### Schema Already Supports This

The DiskANN indexes are **already defined** in `schema.rs`:
- `idx_embeddings_384_vector` on `embeddings_384(libsql_vector_idx(embedding_vector))`
- `idx_embeddings_1024_vector`
- `idx_embeddings_1536_vector`
- `idx_embeddings_3072_vector`

**The indexes exist. The search code just doesn't query them.**

### Impact

| Metric | Current (brute-force) | Native (vector_top_k) | Improvement |
|--------|----------------------|----------------------|-------------|
| SQL queries per search | N+1 (up to 10,001) | 1 | ~10,000× fewer queries |
| Time complexity | O(n) | O(log n) | Logarithmic |
| Memory usage | Load all embeddings | Server-side ANN | Much lower |
| Latency (10K episodes) | ~500-1000ms est. | <100ms | 5-10× |
| Latency (100K episodes) | >5000ms | <200ms | 25-50× |

### Implementation Plan

1. Store embeddings as `F32_BLOB` (not JSON text) — requires migration
2. Replace `find_similar_episodes_native()` body with `vector_top_k()` query
3. Replace `find_similar_patterns_native()` similarly
4. Add fallback: if `vector_top_k` fails (libsql version), fall back to brute-force
5. Add integration test with embedded libsql (no network dependency)

### Risk: libsql version compatibility

The `libsql` crate v0.9.29 must support `vector_top_k`. Turso docs confirm this is available in current releases. The schema already creates the indexes without errors, so the engine supports it.

### Blocker: Embedding storage format

Currently embeddings are stored as **JSON text** in `embedding_data TEXT` column. The `embedding_vector F32_BLOB(384)` column exists but may not be populated with actual binary vectors. Migration needed:

```sql
UPDATE embeddings_384
SET embedding_vector = vector32(embedding_data)
WHERE embedding_vector IS NULL;
```

---

## Section D: Sprint Plan (v0.1.29)

### Phase 0: Version & Hygiene (1 day)
| WG | Task | Effort |
|----|------|--------|
| WG-094 | Bump workspace version to 0.1.29, update CHANGELOG | 0.5d |
| WG-095 | Archive stale GOAP plans, trim GOALS/ACTIONS | 0.5d |

### Phase 1: WASM Removal (1-2 days)
| WG | Task | Effort |
|----|------|--------|
| WG-096 | Remove WASM sandbox (1,899 LOC, 127 refs) | 1d |
| WG-097 | Remove wasmtime + rquickjs from workspace deps | 0.5d |

### Phase 2: Turso Native Vector Search (2-3 days)
| WG | Task | Effort |
|----|------|--------|
| WG-098 | Implement `vector_top_k()` search in `search.rs` | 1-2d |
| WG-099 | Add embedding migration (JSON → F32_BLOB) | 0.5d |
| WG-100 | Integration tests for native vector search | 0.5d |

### Phase 3: Quality (1 day)
| WG | Task | Effort |
|----|------|--------|
| WG-101 | Split remaining >500 LOC files | 0.5d |
| WG-102 | Dead code audit (31 → target ≤25) | 0.5d |

**Total estimated effort: 5-7 days**

---

## Consequences

### Positive
- WASM removal saves ~1,899 LOC, 127 dead references, significant compile time
- Native vector search provides 5-50× performance improvement at scale
- Version numbering resolved — clean release path
- Dead code reduced

### Negative
- WASM removal loses potential future differentiator (mitigated: can rebuild with WASI P2)
- Vector search migration requires data format change (mitigated: backward-compatible fallback)

### Risks
- `vector_top_k()` may behave differently with embedded libsql vs Turso cloud (mitigated: integration tests)
- WASM removal may break downstream forks (mitigated: announce in CHANGELOG)

---

## Cross-References

- [ADR-040](ADR-040-Gap-Analysis-And-GOAP-Sprint-v0.1.19.md) — Original WASM sandbox analysis
- [ADR-049](ADR-049-Comprehensive-Analysis-v0.1.25.md) — Previous comprehensive analysis
- [ROADMAP_ACTIVE.md](../ROADMAPS/ROADMAP_ACTIVE.md) — Sprint tracking
- [Turso Vector Search Docs](https://docs.turso.tech/features/ai-and-embeddings) — Native vector search reference
- [MCP-SandboxScan](https://arxiv.org/html/2601.01241v1) — WASM sandbox research validation
