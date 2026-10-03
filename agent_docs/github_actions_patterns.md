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

## Quick Check Wait Gate (CRITICAL — 2026-07-18)

Expensive workflows may wait for `Quick PR Check (Format + Clippy)` before running. **Cheap / path-filtered workflows must not.**

| Rule | Detail |
|------|--------|
| Wait job timeout | **`timeout-minutes: 40`** minimum (not 15) |
| Quick Check job timeout | **`timeout-minutes: 25`** |
| `running-workflow-name` | Must match **this** workflow's `name:` field exactly |
| `allowed-conclusions` | Prefer `success,skipped` (do not treat cancelled upstream as pass for expensive gates) |
| `fail-on-no-checks` | `false` (avoid false failure when check not yet registered) |
| Concurrency | `group: ${{ github.workflow }}-${{ github.sha }}` for wait workflows |
| **YAML Lint / similar** | **No wait gate** — run yamllint/actionlint immediately |

```yaml
# WRONG — cancels after 15m while Quick Check still running
timeout-minutes: 15
# uses wait-on-check for a 5-second yamllint job

# CORRECT for expensive jobs
timeout-minutes: 40
wait-interval: 15
fail-on-no-checks: false
running-workflow-name: 'CI'   # exact workflow name

# CORRECT for yaml-lint: no check-quick-check job at all
```

Historical: PR #860 cancelled YAML Lint wait every push; #871 fixed name/concurrency only; permanent fix removed the gate (LESSON-021).

## Benchmark/Cargo.toml Sync

**Rule**: `benchmarks.yml` `bench_configs` must mirror `benches/Cargo.toml` `[[bench]]` entries.

Deleting a benchmark from `Cargo.toml` without updating the workflow causes silent failures (stderr is suppressed with `2>/dev/null`), producing no criterion output and triggering the "artifacts not available" fallback comment on PRs.

## Criterion Estimates: Only `new/` Is Absolute (2026-09-15)

**Rule**: ingest `*/new/estimates.json` only (`find … -path '*/new/estimates.json'`).

Criterion 0.8 writes three files per benchmark: `new/` (absolute ns for the run just
completed), `base/` (a byte-identical copy of `new/` — this is why
`Criterion estimates.json files found: 232` collapsed to 111 entries), and — when a
previous run exists in the same target dir — `change/` with **relative** deltas
(`-0.099`), which the old conversion emitted as `-1 ns/iter`. CI target dirs are fresh
today, so the defect is latent there, but any retry/cache/local run that warms the dir
silently corrupts the stored series. Means are emitted as 2-decimal floats because
`compression_overhead/without_compression` measures ~0.31 ns and flooring it dropped
5 benchmarks from every dataset (CI run for PR #1005: 233 files -> 223 lines -> 112
entries). The conversion step also fails closed on a non-positive mean, on a dataset
smaller than the fresh-file count, and on duplicate benchmark names. See LESSON-025 and `scripts/test-benchmark-workflow.sh`.

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

## CI Optimization (2026-04-28)

PR CI time reduced from ~50+ min to ~15-18 min via paths-based benchmark triggering.

| Job | Time | Trigger |
|-----|------|---------|
| Quick Check | ~7–20 min (cold) | All PRs |
| Tests | ~12 min | All PRs |
| MCP Build | ~10 min | All PRs |
| Multi-Platform | ~12-15 min | All PRs |
| Run Benchmarks | ~54 min | **Only perf-critical paths** |

**Quick Check wait gates (2026-07-18 / LESSON-021)**: Never use
`timeout-minutes: 15` on `Check Quick Check Status`. Wait jobs need **40m**;
Quick Check job **25m**. **Do not gate yaml-lint** on Quick Check — run it
immediately.

**Perf-critical paths** (trigger benchmarks): `memory-core/src/**/*.rs`,
`memory-storage-turso/src/**/*.rs`, `memory-storage-redb/src/**/*.rs`,
`memory-mcp/src/**/*.rs`, `benches/**`, `Cargo.toml`, `Cargo.lock`,
`.github/workflows/benchmarks.yml`.

**Skip benchmarks manually**: add the `skip-benchmarks` label to the PR (the
workflow checks it at job start, so the label must be present in the event —
add it, then push). **Main branch** always runs benchmarks with regression
detection.

**Key insight**: GitHub Actions does not support `paths` + `paths-ignore` at the
same trigger level — use `paths` only.

Related skills: `.agents/skills/github-workflows/SKILL.md`,
`.agents/skills/ci-fix/SKILL.md`. Full plan (archived):
`plans/archive/2026-07-consolidation/ci-remediation/GOAP_CI_OPTIMIZATION_2026-04-28.md`.

## Publish Pipeline (2026-10-03)

crates.io publishing (`publish-crates.yml`, trigger `release: [published]` or
`workflow_dispatch`):

- **OIDC trusted publishing only** — each publish job authenticates with the
  pinned `rust-lang/crates-io-auth-action`; there is no
  `CARGO_REGISTRY_TOKEN` secret and no hand-rolled token exchange (ADR-078
  amendment, issue #1109 C1).
- **One implementation** — the gates (semver-checks, propagation wait, metadata
  verify, version-exists check, dispatch dependency closure, dry-run, publish)
  live in the `.github/actions/publish-crate` composite action; the workflow has
  four thin jobs parameterised by crate (issue #1109 C2).
- **Ordered `needs` chain**: core → redb → turso → mcp, with a skipped-tolerant
  gate (`always()` + explicit `result` checks) so a single-crate dispatch still
  reaches its job and fails (or passes) on the dependency-closure gate instead
  of silently skipping.
- `cargo publish --locked` for reproducibility; bounded sparse-index/API
  polling (≤ 20 × 15s) replaces `sleep 30` (LESSON-014, ADR-079 CIT-A4).
- Semver check output surfaced in `$GITHUB_STEP_SUMMARY`; it stays informational
  while the workspace is pre-1.0 (make it blocking at 1.0).
- **Trigger truth**: `release: published` does **not** create a run when the
  release is published with the repository `GITHUB_TOKEN` (the draft-first flow
  does exactly that — see `release.yml`), so `release.yml`'s `dispatch-publish`
  job calls `gh workflow run publish-crates.yml --ref <tag>` after publishing
  the release (`actions: write`). The dispatch API is a documented exception to
  the token-suppression rule; the `release: published` trigger remains as a
  safety net and re-runs no-op on already-published versions.
- **Hardening**: real publishes are tag-ref only (`inputs.dry-run == true ||
  github.ref_type == 'tag'`, matching ADR-072 authority, since crates.io does
  not validate the ref); the tag must equal the manifest version; packaging is
  verified in a token-less step and the authenticated publish uses
  `--no-verify` so the token never reaches a build script; `cargo-semver-checks`
  is version-pinned; and a `crates-io-publish` concurrency group serialises the
  check-then-act version probe.

**Prerequisite (one-time per crate, manual)**: register a trusted publisher on
crates.io for each crate with repository `d-o-hub/rust-self-learning-memory`,
workflow filename `publish-crates.yml`, environment `crates.io`. crates.io
matches the **calling** workflow (`workflow_ref`), so moving the steps into a
composite action or reusable workflow does not change the registered filename.
