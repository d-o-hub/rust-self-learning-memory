# API Documentation

**Document Version**: 1.0
**Created**: 2025-12-25
**Status**: Active
**Target**: Public API consumers and contributors

---

## Purpose

This document provides comprehensive API documentation for all public APIs in the Self-Learning Memory System. It covers the main crates: `do-memory-core`, `do-memory-storage-turso`, `do-memory-storage-redb`, and `do-memory-mcp`.

---

## Table of Contents

1. [do-memory-core API](#do-memory-core-api)
2. [do-memory-storage-turso API](#do-memory-storage-turso-api)
3. [do-memory-storage-redb API](#do-memory-storage-redb-api)
4. [do-memory-mcp API](#do-memory-mcp-api)
5. [Configuration Types](#configuration-types)
6. [Examples](#examples)

---

## do-memory-core API

### Core Types

#### SelfLearningMemory

The main orchestrator for episodic memory management.

**Location**: `do-memory-core/src/memory/mod.rs`

```rust
pub struct SelfLearningMemory {
    storage: Arc<dyn StorageBackend>,
    pattern_extractor: Arc<PatternExtractor>,
    reward_calculator: Arc<RewardCalculator>,
    reflection_generator: Arc<ReflectionGenerator>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    // ...
}

impl SelfLearningMemory {
    /// Create a new memory system with specified storage and configuration
    pub async fn new(
        storage: Arc<dyn StorageBackend>,
        config: MemoryConfig,
    ) -> Result<Self, MemoryError>;

    /// Start a new episode and return its ID
    pub async fn start_episode(
        &self,
        task: String,
        context: TaskContext,
        task_type: TaskType,
    ) -> Result<Uuid, MemoryError>;

    /// Log an execution step to an in-progress episode
    pub async fn log_step(
        &self,
        episode_id: Uuid,
        step: ExecutionStep,
    ) -> Result<(), MemoryError>;

    /// Complete an episode with outcome and reward
    pub async fn complete_episode(
        &self,
        episode_id: Uuid,
        outcome: TaskOutcome,
        reward: RewardScore,
    ) -> Result<(), MemoryError>;

    /// Retrieve relevant episodes based on query and context
    pub async fn retrieve_context(
        &self,
        query: &str,
        task_type: TaskType,
        limit: usize,
    ) -> Result<Vec<Episode>, MemoryError>;

    /// Get current monitoring summary
    pub fn get_monitoring_summary(&self) -> Result<MonitoringSummary, MemoryError>;
}
```

**Usage Example**:
```rust
use memory_core::{SelfLearningMemory, MemoryConfig, TaskContext, TaskType};
use memory_storage_redb::RedbStorage;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create storage backend
    let storage = Arc::new(RedbStorage::new("/path/to/cache.redb").await?);

    // Configure memory system
    let config = MemoryConfig::default();

    // Create memory system
    let memory = SelfLearningMemory::new(storage, config).await?;

    // Start an episode
    let episode_id = memory.start_episode(
        "Implement REST API",
        TaskContext::default(),
        TaskType::CodeGeneration,
    ).await?;

    // Complete episode
    memory.complete_episode(
        episode_id,
        TaskOutcome::Success { completed_steps: 5 },
        RewardScore::default(),
    ).await?;

    // Retrieve relevant context
    let relevant = memory.retrieve_context("REST API implementation", TaskType::CodeGeneration, 5).await?;

    Ok(())
}
```

---

### Episode and Related Types

#### Episode

Represents a complete execution episode.

**Location**: `do-memory-core/src/episode.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub episode_id: Uuid,
    pub task_type: TaskType,
    pub task_description: String,
    pub context: TaskContext,
    pub steps: Vec<ExecutionStep>,
    pub outcome: Option<TaskOutcome>,
    pub reward: Option<RewardScore>,
    pub reflection: Option<Reflection>,
    pub patterns: Vec<Pattern>,
    pub heuristics: Vec<Heuristic>,
    pub metadata: EpisodeMetadata,
    pub domain: String,
    pub language: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}
```

#### ExecutionStep

Represents a single step within an episode.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}
```

#### TaskContext

Provides context for an episode.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskContext {
    pub domain: String,
    pub language: String,
    pub environment: String,
    pub additional_context: HashMap<String, String>,
}
```

#### TaskOutcome

Represents the outcome of a task.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutcome {
    Success {
        completed_steps: usize,
        total_steps: usize,
    },
    PartialSuccess {
        completed_steps: usize,
        total_steps: usize,
        partial_completion_ratio: f32,
    },
    Failure {
        error: String,
        failed_at_step: usize,
        total_steps: usize,
    },
}
```

#### TaskType

Types of tasks the system can handle.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    CodeGeneration,
    Debugging,
    Refactoring,
    Testing,
    Documentation,
    Deployment,
    Analysis,
    Research,
    Other(String),
}
```

---

### Pattern Extraction API

#### PatternExtractor Trait

Abstract interface for pattern extraction strategies.

**Location**: `do-memory-core/src/patterns/extractor.rs`

```rust
#[async_trait]
pub trait PatternExtractor: Send + Sync {
    /// Extract patterns from a completed episode
    async fn extract_patterns(&self, episode: &Episode)
        -> Result<Vec<Pattern>, MemoryError>;

    /// Get extractor name
    fn name(&self) -> &str;

    /// Get extractor version
    fn version(&self) -> &str;
}
```

#### Pattern

Represents a learned pattern.

**Location**: `do-memory-core/src/patterns/types.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    ToolSequence {
        id: PatternId,
        tools: Vec<String>,
        context: TaskContext,
        success_rate: f32,
        avg_latency_ms: u64,
        effectiveness: f32, // Required field
        usage_count: usize,
        last_used: DateTime<Utc>,
    },
    DecisionPoint {
        id: PatternId,
        condition: String,
        action: String,
        outcome_stats: OutcomeStats,
        effectiveness: f32, // Required field
        usage_count: usize,
    },
    ErrorRecovery {
        id: PatternId,
        error_type: String,
        recovery_steps: Vec<String>,
        success_rate: f32,
        effectiveness: f32, // Required field
        usage_count: usize,
    },
    ContextPattern {
        id: PatternId,
        domain: String,
        language: String,
        task_type: TaskType,
        success_rate: f32,
        effectiveness: f32, // Required field
        usage_count: usize,
    },
}
```

#### HybridPatternExtractor

Combines multiple extraction strategies.

**Location**: `do-memory-core/src/patterns/extractors/hybrid.rs`

```rust
pub struct HybridPatternExtractor {
    extractors: Vec<Box<dyn PatternExtractor>>,
    confidence_threshold: f32,
}

impl HybridPatternExtractor {
    pub fn new(confidence_threshold: f32) -> Self {
        // Create default extractors
        let extractors = vec![
            Box::new(ToolSequenceExtractor::new()),
            Box::new(DecisionPointExtractor::new()),
            Box::new(ErrorRecoveryExtractor::new()),
            Box::new(ContextPatternExtractor::new()),
        ];
        Self { extractors, confidence_threshold }
    }
}
```

---

### Reward Calculation API

#### RewardCalculator

Calculates reward scores for completed episodes.

**Location**: `do-memory-core/src/reward.rs`

```rust
pub struct RewardCalculator {
    base_weights: RewardWeights,
}

impl RewardCalculator {
    pub fn new(weights: RewardWeights) -> Self {
        Self { base_weights: weights }
    }

    pub fn calculate_reward(
        &self,
        episode: &Episode,
        outcome: &TaskOutcome,
    ) -> Result<RewardScore, MemoryError> {
        // Calculate base score from outcome
        let base_score = self.calculate_base_score(outcome);

        // Apply multipliers
        let efficiency = self.calculate_efficiency(episode);
        let complexity = self.calculate_complexity_bonus(&episode.context);
        let quality = self.calculate_quality_multiplier(&episode.steps);
        let learning = self.calculate_learning_bonus(&episode.patterns);

        // Final reward
        let total = base_score * efficiency * (1.0 + complexity) * quality * (1.0 + learning);

        Ok(RewardScore {
            total,
            base: base_score,
            efficiency,
            complexity_bonus: complexity,
            quality,
            learning_bonus: learning,
        })
    }
}
```

#### RewardScore

Represents the calculated reward for an episode.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardScore {
    pub total: f32,
    pub base: f32,
    pub efficiency: f32,
    pub complexity_bonus: f32,
    pub quality: f32,
    pub_learning_bonus: f32,
}
```

---

### Embeddings API

#### EmbeddingProvider Trait

Abstract interface for embedding generation.

**Location**: `do-memory-core/src/embeddings/provider.rs`

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Generate embeddings for multiple texts (batch)
    async fn embed_batch(&self, texts: &[String])
        -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// Get embedding dimension
    fn dimension(&self) -> usize;

    /// Get provider name
    fn provider_name(&self) -> &str;
}
```

#### LocalEmbeddingProvider

Offline embeddings using local models.

**Location**: `do-memory-core/src/embeddings/local.rs`

```rust
pub struct LocalEmbeddingProvider {
    model_path: PathBuf,
    model: Option<Model>, // Loaded on-demand
    tokenizer: Option<Tokenizer>,
    dimension: usize,
}

impl LocalEmbeddingProvider {
    pub fn new(model_path: PathBuf) -> Result<Self, EmbeddingError> {
        // Initialize without loading model (lazy load)
        Ok(Self {
            model_path,
            model: None,
            tokenizer: None,
            dimension: 384, // gte-small-en-v1.5 default
        })
    }

    pub async fn load_model(&mut self) -> Result<(), EmbeddingError> {
        // Load model and tokenizer from model_path
        // ...
        Ok(())
    }
}
```

**Usage Example**:
```rust
use memory_core::embeddings::{LocalEmbeddingProvider, EmbeddingProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create local embedding provider
    let provider = LocalEmbeddingProvider::new("./models/gte-small".into())?;
    let mut provider = provider;

    // Load model
    provider.load_model().await?;

    // Generate embedding
    let embedding = provider.embed("Implement REST API").await?;

    println!("Embedding dimension: {}", embedding.len()); // 384

    Ok(())
}
```

---

### Storage API

#### StorageBackend Trait

Abstract interface for storage backends.

**Location**: `do-memory-core/src/storage/mod.rs`

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store a completed episode
    async fn store_episode(&self, episode: &Episode)
        -> Result<(), StorageError>;

    /// Retrieve an episode by ID
    async fn get_episode(&self, id: Uuid)
        -> Result<Option<Episode>, StorageError>;

    /// Store a pattern
    async fn store_pattern(&self, pattern: &Pattern)
        -> Result<(), StorageError>;

    /// Retrieve a pattern by ID
    async fn get_pattern(&self, id: PatternId)
        -> Result<Option<Pattern>, StorageError>;

    /// Query episodes with filters
    async fn query_episodes(&self, query: EpisodeQuery)
        -> Result<Vec<Episode>, StorageError>;

    /// Query patterns with filters
    async fn query_patterns(&self, query: PatternQuery)
        -> Result<Vec<Pattern>, StorageError>;
}
```

#### EpisodeQuery

Query builder for episode retrieval.

```rust
pub struct EpisodeQuery {
    pub task_type: Option<TaskType>,
    pub domain: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_reward: Option<f32>,
    pub limit: usize,
    pub offset: usize,
}
```

#### PatternQuery

Query builder for pattern retrieval.

```rust
pub struct PatternQuery {
    pub pattern_type: Option<String>,
    pub domain: Option<String>,
    pub min_success_rate: Option<f32>,
    pub min_usage_count: Option<usize>,
    pub limit: usize,
}
```

---

## do-memory-storage-turso API

### TursoStorage

Primary storage implementation using Turso/libSQL.

**Location**: `do-memory-storage-turso/src/storage.rs`

```rust
pub struct TursoStorage {
    pool: Arc<ConnectionPool>,
    database_url: String,
    auth_token: Option<String>,
}

impl TursoStorage {
    /// Create new Turso storage instance
    pub async fn new(config: TursoConfig) -> Result<Self, StorageError> {
        // Validate database URL and auth token
        // Create connection pool
        // Initialize schema
        // ...
    }

    /// Get connection from pool
    pub async fn get_connection(&self) -> Result<Connection, StorageError> {
        self.pool.acquire().await
    }

    /// Close all connections
    pub async fn close(&self) -> Result<(), StorageError> {
        self.pool.close().await
    }
}

#[async_trait]
impl StorageBackend for TursoStorage {
    // Implement all StorageBackend trait methods
    // ...
}
```

#### TursoConfig

Configuration for Turso storage.

```rust
pub struct TursoConfig {
    pub database_url: String,
    pub auth_token: Option<String>,
    pub pool_size: usize, // Default: 10
    pub max_retries: u32, // Default: 3
    pub retry_backoff_ms: u64, // Default: 100
    pub connection_timeout_ms: u64, // Default: 5000
}
```

**Usage Example**:
```rust
use memory_storage_turso::{TursoStorage, TursoConfig};
use memory_core::storage::StorageBackend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TursoConfig {
        database_url: "libsql://your-database.turso.io".to_string(),
        auth_token: Some("your-auth-token".to_string()),
        pool_size: 10,
        max_retries: 3,
        retry_backoff_ms: 100,
        connection_timeout_ms: 5000,
    };

    let storage = TursoStorage::new(config).await?;

    // Use as StorageBackend trait
    Ok(())
}
```

---

### ConnectionPool

Manages a pool of Turso connections.

**Location**: `do-memory-storage-turso/src/pool.rs`

```rust
pub struct ConnectionPool {
    connections: Arc<tokio::sync::Semaphore>,
    max_size: usize,
}

impl ConnectionPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: Arc::new(tokio::sync::Semaphore::new(max_size)),
            max_size,
        }
    }

    pub async fn acquire(&self) -> Result<Connection, StorageError> {
        let permit = self.connections.acquire().await?;
        // Create new connection
        // ...
    }

    pub async fn close(&self) -> Result<(), StorageError> {
        // Close all connections
        // ...
    }
}
```

---

## do-memory-storage-redb API

### RedbStorage

Cache layer implementation using redb.

**Location**: `do-memory-storage-redb/src/storage.rs`

```rust
pub struct RedbStorage {
    db: redb::Database,
    cache: Arc<LRUCache>,
    config: RedbConfig,
}

