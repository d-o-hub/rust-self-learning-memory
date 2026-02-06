# Service Communication Patterns

## Overview

The memory system uses several communication patterns between components:

1. **MCP (Model Context Protocol)**: Primary client-server communication (verify version at https://modelcontextprotocol.io/docs/tools/inspector)
2. **Internal Rust Communication**: Inter-component messaging with Tokio
3. **Database Communication**: Storage layer interactions (Turso + redb)
4. **Cache Communication**: Performance optimization layer (Postcard-based, v0.1.13)

> **Note**: MCP specification versions may have changed. Verify the current version at https://modelcontextprotocol.io/docs/tools/inspector

## MCP Protocol Communication

### Client-Server Architecture
```
Client (Claude Code/OpenCode) ↔ MCP Server ↔ Memory Core ↔ Storage (Turso + redb)
```

### MCP Specification Versions (2025)

| Version | Status | Key Changes |
|---------|--------|-------------|
| **2025-11-25** | Latest (Current) | Current specification with all features |
| 2025-06-18 | Previous | Stability improvements |
| 2025-03-26 | Previous | Enhanced security features |
| 2024-11-05 | Legacy | Original specification |

### Tool Definitions (MCP v2025)
```rust
// MCP tool registration with v2025-11 protocol
#[mcp_server::tool]
async fn query_memory(
    query: String,
    domain: Option<String>,
    limit: Option<u32>,
    // New in 2025: Enhanced parameter descriptions
    #[tool(description = "Search query describing the context to retrieve")]
    query_description: String,
) -> Result<String, mcp_server::Error> {
    // Implementation
}
```

### Sampling Request (MCP 2025 Feature)
```rust
// New in MCP 2025: Server-initiated sampling for LLM calls
#[mcp_server::tool]
async fn analyze_patterns(
    #[tool(description = "Pattern analysis request")]
    request: String,
) -> Result<String, mcp_server::Error> {
    // Request sampling from client for complex analysis
    let sampling_request = SamplingMessage {
        request: CreateMessageRequest {
            messages: vec![
                Message {
                    role: Role::User,
                    content: Content::Text { text: request },
                }
            ],
            max_tokens: 1024,
            system_prompt: Some("You are a pattern analysis expert...".into()),
        },
    };

    let result = server.sample(&sampling_request).await?;
    Ok(result)
}
```

### Elicitation (MCP 2025 New Feature)
```rust
// New in MCP 2025: Elicitation for requesting user input
#[mcp_server::tool]
async fn confirm_action(
    #[tool(description = "Action requiring user confirmation")]
    action: String,
    #[tool(description = "Risk level of the action")]
    risk_level: String,
) -> Result<String, mcp_server::Error> {
    let elicitation = ElicitationRequest {
        message: format!("Confirm {} action (risk: {})", action, risk_level),
        kind: "confirmation".into(),
        accept_labels: vec!["Confirm".into(), "Cancel".into()],
    };

    let response = server.elicit(&elicitation).await?;
    Ok(response)
}
```

### Request-Response Pattern (v2025)
```rust
// Client request (JSON-RPC 2.0)
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "query_memory",
        "arguments": {
            "query": "previous debugging sessions",
            "domain": "web-api",
            "limit": 10
        }
    },
    "id": 1
}

// Server response with enhanced content types
{
    "jsonrpc": "2.0",
    "result": {
        "content": [
            {
                "type": "text",
                "text": "Found 3 relevant episodes..."
            },
            {
                "type": "resource",  // New in 2025
                "resource": {
                    "uri": "memory://episode/123",
                    "mimeType": "application/json"
                }
            }
        ]
    },
    "id": 1
}
```

### Capability Negotiation (MCP 2025)
```rust
// Server capability declaration (MCP 2025)
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub prompts: Option<PromptsCapability>,
    // New in 2025:
    pub sampling: Option<SamplingCapability>,
    pub elicitation: Option<ElicitationCapability>,
}

impl ServerCapabilities {
    pub fn default_memory_server() -> Self {
        ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: true,
            }),
            resources: Some(ResourcesCapability {
                subscribe: true,
                list_changed: false,
            }),
            prompts: None,
            sampling: Some(SamplingCapability),      // New 2025
            elicitation: Some(ElicitationCapability), // New 2025
        }
    }
}
```

### MCP Inspector (2025 Best Practice)

> **Verification**: Use the MCP Inspector to validate implementation against the latest specification: https://modelcontextprotocol.io/docs/tools/inspector

```bash
# Test MCP server with Inspector
npx @modelcontextprotocol/inspector \
  node path/to/server/index.js

# Test with custom transport
npx @modelcontextprotocol/inspector \
  --transport stdio \
  ./target/release/memory-mcp-server

# Verify capability negotiation
# Check tools, resources, and prompts tabs
# Test edge cases with custom inputs
```

### Progressive Tool Disclosure
```rust
// Tools shown based on usage patterns
pub fn get_available_tools(user_level: &UserLevel) -> Vec<Tool> {
    match user_level {
        UserLevel::Beginner => vec![
            query_memory_tool(),
            create_episode_tool(),
        ],
        UserLevel::Intermediate => vec![
            query_memory_tool(),
            create_episode_tool(),
            analyze_patterns_tool(),
            health_check_tool(),
        ],
        UserLevel::Advanced => vec![
            query_memory_tool(),
            create_episode_tool(),
            execute_code_tool(),
            health_check_tool(),
            advanced_pattern_analysis_tool(),  // New 2025
        ],
        UserLevel::Expert => all_tools(),
    }
}
```

## Internal Component Communication

### Memory Core Interface (2025 Patterns)
```rust
// Async communication with memory core using 2025 patterns
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Store a new episode with automatic embedding generation
    async fn store_episode(&self, episode: Episode) -> Result<EpisodeId>;

    /// Retrieve episodes with optional semantic search
    async fn retrieve_episodes(
        &self,
        query: &str,
        options: RetrieveOptions,
    ) -> Result<Vec<Episode>>;

    /// Extract patterns from completed episodes
    async fn extract_patterns(&self, episodes: &[Episode]) -> Result<Vec<Pattern>>;

    /// Get embeddings for an episode
    async fn get_embeddings(&self, episode_id: &str) -> Result<Option<Embedding>>;

    // New in 2025: Streaming support
    async fn stream_episodes(
        &self,
        filter: EpisodeFilter,
    ) -> Result<impl Stream<Item = Result<Episode>>>;
}

/// 2025: Unified query options
pub struct RetrieveOptions {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub semantic_threshold: Option<f32>,
    pub include_embeddings: bool,
    pub domain_filter: Option<String>,
    pub task_type_filter: Option<String>,
}

impl Default for RetrieveOptions {
    fn default() -> Self {
        RetrieveOptions {
            limit: Some(10),
            offset: None,
            semantic_threshold: None,
            include_embeddings: false,
            domain_filter: None,
            task_type_filter: None,
        }
    }
}
```

### Trait-Based Extensibility (2025)
```rust
// Pattern extractor trait for extensibility (2025 improvements)
#[async_trait]
pub trait PatternExtractor: Send + Sync {
    async fn extract_patterns(&self, episode: &Episode) -> Result<Vec<Pattern>>;
    fn extractor_type(&self) -> ExtractorType;

    // New in 2025: Priority and filtering
    fn priority(&self) -> u8;
    fn accepts(&self, episode: &Episode) -> bool;
}

// Multiple extractors can be registered with priority
impl PatternExtractor for ToolSequenceExtractor {
    async fn extract_patterns(&self, episode: &Episode) -> Result<Vec<Pattern>> {
        // Extract tool sequence patterns
        Ok(patterns)
    }

    fn extractor_type(&self) -> ExtractorType {
        ExtractorType::ToolSequence
    }

    fn priority(&self) -> u8 {
        10 // Higher priority
    }

    fn accepts(&self, episode: &Episode) -> bool {
        episode.has_tool_calls()
    }
}
```

### Storage Communication

#### Turso Storage (2025 Optimized)
```rust
// Turso storage with connection pooling and 2025 patterns
pub struct TursoStorage {
    database_url: String,
    pool: Arc<Semaphore>,  // Semaphore-based connection limiting
    metrics: Arc<StorageMetrics>, // 2025: Metrics for observability
}

#[async_trait]
impl MemoryStore for TursoStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<EpisodeId> {
        let _permit = self.pool.acquire().await?;
        let timer = self.metrics.start_operation("store_episode");

        let conn = libsql::connect(&self.database_url).await?;

        // Parameterized queries (SQL injection prevention)
        let result = conn.execute(
            "INSERT INTO episodes (id, task_type, domain) VALUES (?1, ?2, ?3)",
            [&episode.id, &episode.task_type, &episode.domain]
        ).await?;

        timer.record(); // 2025: Record operation time
        Ok(episode.id.clone())
    }

    // 2025: Batch operations with transaction
    async fn store_episodes_batch(
        &self,
        episodes: &[Episode],
    ) -> Result<Vec<EpisodeId>> {
        let _permit = self.pool.acquire().await?;
        let conn = libsql::connect(&self.database_url).await?;

        // Use transaction for atomic batch
        let txn = conn.transaction().await?;

        let mut ids = Vec::new();
        for episode in episodes {
            let id = self.store_episode_in_txn(&txn, episode).await?;
            ids.push(id);
        }

        txn.commit().await?;
        Ok(ids)
    }
}
```

#### Redb Cache Communication (2025)
```rust
// Redb cache layer with Postcard serialization and metrics
pub struct CacheLayer {
    cache: Arc<Database>,
    primary_store: Arc<dyn MemoryStore>,
    metrics: Arc<CacheMetrics>, // 2025: Cache hit/miss tracking
}

#[async_trait]
impl MemoryStore for CacheLayer {
    async fn retrieve_episodes(
        &self,
        query: &str,
        options: RetrieveOptions,
    ) -> Result<Vec<Episode>> {
        // Check cache first with Postcard deserialization
        let cache_key = self.build_cache_key(query, &options);
        if let Some(cached) = self.get_cached(&cache_key).await? {
            self.metrics.record_hit(); // 2025: Track cache hits
            let episodes: Vec<Episode> = postcard::from_bytes(&cached)?;
            return Ok(episodes);
        }

        self.metrics.record_miss(); // 2025: Track cache misses

        // Fallback to primary storage
        let episodes = self.primary_store.retrieve_episodes(query, options).await?;

        // Update cache with Postcard serialization
        let serialized = postcard::to_allocvec(&episodes)?;
        self.set_cached(&cache_key, &serialized).await?;

        Ok(episodes)
    }
}

/// 2025: Cache key builder with version support
pub struct CacheKeyBuilder {
    version: u8,
    prefix: &'static str,
}

impl CacheKeyBuilder {
    pub fn new(prefix: &'static str) -> Self {
        Self { prefix, version: 1 }
    }

    pub fn with_version(mut self, version: u8) -> Self {
        self.version = version;
        self
    }

    pub fn build(&self, query: &str, options: &RetrieveOptions) -> String {
        format!(
            "{}:v{}:{}:{:?}",
            self.prefix, self.version, query,
            (options.limit, options.domain_filter)
        )
    }
}
```

### Postcard Serialization Pattern (2025)
```rust
// Consistent Postcard serialization across cache layer
use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};

/// 2025: Versioned serialization for schema evolution
pub trait VersionedSerializable: Serialize + for<'de> Deserialize<'de> {
    fn schema_version() -> u8;
}

#[derive(Serialize, Deserialize)]
pub struct SerializedEpisode {
    pub version: u8,
    pub data: Vec<u8>,
    pub checksum: u32,
}

impl SerializedEpisode {
    pub fn serialize<T: VersionedSerializable + Serialize>(
        value: &T,
    ) -> Result<Self> {
        let data = postcard::to_allocvec(value)?;
        let checksum = crc32fast::hash(&data);

        Ok(Self {
            version: T::schema_version(),
            data,
            checksum,
        })
    }

    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        // Verify checksum
        let computed_checksum = crc32fast::hash(&self.data);
        if computed_checksum != self.checksum {
            return Err(anyhow::anyhow!("Checksum mismatch"));
        }

        postcard::from_bytes(&self.data).map_err(Into::into)
    }
}
```

## Event-Driven Communication (2025)

### Episode Lifecycle Events
```rust
// Event publishing with 2025 patterns
#[derive(Debug, Clone, PartialEq)]
pub enum EpisodeEvent {
    Created { episode_id: EpisodeId, domain: String },
    StepAdded { episode_id: EpisodeId, step: Step },
    Completed { episode_id: EpisodeId, success: bool, reward_score: f64 },
    ReflectionGenerated { episode_id: EpisodeId, reflection: String },
    PatternExtracted { episode_id: EpisodeId, patterns: Vec<Pattern> },
    // New in 2025:
    EmbeddingGenerated { episode_id: EpisodeId, embedding_type: EmbeddingType },
    Cached { episode_id: EpisodeId, cache_key: String },
}

// 2025: Event metadata for tracing
pub struct EpisodeEventMetadata {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub trace_id: String,
    pub span_id: String,
    pub correlation_id: String,
}

impl EpisodeEvent {
    pub fn with_metadata(self, metadata: EpisodeEventMetadata) -> Self {
        // Attach metadata for observability
        self
    }
}

// Event handling with async traits
#[async_trait]
pub trait EpisodeEventHandler: Send + Sync {
    async fn handle_event(&self, event: EpisodeEvent) -> Result<()>;
    fn event_types(&self) -> Vec<EpisodeEventType>;
}
```

### Pattern Processing Pipeline (Queue-Based 2025)
```rust
// Async pattern extraction pipeline with queue and backpressure
pub struct PatternProcessor {
    queue: Arc<learning_queue::LearningQueue>,
    extractors: Vec<Box<dyn PatternExtractor>>,
    store: Arc<dyn MemoryStore>,
    // 2025: New fields
    metrics: Arc<ProcessorMetrics>,
    concurrency_limit: usize,
}

impl PatternProcessor {
    pub async fn start(&self) -> Result<()> {
        // 2025: Use semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));

        loop {
            // 2025: Semaphore-aware dequeue with timeout
            let permit = semaphore.try_acquire_timeout(Duration::from_secs(1))?;

            if let Some(episode) = self.queue.dequeue().await? {
                tokio::spawn(async move {
                    let _permit = permit;
                    // Extract patterns
                    let patterns = self.extract_patterns_from_episode(&episode).await?;

                    // Store high-success patterns
                    for pattern in patterns {
                        if pattern.success_rate >= 0.7 {
                            self.store.store_pattern(pattern).await?;
                        }
                    }
                    Ok::<_, anyhow::Error>(())
                });
            }
        }
    }

    // 2025: Graceful shutdown
    pub async fn shutdown(&self, timeout: Duration) -> Result<()> {
        tokio::time::timeout(timeout, self.queue.drain()).await?;
        self.metrics.report_shutdown();
        Ok(())
    }
}
```

## Error Handling Patterns (2025)

### Circuit Breaker Pattern (2025)
```rust
// Circuit breaker for external API calls with metrics
pub struct CircuitBreaker {
    state: Arc<AtomicU8>,  // 0=closed, 1=open, 2=half-open
    failures: Arc<AtomicUsize>,
    last_failure: Arc<AtomicU64>,
    threshold: usize,
    timeout: Duration,
    // 2025: Additional fields
    metrics: Arc<BreakerMetrics>,
    observers: Vec<Arc<dyn CircuitBreakerObserver>>,
}

impl CircuitBreaker {
    pub async fn call<T, F, E>(&self, operation: F) -> Result<T, Error<E>>
    where
        F: Future<Output = Result<T, E>>,
        Error<E>: From<E>,
    {
        if self.is_open() {
            self.metrics.record_rejected();
            return Err(Error::CircuitBreakerOpen);
        }

        match operation.await {
            Ok(result) => {
                self.on_success();
                self.metrics.record_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                self.metrics.record_failure();
                Err(e.into())
            }
        }
    }

    // 2025: Observer pattern for monitoring
    pub fn add_observer(&mut self, observer: Arc<dyn CircuitBreakerObserver>) {
        self.observers.push(observer);
    }
}

#[async_trait]
pub trait CircuitBreakerObserver: Send + Sync {
    async fn on_state_change(&self, old_state: State, new_state: State);
    async fn on_failure(&self, error: &Error);
}
```

### Storage Synchronization (2025)
```rust
// Dual storage synchronization pattern with consistency guarantees
pub struct DualStorage {
    turso: Arc<TursoStorage>,
    redb: Arc<RedbCache>,
    consistency_checker: Arc<ConsistencyChecker>, // 2025: New
    write_mode: WriteMode,
}

pub enum WriteMode {
    /// Write to cache first, then primary (faster reads)
    CacheFirst,
    /// Write to primary first, then cache (stronger consistency)
    PrimaryFirst,
    /// Write to both concurrently (balanced)
    Concurrent,
}

impl DualStorage {
    pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
        match self.write_mode {
            WriteMode::CacheFirst => {
                // Write to cache first for faster reads
                self.redb.cache_set(&episode.id, episode).await?;
                // Then write to primary
                self.turso.store_episode(episode.clone()).await?;
            }
            WriteMode::PrimaryFirst => {
                // Write to primary first for stronger consistency
                self.turso.store_episode(episode.clone()).await?;
                // Then update cache
                self.redb.cache_set(&episode.id, episode).await?;
            }
            WriteMode::Concurrent => {
                // 2025: Concurrent write with consistency check
                let (turso_result, redb_result) = tokio::join!(
                    self.turso.store_episode(episode.clone()),
                    self.redb.cache_set(&episode.id, episode),
                );
                turso_result?;
                redb_result?;
            }
        }

        // 2025: Verify consistency
        self.consistency_checker.verify(&episode.id).await?;

        Ok(())
    }

    pub async fn get_episode(&self, id: &str) -> Result<Option<Episode>> {
        // Check cache first (Postcard deserialization)
        if let Some(episode) = self.redb.cache_get(id).await? {
            return Ok(Some(episode));
        }

        // Fallback to Turso
        if let Some(episode) = self.turso.get_episode(id).await? {
            // Update cache asynchronously (non-blocking)
            let cache = self.redb.clone();
            let ep = episode.clone();
            tokio::spawn(async move {
                let _ = cache.cache_set(&ep.id, &ep).await;
            });

            return Ok(Some(episode));
        }

        Ok(None)
    }
}
```

## Performance Communication Patterns (2025)

### Connection Pooling (Semaphore-Based 2025)
```rust
// Turso connection pool with semaphore and metrics
pub struct TursoPool {
    database_url: String,
    semaphore: Arc<Semaphore>,  // Default: 10 permits
    metrics: Arc<PoolMetrics>,  // 2025: Pool metrics
    health_check_interval: Duration,
}

impl TursoPool {
    pub async fn get_connection(&self) -> Result<Connection> {
        // Acquire permit (blocks if pool exhausted)
        let _permit = self.semaphore.acquire().await?;
        let acquire_time = Instant::now();

        let conn = libsql::connect(&self.database_url).await?;

        // 2025: Record acquire time metrics
        self.metrics.record_acquire_time(acquire_time.elapsed());

        // 2025: Health check on new connection
        if !conn.is_healthy().await {
            return Err(anyhow::anyhow!("Connection health check failed"));
        }

        // Permit released when dropped
        Ok(ConnectionWrapper::new(conn, _permit))
    }

    // 2025: Pre-warm connections
    pub async fn prewarm(&self, count: usize) -> Result<()> {
        let mut handles = Vec::new();
        for _ in 0..count {
            handles.push(tokio::spawn(self.get_connection()));
        }

        for handle in handles {
            handle.await??;
        }
        Ok(())
    }
}
```

### Batch Operations (2025)
```rust
// Batch episode storage with transaction and retry
pub async fn store_episodes_batch(
    &self,
    episodes: &[Episode],
    options: BatchOptions,
) -> Result<Vec<EpisodeId>> {
    let conn = self.get_connection().await?;
    let mut txn = conn.transaction()?;

    let mut ids = Vec::new();
    for episode in episodes {
        let id = self.store_episode_in_txn(&mut txn, episode).await?;
        ids.push(id);

        // 2025: Batch size limit
        if ids.len() >= options.max_batch_size {
            txn.commit()?;
            txn = conn.transaction()?;
        }
    }

    txn.commit()?;
    Ok(ids)
}

pub struct BatchOptions {
    pub max_batch_size: usize,
    pub retry_count: u32,
    pub retry_delay: Duration,
}

impl Default for BatchOptions {
    fn default() -> Self {
        BatchOptions {
            max_batch_size: 100,
            retry_count: 3,
            retry_delay: Duration::from_millis(100),
        }
    }
}
```

### Async Streaming (2025)
```rust
// Streaming query results with backpressure
pub async fn stream_episodes(
    &self,
    filter: EpisodeFilter,
    buffer_size: usize,
) -> Result<impl Stream<Item = Result<Episode>>> {
    let conn = self.get_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT * FROM episodes WHERE domain = ?1 ORDER BY created_at DESC"
    ).await?;

    let mut rows = stmt.query([filter.domain.unwrap_or_default()]).await?;

    // 2025: Bounded channel for backpressure
    let (tx, rx) = tokio::sync::mpsc::channel(buffer_size);

    tokio::spawn(async move {
        while let Some(row) = rows.next().await? {
            let episode = Episode::from_row(&row);

            // 2025: Backpressure - block if channel is full
            if tx.send(Ok(episode)).await.is_err() {
                break; // Receiver dropped
            }
        }
        Ok::<_, anyhow::Error>(())
    });

    Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
}
```

### LRU Cache Pattern (2025)
```rust
// LRU cache with Postcard serialization and metrics
pub struct LruCache {
    cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    ttl: Duration,
    metrics: Arc<CacheMetrics>, // 2025: Metrics
    max_size: usize,
}

pub struct CacheEntry {
    pub data: Vec<u8>,
    pub timestamp: Instant,
    pub access_count: AtomicU64,
    pub last_access: AtomicU64,
}

impl LruCache {
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut cache = self.cache.lock().await;

        if let Some(entry) = cache.get(key) {
            // Check TTL
            if entry.timestamp.elapsed() < self.ttl {
                // Update access stats
                entry.access_count.fetch_add(1, Ordering::Relaxed);
                entry.last_access.store(
                    Instant::now().elapsed().as_secs(),
                    Ordering::Relaxed,
                );

                self.metrics.record_hit();
                let deserialized: T = postcard::from_bytes(&entry.data)?;
                return Ok(Some(deserialized));
            } else {
                // Evict expired entry
                cache.remove(key);
            }
        }

        self.metrics.record_miss();
        Ok(None)
    }

    pub async fn set<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let mut cache = self.cache.lock().await;
        let serialized = postcard::to_allocvec(value)?;

        // 2025: Check size limit and evict if needed
        if cache.len() >= self.max_size {
            self.evict_lru(&mut cache);
        }

        let entry = CacheEntry {
            data: serialized,
            timestamp: Instant::now(),
            access_count: AtomicU64::new(0),
            last_access: AtomicU64::new(0),
        };

        cache.put(key.to_string(), entry);
        self.metrics.record_set();
        Ok(())
    }

    fn evict_lru(&self, cache: &mut LruCache<String, CacheEntry>) {
        // Evict least recently used entry
        if let Some((key, _)) = cache.peek_lru() {
            cache.remove(key);
        }
    }
}
```

## Monitoring and Observability (2025)

### Health Check Communication (2025)
```rust
// Health check via MCP tool with detailed status
#[mcp_server::tool]
async fn health_check(
    #[tool(description = "Check system health with detailed output")]
    detailed: Option<bool>,
) -> Result<HealthStatus, mcp_server::Error> {
    // 2025: Parallel health checks
    let (db_check, cache_check, embedding_check) = tokio::join!(
        check_database(),
        check_cache(),
        check_embeddings(),
    );

    let checks = vec![db_check?, cache_check?, embedding_check?];

    let overall_healthy = checks.iter().all(|c| c.healthy);

    let metrics = if detailed.unwrap_or(false) {
        Some(get_detailed_metrics().await?)
    } else {
        None
    };

    Ok(HealthStatus {
        overall: overall_healthy,
        checks,
        metrics,
        // 2025: Additional fields
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: get_uptime().await?,
        trace_id: get_current_trace_id(),
    })
}

/// 2025: Health check result with recommendations
pub struct HealthCheckResult {
    pub healthy: bool,
    pub component: String,
    pub latency: Duration,
    pub error_message: Option<String>,
    pub recommendations: Vec<String>,
}
```

### Metrics Communication (2025)
```rust
// Metrics collection with structured logging and tracing
pub struct MetricsCollector {
    episodes_created: AtomicU64,
    query_latency: Histogram<Duration>,  // 2025: Histogram for percentiles
    cache_hit_rate: AtomicF64,           // 2025: Atomic float
    cache_miss_rate: AtomicF64,
    operation_counts: HashMap<String, AtomicU64>,
    #[cfg(feature = "tracing")]
    tracer: Option<Tracer>,
}

impl MetricsCollector {
    pub async fn record_query_latency(&self, latency: Duration) {
        self.query_latency.record(latency);
    }

    // 2025: Get all percentiles at once
    pub async fn get_latency_percentiles(&self) -> LatencyPercentiles {
        let histogram = self.query_latency.read().await;
        LatencyPercentiles {
            p50: histogram.percentile(50.0),
            p95: histogram.percentile(95.0),
            p99: histogram.percentile(99.0),
            max: histogram.max(),
            mean: histogram.mean(),
        }
    }

    // 2025: Cache hit rate with atomic operations
    pub async fn record_cache_hit(&self) {
        self.cache_hit_rate.fetch_add(1.0, Ordering::Relaxed);
    }

    pub async fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hit_rate.load(Ordering::Relaxed);
        let misses = self.cache_miss_rate.load(Ordering::Relaxed);
        if hits + misses == 0.0 {
            return 0.0;
        }
        hits / (hits + misses)
    }
}

pub struct LatencyPercentiles {
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
    pub mean: Duration,
}
```

### Logging Patterns (2025)
```rust
// Structured logging with tracing and spans
#[tracing::instrument(
    level = "info",
    fields(episode_id = %episode.id(), domain = %episode.domain())
)]
pub async fn store_episode(&self, episode: &Episode) -> Result<EpisodeId> {
    let episode_id = episode.id();

    tracing::info!(
        episode_id = %episode_id,
        domain = %episode.domain(),
        task_type = %episode.task_type(),
        steps_count = episode.steps().len(),
        "Storing new episode"
    );

    match self.storage.store_episode(episode).await {
        Ok(id) => {
            tracing::info!(episode_id = %id, "Episode stored successfully");
            Ok(id)
        }
        Err(e) => {
            tracing::error!(
                episode_id = %episode_id,
                error = %e,
                error_chain = ?e.chain(),
                "Failed to store episode"
            );
            Err(e)
        }
    }
}

// 2025: Span-based distributed tracing
pub async fn process_with_trace<F, T>(
    &self,
    operation_name: &str,
    f: F,
) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let span = tracing::info_span!(
        "operation",
        name = operation_name,
        trace_id = %uuid::Uuid::new_v4(),
    );

    async move {
        let _guard = span.enter();
        tracing::info!("Operation started");
        let result = f.await;
        tracing::info!("Operation completed");
        result
    }
    .instrument(span)
    .await
}
```

## Security Communication (2025)

### Authentication (2025)
```rust
// MCP authentication middleware with rate limiting
pub struct AuthMiddleware {
    api_key: String,
    rate_limiter: Arc<RateLimiter>,
    // 2025: Additional security
    token_rotator: TokenRotator,
    audit_logger: Arc<AuditLogger>,
}

impl AuthMiddleware {
    pub async fn authenticate(
        &self,
        request: &mcp_server::Request,
    ) -> Result<User, AuthError> {
        // 2025: Rate limiting check
        if !self.rate_limiter.allow(&request.client_id()).await {
            return Err(AuthError::RateLimited);
        }

        let api_key = request.headers.get("X-API-Key")
            .ok_or(AuthError::MissingKey)?;

        // 2025: Key validation with rotation support
        if !self.token_rotator.is_valid(api_key).await {
            return Err(AuthError::InvalidKey);
        }

        // 2025: Audit logging
        self.audit_logger.log_authentication(
            &request.client_id(),
            true,
            None,
        ).await;

        Ok(User::new(api_key.to_string()))
    }
}

// 2025: Token rotation for key management
pub struct TokenRotator {
    current_key: Arc<RwLock<String>>,
    previous_key: Arc<RwLock<Option<String>>>,
    rotation_interval: Duration,
}

impl TokenRotator {
    pub async fn is_valid(&self, key: &str) -> bool {
        let current = self.current_key.read().await;
        let previous = self.previous_key.read().await;

        key == *current || previous.as_ref().map(|k| k == key).unwrap_or(false)
    }

    pub async fn rotate(&self, new_key: String) {
        let mut current = self.current_key.write().await;
        let mut previous = self.previous_key.write().await;

        *previous = Some(current.clone());
        *current = new_key;
    }
}
```

### Authorization (2025)
```rust
// Domain-based access control with policy engine
pub async fn check_domain_access(
    user: &User,
    domain: &str,
    context: &AuthorizationContext,
) -> Result<(), AuthError> {
    // 2025: Context-aware authorization
    let allowed_domains = user.allowed_domains().await?;

    if !allowed_domains.contains(domain) {
        // 2025: Additional risk-based check
        let risk_score = self.calculate_risk_score(user, domain, context).await?;
        if risk_score > user.max_risk_threshold() {
            return Err(AuthError::RiskTooHigh(risk_score));
        }
    }

    Ok(())
}

// 2025: Policy-based authorization
pub struct PolicyEngine {
    policies: Vec<Arc<dyn AuthorizationPolicy>>,
    cache: Arc<PolicyCache>,
}

impl PolicyEngine {
    pub async fn evaluate(
        &self,
        user: &User,
        resource: &str,
        action: &str,
    ) -> Result<AuthorizationDecision> {
        // Check cache first
        if let Some(decision) = self.cache.get(user.id(), resource, action).await {
            return Ok(decision);
        }

        // Evaluate all policies
        let mut decisions = Vec::new();
        for policy in &self.policies {
            let decision = policy.evaluate(user, resource, action).await?;
            decisions.push(decision);
        }

        // Combine decisions (deny overrides)
        let final_decision = self.combine_decisions(&decisions);
        self.cache.set(user.id(), resource, action, &final_decision).await;

        Ok(final_decision)
    }
}
```

### Defense-in-Depth Security (6-Layer Sandbox 2025)
```rust
// WASM sandbox with 6-layer security and 2025 enhancements
pub struct WasmtimeSandbox {
    // Layer 1: Process isolation
    engine: Engine,
    engine_config: EngineConfig,

    // Layer 2: Network restrictions
    network_config: NetworkConfig,

    // Layer 3: Filesystem sandbox
    fs_config: FsConfig,

    // Layer 4: Resource limits
    resource_limits: ResourceLimiter,

    // Layer 5: Code analysis
    code_analyzer: CodeAnalyzer,
    security_scanner: SecurityScanner, // 2025: Enhanced scanning

    // Layer 6: Runtime monitoring
    monitor: RuntimeMonitor,
    anomaly_detector: AnomalyDetector, // 2025: ML-based detection
}

impl WasmtimeSandbox {
    pub async fn execute_code(&self, code: &str) -> Result<WasmOutput> {
        // Layer 5: Static analysis + security scan
        self.code_analyzer.analyze(code)?;
        self.security_scanner.scan(code)?; // 2025: Additional security scan

        // 2025: Pre-execution validation
        self.validate_pre_execution(code)?;

        // Layer 1-4: Configure sandbox
        let mut store = Store::new(&self.engine, Layer4Limiter::new(self.resource_limits.clone()));

        // Layer 2-3: Apply restrictions
        store.set_network_config(self.network_config.clone());
        store.set_fs_config(self.fs_config.clone());

        // Execute with Layer 6 monitoring
        let module = Module::new(&self.engine, code)?;
        let instance = Instance::new(&mut store, &module, &[])?;

        self.monitor.start_tracking();

        // 2025: Anomaly detection during execution
        self.anomaly_detector.start_monitoring(&mut store);

        let result = self.run_with_monitoring(&mut store, instance).await?;

        self.monitor.stop_tracking();

        // 2025: Post-execution analysis
        let anomalies = self.anomaly_detector.get_anomalies()?;
        if !anomalies.is_empty() {
            self.report_anomalies(&anomalies).await?;
        }

        self.monitor.check_violations()?;

        Ok(result)
    }
}

// 2025: New security layer - Zero Trust verification
pub struct ZeroTrustVerifier {
    identity_verifier: IdentityVerifier,
    integrity_checker: IntegrityChecker,
    policy_enforcer: PolicyEnforcer,
}

impl ZeroTrustVerifier {
    pub async fn verify_execution(&self, output: &WasmOutput) -> Result<bool> {
        let identity_ok = self.identity_verifier.verify(&output.provenance).await?;
        let integrity_ok = self.integrity_checker.verify(&output.checksum).await?;
        let policy_ok = self.policy_enforcer.enforce(&output.actions).await?;

        Ok(identity_ok && integrity_ok && policy_ok)
    }
}
```

## Configuration Communication (2025)

### Dynamic Configuration (2025)
```rust
// Hot-reload configuration with watchers
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    reload_tx: mpsc::UnboundedSender<ConfigChange>,
    watchers: Arc<RwLock<Vec<ConfigWatcher>>>,
}

pub struct ConfigChange {
    pub path: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ConfigManager {
    pub async fn reload_config(&self) -> Result<()> {
        let new_config = self.load_config_from_file().await?;

        // 2025: Calculate diff and notify watchers
        let old_config = self.config.read().await.clone();
        let changes = self.calculate_changes(&old_config, &new_config);

        let mut config = self.config.write().await;
        *config = new_config;

        // Notify all watchers
        for watcher in self.watchers.read().await.iter() {
            let _ = self.reload_tx.send(ConfigChange {
                path: watcher.path.clone(),
                old_value: watcher.current_value.clone(),
                new_value: watcher.new_value.clone(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(())
    }

    pub fn get_config(&self) -> AppConfig {
        let config = self.config.read().now_or_anyhow()?.clone();
        config
    }

    // 2025: Watch specific config paths
    pub async fn watch(&self, path: &str) -> impl Stream<Item = ConfigChange> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut watcher = ConfigWatcher {
            path: path.to_string(),
            tx,
        };
        self.watchers.write().await.push(watcher);
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
    }
}
```

### Progressive Tool Configuration (2025)
```rust
// Progressive disclosure based on usage with ML
pub struct ProgressiveToolManager {
    tools: HashMap<String, Tool>,
    usage_stats: Arc<RwLock<HashMap<String, UsageStats>>>,
    recommendation_model: Option<Arc<RecommendationModel>>, // 2025: ML-based
}

impl ProgressiveToolManager {
    pub fn get_available_tools(&self, user_id: &str) -> Vec<Tool> {
        let stats = self.usage_stats.read().now_or_anyhow()?;
        let user_level = self.calculate_user_level(user_id, &stats);

        // 2025: Use ML model for recommendations
        if let Some(model) = &self.recommendation_model {
            return model.recommend_tools(user_id, user_level, &stats);
        }

        match user_level {
            UserLevel::Beginner => self.get_beginner_tools(),
            UserLevel::Intermediate => self.get_intermediate_tools(),
            UserLevel::Advanced => self.get_all_tools(),
        }
    }

    // 2025: Anomaly detection in usage
    pub async fn detect_usage_anomaly(
        &self,
        user_id: &str,
        tool_name: &str,
    ) -> Result<bool> {
        let stats = self.usage_stats.read().await;
        let user_stats = stats.get(user_id).ok_or_else(|| anyhow!("User not found"))?;
        let tool_stats = user_stats.tool_usage.get(tool_name).ok_or_else(|| anyhow!("Tool not found"))?;

        // Detect unusual patterns
        let baseline = tool_stats.average_usage_rate();
        let current = tool_stats.recent_usage_rate(Duration::from_hours(1));

        let deviation = if baseline > 0 {
            (current - baseline).abs() / baseline
        } else {
            0.0
        };

        Ok(deviation > 0.5) // 50% deviation threshold
    }
}
```

## Async/Await Patterns (2025)

### Concurrent Operations (2025)
```rust
// Use tokio::join! for concurrent operations with cancellation
pub async fn fetch_episode_data(&self, episode_id: &str) -> Result<EpisodeData> {
    // 2025: TryJoin for early termination on error
    let (episode, patterns, embeddings) = tokio::try_join!(
        self.get_episode(episode_id),
        self.get_patterns(episode_id),
        self.get_embeddings(episode_id),
    )?;

    Ok(EpisodeData {
        episode,
        patterns,
        embeddings,
    })
}

// 2025: Select with timeout and cancellation
pub async fn fetch_with_timeout(
    &self,
    episode_id: &str,
    timeout: Duration,
) -> Result<EpisodeData> {
    let operation = self.fetch_episode_data(episode_id);

    tokio::select! {
        result = operation => result,
        _ = tokio::time::sleep(timeout) => {
            Err(anyhow::anyhow!("Operation timed out"))
        }
    }
}
```

### Error Aggregation (2025)
```rust
// Aggregate multiple errors with context
pub async fn validate_episode(&self, episode: &Episode) -> Result<ValidationReport> {
    let (steps_result, patterns_result, embeddings_result) = tokio::join!(
        self.validate_steps(episode),
        self.validate_patterns(episode),
        self.validate_embeddings(episode),
    );

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 2025: Enhanced error handling with context
    match steps_result {
        Ok(None) => {} // No issues
        Ok(Some(warning)) => warnings.push(warning),
        Err(e) => errors.push(ValidationError::StepError(e.to_string())),
    }

    match patterns_result {
        Ok(None) => {}
        Ok(Some(warning)) => warnings.push(warning),
        Err(e) => errors.push(ValidationError::PatternError(e.to_string())),
    }

    match embeddings_result {
        Ok(None) => {}
        Ok(Some(warning)) => warnings.push(warning),
        Err(e) => errors.push(ValidationError::EmbeddingError(e.to_string())),
    }

    // 2025: Distinguish between errors and warnings
    if !errors.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", errors));
    }

    Ok(ValidationReport {
        valid: true,
        warnings,
        suggestions: self.generate_suggestions(&warnings),
    })
}
```

### Stream Processing (2025)
```rust
// Process streams of episodes with controlled concurrency
pub async fn process_episode_stream(
    &self,
    episodes: impl Stream<Item = Episode>,
    concurrency: usize,
) -> Result<ProcessingStats> {
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut count = 0;
    let mut errors = 0;

    tokio::pin!(episodes);

    while let Some(episode) = episodes.next().await {
        let permit = semaphore.try_acquire().map_err(|_| anyhow!("Concurrency limit reached"))?;

        let self_clone = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            if let Err(e) = self_clone.process_episode(episode).await {
                tracing::error!(error = %e, "Failed to process episode");
                // Use atomic operations for thread-safe counting
                self_clone.increment_error_count();
            } else {
                self_clone.increment_success_count();
            }
        });
    }

    Ok(ProcessingStats { processed: count, errors })
}

// 2025: Graceful stream shutdown
pub async fn process_with_graceful_shutdown(
    &self,
    episodes: impl Stream<Item = Episode>,
    shutdown_signal: impl Future<Output = ()>,
) -> Result<ProcessingStats> {
    let episodes = tokio_stream::StreamExt::fuse(episodes);
    tokio::pin!(episodes, shutdown_signal);

    let mut stats = ProcessingStats::default();

    loop {
        tokio::select! {
            Some(episode) = episodes.next() => {
                if let Err(e) = self.process_episode(episode).await {
                    stats.errors += 1;
                    tracing::error!(error = %e, "Failed to process episode");
                } else {
                    stats.processed += 1;
                }
            }
            _ = &mut shutdown_signal => {
                tracing::info!("Shutdown signal received, finishing processing");
                break;
            }
            else => {
                // Stream ended
                break;
            }
        }
    }

    Ok(stats)
}
```

## Summary of Key Patterns (2025)

| Pattern | 2024 | 2025 Enhancement |
|---------|------|------------------|
| **MCP Protocol** | JSON-RPC with tools | v2025-11-25, Sampling, Elicitation, MCP Inspector |
| **Trait-Based Extensibility** | Basic traits | Priority, filtering, observer pattern |
| **Circuit Breaker** | Basic state machine | Metrics, observers, failure tracking |
| **Queue-Based Processing** | Simple loop | Semaphore concurrency, graceful shutdown |
| **Dual Storage** | Concurrent writes | Write modes, consistency verification |
| **LRU Cache** | TTL-based | Size limits, access stats, checksum validation |
| **Defense-in-Depth** | 6-layer WASM | Zero Trust verification, ML anomaly detection |
| **Connection Pooling** | Semaphore limiting | Health checks, pre-warming, metrics |
| **Structured Logging** | tracing | Spans, distributed tracing, span-based operations |
| **Progressive Disclosure** | User levels | ML-based recommendations, anomaly detection |

### New 2025 Patterns Added

1. **MCP Inspector Integration**: Testing and debugging workflows
2. **Versioned Serialization**: Schema evolution with checksums
3. **Backpressure Handling**: Bounded channels in streaming
4. **Rate Limiting**: Token bucket for authentication
5. **Policy Engine**: Cached authorization decisions
6. **Zero Trust Verification**: Identity, integrity, policy enforcement
7. **Config Watching**: Reactive configuration updates
8. **Latency Histograms**: Percentile tracking for metrics
9. **Cancellation-Safe Operations**: TryJoin, select with timeout
10. **Graceful Shutdown**: Stream fusion and signal handling