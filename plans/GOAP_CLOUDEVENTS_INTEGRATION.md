# GOAP: CloudEvents Integration

## Goal
Standardize agent lifecycle events using CNCF CloudEvents.

## Current State
- New crate `do-memory-events` integrated.
- `EventEmitter` trait implemented and wired.
- Basic lifecycle events instrumented.
- Standardized to `f64` scores.

## Tasks
- [x] Create `do-memory-events` crate.
- [x] Define `MemoryEvent` variants.
- [x] Implement CloudEvents mapping.
- [x] Wire emitter into `SelfLearningMemory`.
- [x] Instrumented Turso and redb storage.
- [x] Resolve clippy warnings in E2E tests.
- [x] Add comprehensive variant mapping tests.
- [x] Document CloudEvents schema (ADR-054).