impl RedbStorage {
    /// Create new redb storage instance
    pub async fn new(path: PathBuf) -> Result<Self, StorageError> {
        // Open database
        // Initialize tables
        // Setup cache
        // ...
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Clear cache
    pub async fn clear_cache(&self) -> Result<(), StorageError> {
        self.cache.clear().await
    }
}

#[async_trait]
impl StorageBackend for RedbStorage {
    // Implement all StorageBackend trait methods
    // ...
}
```

#### RedbConfig

Configuration for redb storage.

```rust
pub struct RedbConfig {
    pub cache_size: usize, // Default: 5000 entries
    pub cache_ttl_secs: u64, // Default: 300 (5 minutes)
    pub max_episode_size: usize, // Default: 10MB
    pub max_pattern_size: usize, // Default: 1MB
}
```

---

### LRUCache

In-memory LRU cache.

**Location**: `do-memory-storage-redb/src/cache.rs`

```rust
pub struct LRUCache {
    capacity: usize,
    ttl: Duration,
    data: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
}

impl LRUCache {
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            capacity,
            ttl,
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        let data = self.data.read().await;
        // Check if entry exists and not expired
        // ...
    }

    pub async fn put(&self, key: CacheKey, value: Vec<u8>) {
        let mut data = self.data.write().await;
        // Evict if over capacity
        // Insert new entry
        // ...
    }
}
```

---

## do-memory-mcp API

### MemoryMCPServer

Main MCP server implementation.

**Location**: `do-memory-mcp/src/server/`

```rust
pub struct MemoryMCPServer {
    memory: Arc<SelfLearningMemory>,
    tool_registry: Arc<...>,
    config: SandboxConfig,
    // monitoring, audit logger, rate limiter, cache
}

