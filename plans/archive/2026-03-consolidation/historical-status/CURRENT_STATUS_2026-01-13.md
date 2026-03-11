# Architecture Overview - Current Status

**Version**: v0.1.12
**Date**: 2026-01-13
**Purpose**: Bridge between outdated documentation and actual codebase reality

## Executive Summary

This document provides an accurate snapshot of the Rust Self-Learning Memory System as of v0.1.12. It corrects discrepancies between existing documentation and actual implementation, enabling developers to work with the real codebase structure.

**Key Changes from Documentation**:
- Actual LOC: **~35,466** (not ~15,000 as documented)
- **6 major new modules** not in original docs
- **11+ MCP tools** (not 8 as documented)
- **CLI V2 structure** with episode_v2 subcommands
- **Significant API signature changes** from documented versions

---

## 1. Actual Module Structure

### Core Library: memory-core/src/

The memory-core library contains 134 Rust files totaling **35,466 lines of code**.

#### **Existing Modules** (from original docs)

```
constants.rs
error.rs
episode.rs
lib.rs
types/
├── config.rs
├── constants.rs
├── enums.rs
├── structs.rs
└── tests.rs

embeddings/
├── circuit_breaker.rs
├── config.rs
├── local.rs
├── metrics.rs
├── mock_model.rs
├── mod.rs
├── openai.rs
├── provider.rs
├── real_model.rs
├── similarity.rs
├── storage.rs
├── tests.rs
└── utils.rs

extraction/
├── extractor.rs
├── extractors/mod.rs
├── mod.rs
├── tests.rs
└── utils.rs

learning/
├── mod.rs
└── queue.rs

memory/
├── episode.rs
├── init.rs
├── learning.rs
├── management.rs
├── mod.rs
├── monitoring/
├── pattern_search.rs
├── queries/
├── retrieval/
│   ├── context.rs
│   ├── helpers.rs
│   ├── heuristics.rs
│   ├── patterns.rs
│   └── scoring.rs
├── retrieval.rs
├── step_buffer/
│   ├── config.rs
│   └── mod.rs
├── tests.rs
└── validation.rs

monitoring/
├── core.rs
├── mod.rs
├── storage.rs
└── types.rs

pattern/
├── heuristic.rs
├── mod.rs
├── similarity.rs
└── types.rs

patterns/
├── clustering.rs
├── effectiveness.rs
├── extractors/
│   ├── clustering.rs
│   ├── context_pattern.rs
│   ├── coverage_tests.rs
│   ├── decision_point.rs
│   ├── error_recovery.rs
│   ├── heuristic/
│   ├── hybrid.rs
│   ├── mod.rs
│   └── tool_sequence.rs
├── mod.rs
├── optimized_validator/
│   ├── applicator.rs
│   ├── planned_step.rs
│   ├── risk.rs
│   ├── tool.rs
│   └── validator.rs
├── optimized_validator.rs
└── validation.rs

reflection/
├── coverage_tests.rs
├── helpers.rs
├── improvement_analyzer.rs
├── insight_generator.rs
├── mod.rs
├── success_analyzer.rs
└── tests.rs

reward/
├── adaptive.rs
├── base.rs
├── constants.rs
├── domain_stats.rs
├── efficiency.rs
└── tests.rs

storage/
└── circuit_breaker/
    ├── mod.rs
    ├── states.rs
    └── tests.rs
```

#### **NEW Modules** (not in original documentation)

##### 1. retrieval/ - Query Caching
**Location**: `/workspaces/feat-phase3/memory-core/src/retrieval/`
**Purpose**: Efficient episode retrieval with LRU caching and TTL

```
retrieval/
├── cache/
│   ├── lru.rs          # LRU cache implementation
│   ├── mod.rs
│   ├── tests.rs
│   └── types.rs        # CacheKey, CacheMetrics, QueryCache
└── mod.rs
```

**Key Types**:
- `QueryCache`: LRU cache with TTL support
- `CacheKey`: Unique cache key for queries
- `CacheMetrics`: Hit rate, miss rate, eviction tracking

**Usage Example**:
```rust
use memory_core::retrieval::QueryCache;

let cache = QueryCache::new(1000, std::time::Duration::from_secs(300));
let results = cache.get_or_compute(key, || expensive_query()).await;
```

##### 2. spatiotemporal/ - Hierarchical Indexing
**Location**: `/workspaces/feat-phase3/memory-core/src/spatiotemporal/`
**Purpose**: Domain → task_type → temporal clustering for efficient retrieval

```
spatiotemporal/
├── diversity/
│   ├── maximizer.rs    # MMR-based result diversity
│   ├── mod.rs
│   ├── tests.rs
│   └── types.rs
├── embeddings/
│   └── tests.rs
├── embeddings.rs
├── index/
│   └── types.rs
├── index.rs           # Hierarchical index implementation
├── mod.rs
├── README.md
├── retriever/
│   ├── mod.rs
│   ├── scoring.rs
│   ├── tests.rs
│   └── types.rs
└── types.rs
```

**Key Types**:
- `SpatiotemporalIndex`: Hierarchical index structure
- `HierarchicalRetriever`: Efficient multi-level retrieval
- `DiversityMaximizer`: MMR algorithm for diverse results

