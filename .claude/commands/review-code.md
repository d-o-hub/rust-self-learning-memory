Perform comprehensive code review on recent changes.

Review code for quality, correctness, and adherence to standards.

Usage: /review-code [optional: file or directory]

Review checklist:
1. Formatting and style
2. Architecture and design
3. Correctness and logic
4. Async/await usage
5. Error handling
6. Performance
7. Testing coverage
8. Documentation
9. Security
10. AGENTS.md compliance

Steps:
1. Run `cargo fmt -- --check`
2. Run `cargo clippy --all -- -D warnings`
3. Review code changes
4. Check file sizes (< 500 LOC)
5. Verify tests exist and pass
6. Check documentation
7. Provide structured feedback
8. Report approval or requested changes

Use the code-reviewer agent for this workflow.
