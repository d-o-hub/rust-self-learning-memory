# GOAP: CloudEvents EventEmitter Implementation

**Date**: 2026-05-01
**Type**: Feature Implementation
**Priority**: P1 - Interoperability Block
**WG**: WG-149
**Reference**: ADR-054, Issue #521

---

## Executive Summary

**Goal**: Implement the `EventEmitter` trait and CloudEvents bridge in `do-memory-core` to enable external interoperability for memory lifecycle events.

**Success Criteria**:
- `EventEmitter` trait defined and implemented for `LogEmitter` and `NoOpEmitter`.
- Canonical mapping from `MemoryEvent` to `CloudEvent` implemented.
- `SelfLearningMemory` can be configured with an `EventEmitter`.
- Events are emitted to the configured sink when lifecycle operations occur.
- 90%+ test coverage for new eventing logic.

---

## Phase 1: ANALYZE

### CloudEvents Mapping Verification

Verify `MemoryEvent` fields in `memory-core/src/types/event.rs` to ensure all necessary data is captured in the CloudEvent payload.

### Sink Protocol Requirements

Decide on the default sink implementation (Log/Tracing) and identify future protocol needs (HTTP/Webhook).

---

## Phase 2: DECOMPOSE

### WG-149.1: Core Abstractions & Mapping
- Add `cloudevents-sdk` to `Cargo.toml`.
- Define `EventEmitter` trait in `memory-core/src/types/emitter.rs`.
- Implement `From<MemoryEvent> for CloudEvent` in `memory-core/src/types/event_mapping.rs`.

### WG-149.2: Sink Implementations
- Implement `LogEmitter` in `memory-core/src/types/sinks/log.rs`.
- Implement `NoOpEmitter` in `memory-core/src/types/sinks/noop.rs`.
- (Optional) Implement `MockEmitter` for testing.

### WG-149.3: SelfLearningMemory Integration
- Add `event_emitter: Arc<dyn EventEmitter>` field to `SelfLearningMemory`.
- Update `MemoryConfig` to include event emitter settings.
- Implement the background bridge that consumes `broadcast` events and calls `emit()`.

### WG-149.4: Validation & Documentation
- Unit tests for mapping and sinks.
- Integration tests for end-to-end event emission.
- Update `API_REFERENCE.md` and `README.md`.

---

## Phase 3: STRATEGIZE

**Pattern**: Sequential Implementation with Quality Gates.

1. **Step 1**: Core abstractions and dependencies.
2. **Step 2**: Sink implementations and unit tests.
3. **Step 3**: Core memory integration.
4. **Step 4**: End-to-end validation.

---

## Phase 4: COORDINATE

| Workgroup | Owner | Skills |
|-----------|-------|--------|
| WG-149.1 | feature-implementer | `feature-implement`, `architecture-validation` |
| WG-149.2 | feature-implementer | `feature-implement`, `test-runner` |
| WG-149.3 | feature-implementer | `feature-implement`, `performance` |
| WG-149.4 | code-quality | `code-quality`, `agents-update` |

---

## Phase 5: EXECUTE

### Step 1: Add Dependencies
```bash
cargo add cloudevents-sdk --package do-memory-core
```

### Step 2: Implement WG-149.1 & WG-149.2
- Define trait and mappings.
- Implement Log/NoOp emitters.
- Verify with unit tests.

### Step 3: Implement WG-149.3
- Wire emitter into `SelfLearningMemory`.
- Ensure async/non-blocking emission.

### Step 4: Final Validation
- Run all integration tests.
- Verify coverage metrics.

---

## Quality Gates

| Milestone | Requirement | Status |
|-----------|-------------|--------|
| Gate 1 | `EventEmitter` trait and mapping defined | PENDING |
| Gate 2 | Log/NoOp emitters passing unit tests | PENDING |
| Gate 3 | Integration tests pass with end-to-end emission | PENDING |
| Gate 4 | Coverage ≥ 90% for new code | PENDING |

---

## Monitoring

### Progress Tracking
- Track in `plans/STATUS/CURRENT.md`.
- Milestone completions logged in `progress/LEARNINGS.md`.

---

## References
- ADR-054: CloudEvents EventEmitter Integration
- Issue #521: [Feature] CloudEvents Integration
