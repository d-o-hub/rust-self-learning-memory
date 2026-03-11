# Phase 1 Integration Plan: PREMem Pre-Storage Reasoning

**Document Version**: 1.0
**Created**: 2025-12-25
**Purpose**: Detailed integration plan for QualityAssessor and SalientExtractor into SelfLearningMemory
**Related**: RESEARCH_INTEGRATION_EXECUTION_PLAN.md Phase 1, Days 6-7

---

## Executive Summary

This document provides atomic task decomposition for integrating pre-storage reasoning (PREMem) into the SelfLearningMemory workflow.

**Status**:
- ✅ QualityAssessor implemented (memory-core/src/pre_storage/quality.rs)
- ✅ SalientExtractor implemented (memory-core/src/pre_storage/extractor.rs)
- 🔄 Integration into SelfLearningMemory (this plan)

**Goal**: Enable quality-based episode filtering and salient feature storage before persisting to storage backends.

---

## Requirements Analysis

### Functional Requirements
1. **FR1**: Assess episode quality before storage using QualityAssessor
2. **FR2**: Reject episodes below quality threshold (default: 0.7)
3. **FR3**: Extract and store salient features for accepted episodes
4. **FR4**: Log rejection reasons for low-quality episodes
5. **FR5**: Maintain backward compatibility with existing episode storage

### Non-Functional Requirements
1. **NFR1**: Performance overhead ≤ 10ms per episode
2. **NFR2**: Zero clippy warnings
3. **NFR3**: 90%+ test coverage for new code
4. **NFR4**: Clear configuration options for quality thresholds
5. **NFR5**: Comprehensive documentation with examples

### Success Criteria
- [ ] Low-quality episodes rejected before storage
- [ ] Salient features stored with accepted episodes
- [ ] Rejection reasons logged appropriately
- [ ] All integration tests passing (5+)
- [ ] Performance impact ≤ 10ms overhead
- [ ] Zero clippy warnings
- [ ] Documentation complete

---

## Goal Hierarchy

### Main Goal: Integrate Pre-Storage Reasoning into SelfLearningMemory

```
Main Goal: Integrate PREMem into SelfLearningMemory
├─ Sub-goal 1: Modify Episode data structure
│  ├─ Task 1.1: Add salient_features field to Episode
│  └─ Task 1.2: Update Episode serialization/deserialization
├─ Sub-goal 2: Modify SelfLearningMemory struct
│  ├─ Task 2.1: Add quality_assessor field
│  ├─ Task 2.2: Add salient_extractor field
│  └─ Task 2.3: Add configuration for quality threshold
├─ Sub-goal 3: Modify complete_episode() workflow
│  ├─ Task 3.1: Assess episode quality before storage
│  ├─ Task 3.2: Extract salient features for high-quality episodes
│  ├─ Task 3.3: Log rejection reasons for low-quality episodes
│  └─ Task 3.4: Store salient features with episode
├─ Sub-goal 4: Update storage backends
│  ├─ Task 4.1: Verify Turso supports salient_features storage
│  └─ Task 4.2: Verify redb supports salient_features storage
├─ Sub-goal 5: Add configuration options
│  ├─ Task 5.1: Add quality threshold to MemoryConfig
│  └─ Task 5.2: Add feature weights to MemoryConfig (optional)
├─ Sub-goal 6: Create integration tests
│  ├─ Task 6.1: Test high-quality episode acceptance
│  ├─ Task 6.2: Test low-quality episode rejection
│  ├─ Task 6.3: Test salient feature storage
│  ├─ Task 6.4: Test rejection logging
│  └─ Task 6.5: Test performance overhead
└─ Sub-goal 7: Documentation
   ├─ Task 7.1: Update SelfLearningMemory API docs
   ├─ Task 7.2: Add configuration examples
   └─ Task 7.3: Create troubleshooting guide
```

---

## Atomic Task Decomposition

### Component 1: Episode Data Structure Modifications

#### Task 1.1: Add salient_features field to Episode
**Priority**: P0 (Critical)
**Complexity**: Low
**Effort**: Small (30 min)
**Risk**: Low

**Input Requirements**:
- Current Episode struct definition (memory-core/src/episode.rs)
- SalientFeatures struct from pre_storage::extractor

**Implementation Steps**:
1. Locate Episode struct in `memory-core/src/episode.rs`
2. Add field: `pub salient_features: Option<SalientFeatures>`
3. Import SalientFeatures from pre_storage::extractor
4. Update Episode::new() or builder to accept salient_features

