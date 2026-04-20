# External Signals Integration

Guide for integrating external signal providers (AgentFS, audit trails, etc.) into the rust-self-learning-memory system.

## Overview

The memory system supports external signal providers that augment internal reward calculations with ground-truth data from production tool execution. This enables:

- **Bayesian ranking** with real-world effectiveness data
- **Cross-system learning** from AgentFS toolcall audit trails
- **Pattern validation** against actual tool success rates
- **Extensible architecture** for future providers (GitHub Copilot, IDE telemetry, etc.)

## Quick Start

### 1. Enable AgentFS Provider

```bash
# Set environment variables
export AGENTFS_ENABLED=true
export AGENTFS_DB_PATH=/path/to/agent.db
export AGENTFS_WEIGHT=0.3
export EXTERNAL_SIGNALS_ENABLED=true

# Build with AgentFS feature
cargo build --features agentfs
```

### 2. Configure in Code

```rust
use memory_core::{SelfLearningMemory, ExternalSignalConfig};

let memory = SelfLearningMemory::builder()
    .with_agentfs_provider(AgentFsConfig::from_env()?)
    .build();
```

### 3. Verify Integration

```bash
# CLI command
do-memory-cli external-signals status

# Expected output:
# ✓ AgentFS: Healthy (100 tool calls recorded)
# ✓ External signals enabled with 30% weight
```

## Architecture

### Data Flow

```
Episode Creation → log_step() → Episode Completion
                              ↓
                    ┌─────────────────────┐
                    │ External Signal     │
                    │   Provider Query    │
                    └─────────────────────┘
                              ↓
                    ┌─────────────────────┐
                    │ Signal Normalization│
                    │ (External → Internal)│
                    └─────────────────────┘
                              ↓
                    ┌─────────────────────┐
                    │ Signal Merging      │
                    │ (70% internal +     │
                    │  30% external)       │
                    └─────────────────────┘
                              ↓
                    ┌─────────────────────┐
                    │ Final Reward Score  │
                    └─────────────────────┘
```

### Core Traits

```rust
/// All external providers implement this trait
#[async_trait]
pub trait ExternalSignalProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet>;
    async fn health_check(&self) -> ProviderHealth;
}

/// Normalized signal format
pub struct ExternalSignalSet {
    pub provider: String,
    pub tool_signals: Vec<ToolSignal>,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

/// Per-tool signal data
pub struct ToolSignal {
    pub tool_name: String,
    pub success_rate: f32,
    pub avg_latency_ms: f64,
    pub sample_count: usize,
}
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `EXTERNAL_SIGNALS_ENABLED` | `false` | Master switch for all external signals |
| `EXTERNAL_SIGNAL_WEIGHT` | `0.3` | Weight for external vs internal (0.0-1.0) |
| `EXTERNAL_SIGNAL_MIN_CONFIDENCE` | `0.5` | Minimum confidence to accept signals |
| `AGENTFS_ENABLED` | `false` | Enable AgentFS provider |
| `AGENTFS_DB_PATH` | - | Path to AgentFS SQLite database |
| `AGENTFS_WEIGHT` | `0.3` | Override weight for AgentFS specifically |
| `AGENTFS_MIN_SAMPLES` | `10` | Minimum sample size for correlation |
| `AGENTFS_SANITIZE` | `true` | Sanitize parameters before storing |

### Programmatic Configuration

```rust
use memory_core::external::{
    AgentFsConfig, ExternalSignalConfig, SignalMerger
};

let agentfs_config = AgentFsConfig {
    db_path: "/data/agent.db".to_string(),
    enabled: true,
    external_weight: 0.3,
    min_correlation_samples: 10,
    sanitize_parameters: true,
};

let signal_config = ExternalSignalConfig {
    enabled: true,
    default_weight: 0.3,
    provider_weights: {
        let mut map = HashMap::new();
        map.insert("agentfs".to_string(), 0.4); // AgentFS gets 40%
        map
    },
    min_confidence: 0.5,
};
```

## AgentFS Integration

### Prerequisites

1. **AgentFS SDK**: Install agentfs-sdk crate
   ```toml
   [dependencies]
   agentfs-sdk = { version = "0.6.4", optional = true }
   ```

2. **Database**: AgentFS SQLite database with toolcall history
   ```bash
   # Check AgentFS database
   sqlite3 /path/to/agent.db "SELECT name, COUNT(*) FROM tool_calls GROUP BY name;"
   ```

3. **Feature Flag**: Enable in Cargo.toml
   ```toml
   [features]
   agentfs = ["dep:agentfs-sdk"]
   ```

### Schema Mapping

AgentFS tool calls map to internal signals:

| AgentFS | Internal | Purpose |
|---------|----------|---------|
| `name` | `tool_name` | Tool identifier |
| `status` | `success_rate` | Success/failure rate |
| `duration_ms` | `avg_latency_ms` | Performance metric |
| `started_at` | `timestamp` | Temporal correlation |
| `parameters` | `[REDACTED]` | Privacy-sanitized |
| `result` | `[REDACTED]` | Privacy-sanitized |

### Two-Phase Pattern

AgentFS uses a two-phase recording pattern:

```rust
// Phase 1: Start tool call
let id = tc.start("web_search", Some(json!({"query": "docs"}))).await?;

// Phase 2a: Success
tc.success(id, Some(json!({"results": [...]}))).await?;

// Phase 2b: Error (alternative)
tc.error(id, "Network timeout").await?;
```

The memory system only queries **completed** tool calls (status = 'success' or 'error').

## Privacy and Security

### Data Sanitization

By default, external signals are sanitized before storage:

```rust
// Parameters: keys preserved, values redacted
{"query": "[REDACTED]", "limit": "[REDACTED]"}

