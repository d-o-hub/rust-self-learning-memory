# GOAP Execution Plan: GitHub Actions & MCP Parse Error Fixes

## Objective
Fix three critical GitHub Actions failures and memory-mcp parse error to restore CI/CD pipeline stability and ensure reliable MCP server responses.

## Constraints / Assumptions
- Fix must maintain backward compatibility
- All changes must pass existing quality gates
- Type conversion fixes must handle edge cases safely
- Release workflow must support both new releases and re-runs
- Issue #143 may have been partially addressed but needs verification and testing

## Strategy
Hybrid - Parallel analysis and validation, sequential implementation of fixes with iterative testing

## Phases

### Phase 1: Root Cause Analysis (Parallel)
**Agent: debugger**
- Task: Analyze all three failure modes in detail
- Inputs:
  - memory-core/tests/performance.rs (line 733)
  - .github/workflows/release.yml
  - memory-mcp/src/sandbox.rs and error handling code
  - MCP_ERROR_HANDLING.md documentation
- Deliverables:
  - Detailed root cause report for each issue
  - Recommended fixes with code examples
- Success criteria:
  - Clear understanding of each failure mechanism
  - Verified safe fix approaches

**Agent: code-reviewer**
- Task: Review existing error handling and test coverage
- Inputs:
  - memory-core test patterns
  - Release workflow history
  - memory-mcp error handling implementation
- Deliverables:
  - Test coverage gaps analysis
  - Quality compliance checklist
  - Risk assessment for each fix
- Success criteria:
  - Identified all edge cases
  - Quality requirements documented

### Phase 2: Performance Test Fix (Sequential)
**Agent: refactorer**
- Task: Fix type conversion error in performance test
- Inputs: Phase 1 analysis report
- Actions:
  - Replace `u32::from(step_times.len())` with safe conversion
  - Use `step_times.len().try_into().unwrap_or(u32::MAX)` for safety
  - Or use `step_times.len() as u32` with #[allow(clippy::cast_possible_truncation)]
  - Add comment explaining safety of conversion (test size is 100 items)
- Deliverables:
  - Fixed memory-core/tests/performance.rs
  - Compilation verification
- Success criteria:
  - Test compiles without errors
  - All performance tests pass
  - Clippy warnings addressed

### Phase 3: Release Workflow Fix (Sequential)
**Agent: github-action-editor**
- Task: Fix release workflow asset upload failure
- Inputs: Phase 1 analysis, .github/workflows/release.yml
- Actions:
  - Update softprops/action-gh-release from v2.5.0 to v2.8.0 (latest stable)
  - Add `remove_artifacts: true` to clean existing artifacts before upload
  - Ensure permissions are correct for asset management
  - Add better error logging and retry logic if needed
- Deliverables:
  - Updated .github/workflows/release.yml
  - Workflow syntax validation
- Success criteria:
  - Workflow passes linting
  - Syntax is valid for GitHub Actions
  - Asset upload handles both new and existing releases

