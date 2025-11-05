Run all pre-commit checks before committing code.

Comprehensive check to ensure code is ready for commit.

Steps:
1. Run `cargo fmt` to format code
2. Run `cargo clippy --all -- -D warnings` to lint
3. Run `cargo test --all` to run all tests
4. Run `cargo check --all` to verify compilation
5. Fix any issues found
6. Report status and readiness for commit

Exit criteria:
- All code formatted
- No clippy warnings
- All tests passing
- Clean compilation
