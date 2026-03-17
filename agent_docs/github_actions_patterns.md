# GitHub Actions Patterns

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
3. Use `yaml-validator` skill