// Results: truncated if too large
"[Large result truncated to 1000 chars]"
```

Disable sanitization (not recommended):
```bash
export AGENTFS_SANITIZE=false
```

### Authentication

Never hardcode credentials. Always use environment variables:

```rust
// WRONG ❌
const API_KEY: &str = "sk-123456";

// CORRECT ✅
let api_key = std::env::var("AGENTFS_API_KEY")?;
```

### Audit Logging

All external signal access is logged:

```
INFO external_signal: provider="agentfs" episode_id="..." tool_count=5
INFO signal_merge: internal_weight=0.7 external_weight=0.3
```

## Troubleshooting

### Provider Not Available

```bash
# Check if feature is enabled
cargo build --features agentfs

# Verify environment
env | grep AGENTFS

# Check database exists
ls -la $AGENTFS_DB_PATH
```

### Low Confidence Scores

```bash
# Increase sample size requirement
export AGENTFS_MIN_SAMPLES=5

# Check AgentFS stats
sqlite3 $AGENTFS_DB_PATH "SELECT name, COUNT(*) FROM tool_calls GROUP BY name;"
```

### Performance Issues

```bash
# Enable caching (30 second TTL)
export EXTERNAL_SIGNAL_CACHE_TTL=30

# Reduce external weight
export EXTERNAL_SIGNAL_WEIGHT=0.1
```

## Advanced Usage

### Multiple Providers

```rust
let memory = SelfLearningMemory::builder()
    .with_agentfs_provider(agentfs_config)
    .with_github_copilot_provider(github_config)
    .with_ide_telemetry_provider(ide_config)
    .build();
```

### Custom Provider

```rust
pub struct MyAuditProvider {
    endpoint: String,
}

#[async_trait]
impl ExternalSignalProvider for MyAuditProvider {
    fn name(&self) -> &str { "my-audit" }
    
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet> {
        // Fetch from your audit system
        let response = reqwest::get(&self.endpoint).await?;
        let data: MyAuditData = response.json().await?;
        
        // Normalize to ExternalSignalSet
        Ok(ExternalSignalSet {
            provider: "my-audit".to_string(),
            tool_signals: normalize_my_data(data),
            confidence: 0.8,
            timestamp: Utc::now(),
            episode_quality: None,
        })
    }
}
```

### Testing

```rust
use memory_core::external::MockExternalSignalProvider;

let mock_provider = MockExternalSignalProvider::with_signals(vec![
    ExternalSignalSet {
        provider: "test".to_string(),
        tool_signals: vec![ToolSignal {
            tool_name: "web_search".to_string(),
            success_rate: 0.95,
            avg_latency_ms: 150.0,
            sample_count: 100,
        }],
        confidence: 0.9,
        timestamp: Utc::now(),
        episode_quality: None,
    }
]);

let memory = SelfLearningMemory::builder()
    .with_external_signal(Box::new(mock_provider))
    .build();
```

## API Reference

### ExternalSignalProvider Trait

```rust
#[async_trait]
pub trait ExternalSignalProvider: Send + Sync {
    /// Unique provider name
    fn name(&self) -> &str;
    
    /// Fetch signals for an episode
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet>;
    
    /// Get provider health status
    async fn health_check(&self) -> ProviderHealth;
    
    /// Validate configuration
    fn validate_config(&self) -> Result<()>;
}
```

### SignalMerger

```rust
pub struct SignalMerger {
    pub internal_weight: f32,
    pub external_weight: f32,
}

impl SignalMerger {
    pub fn with_weights(internal: f32, external: f32) -> Self;
    
    pub fn merge(
        &self,
        internal: &RewardScore,
        external: &[ExternalSignalSet],
    ) -> MergedReward;
}
```

### SignalRegistry

```rust
pub struct ExternalSignalRegistry {
    pub fn register(&mut self, provider: Box<dyn ExternalSignalProvider>);
    pub async fn aggregate_signals(&self, episode: &Episode) -> Vec<ExternalSignalSet>;
    pub fn get_provider(&self, name: &str) -> Option<&dyn ExternalSignalProvider>;
}
```

## Performance Considerations

### Latency Impact

| Scenario | Latency | Recommendation |
|----------|---------|----------------|
| No external signals | +0ms | Default behavior |
| Local AgentFS DB | +10-50ms | Acceptable for production |
| Remote API | +100-500ms | Use caching, lower weight |
| Multiple providers | +N × base | Parallelize requests |

### Caching

Enable signal caching to reduce latency:

```rust
let provider = AgentFsProvider::new(config)
    .with_caching(Duration::from_secs(30));
```

### Batch Processing

For bulk episode processing, prefetch signals:

```rust
// Prefetch signals for all episodes
let signals = registry
    .batch_get_signals(&episodes)
    .await;
```

## Migration Guide

### From Direct SDK Integration

If you have existing AgentFS SDK calls:

1. Replace direct calls with provider trait
2. Move normalization logic to provider
3. Add to registry instead of direct usage
4. Update reward calculation to use merger

### From Internal-Only Rewards

To add external signals to existing system:

1. Add `agentfs` feature flag
2. Configure environment variables
3. Initialize provider in `SelfLearningMemory`
4. No changes needed to episode logic

## See Also

- [ADR-050: AgentFS Integration](../plans/adr/ADR-050-AgentFS-Integration.md)
- [ADR-051: External Signal Provider](../plans/adr/ADR-051-External-Signal-Provider.md)
- [Skill: external-signal-provider](../.agents/skills/external-signal-provider/SKILL.md)
- [AgentFS SDK Documentation](https://docs.rs/agentfs-sdk/0.6.4)
