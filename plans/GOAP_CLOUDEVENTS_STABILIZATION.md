# GOAP: CloudEvents Stabilization and Expansion

## Goal
Harden the CloudEvents implementation and expand its reach across the workspace.

## Tasks
- [ ] Implement retry logic for `HttpEventEmitter`.
- [ ] Add WebSocket event sink to `do-memory-events`.
- [ ] Integrate CloudEvents forwarding into `do-memory-mcp` tool calls.
- [ ] Replace `tokio::time::sleep` in tests with more reliable event-based synchronization.
- [ ] Add schema validation for event data using JSON Schema.