impl MemoryMCPServer {
    /// Create new MCP server.
    ///
    /// `SandboxConfig` is still accepted for compatibility, but there is no
    /// production code-execution backend; agent code execution is fail-closed.
    pub async fn new(
        config: SandboxConfig,
        memory: Arc<SelfLearningMemory>,
    ) -> Result<Self> {
        // Build shared tool registry (lazy loading)
        // Initialize monitoring, audit logger, rate limiter, cache
        // ...
    }

    /// Access the underlying memory system
    pub fn memory(&self) -> Arc<SelfLearningMemory> { /* ... */ }

    /// Access the audit logger
    pub fn audit_logger(&self) -> Arc<AuditLogger> { /* ... */ }

    /// Access the rate limiter
    pub fn rate_limiter(&self) -> &RateLimiter { /* ... */ }

    /// Enforce rate limits for a request
    pub fn check_rate_limit(&self, /* client, operation */) -> ... { /* ... */ }
}
```

The stdio JSON-RPC loop itself lives in the binary
(`do-memory-mcp/src/bin/memory-mcp-server.rs`), not on `MemoryMCPServer`.

---

### MCP Tools

The MCP server provides the following tools:

#### 1. query_memory

Retrieve relevant episodes based on query.

```rust
async fn handle_query_memory(
    &self,
    params: QueryMemoryParams,
) -> Result<QueryMemoryResult, MCPServerError>
```

**Parameters**:
```json
{
  "query": "string",
  "domain": "string?",
  "task_type": "string?",
  "limit": "number?"
}
```

**Returns**:
```json
{
  "episodes": [
    {
      "episode_id": "uuid",
      "task_type": "CodeGeneration",
      "task_description": "string",
      "context": {...},
      "steps": [...],
      "outcome": {...},
      "reward": {...}
    }
  ],
  "total_count": 42
}
```

---

#### 2. execute_agent_code

**Unavailable / fail-closed.** The WASM sandbox was removed in v0.1.29
(ADR-052); there is no working code-execution backend. The tool is not
advertised by `tools/list` and direct calls are rejected by the dispatcher.

**Observed rejection**:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Tool execution failed",
    "data": {
      "details": "execute_agent_code tool is not available due to WASM sandbox compilation issues"
    }
  }
}
```

