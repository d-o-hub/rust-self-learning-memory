# Chaotic Semantic Memory (CSM) Integration Guide

**Status**: Planned (WG-128 through WG-131)
**Reference**: <https://github.com/d-o-hub/chaotic_semantic_memory>
**Version**: CSM v0.3.2
**Last Updated**: 2026-04-22

---

## Overview

Chaotic Semantic Memory (CSM) is a 100% CPU-local, zero-API Rust library that provides hyperdimensional computing-based retrieval. Integration with this project targets **50-70% reduction in embedding API calls** by implementing a cascading retrieval pipeline that falls back to API embeddings only when CPU-local methods fail to find good matches.

---

## Architecture Summary

### Core Components

| Component | Description | CPU Cost | API Savings |
|-----------|-------------|----------|-------------|
| **Hyperdimensional Computing (HDC)** | 10,240-bit binary vectors with SIMD XOR+POPCNT similarity | Medium | High (100% for offline) |
| **BM25 Inverted Index** | Full Okapi BM25 with Rayon parallelism | Low | High (exact matches) |
| **ConceptGraph** | Curated ontology for synonym expansion without LLM | Negligible | Medium (known domains) |
| **Echo State Network (ESN)** | 50k-node sparse reservoir for temporal sequences | Medium | Research use |
| **Hybrid BM25+HDC** | Query-length-dependent score fusion | Low | Medium |

### Key Characteristics

- **Zero external dependencies**: No API keys, no network calls for retrieval
- **Same storage stack**: Uses libSQL/SQLite/Turso (compatible with this project)
- **SIMD-accelerated**: XOR+POPCNT operations for binary vector similarity
- **Postcard serialization**: Compatible with this project's serialization requirements

---

## Cascading Retrieval Pipeline

The integration implements a tiered retrieval approach, progressively escalating to more expensive methods:

```
Query -> BM25 Check (CPU, free)
           |
     exact match? --- yes --> Return results (0 API calls)
           |
           no
           |
     HDC encode (CPU, free) --> HDC similarity scan
           |
     good hits? --- yes --> Return results (0 API calls)
           |
           no
           |
     ConceptGraph expand (CPU) --> re-scan HDC
           |
     good hits? --- yes --> Return results (0 API calls)
           |
           no
           |
     API embedding call (OpenAI/Cohere) --> vector search
           |
     Return results (1 API call, as fallback only)
```

### Tier Definitions

| Tier | Method | CPU Cost | API Calls | When Used |
|------|--------|----------|-----------|-----------|
| 1 | BM25 exact/keyword match | O(n) Rayon scan | 0 | Short, keyword-heavy queries |
| 2 | HDC similarity | 10,240-bit SIMD | 0 | Medium-length queries, semantic fallback |
| 3 | ConceptGraph expansion | Graph BFS | 0 | Domain-specific synonym matching |
| 4 | API embedding | Network call | 1 | Final fallback for complex semantic queries |

---

## Already Adopted Patterns

The following CSM patterns have already been integrated into this project:

### WG-103: MemoryEvent Broadcast Channel

**Source**: CSM event patterns
**Location**: `memory-core/src/types/event.rs`

```rust
pub enum MemoryEvent {
    EpisodeCreated { id: String, task: String, timestamp: u64 },
    EpisodeCompleted { id: String, reward: f32, timestamp: u64 },
    EpisodeGarbageCollected { id: String, reason: String, timestamp: u64 },
    PatternExtracted { id: String, source_episodes: Vec<String>, timestamp: u64 },
}
```

Uses `tokio::sync::broadcast` channel for efficient fan-out. See `DEFAULT_EVENT_CHANNEL_CAPACITY = 1024`.

### WG-104: O(n) Top-k Selection

**Source**: CSM `singularity_retrieval.rs`
**Location**: `memory-core/src/search/top_k.rs`

```rust
pub fn select_top_k<T, F>(slice: &mut [T], k: usize, compare: F) -> Vec<T>
where F: FnMut(&T, &T) -> Ordering, T: Clone
{
    // O(n) partition + O(k log k) sort for top k
    slice.select_nth_unstable_by(k - 1, compare);
    // Extract and sort top k elements
}
```