**Output Expectations**:
- Episode struct has optional salient_features field
- Field is public and accessible
- Code compiles without errors

**Success Criteria**:
- [ ] salient_features field added to Episode struct
- [ ] Field type is Option<SalientFeatures>
- [ ] Code compiles successfully
- [ ] Zero clippy warnings for this change

**Dependencies**: None

---

#### Task 1.2: Update Episode serialization/deserialization
**Priority**: P0 (Critical)
**Complexity**: Low
**Effort**: Small (15 min)
**Risk**: Low

**Input Requirements**:
- Modified Episode struct with salient_features
- Existing serialization implementation

**Implementation Steps**:
1. Verify SalientFeatures derives Serialize/Deserialize (already done)
2. Test that Episode serialization includes salient_features
3. Test backward compatibility (episodes without salient_features)

**Output Expectations**:
- Episodes serialize with salient_features when present
- Episodes without salient_features deserialize correctly (None)
- No breaking changes to existing episode data

**Success Criteria**:
- [ ] Episodes with salient_features serialize correctly
- [ ] Episodes without salient_features deserialize as None
- [ ] Backward compatible with existing stored episodes
- [ ] All serialization tests passing

**Dependencies**: Task 1.1

---

### Component 2: SelfLearningMemory Struct Modifications

#### Task 2.1: Add quality_assessor field
**Priority**: P0 (Critical)
**Complexity**: Low
**Effort**: Small (20 min)
**Risk**: Low

**Input Requirements**:
- Current SelfLearningMemory struct (memory-core/src/memory/mod.rs)
- QualityAssessor from pre_storage::quality

**Implementation Steps**:
1. Locate SelfLearningMemory struct in `memory-core/src/memory/mod.rs`
2. Add field: `quality_assessor: QualityAssessor`
3. Import QualityAssessor from pre_storage::quality
4. Update constructor/builder to initialize quality_assessor

**Output Expectations**:
- SelfLearningMemory has quality_assessor field
- Field is initialized with default or configured settings
- Code compiles without errors

**Success Criteria**:
- [ ] quality_assessor field added to SelfLearningMemory
- [ ] Field properly initialized in constructor
- [ ] Code compiles successfully
- [ ] Zero clippy warnings

**Dependencies**: None

---

#### Task 2.2: Add salient_extractor field
**Priority**: P0 (Critical)
**Complexity**: Low
**Effort**: Small (20 min)
**Risk**: Low

**Input Requirements**:
- Current SelfLearningMemory struct
- SalientExtractor from pre_storage::extractor

**Implementation Steps**:
1. Add field: `salient_extractor: SalientExtractor`
2. Import SalientExtractor from pre_storage::extractor
3. Update constructor/builder to initialize salient_extractor

**Output Expectations**:
- SelfLearningMemory has salient_extractor field
- Field is initialized with default or configured settings
- Code compiles without errors

**Success Criteria**:
- [ ] salient_extractor field added to SelfLearningMemory
- [ ] Field properly initialized in constructor
- [ ] Code compiles successfully
- [ ] Zero clippy warnings

**Dependencies**: None (parallel with Task 2.1)

---

#### Task 2.3: Add configuration for quality threshold
**Priority**: P1 (Important)
**Complexity**: Low
**Effort**: Small (30 min)
**Risk**: Low

**Input Requirements**:
- Current MemoryConfig (or SelfLearningMemory initialization)
- QualityConfig from pre_storage::quality

**Implementation Steps**:
1. Locate or create MemoryConfig struct
2. Add field: `quality_threshold: f32` (default: 0.7)
3. Optionally add: `quality_weights: HashMap<String, f32>`
4. Update SelfLearningMemory constructor to use config
5. Pass quality_threshold to QualityAssessor

**Output Expectations**:
- Quality threshold is configurable
- Default value is 0.7 (70%)
- SelfLearningMemory uses configured threshold

**Success Criteria**:
- [ ] quality_threshold configurable via MemoryConfig
- [ ] Default value is 0.7
- [ ] QualityAssessor initialized with configured threshold
- [ ] Code compiles successfully

**Dependencies**: Task 2.1

---

### Component 3: complete_episode() Workflow Modification

