# Service Communication Patterns

## Overview

The memory system uses several communication patterns between components:

1. **MCP (Model Context Protocol)**: Primary client-server communication
2. **Internal Rust Communication**: Inter-component messaging
3. **Database Communication**: Storage layer interactions
4. **Cache Communication**: Performance optimization layer (Postcard-based)

## MCP Protocol Communication

### Client-Server Architecture
```
Client (Claude Code/OpenCode) ↔ MCP Server ↔ Memory Core ↔ Storage (Turso + redb)
```

### Tool Definitions
```rust
// MCP tool registration with v2024-11 protocol
#[mcp_server::tool]
async fn query_memory(
    query: String,
    domain: Option<String>,
    limit: Option<u32>,
) -> Result<String, mcp_server::Error> {
    // Implementation
}
```

### Request-Response Pattern
```rust
// Client request (JSON-RPC)
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

// Server response
{
    "jsonrpc": "2.0",
    "result": {
        "content": [
            {
                "type": "text",
                "text": "Found 3 relevant episodes..."
            }
        ]
    },
    "id": 1
}
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
        UserLevel::Advanced => vec![
            query_memory_tool(),
            create_episode_tool(),
            execute_code_tool(),
            health_check_tool(),
        ],
        UserLevel::Expert => all_tools(),
    }
}
```

## Internal Component Communication

### Memory Core Interface
```rust
// Async communication with memory core
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn store_episode(&self, episode: Episode) -> Result<EpisodeId>;
    async fn retrieve_episodes(&self, query: &str) -> Result<Vec<Episode>>;
    async fn extract_patterns(&self, episodes: &[Episode]) -> Result<Vec<Pattern>>;
    async fn get_embeddings(&self, episode_id: &str) -> Result<Option<Embedding>>;
}
```

### Trait-Based Extensibility
```rust
// Pattern extractor trait for extensibility
#[async_trait]
pub trait PatternExtractor: Send + Sync {
    async fn extract_patterns(&self, episode: &Episode) -> Result<Vec<Pattern>>;
    fn extractor_type(&self) -> ExtractorType;
}

// Multiple extractors can be registered
impl PatternExtractor for ToolSequenceExtractor {
    async fn extract_patterns(&self, episode: &Episode) -> Result<Vec<Pattern>> {
        // Extract tool sequence patterns
        Ok(patterns)
    }
}
```

### Storage Communication

#### Turso Storage
```rust
// Turso storage with connection pooling
pub struct TursoStorage {
    database_url: String,
    pool: Arc<Semaphore>,  // Semaphore-based connection limiting
}

#[async_trait]
impl MemoryStore for TursoStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<EpisodeId> {
        let _permit = self.pool.acquire().await?;
        let conn = libsql::connect(&self.database_url).await?;

        // Parameterized queries (SQL injection prevention)
        let result = conn.execute(
            "INSERT INTO episodes (id, task_type, domain) VALUES (?1, ?2, ?3)",
            [&episode.id, &episode.task_type, &episode.domain]
        ).await?;

        Ok(episode.id.clone())
    }
}
```

#### Redb Cache Communication
```rust
// Redb cache layer with Postcard serialization
pub struct CacheLayer {
    cache: Arc<Database>,
    primary_store: Arc<dyn MemoryStore>,
}

#[async_trait]
impl MemoryStore for CacheLayer {
    async fn retrieve_episodes(&self, query: &str) -> Result<Vec<Episode>> {
        // Check cache first with Postcard deserialization
        let cache_key = format!("query:{}", query);
        if let Some(cached) = self.get_cached(&cache_key).await? {
            let episodes: Vec<Episode> = postcard::from_bytes(&cached)?;
            return Ok(episodes);
        }

        // Fallback to primary storage
        let episodes = self.primary_store.retrieve_episodes(query).await?;

        // Update cache with Postcard serialization
        let serialized = postcard::to_allocvec(&episodes)?;
        self.set_cached(&cache_key, &serialized).await?;

        Ok(episodes)
    }
}
```

### Postcard Serialization Pattern
```rust
// Consistent Postcard serialization across cache layer
use postcard::{from_bytes, to_allocvec};

pub async fn cache_set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
    let write_txn = self.redb.begin_write()?;
    {
        let mut table = write_txn.open_table(CACHE_TABLE)?;
        let serialized = postcard::to_allocvec(value)?;
        table.insert(key.as_bytes(), serialized)?;
    }
    write_txn.commit()?;
    Ok(())
}

pub async fn cache_get<'de, T: Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
    let read_txn = self.redb.begin_read()?;
    let table = read_txn.open_table(CACHE_TABLE)?;
    if let Some(value) = table.get(key.as_bytes())? {
        let deserialized: T = postcard::from_bytes(&value.to_vec())?;
        Ok(Some(deserialized))
    } else {
        Ok(None)
    }
}
```

