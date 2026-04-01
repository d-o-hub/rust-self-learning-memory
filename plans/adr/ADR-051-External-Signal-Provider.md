# ADR-051: External Signal Provider Abstraction

- **Status**: ✅ Accepted
- **Date**: 2026-03-31
- **Deciders**: Project maintainers
- **Related**: ADR-050, ADR-044, ADR-028

## Implementation Status

| Component | Status | Evidence |
|-----------|--------|----------|
| `ExternalSignalProvider` trait | ✅ Complete | `do-memory-core/src/reward/external/provider.rs` |
| `ExternalSignalRegistry` | ✅ Complete | `registry.rs` with multi-provider support |
| `ExternalSignalSet` types | ✅ Complete | `types.rs` with normalized format |
| Mock provider | ✅ Complete | `provider.rs:MockExternalSignalProvider` |
| Tests | ✅ Complete | 29 unit tests, 89% coverage |
| First provider (AgentFS) | ✅ Complete | `agentfs.rs` implementing trait |

---

## Context

The memory system needs to consume signals from external sources beyond AgentFS. Future requirements include:

- GitHub Copilot analytics (completion acceptance rates)
- IDE telemetry (editor usage patterns)
- Custom audit systems (organization-specific tool tracking)
- Third-party agent frameworks (LangChain, AutoGPT telemetry)

Without a generic abstraction, each integration would require custom code scattered across the reward system. A unified provider pattern enables:

1. **Consistent integration model** for all external signals
2. **Pluggable architecture** allowing runtime provider selection
3. **Testability** via mock providers
4. **Feature parity** across different external sources

## Decision

Create a generic `ExternalSignalProvider` trait and supporting infrastructure that normalizes any external audit trail into a common signal format consumable by the reward system.

## Architecture

### Trait Definition

```rust
/// Abstraction for external signal sources
#[async_trait]
pub trait ExternalSignalProvider: Send + Sync {
    /// Unique provider identifier
    fn name(&self) -> &str;
    
    /// Fetch signals for a specific episode
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet>;
    
    /// Get provider health/status
    async fn health_check(&self) -> ProviderHealth;
    
    /// Validate provider configuration
    fn validate_config(&self) -> Result<()>;
}

/// Normalized external signal format
pub struct ExternalSignalSet {
    /// Provider that generated these signals
    pub provider: String,
    /// Tool-specific signals
    pub tool_signals: Vec<ToolSignal>,
    /// Overall episode quality score (0.0-1.0)
    pub episode_quality: Option<f32>,
    /// Signal timestamp
    pub timestamp: DateTime<Utc>,
    /// Confidence in these signals (0.0-1.0)
    pub confidence: f32,
}

/// Per-tool signal data
pub struct ToolSignal {
    /// Tool name (normalized)
    pub tool_name: String,
    /// Success rate from external source (0.0-1.0)
    pub success_rate: f32,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Sample size for statistics
    pub sample_count: usize,
    /// Additional provider-specific metadata
    pub metadata: HashMap<String, Value>,
}
```

### Provider Registry

```rust
/// Registry of available signal providers
pub struct ExternalSignalRegistry {
    providers: HashMap<String, Box<dyn ExternalSignalProvider>>,
    config: ExternalSignalConfig,
}

impl ExternalSignalRegistry {
    /// Register a new provider
    pub fn register(&mut self, provider: Box<dyn ExternalSignalProvider>);
    
    /// Get signals from all registered providers
    pub async fn aggregate_signals(&self, episode: &Episode) -> Vec<ExternalSignalSet>;
    
    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn ExternalSignalProvider>;
}
```

### Provider Implementations

| Provider | Source | Data Type | Feature Flag |
|----------|--------|-----------|--------------|
| `AgentFsProvider` | AgentFS SDK | Toolcall audit | `agentfs` |
| `MockProvider` | Test fixtures | Synthetic data | `do-memory-test-utils` |
| `FileProvider` | Local JSON/CSV | Custom audit | `external-signals` |

## Signal Normalization

External sources have varying schemas. The normalization layer handles:

### 1. Tool Name Mapping

```rust
/// Maps external tool names to internal canonical names
pub struct ToolNameMapper {
    mappings: HashMap<String, String>,
}

impl ToolNameMapper {
    /// "web_search" (AgentFS) → "search_web" (internal)
    pub fn canonicalize(&self, external_name: &str) -> String;
}
```

### 2. Schema Validation

```rust
/// Validates external signal schema before ingestion
pub struct SignalValidator;

impl SignalValidator {
    /// Ensure signal has required fields
    pub fn validate(&self, signal: &ExternalSignalSet) -> Result<()>;
    
    /// Check confidence thresholds
    pub fn check_confidence(&self, signal: &ExternalSignalSet, min: f32) -> bool;
}
```

### 3. Time Window Alignment

```rust
/// Aligns external signal timestamps with episode windows
pub struct TemporalAligner;

impl TemporalAligner {
    /// Find signals within episode time window
    pub fn align(&self, signals: &[ToolSignal], episode: &Episode) -> Vec<ToolSignal>;
}
```

## Integration with Reward System

### Signal Merging Strategy

```rust
pub struct SignalMerger {
    /// Weight for internal calculation (default: 0.7)
    internal_weight: f32,
    /// Weight for external signals (default: 0.3)
    external_weight: f32,
    /// Minimum confidence threshold
    min_confidence: f32,
}

impl SignalMerger {
    /// Merge internal reward with external signals
    pub fn merge(
        &self,
        internal: &RewardScore,
        external: &[ExternalSignalSet],
    ) -> MergedReward;
}
```