#### Task 3.1: Assess episode quality before storage
**Priority**: P0 (Critical)
**Complexity**: Medium
**Effort**: Medium (1 hour)
**Risk**: Medium

**Input Requirements**:
- Current complete_episode() implementation
- QualityAssessor integrated into SelfLearningMemory
- Episode to assess

**Implementation Steps**:
1. Locate `complete_episode()` method in memory/mod.rs
2. Before storage, call: `self.quality_assessor.assess_episode(&episode)`
3. Get quality score: `let quality_score = assessment.score;`
4. Check threshold: `if quality_score < self.quality_threshold { ... }`
5. Handle rejection (see Task 3.3)
6. Handle acceptance (see Task 3.2)

**Output Expectations**:
- Episode quality assessed before storage decision
- Quality score calculated correctly
- Threshold comparison works properly

**Success Criteria**:
- [ ] Quality assessment occurs before storage
- [ ] Quality score calculated correctly
- [ ] Threshold comparison logic correct
- [ ] Code compiles successfully
- [ ] Performance overhead ≤ 5ms for assessment

**Dependencies**: Task 2.1, Task 2.3

---

#### Task 3.2: Extract salient features for high-quality episodes
**Priority**: P0 (Critical)
**Complexity**: Medium
**Effort**: Medium (1 hour)
**Risk**: Low

**Input Requirements**:
- Episode that passed quality threshold
- SalientExtractor integrated into SelfLearningMemory

**Implementation Steps**:
1. After quality check passes, call: `self.salient_extractor.extract(&episode)`
2. Get salient features: `let features = extraction_result?;`
3. Update episode: `episode.salient_features = Some(features);`
4. Proceed with storage

**Output Expectations**:
- Salient features extracted for high-quality episodes
- Features attached to episode before storage
- Extraction errors handled gracefully

**Success Criteria**:
- [ ] Salient features extracted for accepted episodes
- [ ] Features attached to episode correctly
- [ ] Extraction errors logged and handled
- [ ] Code compiles successfully
- [ ] Performance overhead ≤ 3ms for extraction

**Dependencies**: Task 2.2, Task 3.1

---

#### Task 3.3: Log rejection reasons for low-quality episodes
**Priority**: P0 (Critical)
**Complexity**: Low
**Effort**: Small (30 min)
**Risk**: Low

**Input Requirements**:
- Episode that failed quality threshold
- Quality assessment results
- Logging infrastructure

**Implementation Steps**:
1. In quality check failure branch, construct rejection message
2. Include: episode ID, quality score, threshold, failing features
3. Log at appropriate level (info or warn)
4. Return error or skip storage (depending on API design)

**Output Expectations**:
- Rejection logged with detailed information
- Log includes actionable information for debugging
- Episode not stored in backends

**Success Criteria**:
- [ ] Rejection logged with episode ID, score, threshold
- [ ] Log level appropriate (info or warn)
- [ ] Log message is clear and actionable
- [ ] Episode not stored when rejected

**Dependencies**: Task 3.1

---

#### Task 3.4: Store salient features with episode
**Priority**: P0 (Critical)
**Complexity**: Low
**Effort**: Small (30 min)
**Risk**: Low

**Input Requirements**:
- Episode with salient_features attached
- Storage backend API

**Implementation Steps**:
1. Verify storage call includes updated episode with salient_features
2. Ensure serialization includes salient_features (verified in Task 1.2)
3. Verify storage backends handle Option<SalientFeatures> correctly

**Output Expectations**:
- Episodes stored with salient features
- Storage backends persist features correctly
- No data loss during serialization/deserialization

**Success Criteria**:
- [ ] Episodes stored with salient_features field
- [ ] Features retrievable from storage
- [ ] No serialization errors
- [ ] Storage backends handle features correctly

**Dependencies**: Task 1.1, Task 1.2, Task 3.2

---

### Component 4: Storage Backend Verification

#### Task 4.1: Verify Turso supports salient_features storage
**Priority**: P1 (Important)
**Complexity**: Low
**Effort**: Small (30 min)
**Risk**: Low

**Input Requirements**:
- Turso storage backend implementation
- Episode with salient_features

**Implementation Steps**:
1. Review Turso schema for episodes table
2. Verify JSON column can store salient_features
3. Test storing episode with salient_features
4. Test retrieving episode with salient_features
5. Verify deserialization works correctly

**Output Expectations**:
- Turso stores and retrieves salient_features correctly
- No schema changes needed (JSON column should handle it)
- No data corruption

