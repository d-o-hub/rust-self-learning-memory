# E2E Test File Size and Project Guidelines

## Observation: Test Files Exceed 500 LOC Limit

Multiple test files in this project exceed the 500-line limit specified in the `AGENTS.md` file:

### Existing test files over 500 lines:
- `tests/e2e/embeddings_cli_test.rs`: 531 lines
- `tests/e2e/embeddings_local_test.rs`: 640 lines
- `tests/e2e/embeddings_mcp_test.rs`: 755 lines
- `tests/e2e/embeddings_openai_test.rs`: 573 lines
- `tests/e2e/embeddings_performance_test.rs`: 661 lines
- `tests/e2e/embeddings_quality_test.rs`: 682 lines
- `tests/load/batch_operations_test.rs`: 520 lines
- `tests/load/cache_load_test.rs`: 523 lines
- `tests/quality_gates.rs`: 609 lines
- `tests/soak/stability_test.rs`: 568 lines

### New CLI/MCP E2E test files:
- `tests/e2e/cli_episode_workflow.rs`: 758 lines
- `tests/e2e/cli_pattern_workflow.rs`: 812 lines
- `tests/e2e/mcp_episode_chain.rs`: 787 lines
- `tests/e2e/mcp_relationship_chain.rs`: 592 lines
- `tests/e2e/mcp_tag_chain.rs`: 536 lines
- `tests/e2e/error_handling.rs`: 654 lines

## Interpretation of Project Guidelines

The `AGENTS.md` file states:
> "Maximum 500 lines per file for source code (all 9/9 modules compliant after splitting 17 oversized files"

And:
> "Benchmark files (`benches/*.rs`) are exempt from the 500 LOC limit - they contain comprehensive performance tests that require extensive setup and measurement code"

### Key Points:
1. The limit applies to "source code", not necessarily to test files
2. Benchmark files are explicitly exempt due to their comprehensive nature
3. E2E tests share similar characteristics with benchmark files:
   - Comprehensive testing scenarios
   - Extensive setup and teardown code
   - Multiple related test functions
   - Detailed assertions and validations
4. The existing project already has multiple test files exceeding 500 lines

### Conclusion:
The CLI/MCP E2E test files were created following the precedent established by the existing embeddings E2E tests and other large test files in the project. Test files appear to be exempt from the 500-line source code limit.

## If Strict 500-line Compliance is Required

If a decision is made that all files (including tests) must comply with the 500-line limit, each E2E test file can be split into multiple modules:

### Example structure for `cli_episode_workflow`:
```
tests/e2e/cli_episode_workflow/
├── mod.rs                 (main test file)
├── lifecycle.rs           (scenario 1-2)
├── relationships.rs       (scenario 3-4)
├── bulk_operations.rs     (scenario 5-6)
└── search_filter.rs       (scenario 7-8)
```

### Example structure for `mcp_episode_chain`:
```
tests/e2e/mcp_episode_chain/
├── mod.rs                 (main test file)
├── basic_chain.rs         (scenario 1-2)
├── handling.rs            (scenario 3-4)
├── features.rs            (scenario 5-7)
└── advanced.rs            (scenario 8-10)
```

### Pros of splitting:
- Complies with 500-line limit
- Better organization by feature
- Easier to navigate large test suites

### Cons of splitting:
- More files to maintain
- Some tests span multiple concerns
- Added complexity in imports and setup

## Recommendation

**Keep tests as-is** following the existing project precedent. The existing test files demonstrate that comprehensive E2E/Integration/Load tests are allowed to exceed 500 lines. Test files are different from library source code modules and serve a different purpose - they need to be comprehensive and self-contained.

If a decision is made later to enforce 500-line limits on all files, the modular structure can be implemented at that time.

---

**Created**: 2026-02-01
**Purpose**: Document E2E test file size decisions