## Event-Driven Communication

### Episode Lifecycle Events
```rust
// Event publishing
pub enum EpisodeEvent {
    Created { episode_id: EpisodeId, domain: String },
    StepAdded { episode_id: EpisodeId, step: Step },
    Completed { episode_id: EpisodeId, success: bool, reward_score: f64 },
    ReflectionGenerated { episode_id: EpisodeId, reflection: String },
    PatternExtracted { episode_id: EpisodeId, patterns: Vec<Pattern> },
}

// Event handling with async traits
#[async_trait]
pub trait EpisodeEventHandler: Send + Sync {
    async fn handle_event(&self, event: EpisodeEvent) -> Result<()>;
}
```

### Pattern Processing Pipeline (Queue-Based)
```rust
// Async pattern extraction pipeline with queue
pub struct PatternProcessor {
    queue: Arc<learning_queue::LearningQueue>,
    extractors: Vec<Box<dyn PatternExtractor>>,
    store: Arc<dyn MemoryStore>,
}

impl PatternProcessor {
    pub async fn start(&self) -> Result<()> {
        loop {
            // Dequeue episodes for pattern extraction
            if let Some(episode) = self.queue.dequeue().await? {
                // Extract patterns
                let patterns = self.extract_patterns_from_episode(&episode).await?;

                // Store high-success patterns
                for pattern in patterns {
                    if pattern.success_rate >= 0.7 {
                        self.store.store_pattern(pattern).await?;
                    }
                }
            }
        }
    }
}
```

## Error Handling Patterns

### Circuit Breaker Pattern
```rust
// Circuit breaker for external API calls
pub struct CircuitBreaker {
    state: Arc<AtomicU8>,  // 0=closed, 1=open, 2=half-open
    failures: Arc<AtomicUsize>,
    last_failure: Arc<AtomicU64>,
    threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    pub async fn call<T, F, E>(&self, operation: F) -> Result<T, Error<E>>
    where
        F: Future<Output = Result<T, E>>,
        Error<E>: From<E>,
    {
        if self.is_open() {
            return Err(Error::CircuitBreakerOpen);
        }

        match operation.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e.into())
            }
        }
    }
}
```

### Storage Synchronization
```rust
// Dual storage synchronization pattern
pub struct DualStorage {
    turso: Arc<TursoStorage>,
    redb: Arc<RedbCache>,
}

impl DualStorage {
    pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
        // Concurrent write to both storages
        let (turso_result, redb_result) = tokio::join!(
            self.turso.store_episode(episode.clone()),
            self.redb.cache_set(&episode.id, episode),
        );

        turso_result?;
        redb_result?;

        Ok(())
    }

    pub async fn get_episode(&self, id: &str) -> Result<Option<Episode>> {
        // Check cache first (Postcard deserialization)
        if let Some(episode) = self.redb.cache_get(id).await? {
            return Ok(Some(episode));
        }

        // Fallback to Turso
        if let Some(episode) = self.turso.get_episode(id).await? {
            // Update cache
            let _ = self.redb.cache_set(id, &episode).await;
            return Ok(Some(episode));
        }

        Ok(None)
    }
}
```

## Performance Communication Patterns

### Connection Pooling (Semaphore-Based)
```rust
// Turso connection pool with semaphore
pub struct TursoPool {
    database_url: String,
    semaphore: Arc<Semaphore>,  // Default: 10 permits
}

impl TursoPool {
    pub async fn get_connection(&self) -> Result<Connection> {
        // Acquire permit (blocks if pool exhausted)
        let _permit = self.semaphore.acquire().await?;

        let conn = libsql::connect(&self.database_url).await?;

        // Permit released when dropped
        Ok(ConnectionWrapper::new(conn, _permit))
    }
}
```

### Batch Operations
```rust
// Batch episode storage with transaction
pub async fn store_episodes_batch(
    &self,
    episodes: &[Episode],
) -> Result<Vec<EpisodeId>> {
    let conn = self.get_connection().await?;
    let mut txn = conn.begin()?;

    let mut ids = Vec::new();
    for episode in episodes {
        let id = self.store_episode_in_txn(&mut txn, episode).await?;
        ids.push(id);
    }

    txn.commit()?;
    Ok(ids)
}
```