**Success Criteria**:
- [ ] Turso stores episodes with salient_features
- [ ] Turso retrieves salient_features correctly
- [ ] Deserialization works properly
- [ ] No schema migration needed

**Dependencies**: Task 1.1, Task 1.2

---

#### Task 4.2: Verify redb supports salient_features storage
**Priority**: P1 (Important)
**Complexity**: Low
**Effort**: Small (30 min)
**Risk**: Low

**Input Requirements**:
- redb storage backend implementation
- Episode with salient_features

**Implementation Steps**:
1. Review redb serialization for episodes
2. Verify postcard serialization handles salient_features
3. Test storing episode with salient_features
4. Test retrieving episode with salient_features
5. Verify deserialization works correctly

**Output Expectations**:
- redb stores and retrieves salient_features correctly
- Postcard serialization handles Option<SalientFeatures>
- No data corruption

**Success Criteria**:
- [ ] redb stores episodes with salient_features
- [ ] redb retrieves salient_features correctly
- [ ] Deserialization works properly
- [ ] No serialization issues with postcard

**Dependencies**: Task 1.1, Task 1.2

---

### Component 5: Configuration Options

#### Task 5.1: Add quality threshold to MemoryConfig
**Priority**: P1 (Important)
**Complexity**: Low
**Effort**: Small (30 min)
**Risk**: Low

**Input Requirements**:
- Current MemoryConfig or configuration system
- Default quality threshold (0.7)

**Implementation Steps**:
1. Locate or create MemoryConfig struct
2. Add field: `pub quality_threshold: f32`
3. Set default: `quality_threshold: 0.7`
4. Update SelfLearningMemory to use config value
5. Update documentation with configuration example

**Output Expectations**:
- Quality threshold configurable
- Default value sensible (0.7)
- Clear documentation

**Success Criteria**:
- [ ] quality_threshold field in MemoryConfig
- [ ] Default value is 0.7
- [ ] SelfLearningMemory uses configured value
- [ ] Documentation includes configuration example

**Dependencies**: None

---

#### Task 5.2: Add feature weights to MemoryConfig (optional)
**Priority**: P2 (Nice-to-have)
**Complexity**: Medium
**Effort**: Medium (1 hour)
**Risk**: Low

**Input Requirements**:
- MemoryConfig struct
- QualityConfig feature weights

**Implementation Steps**:
1. Add field: `pub quality_feature_weights: Option<HashMap<String, f32>>`
2. Update QualityAssessor initialization to use custom weights if provided
3. Document weight configuration format
4. Add validation for weights (sum to 1.0)

**Output Expectations**:
- Feature weights optionally configurable
- Validation ensures weights are valid
- Clear documentation

**Success Criteria**:
- [ ] Feature weights configurable (optional)
- [ ] Weights validated (sum to 1.0)
- [ ] QualityAssessor uses custom weights if provided
- [ ] Documentation includes weight configuration example

**Dependencies**: Task 5.1

---

### Component 6: Integration Tests

#### Task 6.1: Test high-quality episode acceptance
**Priority**: P0 (Critical)
**Complexity**: Medium
**Effort**: Medium (1 hour)
**Risk**: Low

**Input Requirements**:
- SelfLearningMemory with pre-storage reasoning
- High-quality test episode (complex, diverse, reflective)

**Implementation Steps**:
1. Create integration test file (if needed)
2. Create high-quality episode (>0.7 score expected)
3. Call complete_episode()
4. Verify episode stored in backend
5. Verify salient_features attached
6. Verify no rejection logged

**Output Expectations**:
- High-quality episodes accepted
- Salient features stored
- No errors or rejections

**Success Criteria**:
- [ ] Test creates high-quality episode
- [ ] Episode stored successfully
- [ ] Salient features attached and stored
- [ ] Test passes consistently

**Dependencies**: All Component 3 tasks

---

#### Task 6.2: Test low-quality episode rejection
**Priority**: P0 (Critical)
**Complexity**: Medium
**Effort**: Medium (1 hour)
**Risk**: Low

**Input Requirements**:
- SelfLearningMemory with pre-storage reasoning
- Low-quality test episode (<0.7 score expected)

**Implementation Steps**:
1. Create low-quality episode (simple, errors, no reflection)
2. Call complete_episode()
3. Verify episode NOT stored in backend
4. Verify rejection logged with reason
5. Verify appropriate error/result returned

