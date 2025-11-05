Diagnose and fix failing tests.

Systematically identify, diagnose, and fix test failures.

Usage: /fix-tests [optional: specific test name]

Steps:
1. Run tests to identify failures
2. Isolate each failing test
3. Run with --nocapture and RUST_LOG=debug for details
4. Diagnose root cause:
   - Missing .await?
   - Database connection issue?
   - Race condition?
   - Type mismatch?
   - Logic error?
5. Apply appropriate fix
6. Verify fix works
7. Check for regressions
8. Report results

Use the test-runner agent for this workflow.
