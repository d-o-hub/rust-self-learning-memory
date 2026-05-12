# ADR-054: CloudEvents Integration for Agent Lifecycle

## Status
Proposed -> Implemented

## Context
Standardize internal agent lifecycle events (task start/stop, rewards, etc.) using CNCF CloudEvents v1.0.2 to enable better observability and interoperability with external systems.

## Decision
- Adopt CloudEvents v1.0.2 specification.
- Create a new `do-memory-events` crate for mapping internal events to CloudEvents.
- Use a pluggable `EventEmitter` trait in `do-memory-core`.
- Implement `NullEmitter` as zero-cost default.
- Support HTTP emission behind `events-http` feature flag.

## Progress
- Core event types and trait defined.
- New crate `do-memory-events` created and integrated into workspace.
- Instrumentation added to `SelfLearningMemory` lifecycle methods.
- Storage backends (Turso, redb) instrumented to emit `EpisodeStored` events.
- Unit tests and integration mapping tests implemented and passed.
- Standardized all scores to `f64`.

## Follow-up Tasks
1. Extend CloudEvents to MCP transport layer in `memory-mcp`.
2. Add support for more event sinks (Kafka, NATS) in `do-memory-events`.
3. Improve event batching for high-frequency emission scenarios.
