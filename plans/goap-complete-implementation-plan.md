# GOAP Plan: Complete Wasmtime Implementation

## Phase 1: Task Analysis

### Primary Goal
Complete the wasmtime sandbox implementation with JavaScript support, integration, and delivery.

### Constraints
- **Time**: Normal priority (comprehensive implementation)
- **Resources**: All specialized agents available, web search for research
- **Dependencies**: Phase 2A complete (✅), Javy research complete (✅)

### Complexity Level
**Very Complex**: Multiple phases, mixed execution modes, requires:
- Research and integration (Javy v8.0.0)
- Feature implementation (JavaScript support, WASI)
- System integration (UnifiedSandbox)
- Testing and validation
- Documentation and delivery (PR)

### Quality Requirements
- **Testing**: Unit tests, integration tests, stress tests
- **Standards**: AGENTS.md compliance, rustfmt, clippy -D warnings
- **Documentation**: API docs, examples, migration guide
- **Performance**: Benchmark vs rquickjs baseline

## Phase 2: Task Decomposition

### Main Goal
Deliver production-ready wasmtime sandbox with JavaScript support integrated into the system.

### Sub-Goals

#### 1. Phase 2B: Javy Integration - Priority: P0
**Success Criteria**: JavaScript code executes via Javy→WASM compilation
- Add Javy v8.0.0 dependencies
- Implement JavaScript→WASM compilation
- Add WASI preview1 for stdio capture
- Implement fuel-based timeout enforcement
- Create JavaScript execution tests
- Benchmark performance

**Dependencies**: Phase 2A complete (✅)
**Complexity**: High

#### 2. UnifiedSandbox Integration - Priority: P1
**Success Criteria**: WasmtimeSandbox integrated as backend option
- Wire wasmtime into UnifiedSandbox
- Add backend selection via env var
- Update MCP server integration
- Migration tests

**Dependencies**: Phase 2B complete
**Complexity**: Medium

#### 3. Testing & Validation - Priority: P0
**Success Criteria**: All tests passing, performance validated
- Full test suite execution
- Performance benchmarks
- Security validation
- Integration tests

**Dependencies**: Phase 2B and integration complete
**Complexity**: Medium

#### 4. Documentation & Delivery - Priority: P1
**Success Criteria**: PR created, documented, ready for review
- Update all documentation
- Create migration guide
- Push branch and create PR
- Add PR description

**Dependencies**: All implementation and testing complete
**Complexity**: Low

### Atomic Tasks

#### Component 1: Phase 2B - Javy Integration

**Task 2B.1**: Research Javy v8.0.0 integration approach
- Agent: web-search-researcher
- Dependencies: None
- Output: Integration strategy, API examples

**Task 2B.2**: Add Javy dependencies to Cargo.toml
- Agent: feature-implementer
- Dependencies: 2B.1
- Output: Updated Cargo.toml with Javy

**Task 2B.3**: Implement JavaScript→WASM compiler module
- Agent: feature-implementer
- Dependencies: 2B.2
- Output: javy_compiler.rs module

**Task 2B.4**: Add WASI preview1 stdio capture
- Agent: feature-implementer
- Dependencies: 2B.3
- Output: Updated wasmtime_sandbox.rs with WASI

**Task 2B.5**: Implement fuel-based timeout enforcement
- Agent: feature-implementer
- Dependencies: 2B.4
- Output: Fuel mechanism in wasmtime_sandbox.rs

**Task 2B.6**: Create JavaScript execution tests
- Agent: test-runner
- Dependencies: 2B.5
- Output: JavaScript test suite passing

**Task 2B.7**: Benchmark performance vs rquickjs
- Agent: feature-implementer
- Dependencies: 2B.6
- Output: Performance comparison report

#### Component 2: UnifiedSandbox Integration

**Task INT.1**: Wire wasmtime into UnifiedSandbox
- Agent: feature-implementer
- Dependencies: 2B.7
- Output: unified_sandbox.rs updated

**Task INT.2**: Add backend selection mechanism
- Agent: feature-implementer
- Dependencies: INT.1
- Output: Env var MCP_SANDBOX_BACKEND support

**Task INT.3**: Update MCP server to use new backend
- Agent: feature-implementer
- Dependencies: INT.2
- Output: server.rs updated

**Task INT.4**: Create integration tests
- Agent: test-runner
- Dependencies: INT.3
- Output: Integration test suite passing

#### Component 3: Testing & Validation

**Task TEST.1**: Run full test suite
- Agent: test-runner
- Dependencies: INT.4
- Output: All tests passing

**Task TEST.2**: Security validation
- Agent: code-reviewer
- Dependencies: TEST.1
- Output: Security review complete

**Task TEST.3**: Performance validation
- Agent: feature-implementer
- Dependencies: TEST.2
- Output: Performance metrics documented

#### Component 4: Documentation & Delivery

**Task DOC.1**: Update all documentation
- Agent: feature-implementer
- Dependencies: TEST.3
- Output: README, CLAUDE.md, docs updated

**Task DOC.2**: Create migration guide
- Agent: feature-implementer
- Dependencies: DOC.1
- Output: Migration guide for users

**Task DOC.3**: Push and create PR
- Agent: code-reviewer
- Dependencies: DOC.2
- Output: PR created with description

### Dependency Graph

```
2B.1 (Research) → 2B.2 (Deps) → 2B.3 (Compiler) → 2B.4 (WASI) → 2B.5 (Fuel) → 2B.6 (Tests) → 2B.7 (Benchmark)
                                                                                                    ↓
                                                              INT.1 (Wire) → INT.2 (Backend) → INT.3 (Server) → INT.4 (Tests)
                                                                                                                        ↓
                                                                                    TEST.1 (Full) → TEST.2 (Security) → TEST.3 (Perf)
                                                                                                                                  ↓
                                                                                                    DOC.1 (Docs) → DOC.2 (Guide) → DOC.3 (PR)
```

