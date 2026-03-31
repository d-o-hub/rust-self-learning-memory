# External Signal Provider Examples

AgentFS-specific implementation patterns and examples.

## Basic AgentFS Provider

```rust
use agentfs_sdk::{ToolCalls, KvStore, ToolCallStats};
use async_trait::async_trait;

/// AgentFS external signal provider implementation
pub struct AgentFsProvider {
    db_path: String,
    config: AgentFsConfig,
}

#[derive(Clone)]
pub struct AgentFsConfig {
    pub db_path: String,
    pub enabled: bool,
    pub external_weight: f32,
    pub min_correlation_samples: usize,
    pub sanitize_parameters: bool,
}

impl AgentFsProvider {
    pub fn new(config: AgentFsConfig) -> Self {
        Self { 
            db_path: config.db_path.clone(),
            config,
        }
    }
}

#[async_trait]
impl ExternalSignalProvider for AgentFsProvider {
    fn name(&self) -> &str {
        "agentfs"
    }
    
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet> {
        if !self.config.enabled {
            return Ok(ExternalSignalSet::empty("agentfs"));
        }
        
        // Initialize AgentFS client
        let tc = ToolCalls::new(&self.db_path).await
            .map_err(|e| Error::ExternalProvider(e.to_string()))?;
        
        // Get all tool stats
        let all_stats = tc.stats().await
            .map_err(|e| Error::ExternalProvider(e.to_string()))?;
        
        // Filter and normalize tool signals
        let tool_signals: Vec<ToolSignal> = all_stats
            .into_iter()
            .filter(|stats| stats.total_calls >= self.config.min_correlation_samples as i64)
            .map(|stats| normalize_agentfs_stats(stats, &self.config))
            .collect();
        
        // Calculate overall confidence based on sample sizes
        let total_samples: usize = tool_signals.iter()
            .map(|t| t.sample_count)
            .sum();
        let confidence = (total_samples as f32 / 100.0).min(1.0); // Cap at 1.0
        
        Ok(ExternalSignalSet {
            provider: "agentfs".to_string(),
            tool_signals,
            episode_quality: None, // Calculate from tool signals if needed
            timestamp: Utc::now(),
            confidence,
        })
    }
    
    async fn health_check(&self) -> ProviderHealth {
        match ToolCalls::new(&self.db_path).await {
            Ok(_) => ProviderHealth::Healthy,
            Err(e) => ProviderHealth::Unhealthy(e.to_string()),
        }
    }
    
    fn validate_config(&self) -> Result<()> {
        if self.config.db_path.is_empty() {
            return Err(Error::ConfigMissing("AgentFS db_path"));
        }
        if self.config.external_weight < 0.0 || self.config.external_weight > 1.0 {
            return Err(Error::InvalidConfig("Weight must be 0.0-1.0"));
        }
        Ok(())
    }
}

fn normalize_agentfs_stats(stats: ToolCallStats, config: &AgentFsConfig) -> ToolSignal {
    let success_rate = if stats.total_calls > 0 {
        stats.successful as f32 / stats.total_calls as f32
    } else {
        0.5 // Neutral if no data
    };
    
    ToolSignal {
        tool_name: stats.name.clone(),
        success_rate,
        avg_latency_ms: stats.avg_duration_ms,
        sample_count: stats.total_calls as usize,
        metadata: {
            let mut map = HashMap::new();
            map.insert("failed".to_string(), json!(stats.failed));
            map.insert("provider".to_string(), json!("agentfs"));
            if config.sanitize_parameters {
                map.insert("sanitized".to_string(), json!(true));
            }
            map
        },
    }
}
```

## Configuration from Environment

```rust
impl AgentFsConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let db_path = std::env::var("AGENTFS_DB_PATH")
            .map_err(|_| Error::ConfigMissing(
                "AGENTFS_DB_PATH environment variable not set"
            ))?;
        
        let enabled = std::env::var("AGENTFS_ENABLED")
            .map(|v| v == "true")
            .unwrap_or(false);
        
        let external_weight = std::env::var("AGENTFS_WEIGHT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.3);
        
        let min_correlation_samples = std::env::var("AGENTFS_MIN_SAMPLES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        
        let sanitize_parameters = std::env::var("AGENTFS_SANITIZE")
            .map(|v| v == "true")
            .unwrap_or(true);
        
        Ok(Self {
            db_path,
            enabled,
            external_weight,
            min_correlation_samples,
            sanitize_parameters,
        })
    }
}
```