More efficient than O(n log n) full sort when k << n. Used in retrieval hot paths.

---

## Planned Integration (WG-128 through WG-131)

### WG-128: BM25 as First Retrieval Tier

**Goal**: Add BM25 keyword index from CSM as first retrieval tier

**Implementation Location**: `memory-core/src/search/bm25.rs` (new)

**Key Files from CSM**:
- `retrieval/bm25.rs` - Full Okapi BM25 with TF-IDF scoring
- Tokenization and inverted index structures

**Expected Impact**:
- Eliminates embedding API calls for exact/keyword-heavy queries
- Rayon parallelism for multi-core utilization
- Instant index updates (no embedding latency)

**API Savings**: 50-70% of queries avoid embedding calls entirely

### WG-129: HDC as Local Embedding Fallback

**Goal**: Wire HDC text encoder as CPU-local embedding fallback

**Implementation Location**: Replaces placeholder in `memory-core/src/embeddings/local.rs`

**Key Files from CSM**:
- `encoder.rs` - Text to HDC vector encoding
- `hyperdim.rs` - 10,240-bit binary vector operations
- SIMD XOR+POPCNT similarity calculations

**Integration Pattern**:

```rust
// Replace mock embeddings with HDC encoder
#[cfg(feature = "csm")]
{
    let hdc_encoder = HdcEncoder::new();
    let hdc_embedding = hdc_encoder.encode_text(text);
    // Returns 10,240-bit binary vector
}

#[cfg(not(feature = "csm"))]
{
    // Fall back to existing mock/ONNX path
}
```

**Key Limitation**: HDC is lexical, not semantic. "cat" and "kitten" have different HDC vectors. This is acceptable because the cascade falls through to API embeddings for truly semantic queries.

### WG-130: ConceptGraph for Synonym Expansion

**Goal**: Add ConceptGraph ontology expansion for synonym retrieval without LLM

**Implementation Location**: `memory-core/src/search/concept_graph.rs` (new)

**Key Files from CSM**:
- `semantic_bridge.rs` - Concept graph traversal
- `concept_graph.rs` - Ontology structure

**Use Cases**:
- Domain-specific synonym expansion ("cat" <-> "feline")
- Alias resolution for known terminology
- Zero LLM calls for curated ontology domains

**API Savings**: Reduces need for semantic embeddings on known-domain queries

### WG-131: CascadeRetriever Orchestration

**Goal**: Implement cascading retrieval pipeline with tier escalation

**Implementation Location**: `memory-core/src/search/cascade.rs` (new)

**Proposed API**:

```rust
pub enum RetrievalTier {
    Bm25Exact,      // CPU-only, O(n) keyword search
    HdcLocal,       // CPU-only, 10240-bit HDC similarity
    ConceptExpand,  // CPU-only, ontology graph BFS
    ApiEmbedding,   // API call, vector_top_k
}

pub struct CascadeResult {
    pub tier_used: RetrievalTier,
    pub results: Vec<EpisodeMatch>,
    pub api_calls: u32,  // 0 for tiers 1-3
    pub latency_ms: u64,
}

pub struct CascadeRetriever {
    bm25_index: Bm25Index,
    hdc_encoder: HdcEncoder,
    concept_graph: ConceptGraph,
    api_provider: Option<EmbeddingProvider>,
    thresholds: CascadeThresholds,  // When to escalate tiers
}
```

---

## Cargo.toml Integration

### Workspace Dependencies

```toml
# Cargo.toml (workspace root)
[workspace.dependencies]
chaotic_semantic_memory = { version = "0.3", optional = true }
```

### do-memory-core Cargo.toml

```toml
# do-memory-core/Cargo.toml
[features]
default = ["redb"]
csm = ["dep:chaotic_semantic_memory"]
local-embeddings = []  # Existing ONNX-based local embeddings
embeddings-full = ["openai", "local-embeddings", "csm"]  # All embedding options

[dependencies]
chaotic_semantic_memory = { workspace = true, optional = true }
```

### Feature Flag Usage

```bash
# CPU-only retrieval (no API calls possible)
cargo build -p do-memory-core --features csm

# Full embedding support (CPU fallback + API)
cargo build -p do-memory-core --features embeddings-full

# Development with all retrieval options
cargo build --features "turso,csm,openai"
```

