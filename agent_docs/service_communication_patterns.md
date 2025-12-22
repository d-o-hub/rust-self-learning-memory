# Service Communication Patterns

## Overview

The memory system uses several communication patterns between components:

1. **MCP (Model Context Protocol)**: Primary client-server communication
2. **Internal Rust Communication**: Inter-component messaging
3. **Database Communication**: Storage layer interactions
4. **Cache Communication**: Performance optimization layer

## MCP Protocol Communication

### Client-Server Architecture
```
Client (Claude Code/OpenCode) ↔ MCP Server ↔ Memory Core ↔ Storage
```

### Tool Definitions
```rust
// MCP tool registration
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
// Client request
{
    "tool": "query_memory",
    "arguments": {
        "query": "previous debugging sessions",
        "domain": "web-api",
        "limit": 10
    }
}

// Server response
{
    "content": [
        {
            "type": "text",
            "text": "Found 3 relevant episodes..."
        }
    ]
}
```

## Internal Component Communication

### Memory Core Interface
```rust
// Async communication with memory core
pub trait MemoryStore: Send + Sync {
    async fn store_episode(&self, episode: Episode) -> Result<EpisodeId>;
    async fn retrieve_episodes(&self, query: &str) -> Result<Vec<Episode>>;
    async fn extract_patterns(&self, episodes: &[Episode]) -> Result<Vec<Pattern>>;
}
```

### Storage Communication
```rust
// Turso storage implementation
pub struct TursoStorage {
    connection: libsql::Connection,
}

impl MemoryStore for TursoStorage {
    async fn store_episode(&self, episode: Episode) -> Result<EpisodeId> {
        // Store episode and steps
        // Handle concurrent writes
        // Return episode ID
    }
}
```

### Cache Communication
```rust
// Redb cache layer
pub struct CacheLayer {
    cache: redb::Database,
    primary_store: Arc<dyn MemoryStore>,
}

impl MemoryStore for CacheLayer {
    async fn retrieve_episodes(&self, query: &str) -> Result<Vec<Episode>> {
        // Check cache first
        if let Some(results) = self.cache.get(query).await? {
            return Ok(results);
        }
        
        // Fallback to primary storage
        let results = self.primary_store.retrieve_episodes(query).await?;
        
        // Update cache
        self.cache.set(query, &results).await?;
        
        Ok(results)
    }
}
```

## Event-Driven Communication

### Episode Lifecycle Events
```rust
// Event publishing
pub enum EpisodeEvent {
    Created { episode_id: EpisodeId },
    StepAdded { episode_id: EpisodeId, step: Step },
    Completed { episode_id: EpisodeId, success: bool },
    PatternExtracted { episode_id: EpisodeId, patterns: Vec<Pattern> },
}

// Event handling
pub trait EpisodeEventHandler: Send + Sync {
    async fn handle_event(&self, event: EpisodeEvent);
}
```

### Pattern Processing Pipeline
```rust
// Async pattern extraction pipeline
pub struct PatternProcessor {
    extractors: Vec<Box<dyn PatternExtractor>>,
    store: Arc<dyn MemoryStore>,
}

impl PatternProcessor {
    pub async fn process_episode(&self, episode: &Episode) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();
        
        // Parallel pattern extraction
        let handles: Vec<_> = self.extractors
            .iter()
            .map(|extractor| {
                tokio::spawn(async move {
                    extractor.extract_patterns(episode).await
                })
            })
            .collect();
        
        // Collect results
        for handle in handles {
            if let Ok(Ok(new_patterns)) = handle.await {
                patterns.extend(new_patterns);
            }
        }
        
        // Store patterns
        self.store.store_patterns(&patterns).await?;
        
        Ok(patterns)
    }
}
```

## Error Handling Patterns

