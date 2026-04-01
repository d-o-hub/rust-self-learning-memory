# ADR-050: AgentFS Toolcall Audit Trail Integration

- **Status**: ✅ Accepted
- **Date**: 2026-03-31
- **Deciders**: Project maintainers
- **Related**: ADR-051, ADR-044, ADR-049, ADR-028

## Implementation Status

| Component | Status | Evidence |
|-----------|--------|----------|
| `AgentFsProvider` | ✅ Complete | `do-memory-core/src/reward/external/agentfs.rs` |
| Signal normalization | ✅ Complete | `types.rs:ExternalSignalSet` |
| Signal merging | ✅ Complete | `merger.rs:SignalMerger` |
| Feature flag | ✅ Complete | `agentfs` in `do-memory-core/Cargo.toml` |
| Tests | ✅ Complete | 5 unit tests, 85% coverage |
| Documentation | ✅ Complete | `agent_docs/external_signals.md` (446 lines) |

---

## Context

The rust-self-learning-memory system currently calculates reward scores and pattern effectiveness entirely from internal episode data. While this works, it lacks ground-truth validation from actual tool execution outcomes. AgentFS (agentfs-sdk v0.6.4) provides a comprehensive toolcall audit trail that captures:

- Tool invocation success/failure rates
- Execution latency and duration
- Parameter/result schemas
- Historical tool usage patterns

This external signal source can significantly improve the accuracy of our pattern ranking system by providing real-world validation of which tool sequences actually work in production.

## Decision

Integrate AgentFS as the first implementation of the `ExternalSignalProvider` trait (defined in ADR-051), enabling the memory system to consume toolcall audit trails as reward signals.

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     Episode Lifecycle                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  start_episode() → log_step() → ... → complete_episode()        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Reward Calculation                            │
├─────────────────────────────────────────────────────────────────┤
│  Internal Reward (existing)     External Signal (new)          │
│  ├─ Outcome analysis              ├─ AgentFS stats()             │
│  ├─ Efficiency metrics            ├─ Tool success rates          │
│  ├─ Quality assessment              ├─ Latency patterns          │
│  └─ Learning bonus                  └─ Historical correlation    │
│                              │                                   │
│                              ▼                                   │
│                    ┌─────────────────┐                           │
│                    │ Signal Merger   │                           │
│                    │ ├─ Weight: 70%  │                           │
│                    │ └─ Weight: 30%  │                           │
│                    └─────────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Final Reward Score                              │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Episode Creation**: Episode includes `trace_id` for correlation
2. **Step Logging**: Each step captures tool name for later correlation
3. **Episode Completion**:
   - Calculate internal reward (existing logic)
   - Query AgentFS for toolcall stats matching episode tools
   - Merge signals using weighted combination
   - Store final reward with external signal provenance

### AgentFS Schema Mapping

| AgentFS Column | Internal Type | Purpose |
|----------------|---------------|---------|
| `name` | `String` | Tool identifier matching |
| `status` | `ToolCallStatus` | Success/failure validation |
| `duration_ms` | `i64` | Latency correlation |
| `started_at` | `DateTime<Utc>` | Temporal correlation |
| `parameters` | `Option<Value>` | Parameter pattern extraction |
| `result` | `Option<Value>` | Result validation |

## Implementation

### New Module Structure

```
do-memory-core/src/reward/external/
├── mod.rs              # Public exports
├── provider.rs         # ExternalSignalProvider trait
├── agentfs.rs          # AgentFS SDK integration
├── merger.rs           # Signal combination logic
└── types.rs            # External signal types
```

### Feature Flag

```toml
# do-memory-core/Cargo.toml
[features]
default = []
# ... existing features ...
agentfs = ["dep:agentfs-sdk"]

[dependencies]
# ... existing deps ...
agentfs-sdk = { version = "0.6.4", optional = true }
```

### Configuration

```rust
pub struct ExternalSignalConfig {
    /// AgentFS database path
    pub agentfs_db_path: String,
    /// Weight for external signals (0.0-1.0)
    pub external_weight: f32,
    /// Minimum sample size for correlation
    pub min_correlation_samples: usize,
    /// Enable privacy sanitization
    pub sanitize_parameters: bool,
}
```

### Privacy Considerations

Toolcall audit trails may contain sensitive data. Implementation includes:

1. **Parameter Sanitization**: Optional redaction of parameter values (keep keys)
2. **Result Truncation**: Limit result size stored in memory system
3. **Access Control**: AgentFS credentials never stored, always via env vars
4. **Audit Trail**: Log all external signal access for compliance

## Integration Points

### With Existing Systems

| System | Integration | Impact |
|--------|-------------|--------|
| `RewardCalculator` | Add `ExternalSignalProvider` dependency | Calculates merged score |
| `complete_episode()` | Query external signals before reward calc | +1 async call |
| `PatternEffectiveness` | Incorporate toolcall success rates | External validation |
| `AttributionTracker` | Correlate with AgentFS session data | Cross-system tracking |

### With ADR-049 (Bayesian Ranking)

AgentFS integration directly enables Section C1 (Bayesian/Wilson Pattern Ranking):

```
Before: effectiveness = internal_success_rate (inferred)
After:  effectiveness = 0.7 * internal + 0.3 * agentfs_success_rate (ground truth)
```

This provides the external adoption signal needed for data-driven ranking.

## Consequences

### Positive

1. **Ground Truth Validation**: External signals validate internal effectiveness calculations
2. **Bayesian Ranking Enabled**: Provides required data for ADR-049 Section C1
3. **Cross-System Learning**: Episodes benefit from production tool execution data
4. **Pattern Quality**: Tool sequences ranked by actual production success rates
5. **Extensible Pattern**: First provider establishes pattern for future integrations

### Negative

1. **External Dependency**: Requires running AgentFS instance for full functionality
2. **Latency Increase**: +1 async query per episode completion (~10-50ms)
3. **Configuration Complexity**: Additional credentials and endpoint configuration
4. **Privacy Overhead**: Parameter sanitization adds processing overhead

### Mitigations

- External signals are **optional** - system works without AgentFS (fallback to internal only)
- Configurable weight (0.0 = disabled, 1.0 = external only)
- Graceful degradation on AgentFS outages
- Async processing doesn't block episode storage

## Future Unlocks

This ADR enables:

1. **A/B Testing** (ADR-028 #14): Compare internal vs external ranking strategies
2. **Real-Time Learning** (ADR-028 #12): Stream AgentFS updates continuously
3. **Additional Providers**: GitHub Copilot, IDE telemetry, custom audit systems
4. **Multi-Agent Correlation**: Track tool usage across multiple agent instances

## Cross-References

- [ADR-051: External Signal Provider Abstraction](ADR-051-External-Signal-Provider.md)
- [ADR-044: High-Impact Features v0.1.20](ADR-044-High-Impact-Features-v0.1.20.md)
- [ADR-049: Comprehensive Analysis v0.1.25](ADR-049-Comprehensive-Analysis-v0.1.25.md)
- [ADR-028: Feature Enhancement Roadmap](ADR-028-Feature-Enhancement-Roadmap.md)

## References

- AgentFS SDK v0.6.4: https://docs.rs/agentfs-sdk/0.6.4
- SPEC v0.4: https://github.com/tursodatabase/agentfs/blob/main/SPEC.md

---

**Individual ADR**: `plans/adr/ADR-050-AgentFS-Integration.md`
**Supersedes**: None
**Superseded By**: None
