# CI Monitoring Learnings

- Treat empty required-check rollup as a blocker, not as implicit success
- Avoid plans-only follow-up commits before remediation CI checks are attached
- If rollup remains empty, investigate workflow trigger/path conditions and use GH CLI monitoring evidence in `plans/STATUS/VALIDATION_LATEST.md`

## Workflow Ownership Map

Each workflow owns a specific responsibility. **Do not duplicate work across
workflows** — pick the owning workflow and add the step there. (issue #596)

| Workflow                          | Owns                                                                             | Required for PR merge? |
|-----------------------------------|----------------------------------------------------------------------------------|------------------------|
| `quick-check.yml`                 | rustfmt, clippy, doctests, YAML frontmatter, ignored-test ceiling                | Yes (required check)   |
| `ci.yml`                          | tests (nextest), mcp build, multi-platform tests, semver-check                   | Gated on quick-check   |
| `coverage.yml`                    | `cargo llvm-cov` + Codecov upload, badge generation                              | Gated on quick-check   |
| `security.yml` + `supply-chain.yml` | `cargo audit`, `cargo deny`, secret scanning, SBOM                             | Gated on quick-check   |
| `benchmarks.yml`                  | Criterion benchmarks (only on perf-critical paths)                               | No (paths-filtered)    |
| `nightly-tests.yml`               | `#[ignore]`d slow integration tests, mutation testing, cross-platform slow tests | No (scheduled)         |

### Anti-patterns

- A `quality-gates` job that re-runs coverage and audit inside `ci.yml`.
  Removed in issue #596; coverage lives only in `coverage.yml`, audit lives
  only in `security.yml`/`supply-chain.yml`.
- Adding `cargo audit` to a new workflow without removing it elsewhere.
- Adding `cargo llvm-cov` outside `coverage.yml`.
- Removing the `free-disk-space` action from a workflow without verifying the
  workflow stays below ~14 GB target footprint (will cause ENOSPC).

## CI Parity (CRITICAL)
Local checks must match CI exactly to prevent "works locally, fails in CI".

| Check | Local Command | CI Workflow |
|-------|---------------|-------------|
| Format | `./scripts/code-quality.sh fmt` | `quick-check.yml` |
| Clippy | `./scripts/code-quality.sh clippy --workspace` | `quick-check.yml` |

Use `./scripts/code-quality.sh` wrappers for CI parity.

## Benchmark Workflow Sync (CRITICAL)

The benchmark workflow (`benchmarks.yml`) **dynamically discovers** bench names from `benches/Cargo.toml` at runtime. No manual sync is needed when adding or removing benchmarks — just update `benches/Cargo.toml` and the workflow picks up changes automatically.

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

## Security Workflow Advisory Handling

**CRITICAL**: Never hardcode `--ignore RUSTSEC-XXXX-XXXX` in CI workflow commands.

| Tool | Config File | CI Command |
|------|-------------|------------|
| cargo-audit | `.cargo/audit.toml` | `cargo audit` (no flags) |
| cargo-deny | `deny.toml` | `cargo-deny check` (no flags) |

Config files are auto-discovered by the tools. They serve as the single source of truth and provide auditable documentation of why each advisory is ignored.