---

## API Savings Targets

### Current State

| Query Type | Current Approach | API Calls | Cost |
|------------|------------------|-----------|------|
| Exact keyword match | API embedding + vector search | 1 | High |
| Synonym query | API embedding + vector search | 1 | High |
| Semantic query | API embedding + vector search | 1 | High |
| Offline/no-API | Mock embeddings (non-functional) | 0 | Broken |

### Target State (Post CSM Integration)

| Query Type | Target Approach | API Calls | Savings |
|------------|-----------------|-----------|---------|
| Exact keyword match | BM25 tier | 0 | 100% |
| Synonym query (known domain) | HDC + ConceptGraph | 0 | 100% |
| Semantic query | API fallback | 1 | 0% (fallback only) |
| Offline/no-API | HDC tier | 0 | 100% |

### Aggregate Savings

- **50-70% of queries** avoid API calls entirely
- **100% offline capability** for non-semantic queries
- **Cost reduction**: $0.0001/embedding * 1000 queries/day * 0.5 savings = $0.05/day saved

---

## Implementation Phases

### Phase 1: BM25 Integration (WG-128)

1. Add `chaotic_semantic_memory` dependency behind `csm` feature flag
2. Port BM25 index structure from CSM
3. Implement episode tokenization and indexing
4. Add BM25 retrieval to search module
5. Integration tests for exact match queries

### Phase 2: HDC Embedding (WG-129)

1. Port HDC encoder from CSM
2. Replace `local.rs` placeholder with HDC encoder
3. Implement SIMD similarity calculations
4. Add HDC similarity threshold configuration
5. Benchmark HDC vs mock embeddings

### Phase 3: ConceptGraph (WG-130)

1. Port concept graph structure from CSM
2. Add curated ontology for domain-specific terms
3. Implement graph BFS expansion
4. Add expansion-to-HDC integration
5. Test synonym resolution paths

### Phase 4: CascadeRetriever (WG-131)

1. Implement tier escalation logic
2. Add configurable thresholds
3. Wire to existing retrieval paths
4. Add metrics collection (tier usage, latency)
5. Full integration tests for cascade

---

## Configuration

### Cascade Thresholds

```rust
pub struct CascadeThresholds {
    /// BM25 score threshold for "good match" (0.0-1.0)
    pub bm25_min_score: f32,  // Default: 0.3

    /// HDC similarity threshold for "good match" (0.0-1.0)
    pub hdc_min_similarity: f32,  // Default: 0.7

    /// Minimum results before escalating to next tier
    pub min_results: usize,  // Default: 3

    /// Enable/disable ConceptGraph expansion
    pub enable_concept_expansion: bool,  // Default: true
}
```

### Environment Variables

```bash
# Enable CSM cascade retrieval
CSM_ENABLED=true

# BM25 configuration
CSM_BM25_MIN_SCORE=0.3

# HDC configuration
CSM_HDC_MIN_SIMILARITY=0.7

# ConceptGraph configuration
CSM_CONCEPT_GRAPH_PATH=/path/to/ontology.json
```

---

## Cross-References

| Document | Purpose |
|----------|---------|
| `plans/STATUS/COMPREHENSIVE_ANALYSIS_2026-04-21.md` | Section 5 - Full CSM assessment |
| `plans/ROADMAPS/ROADMAP_ACTIVE.md` | WG-128 through WG-131 tasks |
| `memory-core/src/types/event.rs` | WG-103 broadcast channel (adopted) |
| `memory-core/src/search/top_k.rs` | WG-104 top-k selection (adopted) |
| `memory-core/src/embeddings/local.rs` | Local embedding placeholder (to be replaced) |
| CSM Repository | <https://github.com/d-o-hub/chaotic_semantic_memory> |

---

## Related Research Papers

| Paper | arXiv | Relevance |
|-------|-------|-----------|
| Keyword Search Is All You Need | 2602.23368 | Validates BM25-first cascade approach |
| LottaLoRA | 2604.08749 | Validates frozen backbone + low-rank adapter pattern (HDC+ESN) |
| Federated HDC for IoT | 2603.20037 | Multi-agent HDC prototype exchange pattern |