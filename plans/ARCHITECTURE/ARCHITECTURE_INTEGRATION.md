# Current Architecture - Integration & Systems

**Last Updated**: 2026-10-01
**Workspace Version**: v0.1.44 (Released tag: v0.1.43)
**Baseline SHA**: `64b5a33c75d9901bd9216eaa81ca50038ef66c96`

---

## Integration Points

### Memory ↔ Storage

**Interface**: `StorageBackend` trait

**Flow**:
1. SelfLearningMemory uses `Arc<dyn StorageBackend>`
2. Circuit breaker wraps storage calls
3. Two-layer write (Turso + redb)
4. Async throughout

**Error Handling**:
- Exponential backoff retry
- Circuit breaker prevents cascading failures
- Graceful degradation (cache fallback)

**Implementation**:
```rust
impl SelfLearningMemory {
    pub async fn complete_episode(&self, episode_id: Uuid, ...) -> Result<()> {
        // ... extract patterns, generate reflection ...

        // Store in both backends
        match self.primary_storage.store_episode(&episode).await {
            Ok(_) => {
                // Success, update cache
                let _ = self.cache_storage.store_episode(&episode).await;
                Ok(())
            }
            Err(e) => {
                // Primary failed, check circuit breaker
                if self.circuit_breaker.is_open() {
                    // Use cache only
                    self.cache_storage.store_episode(&episode).await?;
                    tracing::warn!("Using cache fallback due to circuit breaker");
                }
                Err(e)
            }
        }
    }
}
```

### Memory ↔ MCP

**Interface**: Direct `Arc<SelfLearningMemory>` integration

**Flow**:
1. MCP tool receives JSON-RPC request
2. Deserializes to typed parameters
3. Calls SelfLearningMemory methods
4. Returns result as JSON-RPC response

**Security**:
- Input validation
- Optional OAuth 2.1 authentication and per-client rate limiting
- Agent code execution is fail-closed (no production sandbox)

**Implementation**:
```rust
impl MemoryMCPServer {
    pub async fn handle_query_memory(&self, params: QueryParams) -> Result<JsonValue> {
        // Validate input
        self.validate_query_params(&params)?;

        // Retrieve context
        let episodes = self.memory
            .retrieve_context(&params.query, params.task_type)
            .await?;

        // Serialize to JSON
        let result = json!({
            "episodes": episodes,
            "count": episodes.len()
        });

        Ok(result)
    }
}
```

### Storage Synchronization

**Coordinator**: `StorageSynchronizer<Turso, Redb>`

**Strategy**:
- Turso is source of truth (durable)
- redb is fast cache (embedded)
- Write-through caching
- Periodic background sync (5 min default)
- Conflict resolution (Turso wins)

**Sync Flow**:
```rust
impl StorageSynchronizer<P, C> {
    pub async fn write(&self, episode: &Episode) -> Result<()> {
        // Write to primary first (Turso)
        self.primary.store_episode(episode).await?;

        // Write to cache (redb) - non-blocking
        let cache = self.cache.clone();
        let episode = episode.clone();
        tokio::spawn(async move {
            if let Err(e) = cache.store_episode(&episode).await {
                tracing::error!("Cache write failed: {}", e);
            }
        });

        Ok(())
    }

    pub async fn read(&self, id: Uuid) -> Result<Option<Episode>> {
        // Try cache first
        if let Some(episode) = self.cache.get_episode(id).await? {
            return Ok(Some(episode));
        }

        // Cache miss, read from primary
        let episode = self.primary.get_episode(id).await?;

        // Update cache asynchronously
        if let Some(ref ep) = episode {
            let cache = self.cache.clone();
            let ep = ep.clone();
            tokio::spawn(async move {
                if let Err(e) = cache.store_episode(&ep).await {
                    tracing::error!("Cache update failed: {}", e);
                }
            });
        }

        Ok(episode)
    }

    pub async fn background_sync(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.sync_interval_secs));

        loop {
            interval.tick().await;

            // Sync recent episodes from primary to cache
            if let Err(e) = self.sync_recent_episodes().await {
                tracing::error!("Background sync failed: {}", e);
            }
        }
    }
}
```

