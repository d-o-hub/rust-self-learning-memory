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

## Pre-Flight Validation
1. Check action versions: `gh api repos/<owner>/<action>/releases/latest --jq .tag_name`
2. Validate syntax: `actionlint .github/workflows/*.yml`
3. Use `yaml-validator` skill