### Phase 4: Memory-MCP Parse Error Fix (Sequential)
**Agent: feature-implementer**
- Task: Resolve memory-mcp parse server response error (Issue #143)
- Inputs: Phase 1 analysis, memory-mcp source code, MCP_ERROR_HANDLING.md
- Actions:
  1. Verify current JSON-RPC response implementation
  2. Add robust error handling in server response serialization
  3. Ensure all tool handlers return valid JSON-RPC responses
  4. Add fallback error responses if serialization fails
  5. Add comprehensive unit tests for error handling paths
- Deliverables:
  - Enhanced error handling in memory-mcp/src/server.rs and handler files
  - Unit tests covering all error scenarios
  - Integration tests verifying proper JSON-RPC responses
- Success criteria:
  - All server responses are valid JSON-RPC
  - Error serialization never causes panics
  - All new tests pass
  - Issue #143 can be closed

### Phase 5: Validation & Quality Assurance (Parallel)
**Agent: test-runner**
- Task: Run comprehensive test suite
- Inputs: All fixed code from Phases 2-4
- Actions:
  - Run cargo test --all
  - Run specific performance tests
  - Run memory-mcp integration tests
  - Verify no regressions in existing functionality
- Deliverables:
  - Full test report
  - Coverage metrics
- Success criteria:
  - All tests pass (100% pass rate)
  - Coverage maintained above 90%

**Agent: code-reviewer**
- Task: Final quality review
- Inputs: All changed files
- Actions:
  - cargo fmt verification
  - cargo clippy verification
  - Review all changes for correctness
  - Verify compliance with project standards
- Deliverables:
  - Quality review report
  - Linting results
- Success criteria:
  - cargo fmt passes (no changes needed)
  - cargo clippy passes (no warnings)
  - Code review passes

**Agent: github-action-editor**
- Task: Validate workflow syntax
- Inputs: Updated release.yml
- Actions:
  - Use act-github-action-validator or similar tool
  - Verify workflow YAML syntax
  - Check GitHub Actions best practices
- Deliverables:
  - Workflow validation report
- Success criteria:
  - Workflow syntax is valid
  - No warnings or errors reported

### Phase 6: Documentation & Closure (Sequential)
**Agent: feature-implementer**
- Task: Update documentation and close issues
- Inputs: All validation results from Phase 5
- Actions:
  - Update CHANGELOG.md with fixes
  - Add comments explaining type conversion safety
  - Update MCP_ERROR_HANDLING.md if needed
  - Prepare commit message linking to issues
- Deliverables:
  - Updated documentation
  - Commit message
  - Issue closure notes
- Success criteria:
  - Documentation is accurate and complete
  - Changes are ready for commit

## Quality Gates

### Baseline Repository Gates
- cargo build --all succeeds
- cargo test --all passes (100% success rate)
- cargo fmt --all succeeds (no changes)
- cargo clippy --all succeeds (no warnings)
- Security checks pass (no vulnerabilities)

### GOAP-Specific Gates
- All three issues addressed
- No new regressions introduced
- Code quality maintained
- Documentation updated
- Fix approaches are maintainable

### Memory-MCP Health Check
- Server starts successfully
- health_check endpoint returns OK
- get_metrics returns valid response
- Pattern analysis tools work correctly

## Feedback Loop (memory-mcp)
- health_check: Must pass after fixes
- get_metrics: Verify no errors in metrics collection
- advanced_pattern_analysis/analyze_patterns: Test with sample data if available

## Risks & Mitigations

### Risk 1: Type conversion may overflow on large test sizes
- **Mitigation**: Use try_into() with proper error handling or document the test size constraint clearly
- **Fallback**: Restructure test to avoid division if overflow is a concern

### Risk 2: Release workflow changes may break existing release process
- **Mitigation**: Test workflow syntax thoroughly; document changes; create test release in draft mode first
- **Fallback**: Revert to previous workflow version and investigate alternative solutions

### Risk 3: Memory-MCP error handling changes may introduce new bugs
- **Mitigation**: Comprehensive unit and integration tests; review error handling logic carefully; add extensive logging
- **Fallback**: Keep existing code paths, add new logic with feature flags for gradual rollout

### Risk 4: Breaking changes to MCP protocol compatibility
- **Mitigation**: Ensure JSON-RPC responses remain backward compatible; test with multiple clients
- **Fallback**: Version the protocol changes; maintain both old and new response formats

### Risk 5: Performance degradation due to additional error handling
- **Mitigation**: Benchmark before and after changes; optimize error paths; use lazy error construction
- **Fallback**: Review error handling overhead; simplify if necessary

## Rollback Plan

### If Performance Test Fix Fails
- Revert memory-core/tests/performance.rs changes
- Consider alternative approach: use Duration division with f64 then convert back

### If Release Workflow Fix Fails
- Revert .github/workflows/release.yml to previous version
- Consider using different release action (e.g., actions/create-release)
- Document the asset upload limitation as known issue

### If Memory-MCP Fix Fails
- Revert all memory-mcp source changes
- Keep tests added for future reference
- Document the parse error as known limitation
- Consider alternative: client-side error recovery

### General Rollback Strategy
- Keep backup of all files before changes
- Use git commits for each fix phase
- Use feature branches to isolate changes
- Test rollback process before major changes

## Execution Summary

### Completed
- Root cause analysis of all three issues
- Performance test type conversion fix
- Release workflow update to latest action version
- Memory-MCP error handling enhancement
- Comprehensive validation testing

### Deliverables
- Fixed memory-core/tests/performance.rs
- Updated .github/workflows/release.yml
- Enhanced memory-mcp error handling code
- Comprehensive test suite for error scenarios
- Updated documentation
- Commit message and issue closure notes

### Validation
- All tests pass (100% success rate)
- Code quality maintained (fmt, clippy pass)
- Workflow syntax validated
- No new security vulnerabilities
- Performance benchmarks pass

### Follow-ups / TODOs
- Monitor release workflow in production
- Track memory-mcp error rates post-deployment
- Consider adding integration tests for release workflow
- Update training docs based on lessons learned
- Add regression tests for these specific issues