**Architecture**:
```
SpatiotemporalIndex
├── domain_index: HashMap<Domain, DomainNode>
│   ├── task_type_index: HashMap<TaskType, TaskNode>
│   │   ├── temporal_clusters: Vec<TemporalCluster>
│   │   └── episode_ids: Vec<Uuid>
│   └── embeddings: Option<EmbeddingVector>
```

##### 3. semantic/ - Semantic Summarization
**Location**: `/workspaces/feat-phase3/memory-core/src/semantic/`
**Purpose**: Episode compression and summarization

```
semantic/
└── summary/
    ├── extractors.rs
    ├── helpers.rs
    ├── mod.rs
    ├── summarizer.rs    # Main summarization logic
    └── types.rs        # Summary types
```

**Key Types**:
- `SemanticSummarizer`: Summarization engine
- `EpisodeSummary`: Compressed episode representation

##### 4. episodic/ - Capacity Management
**Location**: `/workspaces/feat-phase3/memory-core/src/episodic/`
**Purpose**: Episode storage limits and eviction policies

```
episodic/
├── capacity.rs        # 20,465 LOC - Large capacity management system
└── mod.rs
```

**Key Types**:
- `CapacityManager`: Manages storage limits
- `EvictionPolicy`: LRU, FIFO, LFU strategies
- `CapacityConfig`: Storage thresholds and limits

**Reference**: `/workspaces/feat-phase3/memory-core/src/memory/mod.rs:245`

##### 5. pre_storage/ - PREMem (Quality Assessment)
**Location**: `/workspaces/feat-phase3/memory-core/src/pre_storage/`
**Purpose**: Pre-storage reasoning for quality enhancement (EMNLP 2025 approach)

```
pre_storage/
├── extractor/
│   ├── decisions.rs    # Decision pattern extraction
│   ├── insights.rs    # Insight extraction
│   ├── mod.rs
│   ├── recovery.rs    # Error recovery patterns
│   ├── tests.rs
│   ├── tools.rs       # Tool usage patterns
│   └── types.rs
├── mod.rs
└── quality.rs        # 22,626 LOC - Quality assessment engine
```

**Key Types**:
- `QualityAssessor`: Episode quality scoring
- `SalientExtractor`: Salient feature extraction
- `SalientFeatures`: Extracted key features
- `QualityConfig`: Assessment thresholds

**Reference**: `/workspaces/feat-phase3/memory-core/src/memory/mod.rs:212-213`

##### 6. search/ - Hybrid Search
**Location**: `/workspaces/feat-phase3/memory-core/src/search/`
**Purpose**: Combined semantic + keyword search

```
search/
├── hybrid.rs         # Hybrid search implementation (10,556 LOC)
└── mod.rs
```

**Key Types**:
- `HybridSearchEngine`: Combined search strategy
- `SearchResult`: Ranked results with scores
- `SearchConfig`: Weighting and ranking parameters

---

## 2. API Signature Corrections

### SelfLearningMemory Constructor

**Documented (INCORRECT)**:
```rust
// Documentation says async with storage parameter
async fn new(storage: Arc<dyn StorageBackend>) -> Result<Self>
```

**Actual (CORRECT)**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/memory/mod.rs:285
#[must_use]
pub fn new() -> Self {
    init::with_config(MemoryConfig::default())
}

// Or with config
#[must_use]
pub fn with_config(config: MemoryConfig) -> Self {
    init::with_config(config)
}

// Or with storage (separate method)
pub fn with_storage(
    config: MemoryConfig,
    turso: Arc<dyn StorageBackend>,
    cache: Arc<dyn StorageBackend>,
) -> Self {
    init::with_storage(config, turso, cache)
}
```

**Key Differences**:
- ❌ NOT async
- ❌ NO storage parameter in `new()`
- ✅ Returns `Self` directly (not `Result<Self>`)
- ✅ Has separate `with_storage()` method

### start_episode Method

**Documented (INCORRECT)**:
```rust
// Documentation says returns Result<Uuid>
async fn start_episode(...) -> Result<Uuid>
```

**Actual (CORRECT)**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/memory/episode.rs
pub async fn start_episode(
    &self,
    task_description: String,
    context: TaskContext,
    task_type: TaskType,
) -> Uuid {
    // Validation warnings are logged but don't prevent episode creation
    // ...
    episode.episode_id
}
```

**Key Differences**:
- ❌ NOT `Result<Uuid>`
- ✅ Returns `Uuid` directly
- ✅ Validation only produces warnings

### complete_episode Method

**Documented (INCORRECT)**:
```rust
// Documentation says takes reward parameter
async fn complete_episode(
    episode_id: Uuid,
    outcome: TaskOutcome,
    reward: RewardScore  // ❌ This parameter doesn't exist
) -> Result<()>
```