---

## MCP Server: do-memory-mcp

### Architecture

**Main Server**: `MemoryMCPServer`

**Responsibilities**:
- Integrates with `Arc<SelfLearningMemory>`
- Provides standardized MCP tools
- Handles JSON-RPC protocol
- Keeps agent code execution fail-closed

### MCP Tools (6 Main)

#### 1. query_memory
**Purpose**: Retrieve episodes and patterns

**Parameters**:
```json
{
  "query": "implement REST API",
  "domain": "web-api",
  "task_type": "code_generation",
  "limit": 10
}
```

**Returns**: Relevant episodes with context

**Implementation**:
```rust
pub async fn query_memory(&self, params: QueryParams) -> Result<JsonValue> {
    // Validate parameters
    let task_type = params.task_type.parse::<TaskType>()?;

    // Retrieve context
    let episodes = self.memory
        .retrieve_context(&params.query, task_type)
        .await?;

    // Filter by domain if provided
    let filtered = if let Some(domain) = &params.domain {
        episodes.into_iter()
            .filter(|ep| ep.domain.as_deref() == Some(domain.as_str()))
            .collect()
    } else {
        episodes
    };

    // Limit results
    let limited = filtered.into_iter()
        .take(params.limit.unwrap_or(10))
        .collect::<Vec<_>>();

    Ok(json!(limited))
}
```

#### 2. execute_agent_code
**Purpose**: **Unavailable / fail-closed** in production

The WASM sandbox was removed in v0.1.29 (ADR-052). No execution backend is
registered, so the tool is absent from `tools/list` and direct calls are
rejected by the dispatcher:

```text
jsonrpc error -32000
  message: "Tool execution failed"
  data: { "details": "execute_agent_code tool is not available due to WASM sandbox compilation issues" }
```

Use episode/memory tools for agent work, or run generated code in an external
runner. See ADR-073 for the (future) capability-enforced contract and
[../../memory-mcp/SECURITY.md](../../memory-mcp/SECURITY.md) for the security
boundary.

#### 3. analyze_patterns
**Purpose**: Statistical and predictive pattern analysis

**Parameters**:
```json
{
  "task_type": "debugging",
  "min_success_rate": 0.7,
  "limit": 20
}
```

**Returns**: Successful patterns with statistics

**Statistical Analysis**:
```rust
pub async fn analyze_patterns(&self, params: AnalysisParams) -> Result<JsonValue> {
    // Query patterns
    let patterns = self.memory
        .get_patterns_by_type(params.task_type)
        .await?;

    // Filter by success rate
    let filtered = patterns.into_iter()
        .filter(|p| p.success_rate >= params.min_success_rate.unwrap_or(0.0))
        .collect::<Vec<_>>();

    // Calculate statistics
    let stats = self.calculate_statistics(&filtered);

    Ok(json!({
        "patterns": &filtered[..params.limit.unwrap_or(20).min(filtered.len())],
        "statistics": stats
    }))
}
```

#### 4. advanced_pattern_analysis
**Purpose**: Comprehensive pattern analysis (statistical + predictive + causal)

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

**Returns**: Statistical tests, forecasts, anomalies, causal relationships

**Comprehensive Analysis**:
```rust
pub async fn advanced_pattern_analysis(&self, params: AdvancedParams) -> Result<JsonValue> {
    match params.analysis_type.as_str() {
        "statistical" => self.statistical_analysis(params).await,
        "predictive" => self.predictive_analysis(params).await,
        "comprehensive" => self.comprehensive_analysis(params).await,
        _ => Err(anyhow!("Invalid analysis type")),
    }
}

async fn comprehensive_analysis(&self, params: AdvancedParams) -> Result<JsonValue> {
    // Run all analysis types
    let statistical = self.statistical_analysis(params.clone()).await?;
    let predictive = self.predictive_analysis(params.clone()).await?;

    // Combine results
    Ok(json!({
        "statistical": statistical,
        "predictive": predictive
    }))
}
```

