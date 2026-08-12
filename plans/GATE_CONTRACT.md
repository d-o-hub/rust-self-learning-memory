# Quality Gate Contract (W2.1)

- **Status**: Accepted baseline; same-run fast gate + fail-closed `CI / Required` aggregate authoritative (2026-08-11)
- **Date**: 2026-07-18 (revised 2026-08-11)
- **Related**: ADR-042, ADR-072, ADR-079, AGENTS.md, `scripts/quality-gates.sh`, Codecov config
- **Validator**: `./scripts/validate-gate-contract.sh` (+ `--ci-parity`)

## Purpose

One matrix that maps every **advertised** quality gate to:

1. **Measured** value (what we observe when tools run),
2. **Blocking floor** (what fails CI / local commit gate today),
3. **Aspirational target** (where we are ratcheting),
4. **Authoritative command / CI job**; and
5. whether the live merge ruleset actually requires it.

Claims such as “coverage ≥90%” without a matching blocking check are **documentation debt**, not green status.

## Live merge-protection truth (2026-08-11)

Ruleset `9591004` (`main-protection`) is active for `refs/heads/main`. Its required
status contexts are `Codacy Static Code Analysis` and the first-party `CI / Required`
aggregate (added 2026-08-10, ADR-079 stage 3); it separately enforces CodeQL code
scanning at the configured severity. Only `CI / Required` is first-party
merge-required: no other first-party Quick Check, test, coverage, security,
storage, skill, release-drift, or anchor job is individually required by the
ruleset.

The workflow-side aggregate job (`ci.yml`, 2026-08-11) runs with `always()` and
evaluates its same-run dependencies via `scripts/ci-required-evaluate.sh`, which
accepts **only** `success`. Failure, cancellation, timeout, or `skipped` results
fail the aggregate (fail-closed). The fast gate (`commitlint` + `fast-gate`) is a
same-run dependency of the substantive jobs, so a failed fast gate skips them and
the aggregate reports the skipped result as a failure rather than a warning. A
green `CI / Required` check therefore means the same-run assertions actually
passed. Every first-party row below except CI / Required is **not individually
merge-required** by the ruleset.

## Gate matrix

| Gate | Measured (how) | Blocking floor (local) | Workflow enforcement today | Merge-required? | Aspirational target | Authoritative surface |
|------|----------------|------------------------|----------------------------|-----------------|---------------------|-----------------------|
| CI / Required aggregate | `if: always()` eval of fast-gate/commitlint/test/MCP/multi-platform/quality-gates results via `scripts/ci-required-evaluate.sh` | only `success` accepted; failure/cancelled/timed_out/skipped rejected | `ci.yml` `CI / Required` job (2026-08-11, same-run) | **Yes** (ruleset requires this context, 2026-08-10) | live ruleset requires this stable context | `ci.yml` aggregate + `scripts/ci-required-evaluate.sh` |
| Format | `cargo fmt --check` | required | same-run `fast-gate` job runs `./scripts/code-quality.sh fmt --workspace` | No | 100% formatted | `./scripts/code-quality.sh fmt --workspace` (local + CI) |
| Clippy | `cargo clippy -D warnings` | required | same-run `fast-gate` job runs `./scripts/code-quality.sh clippy --workspace` | No | 0 warnings workspace | `./scripts/code-quality.sh clippy --workspace` (local + CI) |
| Build check | `cargo check` / `./scripts/build-rust.sh check` | recommended | Builds occur in CI jobs, but no exact canonical check | No | always clean | `./scripts/build-rust.sh check` |
| Unit + integration | `cargo nextest run --all` | required before commit (AGENTS) | CI Tests runs `cargo nextest run --profile ci --workspace --exclude do-memory-benches --exclude do-memory-examples --exclude do-memory-test-utils` (support-crate exclusions; MCP and multi-platform duplicate subsets) | No | all pass | PR workflow is narrower than full local `cargo nextest run --all`; local command is authoritative |
| Doctests | `cargo test --doc` | required before commit (AGENTS) | same-run `fast-gate` invokes `scripts/check-doctests.sh` | No | all pass | `cargo test --doc` / `scripts/check-doctests.sh` |
| Docs links | `cargo doc --no-deps` | required before commit | same-run `fast-gate` invokes `scripts/check-doctests.sh` with warnings denied | No | 0 broken | `cargo doc --no-deps --document-private-items` |
| LOC ≤500 | quality-gates source-size check | required in quality-gates | No equivalent production LOC job; File Structure validates locations | No | 0 prod files >500 | `./scripts/quality-gates.sh` LOC check |
| Coverage | `cargo llvm-cov` | **default floor 70%** via `QUALITY_GATE_COVERAGE_THRESHOLD`; AGENTS text still says 90% | Coverage and CI quality jobs run overlapping commands; Codecov upload may soft-fail | No | **90%** (AGENTS + ADR-042 ratchet) | `QUALITY_GATE_COVERAGE_THRESHOLD` + Codecov |
| Security advisories | `cargo deny check advisories` | blocking (W2.2) | Security, Supply Chain, and CI quality jobs overlap | No | clean advisories | `cargo deny` (not soft-pass audit) |
| Cargo audit | `cargo audit` | informational if deny is blocking | optional structured reporting | No | no ignored vulns without justification | prefer deny for gating |
| Semver | cargo-semver-checks | CI-only informational | `continue-on-error: true` and Dependabot excluded | No | no accidental breaks | CI Semver Check |
| Skill evals | `./scripts/run-evals.sh` | fixtures recommended before skill PRs | Skill Evals runs fixtures always, changed on PR, full on schedule/dispatch | No | all skills strict schema | `.github/workflows/skill-evals.yml` + `./scripts/run-evals.sh --fixtures` |
| Release cadence | `./scripts/check-release-drift.sh` | warning@20 / critical@30 | Release Drift Check | No | tag before hard limit | release-drift workflow |
| Gate contract integrity | `./scripts/validate-gate-contract.sh` | required when editing gates | Skill Evals runs default + `--ci-parity`; parity checks same-run fast-gate commands, aggregate needs set, evaluator, and absence of waiter/anchor topology | No | semantic matrix ↔ scripts ↔ workflows ↔ ruleset alignment | `./scripts/validate-gate-contract.sh --ci-parity` |