**Actual (CORRECT)**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/memory/learning.rs
pub async fn complete_episode(
    &self,
    episode_id: Uuid,
    outcome: TaskOutcome  // No reward parameter
) -> Result<()> {
    // Flush any buffered steps before completing
    // Reward is calculated internally from episode data
    // ...
}
```

**Key Differences**:
- ❌ NO reward parameter
- ✅ Reward calculated internally from episode execution
- ✅ Adaptive reward calculation based on outcome, steps, efficiency

### retrieve_relevant_context Method

**Documented (INCORRECT)**:
```rust
// Old documentation signature
async fn retrieve_relevant_context(
    query: String,
    context: TaskContext,
    limit: usize
) -> Result<Vec<Episode>>
```

**Actual (CORRECT)**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/memory/retrieval.rs
// Multiple retrieval methods with different signatures

// Basic retrieval
pub async fn retrieve_relevant_context(
    &self,
    query: String,
    context: TaskContext,
    limit: usize,
) -> Result<Vec<Episode>>

// Semantic retrieval (new in v0.1.12)
pub async fn search_patterns_semantic(
    &self,
    query: &str,
    context: TaskContext,
    limit: usize,
) -> Result<Vec<PatternSearchResult>>

// Pattern recommendations
pub async fn recommend_patterns_for_task(
    &self,
    task_description: &str,
    context: TaskContext,
    limit: usize,
) -> Result<Vec<PatternSearchResult>>
```

---

## 3. Data Structure Corrections

### TaskContext

**Documented (INCORRECT)**:
```rust
pub struct TaskContext {
    pub language: String,     // ❌ Not optional
    pub framework: String,    // ❌ Not optional
    pub domain: String,
    pub tags: HashMap<String, String>,  // ❌ Wrong type
}
```

**Actual (CORRECT)**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/types/structs.rs:45
pub struct TaskContext {
    /// Programming language (e.g., "rust", "python")
    pub language: Option<String>,      // ✅ Optional
    /// Framework used (e.g., "tokio", "fastapi")
    pub framework: Option<String>,     // ✅ Optional
    /// Task complexity level
    pub complexity: ComplexityLevel,    // ✅ New field
    /// Domain or category (e.g., "web-api", "data-processing")
    pub domain: String,
    /// Additional tags for categorization
    pub tags: Vec<String>,            // ✅ Vec, not HashMap
}
```

**Usage Example**:
```rust
use memory_core::{TaskContext, ComplexityLevel};

let context = TaskContext {
    language: Some("rust".to_string()),
    framework: Some("axum".to_string()),
    complexity: ComplexityLevel::Moderate,
    domain: "web-api".to_string(),
    tags: vec!["rest".to_string(), "async".to_string()],
};
```

### TaskOutcome

**Documented (INCORRECT)**:
```rust
// Old enum with simple variants
pub enum TaskOutcome {
    Success(String),
    Failure(String),
}
```

**Actual (CORRECT)**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/types/enums.rs:132
pub enum TaskOutcome {
    /// Task completed successfully with all objectives met
    Success {
        /// Summary of what was accomplished
        verdict: String,
        /// Files or outputs produced
        artifacts: Vec<String>,
    },
    /// Task partially completed with some objectives met
    PartialSuccess {
        /// Summary of partial completion
        verdict: String,
        /// Items successfully completed
        completed: Vec<String>,
        /// Items that failed or weren't completed
        failed: Vec<String>,
    },
    /// Task failed to complete
    Failure {
        /// High-level reason for failure
        reason: String,
        /// Detailed error information (optional)
        error_details: Option<String>,
    },
}
```

**Usage Example**:
```rust
use memory_core::TaskOutcome;

// Complete success
let success = TaskOutcome::Success {
    verdict: "All tests passing".to_string(),
    artifacts: vec!["auth.rs".to_string(), "test.rs".to_string()],
};

// Partial success
let partial = TaskOutcome::PartialSuccess {
    verdict: "Core functionality working".to_string(),
    completed: vec!["login".to_string(), "logout".to_string()],
    failed: vec!["password_reset".to_string()],
};

// Failure
let failure = TaskOutcome::Failure {
    reason: "Compilation errors".to_string(),
    error_details: Some("Type mismatch on line 42".to_string()),
};
```

### ExecutionStep

**Documented (INCORRECT)**:
```rust
pub struct ExecutionStep {
    pub step_id: Uuid,              // ❌ Wrong field name
    pub tool_name: String,          // ❌ Wrong field name
    pub input: serde_json::Value,   // ❌ Wrong field name
    pub output: Option<String>,     // ❌ Wrong field name
    // ...
}
```

**Actual (CORRECT)**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/episode.rs:91
pub struct ExecutionStep {
    /// Step number in sequence (1-indexed)
    pub step_number: usize,                          // ✅ step_number, not step_id
    /// When this step was executed
    pub timestamp: DateTime<Utc>,
    /// Tool or function used
    pub tool: String,                                // ✅ tool, not tool_name
    /// Description of action taken
    pub action: String,
    /// Input parameters (as JSON)
    pub parameters: serde_json::Value,                // ✅ parameters, not input
    /// Result of execution
    pub result: Option<ExecutionResult>,             // ✅ result, not output
    /// Execution time in milliseconds
    pub latency_ms: u64,
    /// Number of tokens used (if applicable)
    pub tokens_used: Option<usize>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}
```

**Usage Example**:
```rust
use memory_core::{ExecutionStep, ExecutionResult};
use serde_json::json;

let mut step = ExecutionStep::new(
    1,                                    // step_number (not step_id)
    "file_reader".to_string(),              // tool (not tool_name)
    "Read config file".to_string(),
);

