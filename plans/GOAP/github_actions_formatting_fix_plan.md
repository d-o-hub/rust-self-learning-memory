# GOAP Execution Plan: Fix GitHub Actions Formatting Issues

**Task ID:** GITHUB-ACTIONS-FIX-001
**Branch:** `fix/security-path-validation`
**Created:** 2025-12-27
**Status:** Ready for execution

---

## Executive Summary

Fix GitHub Actions failures by addressing rustfmt formatting issues in `memory-cli/src/config/validator.rs` while maintaining security validation logic.

**Success Criteria:**
- ✓ All GitHub Actions workflows pass
- ✓ Code formatting meets rustfmt standards
- ✓ No new clippy warnings
- ✓ All tests pass
- ✓ Security validation logic intact

---

## Task Analysis

### Complexity Assessment
**Level:** Simple to Medium
- Primary issue: code formatting
- Straightforward fix with verification
- Single agent can handle
- Quality verification required

### Affected Files
- `memory-cli/src/config/validator.rs` (primary - formatting)
- `memory-cli/src/main.rs` (minor changes)
- Various documentation files in `plans/`

---

## Strategic Planning

### Goal Decomposition
1. **G1:** Ensure code formatting complies with rustfmt
2. **G2:** Verify no new clippy warnings
3. **G3:** Confirm all tests pass
4. **G4:** Run quality gates (optional)

### Dependency Graph
```
G1 (formatting) ────┐
                   ├──> G4 (quality gates)
G2 (clippy) ───────┤
                   │
G3 (tests) ────────┘
```

### Coordination Strategy
**Sequential with Parallel Checks:**
1. Fix formatting (refactorer) - MUST BE FIRST
2. Run parallel: clippy (code-reviewer) + tests (test-runner)
3. Quality gates (code-reviewer) - optional
4. Final validation (GOAP)

---

## Detailed Action Plan

### Action 1: Verify and Fix Code Formatting

**Agent:** refactorer
**Duration:** 1-2 minutes

**Pre-conditions:**
- On `fix/security-path-validation` branch
- Code compiles successfully

**Steps:**
1. Run `cargo fmt --all -- --check` to identify issues
2. If issues found, run `cargo fmt --all` to fix
3. Verify with `cargo fmt --all -- --check`
4. Review git diff for formatting changes only

**Post-conditions:**
- ✓ Formatting complies with rustfmt
- ✓ Only formatting changes in git diff
- ✓ Security validation logic preserved

**Quality Gate:**
```bash
cargo fmt --all -- --check
# Must exit with code 0
```

---

### Action 2: Verify Clippy Compliance

**Agent:** code-reviewer
**Duration:** 3-5 minutes

**Pre-conditions:**
- Formatting is correct (Action 1 completed)

**Steps:**
1. Run `cargo clippy --all-targets --all-features`
2. Review warnings in changed files
3. Fix any new warnings in validator.rs
4. Verify no warnings introduced

**Post-conditions:**
- ✓ Zero new clippy warnings
- ✓ Security validation code follows best practices

**Quality Gate:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
# Must have no warnings in changed files
```

---

### Action 3: Execute Test Suite

**Agent:** test-runner
**Duration:** 5-10 minutes

**Pre-conditions:**
- Formatting correct
- Code compiles

**Steps:**
1. Run `cargo test --all`
2. Run `cargo test -p memory-cli` specifically
3. Verify security validation tests pass
4. Check test coverage >90%

**Post-conditions:**
- ✓ All tests pass
- ✓ Security validation works correctly
- ✓ No regressions

**Quality Gate:**
```bash
cargo test --all
cargo test -p memory-cli
# Must all pass
```

---

### Action 4: Run Quality Gates (Optional)

**Agent:** code-reviewer
**Duration:** 10-15 minutes (if script exists)

**Pre-conditions:**
- Actions 1-3 completed successfully

**Steps:**
1. Check if `./scripts/quality-gates.sh` exists
2. If exists, execute it
3. Review and fix any failures
4. Re-run until passes

**Post-conditions:**
- ✓ All quality gates pass
- ✓ Code meets project standards

**Quality Gate:**
```bash
./scripts/quality-gates.sh
# Optional but recommended
```

---

### Action 5: Final Validation & Commit Prep

**Agent:** GOAP (coordinator)
**Duration:** 2-3 minutes

**Pre-conditions:**
- All quality checks passed

**Steps:**
1. Review all git changes
2. Run final CI simulation
3. Verify security logic integrity
4. Stage changes for commit
5. Prepare commit message

**Post-conditions:**
- ✓ All CI requirements verified locally
- ✓ Changes staged and commit message ready

**Final Quality Gate:**
```bash
cargo fmt --all -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --all
# All must pass
```

---

## Execution Workflow

```
┌────────────────────────────┐
│ Action 1: Fix Formatting   │
│ Agent: refactorer (2 min)  │
└────────────────────────────┘
            │
            ▼
┌────────────────────────────┬────────────────────────────┐
│ Action 2: Clippy Check     │ Action 3: Test Suite       │
│ Agent: code-reviewer (5m)  │ Agent: test-runner (10m)   │
└────────────────────────────┴────────────────────────────┘
            │
            ▼
┌────────────────────────────┐
│ Action 4: Quality Gates    │
│ Agent: code-reviewer (15m) │
│ (Optional)                │
└────────────────────────────┘
            │
            ▼
