# ADR-054: CloudEvents EventEmitter Integration

- **Status**: 🟢 Proposed
- **Date**: 2026-05-01
- **Deciders**: Project maintainers
- **Related**: ADR-022, ADR-050, ADR-051, Issue #521

## Context

The current `SelfLearningMemory` system uses a internal `tokio::sync::broadcast` channel to notify subscribers about lifecycle events via the `MemoryEvent` enum. While effective for internal coordination, this approach has several limitations for external interoperability:

1. **Coupling**: External consumers must depend on `do-memory-core` and the specific `MemoryEvent` type.
2. **Standardization**: There is no standard format for the events, making it difficult to integrate with generic event routers, sinks, or cloud services.
3. **Extensibility**: Adding metadata or context to events requires changing the internal enum.

CloudEvents is a specification for describing event data in common formats to provide interoperability across services, platforms, and systems.

## Decision

We will implement a `CloudEvents` compatible `EventEmitter` to bridge internal memory events to external systems. This integration will:

1. **Standardize**: Adopt the CloudEvents 1.0 specification for all externally emitted events.
2. **Decouple**: Provide a trait-based emission system that allows for various sinks (HTTP, NATS, Kafka, etc.) without coupling the core to specific protocols.
3. **Map**: Create a canonical mapping between internal `MemoryEvent` variants and CloudEvent attributes.

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     SelfLearningMemory                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (MemoryEvent)
┌─────────────────────────────────────────────────────────────────┐
│                    Event Bridge / Dispatcher                     │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐           ┌─────────────────────────────┐   │
│  │ Internal Subs   │           │ CloudEvents EventEmitter    │   │
│  │ (Broadcast)     │           │ (Trait Interface)           │   │
│  └─────────────────┘           └──────────────┬──────────────┘   │
└───────────────────────────────────────────────┼─────────────────┘
                                                │
                                                ▼
                                 ┌─────────────────────────────┐
                                 │       Event Sinks           │
                                 │  (HTTP, Log, NATS, etc.)    │
                                 └─────────────────────────────┘
```

### EventEmitter Trait

```rust
#[async_trait]
pub trait EventEmitter: Send + Sync {
    /// Emit a single CloudEvent to the configured sink.
    async fn emit(&self, event: CloudEvent) -> anyhow::Result<()>;

    /// Emit a batch of CloudEvents.
    async fn emit_batch(&self, events: Vec<CloudEvent>) -> anyhow::Result<()> {
        for event in events {
            self.emit(event).await?;
        }
        Ok(())
    }
}
```

### CloudEvent Mapping

All events will use `source: do-memory://core`.

| MemoryEvent Variant | CloudEvent Type | Subject | Data Schema |
|---------------------|-----------------|---------|-------------|
| `EpisodeCreated` | `com.do-memory.episode.created` | `id` | Episode metadata |
| `EpisodeCompleted` | `com.do-memory.episode.completed` | `id` | Reward score, timestamp |
| `EpisodeGarbageCollected` | `com.do-memory.episode.gc` | `id` | Reason, timestamp |
| `PatternExtracted` | `com.do-memory.pattern.extracted` | `id` | Source episodes |

## Implementation Strategy

1. **Dependency**: Add `cloudevents-sdk` to `do-memory-core`.
2. **Abstraction**: Define the `EventEmitter` trait in `memory-core/src/types/emitter.rs`.
3. **Bridge**: Implement a background task or hook in `SelfLearningMemory` that listens to the `broadcast` channel and forwards events to the `EventEmitter`.
4. **Sinks**:
   - `LogEmitter`: Simple implementation that logs events via `tracing`.
   - `HttpEmitter` (Optional/Future): Implementation that POSTs events to a webhook.
   - `NoOpEmitter`: Default implementation that discards events.

## Consequences

### Positive

1. **Interoperability**: Standardized event format allows integration with the broader CloudEvents ecosystem.
2. **Flexibility**: Users can implement their own `EventEmitter` to route events to any destination.
3. **Observability**: Standardized events make it easier to build dashboards and monitoring tools.
4. **Extensibility**: CloudEvents support custom extensions without breaking existing consumers.

### Negative

1. **Dependency**: Adds `cloudevents-sdk` as a dependency.
2. **Complexity**: Adds a layer of abstraction and conversion between internal events and external formats.
3. **Overhead**: Conversion and async emission add slight latency to lifecycle operations.

## Mitigations

- Use a `NoOpEmitter` by default to avoid overhead for users who don't need external events.
- Perform conversions asynchronously to avoid blocking the main execution path.
- Keep the `EventEmitter` trait simple to encourage custom implementations.

## Future Unlocks

- **Event-Driven Patterns**: Trigger external agent workflows based on episode completion or pattern extraction.
- **Audit Trails**: Feed events into a centralized audit log or compliance system.
- **Real-Time Analytics**: Stream events to a stream processing engine for real-time insights.

## References

- CloudEvents Specification: https://github.com/cloudevents/spec
- Rust CloudEvents SDK: https://github.com/cloudevents/sdk-rust