step.parameters = json!({"path": "/etc/config.toml"});  // parameters (not input)
step.result = Some(ExecutionResult::Success {
    output: "Config loaded successfully".to_string(),
});  // result (not output)
step.latency_ms = 15;
```

### Pattern

**Documented (INCORRECT)**:
```rust
pub struct Pattern {
    pub id: PatternId,
    pub effectiveness: f32,          // ❌ Wrong type
    pub usage_count: usize,          // ❌ Wrong field name
    // ...
}
```

**Actual (CORRECT)**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/pattern/types.rs:107
pub enum Pattern {
    ToolSequence {
        id: PatternId,
        tools: Vec<String>,
        context: TaskContext,
        success_rate: f32,
        avg_latency: Duration,
        occurrence_count: usize,          // ✅ occurrence_count, not usage_count
        effectiveness: PatternEffectiveness,  // ✅ Struct, not f32
    },
    DecisionPoint {
        id: PatternId,
        condition: String,
        action: String,
        outcome_stats: OutcomeStats,
        context: TaskContext,
        effectiveness: PatternEffectiveness,
    },
    ErrorRecovery {
        id: PatternId,
        error_type: String,
        recovery_steps: Vec<String>,
        success_rate: f32,
        context: TaskContext,
        effectiveness: PatternEffectiveness,
    },
    ContextPattern {
        id: PatternId,
        context_features: Vec<String>,
        recommended_approach: String,
        evidence: Vec<Uuid>,
        success_rate: f32,
        effectiveness: PatternEffectiveness,
    },
}
```

**PatternEffectiveness Structure**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/pattern/types.rs:12
pub struct PatternEffectiveness {
    /// Number of times this pattern was retrieved in queries
    pub times_retrieved: usize,
    /// Number of times this pattern was explicitly applied
    pub times_applied: usize,
    /// Number of successful outcomes when applied
    pub success_when_applied: usize,
    /// Number of failed outcomes when applied
    pub failure_when_applied: usize,
    /// Average reward improvement when this pattern was used
    pub avg_reward_delta: f32,
    /// When this pattern was last used
    pub last_used: DateTime<Utc>,
    /// When this pattern was created
    pub created_at: DateTime<Utc>,
}
```

---

## 4. New Features Not Documented

### Query Caching

**Module**: `retrieval/cache/`
**File**: `/workspaces/feat-phase3/memory-core/src/retrieval/cache/lru.rs`

**Features**:
- LRU cache for query results
- TTL-based cache expiration
- Configurable max entries (default: 1000)
- Metrics tracking (hit rate, miss rate, evictions)

**API**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/memory/mod.rs:372-374
pub fn get_cache_metrics(&self) -> crate::retrieval::CacheMetrics {
    monitoring::get_cache_metrics(&self.query_cache)
}

pub fn clear_cache_metrics(&self) {
    monitoring::clear_cache_metrics(&self.query_cache);
}

pub fn clear_cache(&self) {
    monitoring::clear_cache(&self.query_cache);
}
```

### Spatiotemporal Indexing

**Module**: `spatiotemporal/`
**File**: `/workspaces/feat-phase3/memory-core/src/spatiotemporal/mod.rs`

**Features**:
- Hierarchical indexing: Domain → TaskType → Temporal Clusters
- Context-aware embeddings
- Diverse result selection using MMR (Maximal Marginal Relevance)
- Efficient multi-level retrieval

**Architecture**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/spatiotemporal/types.rs
pub struct SpatiotemporalIndex {
    pub domain_index: HashMap<String, DomainNode>,
}

pub struct DomainNode {
    pub task_type_index: HashMap<TaskType, TaskNode>,
    pub embeddings: Option<EmbeddingVector>,
}

pub struct TaskNode {
    pub temporal_clusters: Vec<TemporalCluster>,
    pub episode_ids: Vec<Uuid>,
}
```

**Integration in SelfLearningMemory**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/memory/mod.rs:252-258
pub(super) spatiotemporal_index:
    Option<Arc<RwLock<crate::spatiotemporal::SpatiotemporalIndex>>>,
pub(super) hierarchical_retriever: Option<crate::spatiotemporal::HierarchicalRetriever>,
pub(super) diversity_maximizer: Option<crate::spatiotemporal::DiversityMaximizer>,
```

### Pre-Storage Reasoning (PREMem)

**Module**: `pre_storage/`
**File**: `/workspaces/feat-phase3/memory-core/src/pre_storage/mod.rs`

**Features**:
- Quality assessment before storing episodes
- Salient feature extraction
- Multi-dimensional quality scoring
- Configurable quality thresholds

**Reference**: EMNLP 2025 paper on Pre-Storage Reasoning

**Quality Features**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/pre_storage/quality.rs
pub struct QualityFeature {
    pub name: String,
    pub score: f32,
    pub weight: f32,
    pub description: String,
}

// Assessed features:
// - Execution completeness
// - Step success rate
// - Error recovery patterns
// - Tool diversity
// - Code quality (if applicable)
// - Test coverage
// - Documentation quality
```

### Capacity Management

**Module**: `episodic/capacity.rs`
**File**: `/workspaces/feat-phase3/memory-core/src/episodic/capacity.rs`

**Features**:
- Configurable storage limits (episode count, total size)
- Multiple eviction policies: LRU, FIFO, LFU, Hybrid
- Automatic eviction when thresholds exceeded
- Priority-based retention

**API**:
```rust
pub struct CapacityManager {
    pub max_episodes: usize,
    pub max_total_size_bytes: u64,
    pub eviction_policy: EvictionPolicy,
    pub retention_priority: RetentionPriority,
}

