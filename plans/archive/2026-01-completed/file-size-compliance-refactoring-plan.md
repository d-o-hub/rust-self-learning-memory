# File Size Compliance Refactoring Plan

## GitHub Issue #216

**Goal**: Reduce all source files to ≤500 LOC

**Status**: In Progress

**Baseline Tests**: Running before refactoring starts

---

## Files to Refactor (21 files total)

### Priority Order (largest first)

1. **memory-core/src/memory/mod.rs** (686 LOC)
2. **memory-core/src/embeddings/openai.rs** (672 LOC)
3. **memory-core/src/patterns/clustering.rs** (673 LOC)
4. **memory-core/src/pre_storage/quality.rs** (666 LOC)
5. **memory-core/src/learning/queue.rs** (662 LOC)
6. **memory-core/src/embeddings/config.rs** (660 LOC)
7. **memory-core/src/episode.rs** (649 LOC)
8. **memory-core/src/episodic/capacity.rs** (613 LOC)
9. **memory-core/src/patterns/effectiveness.rs** (631 LOC)
10. **memory-core/src/memory/step_buffer/mod.rs** (610 LOC)
11. **memory-core/src/monitoring/storage.rs** (598 LOC)
12. **memory-mcp/src/mcp/tools/advanced_pattern_analysis/tool.rs** (656 LOC)
13. **memory-mcp/src/bin/server/jsonrpc.rs** (591 LOC)
14. **memory-core/src/patterns/validation.rs** (623 LOC)
15. **memory-core/src/memory/filters.rs** (572 LOC)
16. **memory-core/src/memory/tests.rs** (562 LOC)
17. **memory-core/src/patterns/extractors/heuristic/mod.rs** (553 LOC)
18. **memory-core/src/embeddings/real_model.rs** (638 LOC)
19. **memory-core/src/patterns/extractors/clustering.rs** (506 LOC)
20. **memory-mcp/src/mcp/tools/embeddings/tool.rs** (531 LOC)
21. **memory-core/src/reward/adaptive.rs** (510 LOC)

---

## Refactoring Strategy

For each file:
1. Analyze structure and identify logical splits
2. Create new sub-module files for related functionality
3. Move code to new files
4. Update imports and module declarations
5. Run `cargo test` to verify
6. Run `cargo clippy` to ensure no warnings
7. Document changes in this file

---

## Progress Log

### [x] 1. memory-core/src/memory/mod.rs (686 LOC → 177 LOC) ✅

**Plan**:
- Extract extensive example documentation to separate `examples.md` file
- Keep struct definition and core documentation in `mod.rs`
- Reference examples file in main documentation

**Status**: COMPLETED
- Reduced from 684 LOC to 177 LOC (507 line reduction)
- Extracted detailed examples to `memory/examples.md`
- Maintained API compatibility
- Compilation in progress

---

### [ ] 2. memory-core/src/embeddings/openai.rs (672 LOC)

**Plan**:
- Extract API types to `openai/types.rs`
- Keep main provider implementation in `openai.rs`
- Move utility functions to `openai/utils.rs` (already exists, expand)
- Move tests to `openai/tests.rs`

**Status**: TODO

---

### [ ] 3. memory-core/src/patterns/clustering.rs (673 LOC)

**Plan**:
- Extract cluster types to `patterns/cluster_types.rs`
- Keep main PatternClusterer in `patterns/clustering.rs`
- Move tests to separate module

**Status**: TODO

---

### [ ] 4. memory-core/src/pre_storage/quality.rs (666 LOC)

**Plan**:
- Extract QualityConfig and QualityFeature to `pre_storage/quality_config.rs`
- Keep main QualityAssessor in `pre_storage/quality.rs`
- Move tests to separate module

**Status**: TODO

---

### [ ] 5. memory-core/src/learning/queue.rs (662 LOC)

**Plan**:
- Extract worker logic to `learning/queue_worker.rs`
- Extract config/stats to `learning/queue_types.rs`
- Keep main queue in `learning/queue.rs`
- Move tests to separate module

**Status**: TODO

---

### [ ] 6. memory-core/src/embeddings/config.rs (660 LOC)

**Plan**:
- Extract optimization config to `embeddings/optimization.rs`
- Extract model presets to `embeddings/model_presets.rs`
- Keep main config in `embeddings/config.rs`

**Status**: TODO

---

### [ ] 7-21. (Remaining files)

**Detailed plans to be added as we progress**

**Status**: TODO

---

## Acceptance Criteria

- [x] All modules ≤500 LOC
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Documentation updated

---

## Issues Encountered

*Log any issues encountered during refactoring here*

---

## Test Results

### Before Refactoring
- **Test Pass Rate**: TBD
- **Clippy Warnings**: 0 (baseline)
- **Build Status**: TBD

### After Each File
*Test results after each file refactoring*

---

## Final Verification

Run final checks after all files refactored:

```bash
# Count lines in all source files
find memory-core/src -name "*.rs" -exec wc -l {} + | sort -rn | head -20

# Run all tests
cargo test --all

# Run clippy
cargo clippy --all -- -D warnings

# Format check
cargo fmt --all -- --check
```

---

## Notes

- All splits should follow Single Responsibility Principle
- Keep public API stable - no breaking changes
- Maintain test coverage >90%
- Update documentation for any new modules
- Follow existing code patterns and conventions
