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