#### 5. health_check
**Purpose**: System health status

**Returns**: Overall health and component status

**Health Check**:
```rust
pub async fn health_check(&self) -> Result<JsonValue> {
    // Check storage backends
    let primary_health = self.primary_storage.health_check().await?;
    let cache_health = self.cache_storage.health_check().await?;

    // Check memory system
    let memory_health = self.memory.health_check().await?;

    // Overall health
    let overall = if primary_health.is_healthy() &&
                      cache_health.is_healthy() &&
                      memory_health.is_healthy() {
        "healthy"
    } else {
        "degraded"
    };

    Ok(json!({
        "overall": overall,
        "primary_storage": primary_health,
        "cache_storage": cache_health,
        "memory_system": memory_health
    }))
}
```

#### 6. get_metrics
**Purpose**: Performance metrics and statistics

**Returns**: Various metrics (episodes, patterns, performance)

**Metrics Collection**:
```rust
pub async fn get_metrics(&self) -> Result<JsonValue> {
    let monitoring = self.memory.get_monitoring_summary().await?;

    Ok(json!({
        "episodes": {
            "total": monitoring.episodes_created,
            "completed": monitoring.episodes_completed,
            "completion_rate": monitoring.completion_rate
        },
        "patterns": {
            "total": monitoring.patterns_extracted,
            "unique": monitoring.unique_patterns,
            "effectiveness": monitoring.pattern_effectiveness
        },
        "performance": {
            "average_latency_ms": monitoring.avg_latency_ms,
            "cache_hit_rate": monitoring.cache_hit_rate,
            "error_rate": monitoring.error_rate
        }
    }))
}
```

### Code Execution Architecture

**Production state**: There is **no** unified sandbox and no WASM/Wasmtime/Javy
backend. `execute_agent_code` is fail-closed; the dispatcher rejects it and the
tool is not advertised.

The historical `UnifiedSandbox`/`WasmtimeSandbox`/`CodeSandbox` designs were
removed in v0.1.29 (ADR-052). A legacy Node.js `CodeSandbox`
(`memory-mcp/src/sandbox/`) survives only behind the non-default `sandbox-dev`
feature for trusted local experimentation. It is not a production sandbox:

- **Enforced**: execution timeout with `kill_on_drop`, 100 KB code-length cap,
  static regex source screening, wrapper global shadowing/deletion.
- **Not enforced**: OS-level isolation (`apply_isolation` is never called),
  memory/CPU limits, output sanitization, runtime capability enforcement.

See [../../memory-mcp/SECURITY.md](../../memory-mcp/SECURITY.md).

---

## CLI: do-memory-cli

### Command Structure

**Main Binary**: `do-memory-cli`

**Commands (8 main)**:

#### 1. episode - Manage episodes
- `list` - List all episodes
- `get <id>` - Get episode details
- `create` - Create new episode
- `delete <id>` - Delete episode

#### 2. pattern - Analyze patterns
- `list` - List all patterns
- `analyze` - Analyze patterns by type
- `effectiveness` - Pattern effectiveness stats

#### 3. storage - Storage management
- `status` - Check storage status
- `sync` - Trigger manual sync
- `backup` - Create backup
- `verify` - Verify data integrity

#### 4. config - Configuration
- `show` - Display current config
- `validate` - Validate configuration
- `reset` - Reset to defaults
- `init` - Interactive wizard

#### 5. health - System health
- Check all components
- Report status

#### 6. backup - Data backup
- Create snapshots
- Restore from backup

#### 7. monitor - Real-time monitoring
- Live metrics display

#### 8. logs - Log management
- Filter and view logs

### Configuration System (Modular)

**Location**: `do-memory-cli/src/config/`

**Modules**:
1. **types.rs** - Core config structures
2. **loader.rs** - Configuration loading ✅ REFACTORED
3. **validator.rs** - Configuration validation
4. **simple.rs** - Simple setup functions
5. **progressive.rs** - Progressive configuration
6. **wizard.rs** - Interactive setup
7. **storage.rs** - Storage initialization

