# GOAP: GitHub Actions Main Branch Issues (2026-02-17)

## Goal

Restore a consistently green `main` branch for:

- Scheduled workflows: **Nightly Full Tests**, **Security**
- Push workflows: **CI** (and dependent quality gates)

## Current State (Observed 2026-02-17)

### Failure inventory (recent `main` failures)

From `gh run list --branch main --status failure --limit 60`:

- `22085572056` Nightly Full Tests (schedule) `2026-02-17T04:07:26Z`
- `22059884603` CI (push) `2026-02-16T10:53:53Z` (historical)
- `22049835142` Nightly Full Tests (schedule) `2026-02-16T04:11:54Z`
- `22027889183` Security (schedule) `2026-02-15T01:58:49Z`
- `21958162889` YAML Lint (push) `2026-02-12T17:57:29Z` (historical)
- `21958162817` Coverage (push) `2026-02-12T17:57:29Z` (historical)
- `21958161949` Performance Benchmarks (push) `2026-02-12T17:57:27Z` (historical)

### Nightly Full Tests (schedule) is failing

- Latest example failure: run `22085572056` at `2026-02-17T04:07:26Z`
  - URL: https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/22085572056
- Symptom: E2E CLI tests fail with `error: unrecognized subcommand 'step'`.
- Likely root cause:
  - E2E test invokes `memory-cli episode step ...` in `tests/e2e/cli_workflows.rs`.
  - CLI defines `episode log-step` (kebab-case) in `memory-cli/src/commands/episode/core/types.rs` (`EpisodeCommands::LogStep`).
- Note: earlier nightly failures (e.g., run `22049835142` on `2026-02-16`) were driven by different issues (slow/perf tests and other E2E failures) covered by ADR-030; this document focuses on the current CLI contract regression.

### Security (schedule) is failing due to gitleaks findings

- Example failure: run `22027889183` at `2026-02-15T01:58:49Z`
  - URL: https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/22027889183
- Symptom: gitleaks reports `leaks found: 2` and fails the job.
- Missing data to act precisely:
  - The log suggests details are in the job summary and/or the SARIF artifact.
  - `gh auth status` indicates an invalid token locally, and `gh run download` intermittently fails to reach `api.github.com`, so artifact retrieval may require re-auth + retry.

### Historical failures (likely already remediated)

The following `main` failures are visible in recent history but appear consistent with previously completed CI remediation work:

- `Coverage` failures on `2026-02-12` (e.g., run `21958162817`)
- `YAML Lint` failures on `2026-02-12` (e.g., run `21958162889`)
- `Performance Benchmarks` failures on `2026-02-12` (e.g., run `21958161949`)

Re-verify only if they reoccur.

## Constraints / Prior Decisions

- **ADR-023**: CI/CD GitHub Actions Remediation (implementation complete, 2026-02-13)
- **ADR-029**: GitHub Actions Workflow Modernization (accepted, 2026-02-14)
- **ADR-030**: Test Optimization and CI Stability Patterns (accepted, 2026-02-16)
- **AGENTS.md invariants**: tokio async discipline, parameterized SQL, postcard serialization, zero clippy warnings

## GOAP Plan

### P0: Fix Nightly Full Tests E2E regression (CLI contract mismatch)

**Objective**: Make `Nightly Full Tests` pass consistently by aligning the CLI contract and E2E tests.

**Preconditions**

- `memory-cli` has an episode step logging command (`EpisodeCommands::LogStep`).
- `tests/e2e/cli_workflows.rs` is part of the nightly suite.

**Actions (ordered)**

1. **Decide the canonical CLI spelling for step logging**
   - Proposed: keep `episode log-step` as canonical; add `episode step` as an alias for backward compatibility.
   - Record the decision in **ADR-031**.
2. **Update E2E test call sites**
   - Change `["episode", "step", ...]` to `["episode", "log-step", ...]` in `tests/e2e/cli_workflows.rs`.
3. **Add CLI alias (if ADR-031 accepted)**
   - Add clap alias `step` for `EpisodeCommands::LogStep`.
4. **Add/refresh a CLI argument parsing test**
   - Ensure `memory-cli` unit parsing tests cover both `episode log-step` and `episode step` (alias).

**Verification**

- Local: `cargo test -p memory-cli --tests`
- Local: `cargo test --test cli_workflows` (or `cargo test --all` if E2E is wired differently)
- CI: Re-run Nightly Full Tests on `main` after merge.

**Expected Effects**

- Removes the `unrecognized subcommand 'step'` failure mode.
- Makes nightly resistant to CLI subcommand renames.

### P0: Fix Security scheduled failures (gitleaks)

**Objective**: Make scheduled `Security` workflow pass while maintaining real secret detection.

**Preconditions**

- Security workflow runs gitleaks on schedule for `main`.

**Actions (ordered)**

1. **Retrieve the gitleaks findings**
   - Preferred: download SARIF artifact from the failing run.
   - Command (requires `gh` connectivity/auth): `gh run download 22027889183 --dir /tmp/gh-artifacts-22027889183`
2. **Triage each finding**
   - If real secret: rotate and remove from history per repo policy.
   - If false positive: add a narrowly scoped ignore entry (fingerprint-based) with a comment.
3. **Rerun Security workflow**
   - Ensure schedule passes and PR/push triggers remain effective.

**Verification**

- CI: `Security` workflow passes on `main` schedule and on PR.

**Expected Effects**

- Scheduled security becomes trustworthy instead of noisy.

### P1: Unblock reliable GH Actions inspection from this environment

**Objective**: Make `gh` consistently able to fetch run metadata and artifacts.

**Observed Issue**

- `gh auth status` reports the token at `/home/do/.config/gh/hosts.yml` is invalid.
- Some API calls intermittently fail with `error connecting to api.github.com`.

**Actions**

- Re-authenticate: `gh auth login -h github.com`
- Confirm: `gh auth status`
- Retry artifact download for gitleaks runs.

## References

- Nightly failure run: `22085572056` (2026-02-17)
- Security failure run: `22027889183` (2026-02-15)
- E2E test file: `tests/e2e/cli_workflows.rs`
- CLI command definitions: `memory-cli/src/commands/episode/core/types.rs`
