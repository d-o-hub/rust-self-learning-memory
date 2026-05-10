# Follow-up GitHub Issues: CloudEvents Integration

This document tracks planned improvements and extensions for the CloudEvents system.

## 1. Reliability: Implement Retry Logic for HttpEventEmitter
**Description**: The current `HttpEventEmitter` performs a single `POST` request. If the sink is temporarily down, the event is lost.
**Proposed Solution**: Integrate `reqwest-retry` or a custom backoff loop to retry failed emissions.

## 2. Capability: Add WebSocket Event Sink
**Description**: Support real-time streaming of events to local agents or UIs without HTTP polling.
**Proposed Solution**: Add `WsEventEmitter` to `do-memory-events` gated behind an `events-ws` feature flag.

## 3. Tech Debt: Replace sleep in Tests with Synchronization
**Description**: `memory-core/src/memory/tests/event_tests.rs` uses `tokio::time::sleep(50ms)` to wait for async emission. This is flaky.
**Proposed Solution**: Use a channel or a `Notify` primitive to signal completion of emission in test emitters.

## 4. Quality: Data Schema Validation
**Description**: Standardize the `data` payload of CloudEvents using JSON Schema to ensure consumers can reliably parse events.
**Proposed Solution**: Define schemas in `do-memory-events/schemas/` and add validation logic to `to_cloud_event`.

## 5. Integration: CloudEvents for MCP
**Description**: Forward agent lifecycle events as MCP notifications when `do-memory-mcp` is used.
**Proposed Solution**: Implement an `EventEmitter` that wraps an MCP session and emits notifications.