**Output Expectations**:
- Low-quality episodes rejected
- Rejection logged clearly
- No storage occurred

**Success Criteria**:
- [ ] Test creates low-quality episode
- [ ] Episode rejected (not stored)
- [ ] Rejection logged with details
- [ ] Test passes consistently

**Dependencies**: All Component 3 tasks

---

#### Task 6.3: Test salient feature storage
**Priority**: P0 (Critical)
**Complexity**: Medium
**Effort**: Medium (1 hour)
**Risk**: Low

**Input Requirements**:
- SelfLearningMemory with pre-storage reasoning
- Episode with extractable salient features

**Implementation Steps**:
1. Create episode with clear salient features (decisions, tool sequences, etc.)
2. Call complete_episode()
3. Retrieve episode from storage
4. Verify salient_features present
5. Verify features match expected values

**Output Expectations**:
- Salient features stored correctly
- Features retrievable from storage
- Features match expected extraction

**Success Criteria**:
- [ ] Test creates episode with salient features
- [ ] Features extracted correctly
- [ ] Features stored in backend
- [ ] Features retrievable and correct
- [ ] Test passes consistently

**Dependencies**: All Component 3 and 4 tasks

---

#### Task 6.4: Test rejection logging
**Priority**: P0 (Critical)
**Complexity**: Low
**Effort**: Small (45 min)
**Risk**: Low

**Input Requirements**:
- SelfLearningMemory with pre-storage reasoning
- Test logging infrastructure

**Implementation Steps**:
1. Create low-quality episode
2. Call complete_episode()
3. Capture log output
4. Verify log includes: episode ID, quality score, threshold
5. Verify log level appropriate

**Output Expectations**:
- Rejection logged with complete information
- Log message clear and actionable
- Log level appropriate

**Success Criteria**:
- [ ] Rejection logged on episode rejection
- [ ] Log includes episode ID, score, threshold
- [ ] Log message is clear
- [ ] Test passes consistently

**Dependencies**: Task 3.3

---

#### Task 6.5: Test performance overhead
**Priority**: P1 (Important)
**Complexity**: Medium
**Effort**: Medium (1.5 hours)
**Risk**: Medium

**Input Requirements**:
- SelfLearningMemory with and without pre-storage reasoning
- Benchmark harness

**Implementation Steps**:
1. Create baseline: measure complete_episode() without pre-storage
2. Create treatment: measure complete_episode() with pre-storage
3. Run multiple iterations (100+) for statistical significance
4. Calculate overhead: treatment - baseline
5. Verify overhead ≤ 10ms

**Output Expectations**:
- Overhead measured accurately
- Overhead within acceptable range (≤10ms)
- No performance regression

**Success Criteria**:
- [ ] Baseline performance measured
- [ ] Treatment performance measured
- [ ] Overhead calculated (treatment - baseline)
- [ ] Overhead ≤ 10ms per episode
- [ ] Test passes consistently

**Dependencies**: All Component 3 tasks

---

### Component 7: Documentation

#### Task 7.1: Update SelfLearningMemory API docs
**Priority**: P1 (Important)
**Complexity**: Low
**Effort**: Medium (1 hour)
**Risk**: Low

**Input Requirements**:
- Modified SelfLearningMemory implementation
- Existing API documentation

**Implementation Steps**:
1. Update struct documentation to mention pre-storage reasoning
2. Document quality_assessor field
3. Document salient_extractor field
4. Update complete_episode() documentation:
   - Mention quality assessment
   - Mention salient feature extraction
   - Document rejection behavior
5. Add examples of usage

**Output Expectations**:
- API documentation complete and accurate
- Examples show pre-storage reasoning usage
- Clear explanation of rejection behavior

**Success Criteria**:
- [ ] Struct documentation updated
- [ ] Field documentation complete
- [ ] Method documentation updated
- [ ] Examples included
- [ ] Documentation builds without warnings

**Dependencies**: All Component 2 and 3 tasks

---

#### Task 7.2: Add configuration examples
**Priority**: P1 (Important)
**Complexity**: Low
**Effort**: Small (30 min)
**Risk**: Low

**Input Requirements**:
- MemoryConfig with quality_threshold
- Configuration documentation

**Implementation Steps**:
1. Create example: default configuration
2. Create example: custom quality threshold
3. Create example: custom feature weights (if implemented)
4. Add to user guide or configuration documentation