Use episode/memory tools (`query_memory`, `create_episode`, …) instead. See
ADR-073 for the intended future capability-enforced contract.

---

#### 3. analyze_patterns

Analyze patterns with statistics.

```rust
async fn handle_analyze_patterns(
    &self,
    params: AnalyzePatternsParams,
) -> Result<AnalyzePatternsResult, MCPServerError>
```

**Parameters**:
```json
{
  "task_type": "string?",
  "min_success_rate": "number?",
  "limit": "number?"
}
```

**Returns**:
```json
{
  "patterns": [
    {
      "id": "uuid",
      "type": "ToolSequence",
      "success_rate": 0.85,
      "avg_latency_ms": 150,
      "usage_count": 42
    }
  ],
  "total_count": 15
}
```

---

#### 4. advanced_pattern_analysis

Comprehensive pattern analysis (statistical + predictive + causal).

```rust
async fn handle_advanced_pattern_analysis(
    &self,
    params: AdvancedPatternAnalysisParams,
) -> Result<AdvancedPatternAnalysisResult, MCPServerError>
```

**Parameters**:
```json
{
  "analysis_type": "comprehensive",
  "time_series_data": {
    "success_rate": [0.8, 0.85, 0.9, ...],
    "latency": [100, 95, 90, ...]
  },
  "config": {
    "forecast_horizon": 10,
    "significance_level": 0.05
  }
}
```

