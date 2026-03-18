# Technical Debt Registry

This document tracks known technical debt, organized by category and priority.

## Architecture

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| ARCH-01 | Dual-backend synchronization complexity | Medium | Maintenance | StorageSynchronizer logic is complex and error-prone. |
| ARCH-02 | MonitoringStorage wrapper unused | Low | Cleanup | SimpleMonitoringStorage used directly; wrapper exists for future. |

## Storage

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| STOR-01 | Upstream libsql memory corruption | High | Stability | Blocks 70+ Turso integration tests. See ADR-027. |
| STOR-02 | Prepared statement cache size limits | Medium | Performance | Fixed size of 100 might be too small for complex workloads. |

## Testing

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| TEST-01 | 113 ignored tests | Medium | Coverage | Mix of slow tests and those blocked by upstream bugs. |
| TEST-02 | Non-deterministic pattern accuracy tests | Low | Flakiness | Sometimes fail in CI due to floating point variance. |

## MCP & Integration

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| MCP-01 | execute_agent_code disabled | Medium | Feature | WASM sandbox issues with Javy/Wasmtime. |
| MCP-02 | Batch tool handlers unimplemented | Low | Completeness | Definitions exist but logic is missing. |