## Integration with SelfLearningMemory

```rust
impl SelfLearningMemoryBuilder {
    /// Enable AgentFS external signal provider
    pub fn with_agentfs_provider(self, config: AgentFsConfig) -> Self {
        let provider = Box::new(AgentFsProvider::new(config));
        self.with_external_signal_provider(provider)
    }
}

// Usage
let memory = SelfLearningMemory::builder()
    .with_agentfs_provider(AgentFsConfig::from_env().unwrap())
    .build();
```

## Correlating Episode Steps with AgentFS

```rust
impl AgentFsProvider {
    /// Find AgentFS tool calls matching episode steps
    async fn correlate_steps(
        &self,
        episode: &Episode,
        tc: &ToolCalls,
    ) -> Result<Vec<CorrelatedToolCall>> {
        let mut correlations = Vec::new();
        
        for step in &episode.steps {
            // Query AgentFS for this specific tool
            let stats = tc.stats_for(&step.tool).await?;
            
            if let Some(stats) = stats {
                correlations.push(CorrelatedToolCall {
                    step_number: step.step_number,
                    tool_name: step.tool.clone(),
                    agentfs_success_rate: stats.successful as f32 / stats.total_calls as f32,
                    sample_count: stats.total_calls as usize,
                });
            }
        }
        
        Ok(correlations)
    }
}

pub struct CorrelatedToolCall {
    pub step_number: usize,
    pub tool_name: String,
    pub agentfs_success_rate: f32,
    pub sample_count: usize,
}
```

## Handling Privacy-Sensitive Data

```rust
impl AgentFsProvider {
    /// Sanitize parameters before storing in memory system
    fn sanitize_parameters(params: &Value) -> Value {
        match params {
            Value::Object(map) => {
                let mut sanitized = serde_json::Map::new();
                for (key, _value) in map {
                    // Keep keys, redact values
                    sanitized.insert(key.clone(), Value::String("[REDACTED]".to_string()));
                }
                Value::Object(sanitized)
            }
            _ => Value::String("[REDACTED]".to_string()),
        }
    }
    
    /// Truncate large results
    fn truncate_result(result: &Value, max_len: usize) -> Value {
        let result_str = result.to_string();
        if result_str.len() > max_len {
            let truncated = format!("{}...[truncated]", &result_str[..max_len]);
            Value::String(truncated)
        } else {
            result.clone()
        }
    }
}
```

## Testing with Mock AgentFS

```rust
#[cfg(test)]
pub struct MockAgentFsProvider {
    canned_stats: HashMap<String, ToolCallStats>,
}

#[async_trait]
impl ExternalSignalProvider for MockAgentFsProvider {
    fn name(&self) -> &str { "mock-agentfs" }
    
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet> {
        let tool_signals: Vec<ToolSignal> = episode.steps
            .iter()
            .filter_map(|step| {
                self.canned_stats.get(&step.tool).map(|stats| {
                    ToolSignal {
                        tool_name: stats.name.clone(),
                        success_rate: stats.successful as f32 / stats.total_calls as f32,
                        avg_latency_ms: stats.avg_duration_ms,
                        sample_count: stats.total_calls as usize,
                        metadata: HashMap::new(),
                    }
                })
            })
            .collect();
        
        Ok(ExternalSignalSet {
            provider: "mock-agentfs".to_string(),
            tool_signals,
            timestamp: Utc::now(),
            confidence: 0.9,
            episode_quality: None,
        })
    }
    
    async fn health_check(&self) -> ProviderHealth {
        ProviderHealth::Healthy
    }
}

#[tokio::test]
async fn test_agentfs_provider() {
    let mock_provider = MockAgentFsProvider {
        canned_stats: {
            let mut map = HashMap::new();
            map.insert("web_search".to_string(), ToolCallStats {
                name: "web_search".to_string(),
                total_calls: 100,
                successful: 95,
                failed: 5,
                avg_duration_ms: 150.0,
            });
            map
        },
    };
    
    // Test with episode containing "web_search" step
    let mut episode = create_test_episode();
    add_step(&mut episode, "web_search", "Search for docs");
    
    let signals = mock_provider.get_signals(&episode).await.unwrap();
    assert_eq!(signals.tool_signals.len(), 1);
    assert_eq!(signals.tool_signals[0].tool_name, "web_search");
    assert!((signals.tool_signals[0].success_rate - 0.95).abs() < 0.01);
}
```