**Returns**:
```json
{
  "statistical_tests": {
    "t_test": {
      "t_statistic": 2.34,
      "p_value": 0.019,
      "significant": true
    },
    "correlation": {
      "pearson": 0.85,
      "p_value": 0.0001
    }
  },
  "forecasts": {
    "success_rate": [0.91, 0.92, ...],
    "confidence_intervals": [[0.88, 0.94], ...]
  },
  "anomalies": [
    {
      "index": 42,
      "value": 0.65,
      "anomaly_score": 2.34
    }
  ],
  "changepoints": [
    {
      "index": 50,
      "probability": 0.87
    }
  ]
}
```

---

### Code Execution API

There is **no** production code-execution API. `do-memory-mcp/src/unified_sandbox.rs`
and `do-memory-mcp/src/wasmtime_sandbox.rs` do not exist; the WASM sandbox and
its backends were removed in v0.1.29 (ADR-052). `execute_agent_code` is
fail-closed.

A legacy Node.js executor survives behind the non-default `sandbox-dev` feature
for trusted local experimentation only:

```rust
// Available ONLY with `--features sandbox-dev`; trusted local use only.
// Not a production sandbox — see memory-mcp/SECURITY.md.
#[cfg(feature = "sandbox-dev")]
pub struct CodeSandbox { /* ... */ }

#[cfg(feature = "sandbox-dev")]
impl CodeSandbox {
    pub fn new(config: SandboxConfig) -> Result<Self> { /* ... */ }
    pub async fn execute(&self, code: &str, context: ExecutionContext)
        -> Result<ExecutionResult> { /* ... */ }
}
```

