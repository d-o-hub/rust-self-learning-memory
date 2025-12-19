# GOAP Plan: Phase 2C - Javy JavaScript Integration

**Created**: 2025-12-14
**Status**: Planning
**Parent Plan**: goap-complete-implementation-plan.md

## Phase 1: Task Analysis

### Current State Assessment

**✅ Completed**:
- Phase 2A: Wasmtime POC (7/7 tests passing)
- Phase 2B: WASI + Fuel support
- Integration: WasmtimeSandbox in UnifiedSandbox
- Documentation: Comprehensive planning docs
- PR #146: Created and formatted

**⏳ In Progress**:
- CI validation (formatting fix just pushed)

**❌ Incomplete from Original Plan**:
- Phase 2B: JavaScript support via Javy (deferred)
- Full test suite validation (110+ tests)
- Performance benchmarking vs rquickjs baseline

### Primary Goal
**Enable JavaScript/TypeScript code execution in wasmtime sandbox via Javy compilation**

Success means:
- JavaScript code can be executed through wasmtime backend
- Performance comparable or better than rquickjs (with stability improvement)
- Full test coverage for JavaScript execution
- WASI stdout/stderr captured correctly
- Documentation updated with JavaScript examples

### Constraints

**Time**: Normal priority (comprehensive implementation)
- Current PR (#146) is for WASM-only functionality
- Javy can be separate PR for clean review

**Resources**:
- web-search-researcher: Javy v8.0.0 API research
- feature-implementer: Javy compiler integration
- test-runner: JavaScript execution tests
- code-reviewer: Security and quality validation

**Dependencies**:
- Phase 2A wasmtime POC: ✅ Complete
- WASI Preview 1: ✅ Complete
- Fuel timeouts: ✅ Complete
- UnifiedSandbox integration: ✅ Complete

**Technical Risks**:
- Javy compilation overhead (latency)
- WASM binary size
- Javy API compatibility
- JavaScript feature compatibility (what works, what doesn't)

### Complexity Level
**Complex**: Requires research, integration, testing, and performance validation

### Quality Requirements
- **Testing**: JavaScript execution tests, timeout tests, error handling
- **Standards**: AGENTS.md compliance, rustfmt, clippy -D warnings
- **Documentation**: API examples, performance characteristics, limitations
- **Performance**: Benchmark vs rquickjs, document trade-offs
- **Security**: Validate Javy sandbox isolation

## Phase 2: Task Decomposition

### Main Goal
Complete JavaScript support for wasmtime sandbox via Javy compilation

### Sub-Goals

#### 1. Research Javy Integration - Priority: P0
**Success Criteria**:
- Understand Javy v8.0.0 API
- Know compilation workflow
- Identify integration approach
- Document limitations

**Dependencies**: None
**Complexity**: Medium
**Estimated Time**: Research phase

#### 2. Implement Javy Compiler Module - Priority: P0
**Success Criteria**:
- JavaScript→WASM compilation works
- Error handling for invalid JS
- Compilation caching (optional optimization)

**Dependencies**: Research complete
**Complexity**: High
**Estimated Time**: Implementation phase

#### 3. Enhance WASI Stdio Capture - Priority: P1
**Success Criteria**:
- Stdout captured to ExecutionResult.stdout
- Stderr captured to ExecutionResult.stderr
- Console.log visible in output

**Dependencies**: Javy compiler working
**Complexity**: Medium
**Estimated Time**: Enhancement phase

#### 4. Update UnifiedSandbox Integration - Priority: P1
**Success Criteria**:
- JavaScript code routes to Javy compiler
- Error messages clear and helpful
- Fallback to Node.js if needed

**Dependencies**: Javy compiler + WASI capture
**Complexity**: Low
**Estimated Time**: Integration phase

#### 5. Create JavaScript Test Suite - Priority: P0
**Success Criteria**:
- Basic JS execution tests
- Console.log capture tests
- Error handling tests
- Timeout enforcement tests
- Complex JS feature tests

**Dependencies**: Implementation complete
**Complexity**: Medium
**Estimated Time**: Testing phase

#### 6. Performance Benchmarking - Priority: P1
**Success Criteria**:
- Benchmark vs rquickjs baseline
- Document compilation overhead
- Identify optimization opportunities
- Performance characteristics documented

**Dependencies**: Tests passing
**Complexity**: Medium
**Estimated Time**: Benchmarking phase

#### 7. Documentation & Delivery - Priority: P1
**Success Criteria**:
- Updated README with JS examples
- API documentation complete
- Performance characteristics documented
- Migration guide (if needed)
- PR created

**Dependencies**: All implementation and testing complete
**Complexity**: Low
**Estimated Time**: Documentation phase

### Atomic Tasks

#### Research Phase (2C.1)
- **Task 2C.1.1**: Research Javy v8.0.0 API and integration patterns
  - Agent: web-search-researcher
  - Dependencies: None
  - Output: Integration strategy document

- **Task 2C.1.2**: Analyze Javy compilation workflow
  - Agent: web-search-researcher
  - Dependencies: 2C.1.1
  - Output: Compilation workflow diagram

- **Task 2C.1.3**: Identify Javy limitations and gotchas
  - Agent: web-search-researcher
  - Dependencies: 2C.1.1
  - Output: Limitations document

#### Implementation Phase (2C.2)

- **Task 2C.2.1**: Add Javy dependencies to Cargo.toml
  - Agent: feature-implementer
  - Dependencies: 2C.1.1
  - Output: Updated Cargo.toml

- **Task 2C.2.2**: Create javy_compiler.rs module
  - Agent: feature-implementer
  - Dependencies: 2C.2.1
  - Output: javy_compiler.rs with compile_javascript() function

- **Task 2C.2.3**: Implement compilation error handling
  - Agent: feature-implementer
  - Dependencies: 2C.2.2
  - Output: Robust error handling in compiler

- **Task 2C.2.4**: Add compilation caching (optional)
  - Agent: feature-implementer
  - Dependencies: 2C.2.3
  - Output: Cache mechanism for compiled WASM

#### WASI Enhancement Phase (2C.3)

- **Task 2C.3.1**: Implement stdout/stderr buffer capture
  - Agent: feature-implementer
  - Dependencies: 2C.2.2
  - Output: Updated wasmtime_sandbox.rs with buffer capture

- **Task 2C.3.2**: Update ExecutionResult with captured output
  - Agent: feature-implementer
  - Dependencies: 2C.3.1
  - Output: Stdout/stderr in ExecutionResult

#### Integration Phase (2C.4)

- **Task 2C.4.1**: Update UnifiedSandbox to use Javy compiler
  - Agent: feature-implementer
  - Dependencies: 2C.2.3, 2C.3.2
  - Output: UnifiedSandbox routes JS to Javy

- **Task 2C.4.2**: Add backend selection logic
  - Agent: feature-implementer
  - Dependencies: 2C.4.1
  - Output: Smart routing between Node.js and Wasmtime

- **Task 2C.4.3**: Update ignored tests
  - Agent: feature-implementer
  - Dependencies: 2C.4.1
  - Output: UnifiedSandbox tests un-ignored and passing

#### Testing Phase (2C.5)

- **Task 2C.5.1**: Create basic JavaScript execution tests
  - Agent: test-runner
  - Dependencies: 2C.4.1
  - Output: Basic JS tests passing

- **Task 2C.5.2**: Create console.log capture tests
  - Agent: test-runner
  - Dependencies: 2C.3.2
  - Output: Console output tests passing

- **Task 2C.5.3**: Create error handling tests
  - Agent: test-runner
  - Dependencies: 2C.2.3
  - Output: Error handling tests passing

- **Task 2C.5.4**: Create timeout enforcement tests
  - Agent: test-runner
  - Dependencies: 2C.5.1
  - Output: Timeout tests passing

- **Task 2C.5.5**: Create complex JS feature tests
  - Agent: test-runner
  - Dependencies: 2C.5.1
  - Output: Feature compatibility tests passing

- **Task 2C.5.6**: Run full test suite
  - Agent: test-runner
  - Dependencies: All 2C.5.x
  - Output: 110+ tests passing

#### Benchmarking Phase (2C.6)

- **Task 2C.6.1**: Create performance benchmark suite
  - Agent: feature-implementer
  - Dependencies: 2C.5.6
  - Output: Benchmark harness

- **Task 2C.6.2**: Benchmark vs rquickjs baseline
  - Agent: feature-implementer
  - Dependencies: 2C.6.1
  - Output: Performance comparison data

- **Task 2C.6.3**: Profile compilation overhead
  - Agent: feature-implementer
  - Dependencies: 2C.6.2
  - Output: Profiling report

- **Task 2C.6.4**: Identify optimization opportunities
  - Agent: feature-implementer
  - Dependencies: 2C.6.3
  - Output: Optimization recommendations

#### Documentation Phase (2C.7)

- **Task 2C.7.1**: Update README with JavaScript examples
  - Agent: feature-implementer
  - Dependencies: 2C.5.6
  - Output: Updated README.md

- **Task 2C.7.2**: Document performance characteristics
  - Agent: feature-implementer
  - Dependencies: 2C.6.4
  - Output: Performance documentation

- **Task 2C.7.3**: Update API documentation
  - Agent: feature-implementer
  - Dependencies: 2C.4.3
  - Output: Updated inline docs

- **Task 2C.7.4**: Create Phase 2C completion summary
  - Agent: feature-implementer
  - Dependencies: All docs
  - Output: phase2c-javy-complete.md

- **Task 2C.7.5**: Create PR for Javy integration
  - Agent: code-reviewer
  - Dependencies: 2C.7.4
  - Output: PR created

### Dependency Graph

```
Research Phase (Sequential)
2C.1.1 (Javy API) → 2C.1.2 (Workflow) → 2C.1.3 (Limitations)
         ↓
Implementation Phase (Sequential with parallel branches)
2C.2.1 (Deps) → 2C.2.2 (Compiler) → 2C.2.3 (Errors) → 2C.2.4 (Cache)
                      ↓
                2C.3.1 (WASI) → 2C.3.2 (ExecutionResult)
                      ↓
Integration Phase (Sequential)
2C.4.1 (UnifiedSandbox) → 2C.4.2 (Routing) → 2C.4.3 (Tests)
         ↓
Testing Phase (Parallel then Sequential)
[2C.5.1 (Basic) || 2C.5.2 (Console) || 2C.5.3 (Error) || 2C.5.4 (Timeout) || 2C.5.5 (Features)]
         ↓
2C.5.6 (Full Suite)
         ↓
Benchmarking Phase (Sequential)
2C.6.1 (Harness) → 2C.6.2 (Compare) → 2C.6.3 (Profile) → 2C.6.4 (Optimize)
         ↓
Documentation Phase (Parallel then Sequential)
[2C.7.1 (README) || 2C.7.2 (Perf) || 2C.7.3 (API)]
         ↓
2C.7.4 (Summary) → 2C.7.5 (PR)
```

## Phase 3: Strategy Selection

### Chosen Strategy: **Hybrid (Sequential Phases + Parallel Tasks)**

**Rationale**:
- **Sequential phases**: Research → Implementation → Testing → Benchmarking → Documentation
  - Each phase depends on previous completion
  - Clear quality gates between phases
- **Parallel within phases**:
  - Testing: Multiple test suites in parallel
  - Documentation: Multiple docs in parallel
- **Quality gates**: Validate after each major phase

### Execution Pattern

**Phase 1: Research** (Sequential - ~1 session)
- Deep dive into Javy v8.0.0
- Understand compilation workflow
- Document limitations

**Quality Gate 1**:
- ✅ Javy integration approach documented
- ✅ Limitations identified
- ✅ No blockers found

**Phase 2: Implementation** (Sequential - ~2 sessions)
- Implement Javy compiler module
- Enhance WASI capture
- Integrate into UnifiedSandbox

**Quality Gate 2**:
- ✅ JavaScript compiles to WASM
- ✅ Console output captured
- ✅ Integration complete

**Phase 3: Testing** (Hybrid - ~1 session)
- Parallel: Create test suites
- Sequential: Run full suite

**Quality Gate 3**:
- ✅ All JavaScript tests passing
- ✅ No regressions in WASM tests
- ✅ Full test suite passing (110+)

**Phase 4: Benchmarking** (Sequential - ~1 session)
- Create benchmark harness
- Compare to baseline
- Document performance

**Quality Gate 4**:
- ✅ Performance benchmarked
- ✅ Trade-offs documented
- ✅ Acceptable performance

**Phase 5: Documentation** (Parallel - ~1 session)
- Update all documentation
- Create completion summary
- Prepare PR

**Quality Gate 5**:
- ✅ Documentation complete
- ✅ PR ready for review

### Speed vs Complexity Trade-off
- **Estimated Total Time**: ~5-6 sessions
- **Complexity**: High (Javy integration, WASI enhancement, testing)
- **Risk**: Medium (Javy API changes, performance concerns)
- **Benefit**: Complete JavaScript support, feature parity with rquickjs

## Phase 4: Agent Assignment

| Task | Agent Type | Rationale |
|------|------------|-----------|
| 2C.1.x | web-search-researcher | Javy API research |
| 2C.2.x | feature-implementer | Complex feature implementation |
| 2C.3.x | feature-implementer | WASI enhancement |
| 2C.4.x | feature-implementer | Integration work |
| 2C.5.x | test-runner | Test creation and execution |
| 2C.6.x | feature-implementer | Benchmarking |
| 2C.7.x | feature-implementer | Documentation |
| 2C.7.5 | code-reviewer | PR creation |

## Phase 5: Execution Planning

### Pre-Flight Check

**Before starting Phase 2C**:
1. ✅ Verify PR #146 CI passes
2. ✅ Current work committed and pushed
3. ✅ Clean git state
4. ✅ Create new branch for Javy work

### Phase 1: Research Javy Integration

**Objective**: Understand Javy v8.0.0 integration fully

**Tasks**:
- Launch web-search-researcher for Javy API documentation
- Research compilation workflow (JS → WASM)
- Identify Javy limitations and gotchas
- Document integration approach

**Success Criteria**:
- Clear understanding of Javy API
- Compilation workflow documented
- Integration approach defined
- No blockers identified

**Deliverables**:
- plans/javy-research-findings.md

### Phase 2: Implement Javy Compiler

**Objective**: JavaScript → WASM compilation working

**Tasks**:
- Add Javy dependencies
- Create javy_compiler.rs module
- Implement compile_javascript() function
- Add error handling
- Enhance WASI stdio capture

**Success Criteria**:
- Simple JS code compiles to WASM
- Console.log output captured
- Errors handled gracefully

**Deliverables**:
- memory-mcp/src/javy_compiler.rs
- Updated memory-mcp/src/wasmtime_sandbox.rs

### Phase 3: Integration & Testing

**Objective**: Full JavaScript support in UnifiedSandbox

**Tasks**:
- Update UnifiedSandbox to use Javy
- Un-ignore UnifiedSandbox tests
- Create JavaScript test suite
- Run full test suite

**Success Criteria**:
- JavaScript routes to Javy compiler
- All tests passing
- No regressions

**Deliverables**:
- Updated memory-mcp/src/unified_sandbox.rs
- JavaScript test suite

### Phase 4: Benchmarking

**Objective**: Performance validated

**Tasks**:
- Create benchmark harness
- Compare vs rquickjs
- Document trade-offs

**Success Criteria**:
- Performance characteristics known
- Trade-offs documented

**Deliverables**:
- Benchmark results
- Performance documentation

### Phase 5: Documentation & Delivery

**Objective**: PR ready for review

**Tasks**:
- Update README
- Update API docs
- Create completion summary
- Create PR

**Success Criteria**:
- All documentation updated
- PR created with description

**Deliverables**:
- plans/phase2c-javy-complete.md
- PR for Javy integration

## Phase 6: Risk Assessment & Contingencies

### Risk 1: Javy Integration Complexity
**Probability**: Medium
**Impact**: High
**Mitigation**: Thorough research phase, incremental implementation
**Contingency**: If too complex, use CLI approach (shell out to javy binary)

### Risk 2: Poor Performance
**Probability**: Medium
**Impact**: Medium
**Mitigation**: Add compilation caching, profile early
**Contingency**: Document trade-offs, make wasmtime backend opt-in

### Risk 3: Javy Limitations
**Probability**: Low
**Impact**: Medium
**Mitigation**: Research phase identifies limitations
**Contingency**: Document unsupported features, fallback to Node.js

### Risk 4: WASI Capture Issues
**Probability**: Low
**Impact**: Low
**Mitigation**: Use wasmtime examples, test incrementally
**Contingency**: Inherit stdio as fallback, capture later

## Phase 7: Success Metrics

### Completion Criteria
- [ ] JavaScript code executes via Javy→WASM
- [ ] Console.log output captured correctly
- [ ] Error handling robust
- [ ] Timeout enforcement works
- [ ] Full test suite passing (110+)
- [ ] Performance benchmarked and documented
- [ ] All documentation updated
- [ ] PR created and ready for review

### Quality Metrics
- **Test Coverage**: All JavaScript features tested
- **Performance**: Documented comparison vs rquickjs
- **Code Quality**: Passes clippy, rustfmt
- **Documentation**: API examples, limitations documented

## Next Actions

### Immediate Next Step
**Before Starting Phase 2C**: Verify current PR status

```bash
# Check CI status
gh pr checks 146

# If CI passes, create new branch
git checkout -b feat/phase2c-javy-integration

# If CI fails, fix issues first
```

### Phase 1 Kickoff
**Launch web-search-researcher for Javy research**

---

**Status**: Ready to Execute
**First Action**: Verify PR #146 CI, then launch research phase
**Estimated Timeline**: 5-6 sessions to completion