### Async Streaming
```rust
// Streaming query results
pub async fn stream_episodes(
    &self,
    domain: &str,
) -> Result<impl Stream<Item = Result<Episode>>> {
    let conn = self.get_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT * FROM episodes WHERE domain = ?1 ORDER BY created_at DESC"
    ).await?;

    let mut rows = stmt.query([domain]).await?;

    Ok(stream! {
        while let Some(row) = rows.next().await? {
            yield Ok(Episode::from_row(&row));
        }
    })
}
```

### LRU Cache Pattern
```rust
// LRU cache with Postcard serialization
pub struct LruCache {
    cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
    ttl: Duration,
}

impl LruCache {
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut cache = self.cache.lock().await;

        if let Some((value, timestamp)) = cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                let deserialized: T = postcard::from_bytes(value)?;
                return Ok(Some(deserialized));
            }
        }

        Ok(None)
    }

    pub async fn set<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let mut cache = self.cache.lock().await;
        let serialized = postcard::to_allocvec(value)?;
        cache.put(key.to_string(), (serialized, Instant::now()));
        Ok(())
    }
}
```

## Monitoring and Observability

### Health Check Communication
```rust
// Health check via MCP tool
#[mcp_server::tool]
async fn health_check(
    #[tool(description = "Check system health")] detailed: Option<bool>,
) -> Result<HealthStatus, mcp_server::Error> {
    let checks = vec![
        check_database().await?,
        check_cache().await?,
        check_embeddings().await?,
    ];

    let metrics = if detailed.unwrap_or(false) {
        Some(get_detailed_metrics().await?)
    } else {
        None
    };

    Ok(HealthStatus {
        overall: checks.iter().all(|c| c.healthy),
        checks,
        metrics,
    })
}
```

### Metrics Communication
```rust
// Metrics collection with structured logging
pub struct MetricsCollector {
    episodes_created: AtomicU64,
    query_latency: Arc<Mutex<VecDeque<Duration>>>,
    cache_hit_rate: Arc<AtomicUsize>,
    cache_miss_rate: Arc<AtomicUsize>,
}

impl MetricsCollector {
    pub async fn record_query_latency(&self, latency: Duration) {
        let mut latencies = self.query_latency.lock().await;
        latencies.push_back(latency);
        if latencies.len() > 1000 {
            latencies.pop_front();
        }
    }

    pub async fn get_p95_latency(&self) -> Duration {
        let latencies = self.query_latency.lock().await;
        let mut sorted: Vec<_> = latencies.iter().cloned().collect();
        sorted.sort();
        sorted[sorted.len() * 95 / 100]
    }
}
```

### Logging Patterns
```rust
// Structured logging with tracing
pub async fn store_episode(&self, episode: &Episode) -> Result<EpisodeId> {
    let episode_id = episode.id();

    tracing::info!(
        episode_id = %episode_id,
        domain = %episode.domain(),
        task_type = %episode.task_type(),
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
```

## Security Communication

### Authentication
```rust
// MCP authentication middleware
pub struct AuthMiddleware {
    api_key: String,
}

impl AuthMiddleware {
    pub async fn authenticate(
        &self,
        request: &mcp_server::Request,
    ) -> Result<User, AuthError> {
        let api_key = request.headers.get("X-API-Key")
            .ok_or(AuthError::MissingKey)?;

        if api_key != self.api_key {
            return Err(AuthError::InvalidKey);
        }

        Ok(User::new(api_key.to_string()))
    }
}
```

### Authorization
```rust
// Domain-based access control
pub async fn check_domain_access(
    user: &User,
    domain: &str,
) -> Result<(), AuthError> {
    let allowed_domains = user.allowed_domains().await?;

    if !allowed_domains.contains(domain) {
        return Err(AuthError::DomainNotAllowed(domain.to_string()));
    }

    Ok(())
}
```