Enforced in that path: execution timeout (`kill_on_drop`), 100 KB code-length
cap, static regex source screening, wrapper global shadowing/deletion.
**Not** enforced: OS-level isolation, memory/CPU limits, output sanitization,
runtime capability enforcement.

---

## Configuration Types

### MemoryConfig

Configuration for do-memory-core.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub pattern_extraction_enabled: bool, // Default: true
    pub reward_calculation_enabled: bool, // Default: true
    pub reflection_generation_enabled: bool, // Default: true
    pub embedding_enabled: bool, // Default: false
    pub cache_enabled: bool, // Default: true
}
```

### Server configuration

There is no `ServerConfig` type. The server binary runs a stdio JSON-RPC loop;
runtime behavior is configured through environment variables
(`MCP_OAUTH_*`, `MCP_RATE_LIMIT_*`, `AUDIT_LOG_*`).

### SandboxConfig

Configuration accepted by `MemoryMCPServer::new`. With no registered execution
backend it no longer governs a production code path; treat its limits as
advisory configuration only.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub max_execution_time_ms: u64,   // Default: 5000
    pub max_memory_mb: usize,         // Default: 128 (advisory)
    pub max_cpu_percent: u8,          // Default: 50  (advisory)
    pub allowed_paths: Vec<String>,   // Default: []
    pub allowed_network: Vec<String>, // Default: []
    pub allow_network: bool,          // Default: false
    pub allow_filesystem: bool,       // Default: false
    pub allow_subprocesses: bool,     // Default: false
    pub resource_limits: ResourceLimits,
    pub process_uid: Option<u32>,     // Default: None
    pub read_only_mode: bool,         // Default: false
}
```

---

## Examples

### Example 1: Basic Memory System Setup

```rust
use memory_core::{SelfLearningMemory, MemoryConfig, TaskContext, TaskType};
use memory_storage_redb::RedbStorage;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup storage (redb cache)
    let storage = Arc::new(RedbStorage::new("memory.redb").await?);

    // Configure memory system
    let config = MemoryConfig::default();

    // Create memory system
    let memory = SelfLearningMemory::new(storage, config).await?;

    // Start episode
    let episode_id = memory.start_episode(
        "Debug failing test",
        TaskContext {
            domain: "testing".to_string(),
            language: "rust".to_string(),
            ..Default::default()
        },
        TaskType::Debugging,
    ).await?;

    println!("Started episode: {}", episode_id);

    // Complete episode
    use memory_core::{TaskOutcome, RewardScore};
    memory.complete_episode(
        episode_id,
        TaskOutcome::Success {
            completed_steps: 3,
            total_steps: 3,
        },
        RewardScore { total: 0.9, ..Default::default() },
    ).await?;

    // Retrieve relevant episodes
    let relevant = memory.retrieve_context("test debugging", TaskType::Debugging, 5).await?;
    println!("Found {} relevant episodes", relevant.len());

    Ok(())
}
```

---

### Example 2: Dual Storage (Turso + redb)

```rust
use memory_core::{SelfLearningMemory, MemoryConfig};
use memory_storage_turso::{TursoStorage, TursoConfig};
use memory_storage_redb::{RedbStorage, RedbConfig};
use memory_core::storage::{StorageBackend, StorageSynchronizer};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup Turso (primary)
    let turso_config = TursoConfig {
        database_url: "libsql://your-database.turso.io".to_string(),
        auth_token: Some(std::env::var("TURSO_TOKEN")?),
        ..Default::default()
    };
    let primary = Arc::new(TursoStorage::new(turso_config).await?) as Arc<dyn StorageBackend>;

    // Setup redb (cache)
    let redb_config = RedbConfig::default();
    let cache = Arc::new(RedbStorage::new("cache.redb".into()).await?) as Arc<dyn StorageBackend>;

    // Create synchronizer
    let storage = StorageSynchronizer::new(primary.clone(), cache.clone(), SyncConfig::default());

    // Create memory system with synchronized storage
    let config = MemoryConfig::default();
    let memory = SelfLearningMemory::new(Arc::new(storage), config).await?;

    // Use memory system
    // ...

    Ok(())
}
```

---

### Example 3: MCP Server with Pattern Analysis

