---
name: external-signal-provider
description: Integrate external signal providers (AgentFS, audit trails, toolcall logs) into the reward system. Use when adding external reward signals, processing toolcall audit trails, or connecting third-party agent telemetry to episode scoring.
---

# External Signal Provider Integration

Guide for integrating external signal providers into the rust-self-learning-memory reward system.

## When to Use This Skill

- Adding a new external signal provider (AgentFS, GitHub Copilot, IDE telemetry)
- Processing toolcall audit trails as reward signals
- Implementing signal normalization from external sources
- Merging external signals with internal reward calculations
- Configuring external signal weights and thresholds

## Related Documents

- **[Signal Ingestion](./signal-ingestion.md)** - Protocol handling and authentication
- **[Reward Integration](./reward-integration.md)** - Merging signals with RewardCalculator
- **[Examples](./examples.md)** - AgentFS-specific implementation patterns
- **[Architecture](../../plans/adr/ADR-051-External-Signal-Provider.md)** - ADR with trait design

## Quick Reference

```rust
// 1. Implement the trait
pub struct AgentFsProvider {
    db_path: String,
}

#[async_trait]
impl ExternalSignalProvider for AgentFsProvider {
    fn name(&self) -> &str { "agentfs" }
    
    async fn get_signals(&self, episode: &Episode) -> Result<ExternalSignalSet> {
        // Query AgentFS SDK
        let tc = ToolCalls::new(&self.db_path).await?;
        let stats = tc.stats().await?;
        
        // Normalize to ExternalSignalSet
        Ok(ExternalSignalSet {
            provider: "agentfs".to_string(),
            tool_signals: normalize_tool_stats(stats, episode),
            ..Default::default()
        })
    }
}

// 2. Register with reward system
let registry = ExternalSignalRegistry::new();
registry.register(Box::new(AgentFsProvider::new(db_path)));

// 3. Use in reward calculation
let merger = SignalMerger::with_weights(0.7, 0.3); // internal, external
let merged = merger.merge(&internal_reward, &external_signals);
```

## Core Principles

1. **Normalization First**: All external signals normalized to `ExternalSignalSet` format
2. **Weighted Merging**: Configurable weights for internal vs external signals (default: 70/30)
3. **Graceful Degradation**: System works without external providers (fallback to internal only)
4. **Privacy by Default**: Parameter sanitization enabled, credentials via env vars only

## Feature Flags

```toml
# Cargo.toml
[features]
agentfs = ["dep:agentfs-sdk"]
external-signals = []  # Base feature for external signal infrastructure
```

## Environment Variables

```bash
# General external signals
EXTERNAL_SIGNALS_ENABLED=true
EXTERNAL_SIGNAL_WEIGHT=0.3
EXTERNAL_SIGNAL_MIN_CONFIDENCE=0.5

# AgentFS provider
AGENTFS_DB_PATH=/path/to/agent.db
AGENTFS_ENABLED=true
```

## Implementation Checklist

- [ ] Implement `ExternalSignalProvider` trait
- [ ] Add feature flag to `do-memory-core/Cargo.toml`
- [ ] Implement signal normalization logic
- [ ] Add configuration struct
- [ ] Implement privacy sanitization (if handling sensitive data)
- [ ] Add unit tests with MockExternalSignalProvider
- [ ] Add integration test (requires AgentFS dev instance)
- [ ] Update AGENTS.md with feature flag
- [ ] Update `agent_docs/external_signals.md` with provider details
- [ ] Update `skill-rules.json` with provider triggers

## See Also

- [Agent Coordination](../agent-coordination/) - For multi-provider coordination
- [Feature Implement](../feature-implement/) - For adding new features
- [Architecture Validation](../architecture-validation/) - For validating provider patterns