pub enum EvictionPolicy {
    LeastRecentlyUsed,
    FirstInFirstOut,
    LeastFrequentlyUsed,
    Hybrid,
}
```

### Hybrid Search

**Module**: `search/hybrid.rs`
**File**: `/workspaces/feat-phase3/memory-core/src/search/hybrid.rs`

**Features**:
- Combines semantic and keyword search
- Configurable weighting between search strategies
- Multi-signal ranking
- Result diversification

**Search Pipeline**:
```
Query
  ↓
┌─────────────────┬─────────────────┐
│ Semantic Search  │ Keyword Search  │
│   (embeddings)   │    (trie)      │
└─────────────────┴─────────────────┘
  ↓
  Result Fusion
  ↓
  Re-ranking
  ↓
  Diversification
  ↓
  Final Results
```

### Pattern Search

**Module**: `memory/pattern_search.rs`
**File**: `/workspaces/feat-phase3/memory-core/src/memory/pattern_search.rs`

**Features**:
- Semantic pattern search with embeddings
- Multi-signal ranking:
  - Semantic similarity
  - Context matching (domain, task_type, tags)
  - Pattern effectiveness
  - Recency (recently used patterns score higher)
  - Success rate
- Customizable search configuration
- Pattern recommendation engine

**API**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/memory/mod.rs:496-512
pub async fn search_patterns_semantic(
    &self,
    query: &str,
    context: TaskContext,
    limit: usize,
) -> Result<Vec<pattern_search::PatternSearchResult>>

pub async fn recommend_patterns_for_task(
    &self,
    task_description: &str,
    context: TaskContext,
    limit: usize,
) -> Result<Vec<pattern_search::PatternSearchResult>>

pub async fn discover_analogous_patterns(
    &self,
    source_domain: &str,
    target_context: TaskContext,
    limit: usize,
) -> Result<Vec<pattern_search::PatternSearchResult>>
```

**Score Breakdown**:
```rust
pub struct ScoreBreakdown {
    pub semantic_similarity: f32,
    pub context_match: f32,
    pub effectiveness: f32,
    pub recency: f32,
    pub success_rate: f32,
}
```

### Adaptive Reward Calculation

**Module**: `reward/adaptive.rs`
**File**: `/workspaces/feat-phase3/memory-core/src/reward/adaptive.rs`

**Features**:
- Dynamic reward calculation based on multiple factors
- Efficiency bonuses
- Complexity multipliers
- Quality factors
- Learning bonuses for novel patterns
- Domain-specific adaptation

**Reward Components**:
```rust
pub struct RewardScore {
    pub total: f32,                // Combined score (0.0 to 2.0+)
    pub base: f32,                // Outcome-based (1.0, 0.5, 0.0)
    pub efficiency: f32,           // Execution speed (0.5 to 1.5)
    pub complexity_bonus: f32,     // Difficulty handling (1.0 to 1.3)
    pub quality_multiplier: f32,    // Output quality (0.8 to 1.2)
    pub learning_bonus: f32,        // Pattern discovery (0.0 to 0.5)
}
```

### Agent Monitoring

**Module**: `monitoring/core.rs`
**File**: `/workspaces/feat-phase3/memory-core/src/monitoring/core.rs`

**Features**:
- Track agent execution metrics
- Success rate tracking
- Performance monitoring
- Error aggregation
- Agent comparison

**API**:
```rust
// File: /workspaces/feat-phase3/memory-core/src/memory/mod.rs:326-364
pub async fn record_agent_execution(
    &self,
    agent_name: &str,
    success: bool,
    duration: std::time::Duration,
) -> Result<()>

pub async fn get_agent_metrics(&self, agent_name: &str) -> Option<AgentMetrics>

pub async fn get_all_agent_metrics(&self) -> HashMap<String, AgentMetrics>

pub async fn get_monitoring_summary(&self) -> crate::monitoring::MonitoringSummary
```

**Metrics**:
```rust
pub struct AgentMetrics {
    pub agent_name: String,
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub success_rate: f32,
    pub avg_duration_ms: f64,
    pub total_duration_ms: f64,
    pub last_execution: Option<DateTime<Utc>>,
    pub error_summary: HashMap<String, usize>,
}
```

---

## 5. MCP Tools Update

### All MCP Tools (11+ tools)

**File**: `/workspaces/feat-phase3/memory-mcp/src/bin/server/tools.rs`

#### 1. query_memory
**Handler**: `handle_query_memory()`
**Parameters**:
- `query`: String - Search query
- `domain`: String - Domain filter
- `task_type`: String? - Task type filter
- `limit`: Number - Max results (default: 10)

**Returns**: Episodes matching query

#### 2. execute_agent_code
**Handler**: `handle_execute_code()`
**Parameters**:
- `code`: String - WASM code to execute
- `context`: Object with `task` and `input`

**Returns**: Execution result or error

#### 3. analyze_patterns
**Handler**: `handle_analyze_patterns()`
**Parameters**:
- `task_type`: String - Filter by task type
- `min_success_rate`: Float - Minimum success rate (default: 0.7)
- `limit`: Number - Max patterns (default: 20)

**Returns**: Pattern analysis results