```rust
use memory_core::{SelfLearningMemory, MemoryConfig};
use memory_storage_redb::RedbStorage;
use memory_mcp::{MemoryMCPServer, ServerConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup memory system
    let storage = Arc::new(RedbStorage::new("memory.redb").await?);
    let memory = Arc::new(SelfLearningMemory::new(storage, MemoryConfig::default()).await?);

    // Configure MCP server
    let server_config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        ..Default::default()
    };

    // Create MCP server
    let server = MemoryMCPServer::new(memory, server_config).await?;

    // Start server (runs until stopped)
    println!("MCP server listening on {}:{}", server_config.host, server_config.port);
    server.run().await?;

    Ok(())
}
```

---

### Example 4: Custom Pattern Extractor

```rust
use memory_core::patterns::{PatternExtractor, Pattern, MemoryError};
use memory_core::episode::Episode;

#[async_trait::async_trait]
pub struct MyCustomExtractor;

impl MyCustomExtractor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PatternExtractor for MyCustomExtractor {
    async fn extract_patterns(&self, episode: &Episode)
        -> Result<Vec<Pattern>, MemoryError>
    {
        // Custom extraction logic here
        let patterns = vec![];
        Ok(patterns)
    }

    fn name(&self) -> &str {
        "MyCustomExtractor"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}
```

---

### Example 5: Local Embeddings

```rust
use memory_core::embeddings::{LocalEmbeddingProvider, EmbeddingProvider};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create local embedding provider
    let provider = LocalEmbeddingProvider::new("./models/gte-small".into())?;
    let mut provider = provider;

    // Load model (lazy load)
    provider.load_model().await?;

    // Generate embeddings
    let texts = vec![
        "Implement REST API".to_string(),
        "Debug failing test".to_string(),
        "Refactor code".to_string(),
    ];

    let embeddings = provider.embed_batch(&texts).await?;

    // Calculate similarity
    let embedding1 = &embeddings[0];
    let embedding2 = &embeddings[1];

    let similarity = cosine_similarity(embedding1, embedding2);
    println!("Similarity: {:.4}", similarity);

    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    // Cosine similarity calculation
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot_product / (norm_a * norm_b)
}
```

---

## Error Handling

### MemoryError

Error type for do-memory-core operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Pattern extraction error: {0}")]
    PatternExtraction(String),

    #[error("Reward calculation error: {0}")]
    RewardCalculation(String),

    #[error("Embedding error: {0}")]
    Embedding(#[from] EmbeddingError),

    #[error("Episode not found: {0}")]
    EpisodeNotFound(Uuid),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}
```

### StorageError

Error type for storage operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] postcard::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Capacity exceeded: {0}")]
    CapacityExceeded(String),
}
```

---

## Best Practices

1. **Use Arc for Shared State**: All storage and major components use `Arc<T>` for thread-safe sharing.

2. **Handle Errors Gracefully**: All async methods return `Result<T, Error>`. Handle errors with `?` or `match`.

3. **Use Dependency Injection**: Pass `Arc<dyn StorageBackend>` to allow easy backend swapping.

4. **Leverage Caching**: Use redb cache for frequently accessed data to reduce Turso API calls.

5. **Monitor Metrics**: Use `get_monitoring_summary()` to track system health.

6. **Pattern Extraction is Async**: Pattern extraction runs in background, don't block on it.

7. **Use Simple Config**: Start with `MemoryConfig::default()` and only override necessary fields.

---

## Versioning

API version follows Semantic Versioning (SemVer):
- **Major (X.0.0)**: Breaking changes to public APIs
- **Minor (0.X.0)**: New features, backward compatible
- **Patch (0.0.X)**: Bug fixes, backward compatible

Current version: **0.1.7**

---

## References

- [do-memory-core Documentation](../../memory-core/README.md)
- [do-memory-storage-turso Documentation](../../memory-storage-turso/README.md)
- [do-memory-storage-redb Documentation](../../memory-storage-redb/README.md)
- [do-memory-mcp Documentation](../../memory-mcp/README.md)
- [Architecture Decision Records](../adr/)
- Current Architecture - See ROADMAP_ACTIVE.md for current state

---

**Document Maintainer**: Project Maintainers
**Last Updated**: 2025-12-25
**Next Review**: With each minor version release (0.1.8+)