**Smart Defaults**:
```rust
// Platform-aware paths
fn detect_data_directory() -> PathBuf;
fn detect_cache_directory() -> PathBuf;

// Resource-aware defaults
fn get_system_info() -> SystemInfo;
fn suggest_pool_size(cpu_count: usize) -> usize;  // cpu_count * 2, clamped [3, 20]
fn suggest_cache_size(available_gb: f64) -> usize; // gb * 200MB, clamped [1000, 5000]
```

### Output System

**Features**:
- Colored output (terminal-aware)
- Progress indicators (indicatif)
- Interactive dialogs (dialoguer)
- Table formatting (prettytable-rs)

---

## Feature Matrix

| Feature | do-memory-core | do-memory-mcp | do-memory-cli | do-memory-storage-turso | do-memory-storage-redb |
|---------|------------|-----------|-----------|---------------------|---------------------|
| **Embeddings** | ✅ Local/OpenAI | ❌ | ❌ | ❌ | ✅ Cache layer |
| **Patterns** | ✅ Extraction/Validation | ✅ Analysis | ✅ Management | ✅ Storage | ✅ Cache |
| **Reward Scoring** | ✅ Multi-component | ❌ | ❌ | ❌ | ❌ |
| **Reflection** | ✅ Insight generation | ❌ | ❌ | ❌ | ❌ |
| **Monitoring** | ✅ Basic metrics | ✅ Full MCP | ❌ | ✅ Usage tracking | ✅ Cache metrics |
| **Code Execution** | ❌ | ❌ fail-closed (`sandbox-dev` = trusted local only) | ❌ | ❌ | ❌ |
| **Pattern Analysis** | ✅ Basic | ✅ Statistical/Predictive | ❌ | ❌ | ❌ |
| **Configuration** | ✅ MemoryConfig | ✅ SandboxConfig | ✅ Full system | ✅ TursoConfig | ✅ CacheConfig |

---

## Dependency Graph

```
do-memory-cli
├── do-memory-core
│   ├── storage/ (trait abstraction)
│   ├── embeddings/ (local/openai via features)
│   ├── patterns/ (extraction/validation)
│   ├── reflection/ (insight generation)
│   ├── reward/ (multi-component scoring)
│   └── learning/ (async queue)
├── do-memory-storage-redb
│   └── redb (embedded KV)
└── do-memory-storage-turso
    └── libsql (remote/local)

do-memory-mcp
├── do-memory-core (shared)
├── do-memory-storage-turso (shared)
├── do-memory-storage-redb (shared)
├── agentfs-sdk
├── sysinfo (health/metrics)
└── jsonwebtoken (optional, `oauth` feature)

Shared Dependencies:
- tokio (async runtime)
- serde (serialization)
- anyhow (error handling)
- tracing (logging)
- uuid (IDs)
```

---

## Feature Flags

### do-memory-core
```toml
[features]
default = []
openai = ["reqwest"]
embeddings-full = ["openai", "local-embeddings"]
local-embeddings = ["candle-core", "candle-nn", "tokenizers"]
```

### do-memory-mcp
```toml
[features]
default = []
oauth = ["dep:jsonwebtoken"]
embeddings = []
streaming-impl = []
sandbox-dev = []   # trusted local experimentation only; not a production sandbox
```

### do-memory-cli
```toml
[features]
default = ["redb"]
turso = ["do-memory-storage-turso"]
redb = ["do-memory-storage-redb"]
full = ["turso", "redb"]
```

---

## Cross-References

- **Core Components**: See [ARCHITECTURE_CORE.md](ARCHITECTURE_CORE.md)
- **Patterns**: See [ARCHITECTURE_PATTERNS.md](ARCHITECTURE_PATTERNS.md)
- **Configuration**: See [Configuration Guide](../../memory-cli/CONFIGURATION_GUIDE.md)
- **Current Status**: See [Active Roadmap](../ROADMAPS/ROADMAP_ACTIVE.md)

---

*Last Updated: 2026-10-01*
*Architecture Score: 5.0/5*