**Output Expectations**:
- Clear configuration examples
- Examples show common use cases
- Examples are tested and correct

**Success Criteria**:
- [ ] Default configuration example
- [ ] Custom threshold example
- [ ] Examples tested and correct
- [ ] Examples added to documentation

**Dependencies**: Task 5.1, Task 5.2 (optional)

---

#### Task 7.3: Create troubleshooting guide
**Priority**: P2 (Nice-to-have)
**Complexity**: Low
**Effort**: Medium (45 min)
**Risk**: Low

**Input Requirements**:
- Common issues and solutions
- User feedback

**Implementation Steps**:
1. Document common issue: "Episodes being rejected unexpectedly"
   - Solution: Check quality threshold, review quality features
2. Document common issue: "Salient features not being stored"
   - Solution: Verify episode quality above threshold
3. Document common issue: "Performance degradation"
   - Solution: Check quality assessment overhead, optimize if needed
4. Add to troubleshooting documentation

**Output Expectations**:
- Clear troubleshooting guide
- Solutions actionable
- Covers common issues

**Success Criteria**:
- [ ] Common issues documented
- [ ] Solutions provided
- [ ] Guide added to documentation
- [ ] Guide reviewed for clarity

**Dependencies**: All integration complete

---

## Dependency Graph

```
Episode Modifications (Component 1):
Task 1.1 (Add salient_features field) ──> Task 1.2 (Update serialization)
                │
                ├──> Task 4.1 (Verify Turso)
                └──> Task 4.2 (Verify redb)

SelfLearningMemory Modifications (Component 2):
Task 2.1 (Add quality_assessor) ──┬──> Task 2.3 (Add config)
Task 2.2 (Add salient_extractor) ─┘

Workflow Modifications (Component 3):
Task 2.1, Task 2.3 ──> Task 3.1 (Assess quality)
                       Task 3.1 ──┬──> Task 3.2 (Extract features)
                                  └──> Task 3.3 (Log rejection)
Task 3.2, Task 1.1 ──> Task 3.4 (Store features)

Configuration (Component 5):
Task 5.1 (Add threshold) ──> Task 5.2 (Add weights - optional)

Integration Tests (Component 6):
All Component 3 ──> Task 6.1, 6.2, 6.3, 6.4, 6.5

Documentation (Component 7):
All Components ──> Task 7.1, 7.2, 7.3
```

---

## Execution Strategy

### Recommended Approach: Sequential with Parallel Testing

**Phase 1**: Data Structure Modifications (Sequential)
- Task 1.1 → Task 1.2
- Duration: ~45 minutes

**Phase 2**: SelfLearningMemory Modifications (Parallel)
- Task 2.1, Task 2.2 (parallel)
- Task 2.3 (after 2.1)
- Duration: ~1 hour

**Phase 3**: Workflow Integration (Sequential)
- Task 3.1 → Task 3.2, Task 3.3 (parallel) → Task 3.4
- Duration: ~2.5 hours

**Phase 4**: Storage Verification (Parallel)
- Task 4.1, Task 4.2 (parallel)
- Duration: ~30 minutes

**Phase 5**: Configuration (Sequential)
- Task 5.1 → Task 5.2 (optional)
- Duration: ~1.5 hours

**Phase 6**: Integration Testing (Parallel for independent tests)
- Task 6.1, 6.2, 6.3, 6.4 (parallel)
- Task 6.5 (after others, requires baseline)
- Duration: ~4 hours

**Phase 7**: Documentation (Sequential)
- Task 7.1 → Task 7.2 → Task 7.3
- Duration: ~2 hours

**Total Estimated Time**: 10-12 hours

---

## Quality Gates

### After Phase 1 (Data Structure)
- [ ] Episode struct compiles with salient_features field
- [ ] Serialization/deserialization tests passing
- [ ] No clippy warnings

### After Phase 2 (SelfLearningMemory)
- [ ] SelfLearningMemory compiles with new fields
- [ ] Constructor initializes fields correctly
- [ ] No clippy warnings

### After Phase 3 (Workflow)
- [ ] complete_episode() implements quality check
- [ ] Salient features extracted and stored
- [ ] Rejection logging works
- [ ] Code compiles without errors
- [ ] No clippy warnings

### After Phase 4 (Storage)
- [ ] Both storage backends handle salient_features
- [ ] No data corruption
- [ ] Serialization/deserialization working

