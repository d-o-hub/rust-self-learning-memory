# CI Monitoring Learnings

- Treat empty required-check rollup as a blocker, not as implicit success
- Avoid plans-only follow-up commits before remediation CI checks are attached
- If rollup remains empty, investigate workflow trigger/path conditions and use GH CLI monitoring evidence in `plans/STATUS/VALIDATION_LATEST.md`

## CI Parity (CRITICAL)
Local checks must match CI exactly to prevent "works locally, fails in CI".

| Check | Local Command | CI Workflow |
|-------|---------------|-------------|
| Format | `cargo fmt --all -- --check` | `quick-check.yml` |
| Clippy (lib) | `cargo clippy --lib -- -D warnings ...` | `quick-check.yml` |
| Clippy (tests) | `cargo clippy --tests -- -D warnings ...` | `quick-check.yml` |

Use `./scripts/code-quality.sh clippy` or `./scripts/code-quality.sh check` for CI parity.

## Benchmark Workflow Sync (CRITICAL)

When adding or removing benchmarks in `benches/Cargo.toml`, **always** update `.github/workflows/benchmarks.yml` `bench_configs` to match.

| Action | Files to Update |
|--------|----------------|
| Add benchmark | `benches/Cargo.toml` + `benchmarks.yml` bench_configs |
| Remove benchmark | `benches/Cargo.toml` + `benchmarks.yml` bench_configs |

**Why**: The workflow's `bench_configs` associative array lists benchmarks by name. If a benchmark is deleted from `Cargo.toml` but left in `bench_configs`, `cargo bench --bench <name>` fails silently (stderr suppressed), producing no artifacts → the PR gets a misleading "artifacts not available" comment.

## Docs-Only PRs and CI Workflows

Workflows that run expensive operations (benchmarks, coverage) should use `paths-ignore` to skip for docs-only changes:

```yaml
paths-ignore:
  - 'docs/**'
  - 'plans/**'
  - '**/*.md'
  - '.agents/**'
  - 'agent_docs/**'
```