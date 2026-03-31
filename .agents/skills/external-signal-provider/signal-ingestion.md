# Signal Ingestion

How to ingest and process signals from external providers.

## Protocol Handling

### AgentFS SDK Integration

```rust
use agentfs_sdk::{ToolCalls, KvStore};

pub struct AgentFsClient {
    tool_calls: ToolCalls,
    kv_store: KvStore,
}

impl AgentFsClient {
    pub async fn new(db_path: &str) -> Result<Self> {
        let tool_calls = ToolCalls::new(db_path).await?;
        let kv_store = KvStore::new(db_path).await?;
        
        Ok(Self {
            tool_calls,
            kv_store,
        })
    }
    
    /// Get toolcall statistics for correlation
    pub async fn get_tool_stats(
        &self,
        tool_name: &str,
        window: TimeWindow,
    ) -> Result<ToolCallStats> {
        // Query using built-in stats() method
        let stats = self.tool_calls.stats_for(tool_name).await?;
        
        Ok(stats)
    }
}
```

### Two-Phase Pattern Handling

AgentFS uses a two-phase pattern: `start()` → `success()`/`error()`. For reward signals, we only care about completed calls:

```rust
// Only consider completed tool calls with status
let relevant_calls = tool_calls
    .iter()
    .filter(|tc| tc.status == ToolCallStatus::Success || tc.status == ToolCallStatus::Error)
    .filter(|tc| tc.completed_at.is_some());
```

## Authentication Patterns

### Environment Variables (Required)

```rust
pub struct AgentFsConfig {
    pub db_path: String,
    pub enabled: bool,
}

impl AgentFsConfig {
    pub fn from_env() -> Result<Self> {
        let db_path = std::env::var("AGENTFS_DB_PATH")
            .map_err(|_| Error::ConfigMissing("AGENTFS_DB_PATH"))?;
        let enabled = std::env::var("AGENTFS_ENABLED")
            .map(|v| v == "true")
            .unwrap_or(false);
            
        Ok(Self { db_path, enabled })
    }
}
```

### No Hardcoded Credentials

Never hardcode credentials in source:

```rust
// WRONG
const API_KEY: &str = "sk-1234567890";

// CORRECT
let api_key = std::env::var("AGENTFS_API_KEY")?;
```

## Signal Normalization

### External → Internal Mapping

```rust
pub fn normalize_tool_stats(
    agentfs_stats: ToolCallStats,
    episode: &Episode,
) -> ToolSignal {
    ToolSignal {
        tool_name: agentfs_stats.name.clone(),
        success_rate: agentfs_stats.successful as f32 
            / agentfs_stats.total_calls as f32,
        avg_latency_ms: agentfs_stats.avg_duration_ms,
        sample_count: agentfs_stats.total_calls as usize,
        metadata: {
            let mut map = HashMap::new();
            map.insert("failed".to_string(), 
                json!(agentfs_stats.failed));
            map
        },
    }
}
```

### Schema Validation

```rust
pub struct SignalValidator;

impl SignalValidator {
    pub fn validate(signal: &ExternalSignalSet) -> Result<()> {
        // Check required fields
        if signal.provider.is_empty() {
            return Err(Error::Validation("Missing provider name"));
        }
        
        // Check confidence threshold
        if signal.confidence < 0.0 || signal.confidence > 1.0 {
            return Err(Error::Validation("Invalid confidence value"));
        }
        
        // Check tool signals
        for tool_signal in &signal.tool_signals {
            if tool_signal.sample_count == 0 {
                return Err(Error::Validation(
                    format!("Zero sample count for {}", tool_signal.tool_name)
                ));
            }
        }
        
        Ok(())
    }
}
```

## Error Handling

### Provider Failure Modes

| Failure | Action | Log Level |
|---------|--------|-----------|
| Provider offline | Skip external signals, use internal only | WARN |
| Auth failure | Disable provider, alert user | ERROR |
| Timeout | Retry with backoff, then skip | WARN |
| Schema mismatch | Skip malformed signals, continue | WARN |
| Rate limit | Queue and retry later | INFO |

### Circuit Breaker Pattern

```rust
pub struct ProviderCircuitBreaker {
    failure_count: AtomicUsize,
    last_failure: Mutex<Option<Instant>>,
    state: RwLock<CircuitState>,
}

impl ProviderCircuitBreaker {
    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        // Check circuit state
        match *self.state.read().await {
            CircuitState::Open => {
                return Err(Error::CircuitOpen);
            }
            CircuitState::HalfOpen => {
                // Allow one test call
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }
        
        // Execute call
        match f().await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(e)
            }
        }
    }
}
```

## Best Practices

1. **Async All the Way**: Use `spawn_blocking` for CPU-intensive normalization
2. **Batch Queries**: Group tool lookups to minimize round trips
3. **Caching**: Cache normalized signals for short periods (30s)
4. **Sanitization**: Always sanitize parameters before storing
5. **Metrics**: Track provider health, latency, success rates

## Testing

### Mock Provider Pattern

```rust
#[cfg(test)]
pub struct MockExternalSignalProvider {
    canned_signals: HashMap<String, ExternalSignalSet>,
}

#[async_trait]
impl ExternalSignalProvider for MockExternalSignalProvider {
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet> {
        Ok(self.canned_signals
            .get(&episode.episode_id.to_string())
            .cloned()
            .unwrap_or_default())
    }
    
    fn name(&self) -> &str { "mock" }
    
    async fn health_check(&self) -> ProviderHealth {
        ProviderHealth::Healthy
    }
}
```