### Weighting Options

| Mode | Internal | External | Use Case |
|------|----------|----------|----------|
| `InternalOnly` | 100% | 0% | No external providers configured |
| `Balanced` | 70% | 30% | Default, one external provider |
| `ExternalHeavy` | 50% | 50% | High-confidence external signal |
| `ExternalOnly` | 0% | 100% | Testing/debugging |

## Security Model

### Authentication Patterns

```rust
/// Provider authentication abstraction
pub enum ProviderAuth {
    /// API key (AgentFS, GitHub)
    ApiKey { key: String, header: String },
    /// OAuth token
    OAuth { token: String, refresh: String },
    /// mTLS certificate
    Mtls { cert: PathBuf, key: PathBuf },
    /// No auth (local file provider)
    None,
}
```

### Data Sanitization

All providers must implement:

1. **PII Redaction**: Remove personally identifiable information
2. **Size Limits**: Cap parameter/result sizes
3. **Schema Validation**: Reject malformed signals
4. **Audit Logging**: Log all external data access

## Configuration

### Environment Variables

```bash
# General external signals
EXTERNAL_SIGNALS_ENABLED=true
EXTERNAL_SIGNAL_WEIGHT=0.3
EXTERNAL_SIGNAL_MIN_CONFIDENCE=0.5

# AgentFS provider
AGENTFS_DB_PATH=/path/to/agent.db
AGENTFS_ENABLED=true

# Future providers (placeholder)
GITHUB_COPILOT_TOKEN=
IDE_TELEMETRY_ENDPOINT=
```

### Runtime Configuration

```rust
pub struct ExternalSignalRuntimeConfig {
    /// Globally enable/disable external signals
    pub enabled: bool,
    /// Default weight for all providers
    pub default_weight: f32,
    /// Provider-specific overrides
    pub provider_weights: HashMap<String, f32>,
    /// Circuit breaker config
    pub circuit_breaker: CircuitBreakerConfig,
}
```

## Error Handling

### Provider Failure Modes

| Failure | Behavior | Fallback |
|---------|----------|----------|
| Provider offline | Log warning, skip external signals | Use internal only |
| Auth failure | Log error, disable provider | Use internal only |
| Schema mismatch | Log warning, skip malformed signals | Use valid signals only |
| Timeout | Log warning, partial results | Use available signals |
| Rate limit | Backoff, queue requests | Degraded mode |

### Circuit Breaker Pattern

```rust
/// Prevents cascading failures when provider is unhealthy
pub struct ProviderCircuitBreaker {
    failure_threshold: usize,
    recovery_timeout: Duration,
    state: CircuitState,
}

enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing fast
    HalfOpen,  // Testing recovery
}
```

## Testing Strategy

### Mock Provider

```rust
/// Test fixture provider for unit tests
pub struct MockExternalSignalProvider {
    canned_signals: HashMap<String, ExternalSignalSet>,
}

impl ExternalSignalProvider for MockExternalSignalProvider {
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet> {
        // Return pre-configured signals for testing
        Ok(self.canned_signals.get(&episode.episode_id.to_string())
            .cloned()
            .unwrap_or_default())
    }
}
```

### Test Scenarios

1. **Happy path**: Valid signals merge correctly
2. **Provider failure**: Graceful fallback to internal
3. **Schema mismatch**: Invalid signals rejected
4. **Low confidence**: Signals below threshold ignored
5. **Multiple providers**: Proper weighting and aggregation

## Consequences

### Positive

1. **Unified Integration**: Single pattern for all external signals
2. **Testability**: Mock providers enable comprehensive unit testing
3. **Extensibility**: New providers implement trait, no core changes
4. **Flexibility**: Runtime provider registration and configuration
5. **Observability**: Standardized health checks and metrics

### Negative

1. **Abstraction Overhead**: Additional layer vs direct integration
2. **Complexity**: More moving parts than simple SDK calls
3. **Learning Curve**: Developers must understand trait API
4. **Maintenance**: Abstraction must evolve with provider needs

### Mitigations

- Clear documentation and examples
- Derive macros for simple provider implementations
- Built-in providers demonstrate best practices
- Comprehensive error messages guide implementation

## Future Unlocks

This abstraction enables:

1. **Multi-Provider Aggregation**: Combine signals from multiple sources
2. **Federated Learning**: Aggregate signals across agent instances
3. **Provider Marketplace**: Third-party provider plugins
4. **Signal Composition**: Chain providers (e.g., AgentFS → transformer → reward)
5. **Custom Enterprise Providers**: Organization-specific audit systems

## Migration Path

If direct SDK integration exists:

1. Create provider implementing `ExternalSignalProvider`
2. Migrate SDK calls to provider methods
3. Update configuration to use provider registry
4. Remove direct SDK dependencies from reward module

## Cross-References

- [ADR-050: AgentFS Integration](ADR-050-AgentFS-Integration.md)
- [ADR-044: High-Impact Features v0.1.20](ADR-044-High-Impact-Features-v0.1.20.md)
- [ADR-028: Feature Enhancement Roadmap](ADR-028-Feature-Enhancement-Roadmap.md)

---

**Individual ADR**: `plans/adr/ADR-051-External-Signal-Provider.md`
**Supersedes**: None
**Superseded By**: None
