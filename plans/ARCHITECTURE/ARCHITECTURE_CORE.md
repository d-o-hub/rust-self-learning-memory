# Current Architecture - Core Components

**Last Updated**: 2025-12-30
**Version**: v0.1.10
**Branch**: release/v0.1.10
**Production Readiness**: 100% ✅

---

## Executive Summary

The Self-Learning Memory System is a production-ready Rust-based episodic learning platform with dual storage backends, semantic embeddings, and MCP protocol integration. The system demonstrates excellent architectural design with clear separation of concerns across 8 workspace crates.

**Key Characteristics**:
- **Modular Architecture**: 4.5/5 stars - Clean crate boundaries with well-defined interfaces
- **2025 Best Practices**: 5/5 stars - Modern async/Tokio patterns, comprehensive testing
- **Production Ready**: 100% - All quality gates passing, research integration complete
- **File Size Compliance**: 100% - All storage modules comply with 500 LOC limit

---

## Workspace Structure

### Crate Overview (8 Total)

| Crate | Purpose | Dependencies | Status |
|-------|---------|--------------|--------|
| **memory-core** | Core episodic learning system | tokio, serde, anyhow | ✅ Stable |
| **memory-storage-turso** | Durable storage (libSQL/Turso) | libsql, tokio | ✅ Stable |
| **memory-storage-redb** | High-speed cache (embedded) | redb, tokio | ✅ Stable |
| **memory-mcp** | MCP protocol server | wasmtime, tokio | ✅ Stable |
| **memory-cli** | CLI for operations | clap, dialoguer | ✅ Stable |
| **test-utils** | Shared test utilities | tokio-test | ✅ Stable |
| **benches** | Performance benchmarks | criterion | ✅ Stable |
| **examples** | Integration examples | - | ✅ Stable |

---

## Core Architecture: memory-core

### Module Organization

```
memory-core/src/
├── lib.rs                    # Public API and re-exports
├── memory/                   # Main orchestration
│   ├── mod.rs               # SelfLearningMemory coordinator
│   ├── retrieval.rs         # Context retrieval and search
│   └── step_buffer.rs       # Execution step batching
├── episode.rs               # ExecutionStep and core types
├── patterns/                # Pattern extraction and validation
│   ├── mod.rs               # PatternExtractor trait
│   ├── extractors/          # Multiple extraction strategies
│   │   ├── hybrid.rs        # Combined multi-strategy extraction
│   │   ├── context.rs       # Domain/language-aware patterns
│   │   ├── decision_point.rs # Branch condition patterns
│   │   ├── error_recovery.rs # Error handling strategies
│   │   ├── tool_sequence.rs  # Sequential tool usage
│   │   └── heuristic/       # Condition-action rules
│   ├── validation.rs        # Pattern quality assessment
│   ├── clustering.rs        # Pattern deduplication
│   ├── effectiveness.rs     # Usage tracking
│   └── optimized_validator.rs # Risk and compatibility
├── reward/                  # Reward calculation
│   ├── mod.rs              # RewardCalculator
│   └── calculator.rs       # Multi-component scoring
├── reflection/             # Episode analysis
│   ├── mod.rs              # ReflectionGenerator
│   ├── insight_generator.rs # Insight extraction
│   └── helpers.rs          # Analysis utilities
├── extraction/             # Pattern extraction orchestration
│   ├── mod.rs              # Main extractor
│   └── extractors/         # Extraction strategies
├── learning/               # Async pattern processing
│   └── queue.rs            # PatternExtractionQueue
├── embeddings/             # Semantic embeddings
│   ├── mod.rs              # SemanticService coordinator
│   ├── provider.rs         # EmbeddingProvider trait
│   ├── local.rs            # Offline models (candle)
│   ├── openai.rs           # Cloud API (feature-gated)
│   ├── mock_model.rs       # Testing support
│   └── similarity.rs       # Cosine similarity
├── embeddings_simple.rs    # Simplified embedding interface
├── monitoring/             # Metrics and observability
│   ├── core.rs             # AgentMonitor
│   ├── storage.rs          # Metric persistence
│   └── query.rs            # Metric queries
├── storage/                # Storage abstraction
│   ├── mod.rs              # StorageBackend trait
│   └── circuit_breaker.rs  # Resilience pattern
├── sync.rs                 # StorageSynchronizer
├── types.rs                # Core data structures
└── error.rs                # Error types

Total: ~15,000 LOC
```