#### 4. advanced_pattern_analysis
**Handler**: `handle_advanced_pattern_analysis()`
**Parameters**:
- `analysis_type`: String - "statistical", "predictive", or "comprehensive"
- `time_series_data`: Object - Time series data by metric
- `config`: Object? - Optional analysis configuration

**Returns**: Advanced analysis results

#### 5. health_check
**Handler**: `handle_health_check()`
**Parameters**: None

**Returns**: System health status

#### 6. get_metrics
**Handler**: `handle_get_metrics()`
**Parameters**:
- `metric_type`: String? - Optional metric type filter

**Returns**: System metrics

#### 7. quality_metrics
**Handler**: `handle_quality_metrics()`
**Parameters**:
- `episode_id`: String? - Optional episode ID
- `pattern_id`: String? - Optional pattern ID
- `time_range`: String? - Time range filter

**Returns**: Quality metrics

#### 8. configure_embeddings **[NEW]**
**Handler**: `handle_configure_embeddings()`
**Parameters**:
- `provider`: String - "openai", "cohere", "ollama", "local"
- `model`: String - Model name
- `api_key`: String? - API key for provider
- `dimensions`: Number? - Embedding dimensions

**Returns**: Configuration status

#### 9. query_semantic_memory **[NEW]**
**Handler**: `handle_query_semantic_memory()`
**Parameters**:
- `query`: String - Natural language query
- `context`: Object - Task context
- `limit`: Number - Max results
- `use_cache`: Boolean - Use query cache

**Returns**: Semantically similar episodes

#### 10. test_embeddings **[NEW]**
**Handler**: `handle_test_embeddings()`
**Parameters**: None

**Returns**: Embedding provider test results

#### 11. search_patterns **[NEW]**
**Handler**: `handle_search_patterns()`
**Parameters**:
- `query`: String - Search query
- `context`: Object - Task context
- `limit`: Number - Max results
- `min_effectiveness`: Float - Minimum effectiveness score
- `config`: Object? - Search configuration

**Returns**: Ranked patterns with scores

#### 12. recommend_patterns **[NEW]**
**Handler**: `handle_recommend_patterns()`
**Parameters**:
- `task_description`: String - Task being worked on
- `context`: Object - Task context
- `limit`: Number - Max recommendations
- `strict_matching`: Boolean - Use stricter filtering

**Returns**: Recommended patterns for task

---

## 6. CLI Commands Update

### CLI V2 Structure

**File**: `/workspaces/feat-phase3/memory-cli/src/main.rs`

### Top-Level Commands

```rust
pub enum Commands {
    /// Episode management commands
    #[command(alias = "ep")]
    Episode { command: EpisodeCommands },

    /// Pattern analysis commands
    #[command(alias = "pat")]
    Pattern { command: PatternCommands },

    /// Storage operations commands
    #[command(alias = "st")]
    Storage { command: StorageCommands },

    /// Configuration validation and management
    #[command(alias = "cfg")]
    Config { command: ConfigCommands },

    /// Health monitoring and diagnostics
    #[command(alias = "hp")]
    Health { command: HealthCommands },

    /// Backup and restore operations
    #[command(alias = "bak")]
    Backup { command: BackupCommands },

    /// Monitoring and metrics
    #[command(alias = "mon")]
    Monitor { command: MonitorCommands },

    /// Log analysis and search
    #[command(alias = "log")]
    Logs { command: LogsCommands },

    /// Evaluation and calibration commands
    #[command(alias = "ev")]
    Eval { command: EvalCommands },

    /// Embedding provider management and testing
    #[command(alias = "emb")]
    Embedding { command: EmbeddingCommands },

    /// Generate shell completion scripts
    #[command(alias = "comp")]
    Completion { shell: clap_complete::Shell },
}
```

### Episode V2 Subcommands

**Module**: `commands/episode_v2/`
**File**: `/workspaces/feat-phase3/memory-cli/src/commands/episode_v2/mod.rs`

```rust
pub enum EpisodeCommands {
    /// Create a new episode
    Create {
        description: String,
        #[arg(short, long)]
        task_type: Option<String>,
        #[arg(short, long)]
        domain: Option<String>,
        #[arg(short, long)]
        tags: Option<Vec<String>>,
    },

    /// List episodes
    List {
        #[arg(short, long)]
        limit: Option<usize>,
        #[arg(short, long)]
        offset: Option<usize>,
        #[arg(short, long)]
        completed_only: Option<bool>,
        #[arg(short, long)]
        domain: Option<String>,
    },

    /// View episode details
    View {
        episode_id: String,
        #[arg(short, long)]
        output: Option<OutputFormat>,
    },

    /// Search episodes
    Search {
        query: String,
        #[arg(short, long)]
        domain: Option<String>,
        #[arg(short, long)]
        limit: Option<usize>,
        #[arg(short, long)]
        semantic: Option<bool>,
    },

    /// Complete an episode
    Complete {
        episode_id: String,
        outcome: TaskOutcome,
    },

    /// Delete an episode
    Delete {
        episode_id: String,
        #[arg(short, long)]
        force: Option<bool>,
    },

    /// Log execution step
    LogStep {
        episode_id: String,
        step_number: usize,
        tool: String,
        action: String,
        #[arg(short, long)]
        parameters: Option<String>,
        #[arg(short, long)]
        latency_ms: Option<u64>,
    },
}
```

### New Commands

