Run all tests in the project and report results.

Execute the full test suite including unit tests, integration tests, and doc tests. Fix any failures found.

Steps:
1. Run `cargo test --all --verbose`
2. If tests fail, identify failing tests
3. Run each failing test individually with `--nocapture` for details
4. Apply fixes using test-fix skill
5. Verify all tests pass
6. Report results