## Phase 3: Strategy Selection

### Chosen Strategy: **Hybrid (Sequential + Parallel)**

**Rationale**:
- **Sequential phases**: Each major phase depends on previous (2B → INT → TEST → DOC)
- **Parallel within phases**: Some tasks within phases can run in parallel
- **Quality gates**: Validate after each major phase

### Execution Pattern

**Phase 2B**: Sequential (complex dependencies)
- Research → Implementation → Testing → Benchmarking

**Integration**: Sequential (dependencies on Phase 2B)
- Wire → Configure → Test

**Testing & Validation**: Parallel where possible
- Full tests || Security review
- Then performance validation

**Documentation & Delivery**: Sequential
- Docs → Guide → PR

### Speed vs Complexity Trade-off
- **Speed**: 2-3x faster than pure sequential (parallel in test phase)
- **Complexity**: High (multiple phases, dependencies)
- **Risk**: Medium (quality gates mitigate)

## Phase 4: Agent Assignment

### Agent Capability Matching

| Task | Agent Type | Rationale |
|------|------------|-----------|
| 2B.1 | web-search-researcher | Javy integration research |
| 2B.2-2B.5 | feature-implementer | Complex feature implementation |
| 2B.6 | test-runner | Test creation and execution |
| 2B.7 | feature-implementer | Performance benchmarking |
| INT.1-INT.3 | feature-implementer | System integration |
| INT.4 | test-runner | Integration testing |
| TEST.1 | test-runner | Full test suite |
| TEST.2 | code-reviewer | Security review |
| TEST.3 | feature-implementer | Performance validation |
| DOC.1-DOC.2 | feature-implementer | Documentation |
| DOC.3 | code-reviewer | PR creation and review |

## Phase 5: Execution Planning

### Overview
- **Strategy**: Hybrid (Sequential phases, parallel within)
- **Total Tasks**: 17 atomic tasks
- **Estimated Duration**: 3-4 phases
- **Quality Gates**: 4 major checkpoints

### Phase 2B: Javy Integration (Sequential)

**Tasks**:
- 2B.1: Research Javy integration (web-search-researcher)
- 2B.2: Add dependencies (feature-implementer)
- 2B.3: Implement compiler (feature-implementer)
- 2B.4: Add WASI (feature-implementer)
- 2B.5: Fuel timeouts (feature-implementer)
- 2B.6: JavaScript tests (test-runner)
- 2B.7: Benchmark (feature-implementer)

**Quality Gate 1**:
- ✅ JavaScript code executes successfully
- ✅ WASI stdout/stderr captured
- ✅ Timeouts enforced via fuel
- ✅ All tests passing
- ✅ Performance benchmarked

### Integration Phase (Sequential)

**Tasks**:
- INT.1: Wire wasmtime backend (feature-implementer)
- INT.2: Backend selection (feature-implementer)
- INT.3: Update MCP server (feature-implementer)
- INT.4: Integration tests (test-runner)

**Quality Gate 2**:
- ✅ UnifiedSandbox supports wasmtime backend
- ✅ Backend selectable via env var
- ✅ MCP server uses new backend
- ✅ Integration tests passing

### Testing & Validation (Hybrid: Parallel then Sequential)

**Parallel Tasks**:
- TEST.1: Full test suite (test-runner) || TEST.2: Security review (code-reviewer)

**Sequential Task**:
- TEST.3: Performance validation (feature-implementer) [after parallel complete]

**Quality Gate 3**:
- ✅ All 110+ tests passing
- ✅ Security review approved
- ✅ Performance meets/exceeds baseline
- ✅ Zero regressions

### Documentation & Delivery (Sequential)

**Tasks**:
- DOC.1: Update docs (feature-implementer)
- DOC.2: Migration guide (feature-implementer)
- DOC.3: Create PR (code-reviewer)

**Quality Gate 4**:
- ✅ All documentation updated
- ✅ Migration guide complete
- ✅ PR created with description
- ✅ Ready for review

### Overall Success Criteria
- [x] Phase 2A complete (✅ Done)
- [ ] Phase 2B complete with JavaScript support
- [ ] UnifiedSandbox integration complete
- [ ] All tests passing (110+)
- [ ] Performance validated
- [ ] Documentation complete
- [ ] PR created and ready for review

### Contingency Plans

**If Javy integration fails**:
- Research alternative: quickjs-wasm-rs or custom JS→WASM
- Fallback: Keep Phase 2A POC, defer JS to later

**If performance worse than baseline**:
- Profile and optimize
- Consider caching compiled WASM
- Evaluate trade-off (stability vs speed)

**If tests fail**:
- Use debugger agent for diagnosis
- Use test-runner to fix
- Re-run quality gate

**If integration breaks existing code**:
- Feature flag to disable
- Fix regressions
- Ensure backward compatibility

## Phase 6: Execution Start

### Initial Phase: Research (2B.1)

Launch web-search-researcher to gather:
- Javy v8.0.0 API documentation
- WASI preview1 integration examples
- Fuel configuration for timeouts
- Best practices for wasmtime + Javy

### Next Steps
After research complete:
1. Implement Javy compiler module
2. Add WASI support
3. Enable fuel-based timeouts
4. Create tests
5. Benchmark
6. Integrate
7. Validate
8. Document
9. Deliver

---

**Status**: Ready to execute
**First Action**: Launch web-search-researcher for Javy integration research