┌────────────────────────────┐
│ Action 5: Final Validation│
│ Agent: GOAP (3 min)        │
└────────────────────────────┘
```

---

## Quality Checkpoints

### Checkpoint 1: After Action 1
- [ ] `cargo fmt --all -- --check` passes
- [ ] Only formatting changes
- [ ] Security logic unchanged

### Checkpoint 2: After Actions 2,3
- [ ] No new clippy warnings
- [ ] All tests pass
- [ ] Security validation tests pass

### Checkpoint 3: After Action 4 (Optional)
- [ ] Quality gates script passes
- [ ] Coverage >90% maintained

### Checkpoint 4: Final
- [ ] All CI checks pass locally
- [ ] Changes reviewed
- [ ] Ready for commit

---

## Error Handling

**If Formatting Fails:**
1. Apply `cargo fmt --all`
2. Review changes
3. Re-run verification

**If Clippy Fails:**
1. Fix warnings in new code only
2. Re-run clippy check
3. Verify no regressions

**If Tests Fail:**
1. Identify failing tests
2. Diagnose root cause
3. Fix or adjust security logic if needed
4. Re-run test suite

---

## Security Validation Preservation

**Critical Constraint:** Maintain security validation logic

The `validate_file_url_security` function must:
- Check for path traversal attacks (`..` in paths)
- Block sensitive system paths (`/etc/`, `/bin/`, etc.)
- Block sensitive files (`/etc/passwd`, etc.)
- Provide clear error messages

**Verification:**
- Review git diff for security logic changes
- Run specific security tests
- Ensure error messages remain helpful

---

## Commit Preparation

### Commit Message Template
```
fix(cli): resolve GitHub Actions formatting issues

- Apply rustfmt to memory-cli/src/config/validator.rs
- Ensure array formatting meets project standards
- Verify all quality checks pass locally
- Maintain security validation logic integrity

Fixes formatting failures in Quick Check workflow
```

### Pre-Commit Verification
```bash
# Verify all checks
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all

# Review changes
git diff --staged
```

---

## Expected Format Example

**Lines 416-442 in validator.rs should be:**
```rust
// Check for access to sensitive system paths
let sensitive_paths = [
    "/etc/",
    "/bin/",
    "/sbin/",
    "/usr/bin/",
    "/usr/sbin/",
    "/sys/",
    "/proc/",
    "/dev/",
    "/boot/",
    "/root/",
    "/var/log/",
    "/var/run/",
    "/tmp/",
];

for sensitive_path in &sensitive_paths {
    if path.starts_with(sensitive_path) {
        return Err(ValidationError {
            field: "database.turso_url".to_string(),
            message: format!(
                "Storage error: Access to sensitive system path is not allowed: {}",
                path
            ),
            suggestion: Some(
                "Use a path in your home directory or project directory".to_string(),
            ),
            context: Some("Security: Access to system paths is blocked".to_string()),
        });
    }
}
```

---

## Success Metrics

**Must Achieve:**
- 100% goal achievement (all GitHub Actions pass)
- Zero new clippy warnings
- All tests passing
- Security logic preserved

**Expected Performance:**
- Total time: 20-30 minutes
- Agent utilization: Balanced
- Zero errors expected

---

## Best Practices

**DO:**
✓ Verify formatting before other checks
✓ Run parallel quality checks
✓ Validate security logic integrity
✓ Review all changes before committing
✓ Follow commit message conventions

**DON'T:**
✗ Skip quality checkpoints
✗ Modify security logic without explicit need
✗ Commit without verification
✗ Ignore clippy warnings in new code

---

## Risk Assessment

**Low Risk:**
- Formatting breaking functionality (unlikely, easily detected)

**Medium Risk:**
- Clippy requiring logic changes (unlikely, mostly style)

**Mitigation:**
- Comprehensive testing after changes
- Security logic review
- Full CI simulation locally

---

## Execution Checklist

**Pre-Execution:**
- [ ] On correct branch
- [ ] Code compiles
- [ ] Read validator.rs
- [ ] Understood security logic

**Execution:**
- [ ] Formatting fixed
- [ ] Clippy passed
- [ ] Tests passed
- [ ] Quality gates (optional)
- [ ] Final validation

**Post-Execution:**
- [ ] Ready to commit
- [ ] Plan archived

---

## Conclusion

This GOAP plan provides a systematic approach to fixing GitHub Actions formatting issues while maintaining security validation logic. The plan ensures:

- All formatting issues resolved
- Code quality standards maintained
- Security validation preserved
- Comprehensive verification completed

**Estimated Time:** 20-30 minutes
**Success Probability:** High
**Next Step:** Execute Action 1

---

## Appendix: Command Reference

**Formatting:**
```bash
cargo fmt --all -- --check     # Check formatting
cargo fmt --all                 # Auto-fix
```

**Clippy:**
```bash
cargo clippy --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

**Testing:**
```bash
cargo test --all
cargo test -p memory-cli
RUST_LOG=debug cargo test --all
```

**Quality Gates:**
```bash
./scripts/quality-gates.sh
```

---

**Plan Status:** Ready for execution
**File Location:** `plans/GOAP/github_actions_formatting_fix_plan.md`
**Follows:** GOAP methodology and project conventions
