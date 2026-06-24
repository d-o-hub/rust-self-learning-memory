# GitHub Actions Patterns

## Centralized Rust Setup (ADR-032 / 2026 Best Practices)
All Rust workflows MUST use the local composite action `.github/actions/setup-rust` instead of repeating setup logic. This ensures consistency across target directory isolation, toolchain installation, mold linker setup, and caching.

```yaml
- name: Setup Rust
  uses: ./.github/actions/setup-rust
  with:
    job-name: my-job-id          # Required: unique ID for target dir isolation
    components: clippy, rustfmt  # Optional: default "" (dtolnay/rust-toolchain detects from file)
    install-nextest: "true"      # Optional: default "false"
```

## Action Pinning Policy (CRITICAL)
All third-party actions MUST be pinned to immutable SHAs to prevent supply-chain attacks and ensure reproducibility.

- **Rule**: Use `@<SHA>` instead of `@vX`.
- **Exception**: Local actions (e.g., `uses: ./.github/actions/setup-rust`) do not require SHAs.
- **Maintenance**: Use Dependabot to manage SHA updates while preserving major version comments.

Example:
```yaml
- uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
```

## Job Dependency (CRITICAL)
When a job has `needs: [upstream-job]` and upstream is conditionally skipped:
- **Problem**: Downstream jobs skip by default when dependency skips
- **Solution**: Use `always()` in conditional

```yaml
# WRONG: Job skips when check-quick-check skipped (push events)
needs: [check-quick-check]
if: ${{ github.event_name != 'pull_request' || needs.check-quick-check.result == 'success' }}

# CORRECT: Job runs on push even when dependency skipped
needs: [check-quick-check]
if: ${{ always() && (github.event_name != 'pull_request' || needs.check-quick-check.result == 'success') }}
```

**Pattern**: If job A only runs on PR → it's skipped on push → job B needing A skips → use `always()`

## Benchmark/Cargo.toml Sync

**Rule**: `benchmarks.yml` `bench_configs` must mirror `benches/Cargo.toml` `[[bench]]` entries.

Deleting a benchmark from `Cargo.toml` without updating the workflow causes silent failures (stderr is suppressed with `2>/dev/null`), producing no criterion output and triggering the "artifacts not available" fallback comment on PRs.

## Upload Artifact LCA Pitfall (2026-06-05)

`actions/upload-artifact` computes the **least common ancestor (LCA)** of all input paths and stores files relative to that root. When downstream jobs download the artifact by name, files are extracted to `$GITHUB_WORKSPACE` preserving that structure.

**Symptom**: Artifact uploads successfully (5+ MB), but downstream jobs report "Benchmark artifacts not available" because they look for `bench_results.txt` at the workspace root.

**Root cause**: Mixing workspace-relative paths with `${{ runner.temp }}/...` (or any path outside the workspace) makes the LCA `/home/runner/work`, nesting files several directories deep on download. Example:
- Path 1: `bench_results.txt` → resolves to `<workspace>/bench_results.txt`
- Path 2: `${{ runner.temp }}/cargo-target/criterion/` → outside workspace
- LCA = `/home/runner/work`
- Archive structure: `rust-self-learning-memory/rust-self-learning-memory/bench_results.txt` and `_temp/cargo-target/criterion/...`
- After download: files end up at `<workspace>/rust-self-learning-memory/rust-self-learning-memory/bench_results.txt`

**Fix**: Co-locate all upload paths under a single parent directory before archiving. Copy workspace-relative files to `${{ runner.temp }}/cargo-target/` and upload only that directory. Also add `if-no-files-found: error` to catch real silent failures early.

```yaml
- name: Stage benchmark results for upload
  run: |
    set -euo pipefail
    if [ ! -s bench_results.txt ]; then
      echo "❌ bench_results.txt missing or empty before staging"
      exit 1
    fi
    mkdir -p "${{ runner.temp }}/cargo-target"
    cp bench_results.txt "${{ runner.temp }}/cargo-target/bench_results.txt"

- name: Archive benchmark results
  uses: actions/upload-artifact@v4
  with:
    name: benchmark-results-${{ github.sha }}
    path: ${{ runner.temp }}/cargo-target/
    if-no-files-found: error   # Surface silent failures, don't mask them
```

Reference: <https://github.com/actions/upload-artifact#upload-using-multiple-paths-and-exclusions>.

## Bash Subshell Pitfall in Workflows

Avoid `find ... | while read` in workflow scripts — the pipe creates a subshell. Use process substitution instead:

```bash
# WRONG: while loop runs in subshell, variable changes lost
find dir -name "*.json" | while read -r f; do ... done

# CORRECT: process substitution keeps same shell
while IFS= read -r f; do ... done < <(find dir -name "*.json")
```

## Pre-Flight Validation
1. Check action versions: `gh api repos/<owner>/<action>/releases/latest --jq .tag_name`
2. Validate syntax: `actionlint .github/workflows/*.yml`