#### 1. embedding
**Module**: `commands/embedding.rs`
**File**: `/workspaces/feat-phase3/memory-cli/src/commands/embedding.rs`

**Subcommands**:
```bash
memory-cli embedding configure    # Configure embedding provider
memory-cli embedding test        # Test embedding provider
memory-cli embedding status      # Show embedding status
memory-cli embedding list        # List available models
```

#### 2. eval
**Module**: `commands/eval.rs`
**File**: `/workspaces/feat-phase3/memory-cli/src/commands/eval.rs`

**Subcommands**:
```bash
memory-cli eval quality         # Evaluate episode quality
memory-cli eval patterns        # Evaluate pattern effectiveness
memory-cli eval metrics        # Show evaluation metrics
memory-cli eval benchmark      # Run benchmarks
```

### Episode V2 Usage Examples

```bash
# Create episode
memory-cli ep create "Implement auth" --task-type code_generation --domain web-api --tags rust async

# Log step
memory-cli log-step <episode-id> 1 "read_file" "Read config" --latency-ms 15

# Complete episode
memory-cli ep complete <episode-id> --success "Auth implemented" --artifacts auth.rs auth_test.rs

# Search episodes
memory-cli ep search "authentication" --domain web-api --semantic --limit 10

# List episodes
memory-cli ep list --limit 20 --completed-only true --domain web-api

# View episode
memory-cli ep view <episode-id> --output json

# Delete episode
memory-cli ep delete <episode-id> --force
```

---

## 7. File References

### Core Memory Structure

**Main Module**: `/workspaces/feat-phase3/memory-core/src/memory/mod.rs`
- Constructor methods: Lines 285-302
- Query caching: Line 273
- Spatiotemporal indexing: Lines 253-258
- Capacity management: Line 245
- PREMem integration: Lines 212-213
- Pattern search: Lines 496-634
- Agent monitoring: Lines 326-364

**Type Definitions**: `/workspaces/feat-phase3/memory-core/src/types/`
- Structs: `structs.rs` (TaskContext, RewardScore, Reflection, etc.)
- Enums: `enums.rs` (TaskOutcome, ExecutionResult, TaskType)
- Config: `config.rs` (MemoryConfig, StorageConfig, ConcurrencyConfig)

**Episode Module**: `/workspaces/feat-phase3/memory-core/src/episode.rs`
- Episode struct: Lines 262-295
- ExecutionStep: Lines 91-110
- PatternApplication: Lines 11-33

### New Modules

**Query Caching**:
- `/workspaces/feat-phase3/memory-core/src/retrieval/mod.rs`
- `/workspaces/feat-phase3/memory-core/src/retrieval/cache/lru.rs`
- `/workspaces/feat-phase3/memory-core/src/retrieval/cache/types.rs`

**Spatiotemporal Indexing**:
- `/workspaces/feat-phase3/memory-core/src/spatiotemporal/mod.rs`
- `/workspaces/feat-phase3/memory-core/src/spatiotemporal/types.rs`
- `/workspaces/feat-phase3/memory-core/src/spatiotemporal/index.rs`
- `/workspaces/feat-phase3/memory-core/src/spatiotemporal/retriever/`

**Semantic Summarization**:
- `/workspaces/feat-phase3/memory-core/src/semantic/mod.rs`
- `/workspaces/feat-phase3/memory-core/src/semantic/summary/`

**Capacity Management**:
- `/workspaces/feat-phase3/memory-core/src/episodic/mod.rs`
- `/workspaces/feat-phase3/memory-core/src/episodic/capacity.rs`

**PREMem**:
- `/workspaces/feat-phase3/memory-core/src/pre_storage/mod.rs`
- `/workspaces/feat-phase3/memory-core/src/pre_storage/quality.rs`
- `/workspaces/feat-phase3/memory-core/src/pre_storage/extractor/`

**Hybrid Search**:
- `/workspaces/feat-phase3/memory-core/src/search/mod.rs`
- `/workspaces/feat-phase3/memory-core/src/search/hybrid.rs`

### Pattern System

**Pattern Types**: `/workspaces/feat-phase3/memory-core/src/pattern/types.rs`
- PatternEffectiveness: Lines 12-102
- Pattern enum: Lines 107-149

**Pattern Search**: `/workspaces/feat-phase3/memory-core/src/memory/pattern_search.rs`
- Search results, scoring, ranking

**Pattern Extractors**: `/workspaces/feat-phase3/memory-core/src/patterns/extractors/`
- Tool sequence: `tool_sequence.rs`
- Decision point: `decision_point.rs`
- Error recovery: `error_recovery.rs`
- Context pattern: `context_pattern.rs`

### MCP Server

**Tools Handler**: `/workspaces/feat-phase3/memory-mcp/src/bin/server/tools.rs`
- All 11+ tool handlers

**Core MCP**: `/workspaces/feat-phase3/memory-mcp/src/bin/server/core.rs`
- Protocol handlers

**Tool Types**:
- `/workspaces/feat-phase3/memory-mcp/src/mcp/tools/embeddings/`
- `/workspaces/feat-phase3/memory-mcp/src/mcp/tools/pattern_search.rs`
- `/workspaces/feat-phase3/memory-mcp/src/mcp/tools/quality_metrics/`
- `/workspaces/feat-phase3/memory-mcp/src/mcp/tools/advanced_pattern_analysis/`

### CLI

