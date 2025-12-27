# Execution Plan: Harden and Fix CI Workflow

## Objective
- Correctness: Replace bc-based coverage comparison with awk-only logic.
- Security: Pin all actions to commit SHAs; least-privilege permissions.
- Performance: Improve cache keying; remove redundant disk cleanup.
- Reliability: Ensure timeouts and defaults across jobs.

## Planned Changes
1) Coverage step (remove bc)
- Use awk-only parsing and comparison for coverage threshold checks.

2) Standardize and pin actions
- actions/checkout@<sha>
- actions/upload-artifact@<sha> (standardize to one major across repo)
- codecov/codecov-action@<sha>
- dtolnay/rust-toolchain@<sha>
- Swatinem/rust-cache@<sha>
- taiki-e/install-action@<sha>

3) Least-privilege permissions
- Top-level permissions: contents: read
- Per-job overrides only if required (none currently require writes).

4) Reliability defaults
- Add timeout-minutes to all jobs.
- defaults.run.shell: bash; use set -euo pipefail in scripts.

5) Performance and caching
- Limit or remove duplicate "Free disk space" steps.
- Add prefix-key to rust-cache where matrix feature varies.

6) Concurrency and hygiene
- concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

7) Artifact handling
- Standardize upload-artifact usage; set if-no-files-found: ignore for optional files.

8) Minor robustness
- Replace grep -P with awk if present; guard OS-specific steps.

## Validation Plan
- Static: actionlint and yamllint.
- Dry-run: act for Linux jobs.
- Targeted: test awk coverage logic with a sample lcov.info.

## Security Considerations
- Strict SHA pinning; minimal permissions; careful token usage for codecov.

## Risks
- Pins require periodic refresh; cache key changes reduce initial hits.

## Rollback
- Revert workflow or selectively revert pinning/coverage changes.

## Implementation Checklist
- [ ] Replace bc with awk
- [ ] Pin all actions
- [ ] Add timeouts & defaults
- [ ] Tune caches & remove redundant cleanup
- [ ] Concurrency block
- [ ] Validate with linters and PR run