### Key Components

#### 1. SelfLearningMemory (Main Orchestrator)

**Location**: `memory/mod.rs`

**Responsibilities**:
- Episode lifecycle management (start → log → complete)
- Pattern extraction coordination
- Context retrieval for decision-making
- Storage backend integration

**Key Methods**:
```rust
pub async fn start_episode(task: String, context: TaskContext, task_type: TaskType) -> Uuid
pub async fn log_step(episode_id: Uuid, step: ExecutionStep) -> Result<()>
pub async fn complete_episode(episode_id: Uuid, outcome: TaskOutcome, reward: RewardScore) -> Result<()>
pub async fn retrieve_context(query: &str, task_type: TaskType) -> Result<Vec<Episode>>
```

**Architecture**:
- StepBuffer for batching execution steps (reduces I/O)
- Async pattern extraction queue (decouples completion from extraction)
- Dual storage coordination (Turso + redb)

#### 2. Pattern Extraction System

**Extractors (7 types)**:
1. **HybridPatternExtractor** - Combines all strategies
2. **ContextPatternExtractor** - Domain/language-aware
3. **DecisionPointExtractor** - Conditional branches
4. **ErrorRecoveryExtractor** - Error handling patterns
5. **ToolSequenceExtractor** - Sequential tool usage
6. **HeuristicExtractor** - Condition-action rules
7. **CustomPatternExtractor** - User-defined patterns

**Pattern Types**:
```rust
pub enum Pattern {
    ToolSequence { id, tools, context, success_rate, avg_latency },
    DecisionPoint { id, condition, action, outcome_stats },
    ErrorRecovery { id, error_type, recovery_steps, success_rate },
    ContextPattern { id, domain, language, task_type, success_rate },
    Custom { id, pattern_data },
}
```

**Validation**:
- PatternValidator for quality assessment (precision, recall, F1)
- PatternClusterer for deduplication (cosine similarity)
- EffectivenessTracker for usage statistics
- Risk assessment and compatibility checks

#### 3. Reward Calculation

**Components**:
- Base outcome score (success/partial/failure)
- Efficiency multiplier (duration + step count)
- Complexity bonus (expert-level tasks rewarded)
- Quality multiplier (step success rate)
- Learning bonus (new patterns discovered)

**Formula**:
```rust
reward = base_score * efficiency * (1 + complexity_bonus) * quality * (1 + learning_bonus)
```

#### 4. Embeddings System

**Providers**:
1. **LocalEmbeddingProvider** - Offline via sentence-transformers (candle)
   - Model: gte-small-en-v1.5 (384-dimensional)
   - No API costs, privacy-preserving

2. **OpenAIEmbeddingProvider** - Cloud-based (feature-gated)
   - Model: text-embedding-ada-002 (1536-dimensional)
   - Highest accuracy, API costs apply

3. **MockEmbeddingProvider** - Testing only
   - Hash-based pseudo-embeddings
   - NOT semantically meaningful

**SemanticService**:
- Episode semantic search
- Pattern embedding and retrieval
- Context-aware recommendations
- Batch embedding generation

#### 5. Storage Abstraction

**StorageBackend Trait**:
```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn store_episode(&self, episode: &Episode) -> Result<()>;
    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>>;
    async fn store_pattern(&self, pattern: &Pattern) -> Result<()>;
    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>>;
    async fn query_episodes(&self, query: EpisodeQuery) -> Result<Vec<Episode>>;
    async fn query_patterns(&self, query: PatternQuery) -> Result<Vec<Pattern>>;
}
```

**Implementations**:
- TursoStorage (durable, cloud/local)
- RedbStorage (cache, embedded)

---

## Storage Backends

### Turso Storage: Durable Backend

**Location**: `memory-storage-turso/`