## CLI Integration Example

```rust
// do-memory-cli/src/commands/external_signals/configure.rs

use clap::Args;

#[derive(Args)]
pub struct ConfigureAgentFsArgs {
    /// Path to AgentFS database
    #[arg(long, env = "AGENTFS_DB_PATH")]
    db_path: String,
    
    /// Enable AgentFS integration
    #[arg(long, default_value = "true")]
    enabled: bool,
    
    /// Weight for external signals (0.0-1.0)
    #[arg(long, default_value = "0.3")]
    weight: f32,
}

pub async fn configure_agentfs(args: &ConfigureAgentFsArgs) -> Result<()> {
    let config = AgentFsConfig {
        db_path: args.db_path.clone(),
        enabled: args.enabled,
        external_weight: args.weight,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };
    
    // Validate configuration
    let provider = AgentFsProvider::new(config.clone());
    provider.validate_config()?;
    
    // Test connection
    match provider.health_check().await {
        ProviderHealth::Healthy => {
            println!("✓ AgentFS connection successful");
            
            // Store configuration
            save_config(&config)?;
            println!("✓ Configuration saved");
        }
        ProviderHealth::Unhealthy(reason) => {
            return Err(Error::ExternalProvider(format!(
                "AgentFS health check failed: {}", reason
            )));
        }
    }
    
    Ok(())
}
```

## Docker Compose Setup

```yaml
# docker-compose.yml for testing
version: '3.8'

services:
  memory-system:
    build: .
    environment:
      - AGENTFS_ENABLED=true
      - AGENTFS_DB_PATH=/data/agent.db
      - AGENTFS_WEIGHT=0.3
      - RUST_LOG=info
    volumes:
      - ./data:/data
    depends_on:
      - agentfs
      
  agentfs:
    image: tursodatabase/agentfs:latest
    volumes:
      - ./data:/data
    environment:
      - AGENTFS_DB_PATH=/data/agent.db
```

## Troubleshooting

### Common Issues

**Issue**: AgentFS provider returns empty signals
```
Cause: Database path incorrect or no tool calls recorded
Solution: Verify AGENTFS_DB_PATH and check if tools have been called
```

**Issue**: Low confidence scores
```
Cause: Insufficient sample size (default min: 10)
Solution: Lower AGENTFS_MIN_SAMPLES or accumulate more tool calls
```

**Issue**: Connection timeout
```
Cause: AgentFS service unavailable
Solution: Check service health, implement circuit breaker
```

**Issue**: Schema mismatch errors
```
Cause: AgentFS SDK version mismatch
Solution: Update to agentfs-sdk = "0.6.4" in Cargo.toml
```

## Performance Tuning

```rust
impl AgentFsProvider {
    /// Optimize for low-latency signal retrieval
    pub fn with_caching(self, cache_duration: Duration) -> CachingAgentFsProvider {
        CachingAgentFsProvider {
            inner: self,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_duration,
        }
    }
}

pub struct CachingAgentFsProvider {
    inner: AgentFsProvider,
    cache: Arc<RwLock<HashMap<String, (ExternalSignalSet, Instant)>>>,
    cache_duration: Duration,
}

#[async_trait]
impl ExternalSignalProvider for CachingAgentFsProvider {
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet> {
        let cache_key = episode.episode_id.to_string();
        
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some((signals, timestamp)) = cache.get(&cache_key) {
                if timestamp.elapsed() < self.cache_duration {
                    return Ok(signals.clone());
                }
            }
        }
        
        // Fetch fresh signals
        let signals = self.inner.get_signals(episode).await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, (signals.clone(), Instant::now()));
        }
        
        Ok(signals)
    }
    
    fn name(&self) -> &str { self.inner.name() }
}
```