### After Phase 5 (Configuration)
- [ ] Quality threshold configurable
- [ ] Default value appropriate
- [ ] Configuration validated

### After Phase 6 (Testing)
- [ ] All 5 integration tests passing
- [ ] Performance overhead ≤ 10ms
- [ ] 90%+ test coverage

### After Phase 7 (Documentation)
- [ ] API documentation complete
- [ ] Configuration examples provided
- [ ] Troubleshooting guide available
- [ ] Documentation builds without warnings

---

## Risk Mitigation

### Risk 1: Performance overhead exceeds 10ms
**Likelihood**: Medium
**Impact**: High
**Mitigation**:
- Profile quality assessment and salient extraction
- Optimize hot paths if needed
- Consider async processing if overhead too high
- Use feature flags to disable if needed

### Risk 2: Storage backends don't support salient_features
**Likelihood**: Low
**Impact**: High
**Mitigation**:
- Verify early (Phase 4)
- JSON columns (Turso) should handle it
- Postcard serialization (redb) should handle it
- Fallback: store features separately if needed

### Risk 3: Breaking changes to existing episodes
**Likelihood**: Low
**Impact**: High
**Mitigation**:
- Use Option<SalientFeatures> for backward compatibility
- Test deserialization of old episodes
- Ensure None handling works correctly

### Risk 4: Quality threshold too restrictive
**Likelihood**: Medium
**Impact**: Medium
**Mitigation**:
- Start with reasonable default (0.7)
- Make threshold configurable
- Document threshold tuning
- Provide quality score metrics for tuning

---

## Implementation Checklist

### Pre-Implementation
- [x] QualityAssessor module implemented
- [x] SalientExtractor module implemented
- [x] Integration plan created
- [ ] Team review of plan

### Component 1: Episode Data Structure
- [ ] Task 1.1: Add salient_features field
- [ ] Task 1.2: Update serialization

### Component 2: SelfLearningMemory Struct
- [ ] Task 2.1: Add quality_assessor field
- [ ] Task 2.2: Add salient_extractor field
- [ ] Task 2.3: Add quality threshold config

### Component 3: Workflow Integration
- [ ] Task 3.1: Assess episode quality
- [ ] Task 3.2: Extract salient features
- [ ] Task 3.3: Log rejections
- [ ] Task 3.4: Store salient features

### Component 4: Storage Verification
- [ ] Task 4.1: Verify Turso support
- [ ] Task 4.2: Verify redb support

### Component 5: Configuration
- [ ] Task 5.1: Add quality threshold config
- [ ] Task 5.2: Add feature weights config (optional)

### Component 6: Integration Tests
- [ ] Task 6.1: Test high-quality acceptance
- [ ] Task 6.2: Test low-quality rejection
- [ ] Task 6.3: Test salient feature storage
- [ ] Task 6.4: Test rejection logging
- [ ] Task 6.5: Test performance overhead

### Component 7: Documentation
- [ ] Task 7.1: Update API docs
- [ ] Task 7.2: Add configuration examples
- [ ] Task 7.3: Create troubleshooting guide

### Post-Implementation
- [ ] All quality gates passed
- [ ] Code review completed
- [ ] Integration tests passing
- [ ] Documentation reviewed
- [ ] Ready for Phase 1 completion report

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Episode Rejection Rate** | 20-30% | Episodes rejected / total episodes |
| **Quality Score Distribution** | Normal distribution around 0.7 | Histogram of quality scores |
| **Performance Overhead** | ≤ 10ms | Benchmark: treatment - baseline |
| **Test Coverage** | ≥ 90% | Code coverage tools |
| **Clippy Warnings** | 0 | cargo clippy --all |
| **Integration Tests** | 5 passing | cargo test --all |

---

## Next Steps

1. **Review this plan** with the team
2. **Begin implementation** with Phase 1 (Data Structure)
3. **Execute sequentially** through phases with quality gates
4. **Monitor progress** against timeline
5. **Adjust as needed** based on discoveries

---

**Document Status**: ✅ READY FOR IMPLEMENTATION
**Total Tasks**: 26 atomic tasks
**Estimated Duration**: 10-12 hours
**Complexity**: Medium
**Risk**: Low-Medium

---

*This integration plan provides atomic task breakdown for Phase 1 (PREMem) Days 6-7 integration work.*