### Result Propagation
```rust
// Error types
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] libsql::Error),
    
    #[error("Cache error: {0}")]
    Cache(#[from] redb::Error),
    
    #[error("Embedding error: {0}")]
    Embedding(String),
}

// Error handling in async functions
pub async fn retrieve_episodes(
    &self,
    query: &str,
) -> Result<Vec<Episode>, MemoryError> {
    match self.cache.get(query).await {
        Ok(Some(episodes)) => Ok(episodes),
        Ok(None) => {
            // Cache miss, query database
            match self.database.query_episodes(query).await {
                Ok(episodes) => {
                    // Update cache asynchronously
                    let cache = self.cache.clone();
                    tokio::spawn(async move {
                        let _ = cache.set(query, &episodes).await;
                    });
                    Ok(episodes)
                }
                Err(e) => Err(MemoryError::Database(e)),
            }
        }
        Err(e) => Err(MemoryError::Cache(e)),
    }
}
```

### Circuit Breaker Pattern
```rust
pub struct CircuitBreaker {
    failures: AtomicUsize,
    last_failure: AtomicU64,
    threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    pub async fn call<T>(&self, operation: impl Future<Output = Result<T>>) -> Result<T> {
        if self.is_open() {
            return Err(MemoryError::CircuitBreakerOpen);
        }
        
        match operation.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e)
            }
        }
    }
}
```

## Performance Communication Patterns

### Connection Pooling
```rust
// Turso connection pool
pub struct TursoPool {
    pool: libsql::Pool,
    max_connections: usize,
}

impl TursoPool {
    pub async fn get_connection(&self) -> Result<libsql::Connection> {
        self.pool.get().await.map_err(MemoryError::Database)
    }
}
```

### Batch Operations
```rust
// Batch episode storage
pub async fn store_episodes_batch(
    &self,
    episodes: &[Episode],
) -> Result<Vec<EpisodeId>> {
    let mut transaction = self.connection.transaction().await?;
    
    let mut ids = Vec::new();
    for episode in episodes {
        let id = self.store_episode_transaction(&mut transaction, episode).await?;
        ids.push(id);
    }
    
    transaction.commit().await?;
    Ok(ids)
}
```

### Async Streaming
```rust
// Streaming query results
pub async fn stream_episodes(
    &self,
    query: &str,
) -> Result<impl Stream<Item = Result<Episode>>> {
    let mut statement = self.connection
        .prepare("SELECT * FROM episodes WHERE domain = ?")
        .await?;
    
    Ok(stream! {
        let mut rows = statement.query([query]).await?;
        while let Some(row) = rows.next().await? {
            yield Ok(Episode::from_row(&row));
        }
    })
}
```

## Monitoring and Observability

### Health Check Communication
```rust
// Health check endpoint
#[mcp_server::tool]
async fn health_check() -> Result<HealthStatus, mcp_server::Error> {
    let checks = vec![
        self.check_database().await?,
        self.check_cache().await?,
        self.check_embeddings().await?,
    ];
    
    Ok(HealthStatus {
        overall: checks.iter().all(|c| c.healthy),
        checks,
    })
}
```

### Metrics Communication
```rust
// Metrics collection
pub struct MetricsCollector {
    episodes_created: Counter,
    query_latency: Histogram,
    cache_hit_rate: Gauge,
}

impl MetricsCollector {
    pub async fn record_episode_created(&self, domain: &str) {
        self.episodes_created.with_label_values([domain]).inc();
    }
}
```

### Logging Patterns
```rust
// Structured logging
pub async fn store_episode(&self, episode: &Episode) -> Result<EpisodeId> {
    let episode_id = episode.id();
    
    tracing::info!(
        episode_id = %episode_id,
        domain = %episode.domain(),
        "Storing new episode"
    );
    
    match self.storage.store_episode(episode).await {
        Ok(id) => {
            tracing::info!(episode_id = %id, "Episode stored successfully");
            Ok(id)
        }
        Err(e) => {
            tracing::error!(episode_id = %episode_id, error = %e, "Failed to store episode");
            Err(e)
        }
    }
}
```

## Security Communication

### Authentication
```rust
// MCP authentication
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
}
```