### Coverage truth (explicit)

| Layer | Value | Source |
|-------|-------|--------|
| Local script default | **70%** | `scripts/quality-gates.sh` `QUALITY_GATE_COVERAGE_THRESHOLD` |
| AGENTS / commit checklist | **90%** | aspirational / ratchet target |
| Codecov | project + patch (repo config) | `.github` / codecov settings |
| Measured today | run `cargo llvm-cov` and record in VALIDATION | do not invent a number |

**Rule**: Never claim “coverage passed at 90%” unless the blocking floor and measured report both show ≥90%. A green job at 70% is “passed blocking floor 70%,” not “met AGENTS 90%.”

## Local vs CI parity

| Concern | Local entrypoint | CI entrypoint |
|---------|------------------|---------------|
| fmt + clippy | `./scripts/code-quality.sh fmt --workspace` / `clippy --workspace` | same-run `fast-gate` job in `ci.yml` (canonical local commands) |
| tests | `cargo nextest run --all` | CI Tests job excludes `do-memory-benches`, `do-memory-examples`, `do-memory-test-utils`; **PR workflow narrower than local `--all`** |
| quality bundle | `./scripts/quality-gates.sh` | Quality Gates job duplicates only a subset; **not parity** |
| deny advisories | `cargo deny check` | Cargo Deny / Supply Chain (`security.yml` / `supply-chain.yml`) |
| skill schema | `./scripts/run-evals.sh --fixtures` | Skill Evals workflow (`skill-evals.yml`) always |
| changed skill evals | `./scripts/run-evals.sh --changed` | Skill Evals on `pull_request` |
| full skill suite | `./scripts/run-evals.sh` | Skill Evals on `schedule` / `workflow_dispatch` |
| release drift | `./scripts/check-release-drift.sh` | `release-drift.yml` |
| gate contract | `./scripts/validate-gate-contract.sh` | Skill Evals job (default + `--ci-parity`) |

`./scripts/validate-gate-contract.sh` fails if this matrix file is missing required sections or if default coverage floor in `quality-gates.sh` disagrees with the **Blocking floor (local)** cell above.

`./scripts/validate-gate-contract.sh --ci-parity` verifies authoritative files,
the same-run `commitlint`/`fast-gate` jobs and their canonical commands, the exact
`CI / Required` needs set, `scripts/ci-required-evaluate.sh`, the absence of
`wait-on-check-action` and of the obsolete `quick-check.yml`/`pr-check-anchor.yml`
files, actor exclusions, accepted conclusions, and the live ruleset configuration.

## Non-goals (W2.1)

- Raising the blocking coverage floor to 90% in this PR (requires measured baseline + ratchet PR).
- Making the **full** skill-eval suite required on every PR (full suite is schedule / dispatch; PRs always run fixtures + changed skills).

## Acceptance (W2.1a)

- [x] Matrix documents measured / floor / target / authority for each advertised gate
- [x] Coverage contradiction (70 vs 90) is explicit, not hidden
- [x] `./scripts/validate-gate-contract.sh` enforces presence of matrix + floor alignment

## Acceptance (W2.1b)

- [x] `--ci-parity` verifies ci (same-run fast-gate + aggregate), release-drift, security/supply-chain (deny), skill-evals surfaces
- [x] CI runs `./scripts/validate-gate-contract.sh` and `--ci-parity` (Skill Evals workflow)
- [x] Local vs CI parity table lists skill schema + gate contract entrypoints
- [x] `--ci-parity` rejects cancelled/skipped waiter acceptance, missing `fail-on-no-checks: true`, absent `CI / Required` aggregate, release `workflow_dispatch`, `sleep 30` publish waits, Dependabot actor exclusion, and missing ruleset-context record (semantic negative fixtures)
- [x] Same-run fast gate is authoritative: `commitlint` + `fast-gate` run inside `ci.yml`; the `CI / Required` aggregate accepts ONLY `success` (skipped is rejected, fail-closed) via `scripts/ci-required-evaluate.sh` — regression fixture for the previous false-green aggregate (2026-08-11)
- [x] Exact command scopes, actor conditions, and aggregate outcomes agree (CIT-A2 Dependabot/fork actor parity + fixture, 2026-08-10)
- [x] Live ruleset requires the CI / Required context (ruleset 9591004 updated 2026-08-10; ADR-079 stage 3 complete)

## Acceptance (K3.1b)

- [x] CI always runs `./scripts/run-evals.sh --fixtures` (fail closed on schema fixtures)
- [x] PRs run `./scripts/run-evals.sh --changed` with history sufficient for `origin/main` diff
- [x] Full suite available on schedule / workflow_dispatch
- [x] No skill changes on PR → `--changed` may exit 0 (“No changed skills”) — allowed