**Main Entry**: `/workspaces/feat-phase3/memory-cli/src/main.rs`
- Command definitions: Lines 52-121

**Episode V2**: `/workspaces/feat-phase3/memory-cli/src/commands/episode_v2/`
- Episode subcommands
- Log step: `episode/log_step.rs`

**New Commands**:
- Embedding: `/workspaces/feat-phase3/memory-cli/src/commands/embedding.rs`
- Eval: `/workspaces/feat-phase3/memory-cli/src/commands/eval.rs`

---

## 8. Quick Reference

### Key API Differences Summary

| Aspect | Documented | Actual | Reference |
|--------|-------------|---------|-----------|
| Constructor | `async fn new(storage)` | `fn new()` → `fn with_storage()` | memory/mod.rs:285 |
| start_episode return | `Result<Uuid>` | `Uuid` | memory/episode.rs |
| complete_episode param | `reward: RewardScore` | (none - calculated) | memory/learning.rs |
| TaskContext.language | `String` | `Option<String>` | types/structs.rs:47 |
| TaskContext.framework | `String` | `Option<String>` | types/structs.rs:49 |
| TaskContext.tags | `HashMap` | `Vec<String>` | types/structs.rs:55 |
| ExecutionStep.step_id | `Uuid` | `step_number: usize` | episode.rs:93 |
| ExecutionStep.tool_name | `String` | `tool: String` | episode.rs:97 |
| ExecutionStep.input | `Value` | `parameters: Value` | episode.rs:101 |
| ExecutionStep.output | `String` | `result: Option<...>` | episode.rs:103 |
| Pattern.effectiveness | `f32` | `PatternEffectiveness` | pattern/types.rs:12 |
| Pattern.usage_count | `usize` | `occurrence_count: usize` | pattern/types.rs:115 |
| memory-core LOC | ~15,000 | **~35,466** | 134 files |
| MCP Tools | 8 | **11+** | bin/server/tools.rs |
| CLI Episodes | Old structure | **V2 structure** | commands/episode_v2/ |

### Module Count Summary

| Module | Files | Primary Purpose |
|--------|-------|----------------|
| memory | 11+ | Core orchestration |
| embeddings | 13 | Embedding generation |
| extraction | 6 | Pattern extraction |
| learning | 2 | Learning queue |
| patterns | 16+ | Pattern management |
| reflection | 7 | Reflection generation |
| reward | 5 | Reward calculation |
| monitoring | 4 | Agent monitoring |
| **retrieval** | 4 | **Query caching (NEW)** |
| **spatiotemporal** | 9+ | **Hierarchical indexing (NEW)** |
| **semantic** | 2+ | **Summarization (NEW)** |
| **episodic** | 2 | **Capacity management (NEW)** |
| **pre_storage** | 8 | **PREMem quality (NEW)** |
| **search** | 2 | **Hybrid search (NEW)** |
| types | 5 | Type definitions |
| storage | 3 | Storage backends |

---

## 9. Migration Guide

### Upgrading from Old API

#### Old Usage (INCORRECT):
```rust
// ❌ Doesn't work - new() is not async
let memory = SelfLearningMemory::new(storage).await?;

// ❌ Doesn't work - returns Uuid, not Result<Uuid>
let episode_id = memory.start_episode(...).await?;

// ❌ Doesn't work - no reward parameter
memory.complete_episode(episode_id, outcome, reward).await?;

// ❌ Wrong field names
let step = ExecutionStep {
    step_id: Uuid::new_v4(),
    tool_name: "reader".to_string(),
    input: json!(...),
    output: Some("result".to_string()),
};
```

#### New Usage (CORRECT):
```rust
// ✅ Use new() for in-memory, with_storage() for backends
let memory = SelfLearningMemory::new();

// ✅ Returns Uuid directly
let episode_id = memory.start_episode(
    "Implement feature".to_string(),
    TaskContext::default(),
    TaskType::CodeGeneration,
).await;

// ✅ Reward calculated internally
memory.complete_episode(
    episode_id,
    TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec!["feature.rs".to_string()],
    },
).await?;

// ✅ Correct field names
let step = ExecutionStep::new(
    1,                                    // step_number
    "reader".to_string(),                  // tool
    "Read file".to_string(),                // action
);
step.parameters = json!(...);               // parameters
step.result = Some(ExecutionResult::Success { output: "result".to_string() });  // result
```

---

## 10. Conclusion

This document provides a comprehensive bridge between outdated documentation and the current codebase reality at v0.1.12. Key takeaways:

1. **LOC Discrepancy**: Actual codebase is ~35,466 LOC (2.3x larger than documented)
2. **6 New Modules**: retrieval, spatiotemporal, semantic, episodic, pre_storage, search
3. **API Changes**: Constructor not async, start_episode returns Uuid not Result, reward removed from complete_episode
4. **Data Structure Changes**: Multiple field renames and type changes
5. **MCP Expansion**: 11+ tools vs 8 documented
6. **CLI V2**: Complete restructure with episode_v2 and new commands (embedding, eval)

**Recommendations**:
- Update all documentation to reflect actual signatures
- Add migration guide for breaking changes
- Document all 6 new modules
- Update CLI examples to use V2 structure
- Add pattern search API documentation

---

**Document Version**: 1.0
**Last Updated**: 2026-01-13
**Maintained By**: Architecture Team