### Defense-in-Depth Security (6-Layer Sandbox)
```rust
// WASM sandbox with 6-layer security
pub struct WasmtimeSandbox {
    // Layer 1: Process isolation
    engine: Engine,

    // Layer 2: Network restrictions
    network_config: NetworkConfig,

    // Layer 3: Filesystem sandbox
    fs_config: FsConfig,

    // Layer 4: Resource limits
    resource_limits: ResourceLimiter,

    // Layer 5: Code analysis
    code_analyzer: CodeAnalyzer,

    // Layer 6: Runtime monitoring
    monitor: RuntimeMonitor,
}

impl WasmtimeSandbox {
    pub async fn execute_code(&self, code: &str) -> Result<WasmOutput> {
        // Layer 5: Static analysis
        self.code_analyzer.analyze(code)?;

        // Layer 1-4: Configure sandbox
        let mut store = Store::new(&self.engine, Layer4Limiter::new(self.resource_limits.clone()));

        // Layer 2-3: Apply restrictions
        store.set_network_config(self.network_config.clone());
        store.set_fs_config(self.fs_config.clone());

        // Execute with Layer 6 monitoring
        let module = Module::new(&self.engine, code)?;
        let instance = Instance::new(&mut store, &module, &[])?;

        self.monitor.start_tracking();

        let result = self.run_with_monitoring(&mut store, instance).await?;

        self.monitor.stop_tracking();
        self.monitor.check_violations()?;

        Ok(result)
    }
}
```

## Configuration Communication

### Dynamic Configuration
```rust
// Hot-reload configuration
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    reload_tx: mpsc::UnboundedSender<()>,
}

impl ConfigManager {
    pub async fn reload_config(&self) -> Result<()> {
        let new_config = self.load_config_from_file().await?;

        let mut config = self.config.write().await;
        *config = new_config;

        // Notify components of config change
        let _ = self.reload_tx.send(());

        Ok(())
    }

    pub fn get_config(&self) -> AppConfig {
        let config = self.config.read().now_or_never()?;
        config.clone()
    }
}
```

### Progressive Tool Configuration
```rust
// Progressive disclosure based on usage
pub struct ProgressiveToolManager {
    tools: HashMap<String, Tool>,
    usage_stats: Arc<RwLock<HashMap<String, UsageStats>>>,
}

impl ProgressiveToolManager {
    pub fn get_available_tools(&self, user_id: &str) -> Vec<Tool> {
        let stats = self.usage_stats.read().now_or_never()?;
        let user_level = self.calculate_user_level(user_id, &stats);

        match user_level {
            UserLevel::Beginner => self.get_beginner_tools(),
            UserLevel::Intermediate => self.get_intermediate_tools(),
            UserLevel::Advanced => self.get_all_tools(),
        }
    }
}
```

## Async/Await Patterns

### Concurrent Operations
```rust
// Use tokio::join! for concurrent operations
pub async fn fetch_episode_data(&self, episode_id: &str) -> Result<EpisodeData> {
    let (episode, patterns, embeddings) = tokio::join!(
        self.get_episode(episode_id),
        self.get_patterns(episode_id),
        self.get_embeddings(episode_id),
    );

    Ok(EpisodeData {
        episode: episode?,
        patterns: patterns?,
        embeddings: embeddings?,
    })
}
```

### Error Aggregation
```rust
// Aggregate multiple errors
pub async fn validate_episode(&self, episode: &Episode) -> Result<ValidationReport> {
    let (steps_result, patterns_result, embeddings_result) = tokio::join!(
        self.validate_steps(episode),
        self.validate_patterns(episode),
        self.validate_embeddings(episode),
    );

    let mut errors = Vec::new();

    if let Err(e) = steps_result {
        errors.push(e);
    }
    if let Err(e) = patterns_result {
        errors.push(e);
    }
    if let Err(e) = embeddings_result {
        errors.push(e);
    }

    if !errors.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", errors));
    }

    Ok(ValidationReport::valid())
}
```

### Stream Processing
```rust
// Process streams of episodes
pub async fn process_episode_stream(
    &self,
    episodes: impl Stream<Item = Episode>,
) -> Result<ProcessingStats> {
    let mut count = 0;
    let mut errors = 0;

    tokio::pin!(episodes);

    while let Some(episode) = episodes.next().await {
        match self.process_episode(episode).await {
            Ok(_) => count += 1,
            Err(e) => {
                errors += 1;
                tracing::error!(error = %e, "Failed to process episode");
            }
        }
    }

    Ok(ProcessingStats { processed: count, errors })
}
```

## Summary of Key Patterns

1. **MCP Protocol**: JSON-RPC with tool-based communication
2. **Trait-Based Extensibility**: Pattern extractors, storage backends
3. **Circuit Breaker**: Fault tolerance for external APIs
4. **Queue-Based Processing**: Async pattern extraction pipeline
5. **Dual Storage**: Turso + redb with Postcard serialization
6. **LRU Cache**: Postcard-based caching with TTL
7. **Defense-in-Depth**: 6-layer WASM sandbox
8. **Connection Pooling**: Semaphore-based limiting
9. **Structured Logging**: tracing with rich metadata
10. **Progressive Disclosure**: Tool availability based on usage