**Components**:
- `TursoStorage` - Main storage implementation
- `ConnectionPool` - Connection pooling with statistics
- `ResilientStorage` - Retry logic wrapper
- `EpisodeQuery`/`PatternQuery` - Type-safe query builders

**Schema (libSQL)**:
```sql
CREATE TABLE episodes (
    episode_id TEXT PRIMARY KEY,
    task_type TEXT NOT NULL,
    task_description TEXT,
    context TEXT,
    steps TEXT,
    outcome TEXT,
    reward TEXT,
    reflection TEXT,
    patterns TEXT,
    heuristics TEXT,
    metadata TEXT,
    domain TEXT,
    language TEXT,
    start_time INTEGER,
    end_time INTEGER,
    created_at INTEGER DEFAULT (unixepoch()),
    updated_at INTEGER DEFAULT (unixepoch())
);

CREATE INDEX idx_episodes_task_type ON episodes(task_type);
CREATE INDEX idx_episodes_start_time ON episodes(start_time);
CREATE INDEX idx_episodes_domain ON episodes(domain);

-- Similar for patterns, heuristics, execution_records, agent_metrics
```

**Resilience Features**:
- Exponential backoff retry (3 attempts)
- Circuit breaker pattern (prevents cascading failures)
- Connection pooling (configurable size)
- Transaction support

**Configuration**:
```rust
pub struct TursoConfig {
    pub max_retries: u32,          // Default: 3
    pub retry_backoff_ms: u64,     // Default: 100ms
    pub pool_size: usize,          // Default: 10
    pub connection_timeout_ms: u64, // Default: 5000ms
}
```

### Redb Storage: Cache Layer

**Location**: `memory-storage-redb/`

**Components**:
- `RedbStorage` - Main cache implementation
- `LRUCache` - In-memory LRU caching
- `CacheMetrics` - Hit/miss/eviction tracking

**Tables (embedded redb)**:
```rust
const EPISODES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("episodes");
const PATTERNS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("patterns");
const HEURISTICS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("heuristics");
const EMBEDDINGS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("embeddings");
const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");
```

**Performance**:
- Zero-copy reads
- Async wrappers for blocking operations
- Memory-mapped I/O
- Concurrent read access

**Security Limits**:
```rust
const MAX_EPISODE_SIZE: usize = 10 * 1024 * 1024;   // 10MB
const MAX_PATTERN_SIZE: usize = 1 * 1024 * 1024;    // 1MB
const MAX_HEURISTIC_SIZE: usize = 100 * 1024;       // 100KB
const MAX_EMBEDDING_SIZE: usize = 1 * 1024 * 1024;  // 1MB
```

### Storage Synchronization

**Location**: `memory-core/src/sync.rs`

**Strategy**: Turso is source of truth, redb is fast cache

**Synchronizer**:
```rust
pub struct StorageSynchronizer<P, C>
where
    P: StorageBackend,
    C: StorageBackend,
{
    primary: Arc<P>,     // Turso (durable)
    cache: Arc<C>,       // redb (fast)
    config: SyncConfig,
}
```

**Sync Operations**:
1. **Write**: Turso → redb (write-through cache)
2. **Read**: redb first (cache hit), fallback to Turso (cache miss)
3. **Periodic Sync**: Background task syncs Turso → redb
4. **Conflict Resolution**: Turso always wins

**Configuration**:
```rust
pub struct SyncConfig {
    pub sync_interval_secs: u64,    // Default: 300 (5 minutes)
    pub batch_size: usize,          // Default: 100
    pub enable_background_sync: bool, // Default: true
}
```

---

## Cross-References

- **Architecture Patterns**: See [ARCHITECTURE_PATTERNS.md](ARCHITECTURE_PATTERNS.md)
- **Integration Details**: See [ARCHITECTURE_INTEGRATION.md](ARCHITECTURE_INTEGRATION.md)
- **Configuration**: See [CONFIG_IMPLEMENTATION_ROADMAP.md](CONFIG_IMPLEMENTATION_ROADMAP.md)
- **Current Status**: See [ROADMAP_V017_CURRENT.md](ROADMAP_V017_CURRENT.md)

---

*Last Updated: 2025-12-21*
