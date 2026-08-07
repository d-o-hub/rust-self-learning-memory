# Quality Gate Contract (W2.1)

- **Status**: Accepted baseline; **implementation drift detected 2026-07-30**
- **Date**: 2026-07-18
- **Related**: ADR-042, ADR-072, proposed ADR-079, AGENTS.md, `scripts/quality-gates.sh`, Codecov config
- **Validator**: `./scripts/validate-gate-contract.sh` (+ `--ci-parity`)

## Purpose

One matrix that maps every **advertised** quality gate to:

1. **Measured** value (what we observe when tools run),
2. **Blocking floor** (what fails CI / local commit gate today),
3. **Aspirational target** (where we are ratcheting),
4. **Authoritative command / CI job**; and
5. whether the live merge ruleset actually requires it.

Claims such as “coverage ≥90%” without a matching blocking check are **documentation debt**, not green status.

## Live merge-protection truth (2026-07-30)

Ruleset `9591004` (`main-protection`) is active for `refs/heads/main`. Its only
required status context is `Codacy Static Code Analysis`; it separately enforces
CodeQL code scanning at the configured severity. No first-party Quick Check, CI,
test, coverage, security, storage, skill, release-drift, or anchor job is required.
The standalone `Required Check Anchor` is also not required and performs no
validation. Therefore a green workflow job means that workflow ran successfully;
it does **not** mean the merge ruleset requires the gate.

ADR-079 proposes a staged `CI / Required` aggregate. The workflow-side aggregate
job now exists (2026-08-06, `ci.yml`): it runs with `always()` and fails closed
on failure/cancellation of the substantive same-run jobs, and all five
cross-workflow waiters now fail closed (no cancelled/skipped/missing acceptance,
commit lint included). Until the `CI / Required` context is added to the live
ruleset with maintainer approval, every first-party row below remains **not
merge-required**.

## Gate matrix

| Gate | Measured (how) | Blocking floor (local) | Workflow enforcement today | Merge-required? | Aspirational target | Authoritative surface |
|------|----------------|------------------------|----------------------------|-----------------|---------------------|-----------------------|
| CI / Required aggregate | `if: always()` fail-closed eval of test/MCP/multi-platform/quality-gates results | failure/cancelled never accepted | `ci.yml` `CI / Required` job (2026-08-06) | **No** (ruleset migration pending ADR-079 approval) | live ruleset requires this stable context | `ci.yml` aggregate + waiters (fail-closed) |
| Format | `cargo fmt --check` | required | Quick Check job | No | 100% formatted | `./scripts/code-quality.sh fmt` / Quick Check |
| Clippy | `cargo clippy -D warnings` | required | Quick Check uses separate lib/tests commands and a broad copied allow-list | No | 0 warnings workspace | Local: `./scripts/code-quality.sh clippy --workspace`; CI drift open |
| Build check | `cargo check` / `./scripts/build-rust.sh check` | recommended | Builds occur in CI jobs, but no exact canonical check | No | always clean | `./scripts/build-rust.sh check` |
| Unit + integration | `cargo nextest run --all` | required before commit (AGENTS) | CI excludes benches, examples, and test-utils; MCP and multi-platform duplicate subsets | No | all pass | Local command is authoritative; CI scope drift open |
| Doctests | `cargo test --doc` | required before commit (AGENTS) | Quick Check invokes `scripts/check-doctests.sh` | No | all pass | `cargo test --doc` / `scripts/check-doctests.sh` |
| Docs links | `cargo doc --no-deps` | required before commit | Quick Check invokes `scripts/check-doctests.sh` with warnings denied | No | 0 broken | `cargo doc --no-deps --document-private-items` |
| LOC ≤500 | quality-gates source-size check | required in quality-gates | No equivalent production LOC job; File Structure validates locations | No | 0 prod files >500 | `./scripts/quality-gates.sh` LOC check |
| Coverage | `cargo llvm-cov` | **default floor 70%** via `QUALITY_GATE_COVERAGE_THRESHOLD`; AGENTS text still says 90% | Coverage and CI quality jobs run overlapping commands; Codecov upload may soft-fail | No | **90%** (AGENTS + ADR-042 ratchet) | `QUALITY_GATE_COVERAGE_THRESHOLD` + Codecov |
| Security advisories | `cargo deny check advisories` | blocking (W2.2) | Security, Supply Chain, and CI quality jobs overlap | No | clean advisories | `cargo deny` (not soft-pass audit) |
| Cargo audit | `cargo audit` | informational if deny is blocking | optional structured reporting | No | no ignored vulns without justification | prefer deny for gating |
| Semver | cargo-semver-checks | CI-only informational | `continue-on-error: true` and Dependabot excluded | No | no accidental breaks | CI Semver Check |
| Skill evals | `./scripts/run-evals.sh` | fixtures recommended before skill PRs | Skill Evals runs fixtures always, changed on PR, full on schedule/dispatch | No | all skills strict schema | `.github/workflows/skill-evals.yml` + `./scripts/run-evals.sh --fixtures` |
| Release cadence | `./scripts/check-release-drift.sh` | warning@20 / critical@30 | Release Drift Check | No | tag before hard limit | release-drift workflow |
| Gate contract integrity | `./scripts/validate-gate-contract.sh` | required when editing gates | Skill Evals runs default + `--ci-parity`, but parity is presence/keyword based | No | semantic matrix ↔ scripts ↔ workflows ↔ ruleset alignment | `./scripts/validate-gate-contract.sh --ci-parity` |

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
| fmt + clippy | `./scripts/code-quality.sh` | Quick PR Check (`quick-check.yml`) |
| tests | `cargo nextest run --all` | CI Tests excludes three workspace packages; **not parity** |
| quality bundle | `./scripts/quality-gates.sh` | Quality Gates job duplicates only a subset; **not parity** |
| deny advisories | `cargo deny check` | Cargo Deny / Supply Chain (`security.yml` / `supply-chain.yml`) |
| skill schema | `./scripts/run-evals.sh --fixtures` | Skill Evals workflow (`skill-evals.yml`) always |
| changed skill evals | `./scripts/run-evals.sh --changed` | Skill Evals on `pull_request` |
| full skill suite | `./scripts/run-evals.sh` | Skill Evals on `schedule` / `workflow_dispatch` |
| release drift | `./scripts/check-release-drift.sh` | `release-drift.yml` |
| gate contract | `./scripts/validate-gate-contract.sh` | Skill Evals job (default + `--ci-parity`) |

`./scripts/validate-gate-contract.sh` fails if this matrix file is missing required sections or if default coverage floor in `quality-gates.sh` disagrees with the **Blocking floor (local)** cell above.

`./scripts/validate-gate-contract.sh --ci-parity` currently verifies authoritative
files and selected keywords exist (including `skill-evals.yml` wiring). It does
not parse command arguments, dependency topology, actor exclusions, accepted
conclusions, or live ruleset configuration. Calling this semantic CI parity is a
known gap tracked by ADR-079 / CIT-A3.

## Non-goals (W2.1)

- Raising the blocking coverage floor to 90% in this PR (requires measured baseline + ratchet PR).
- Making the **full** skill-eval suite required on every PR (full suite is schedule / dispatch; PRs always run fixtures + changed skills).

## Acceptance (W2.1a)

- [x] Matrix documents measured / floor / target / authority for each advertised gate
- [x] Coverage contradiction (70 vs 90) is explicit, not hidden
- [x] `./scripts/validate-gate-contract.sh` enforces presence of matrix + floor alignment

## Acceptance (W2.1b)

- [x] `--ci-parity` verifies quick-check, ci, release-drift, security/supply-chain (deny), skill-evals surfaces
- [x] CI runs `./scripts/validate-gate-contract.sh` and `--ci-parity` (Skill Evals workflow)
- [x] Local vs CI parity table lists skill schema + gate contract entrypoints
- [x] `--ci-parity` rejects cancelled/skipped waiter acceptance, missing `fail-on-no-checks: true`, absent `CI / Required` aggregate, release `workflow_dispatch`, and `sleep 30` publish waits (semantic negative fixtures, 2026-08-06)
- [ ] Exact command scopes, actor conditions, aggregate outcomes, and live ruleset context agree (ruleset migration remains; ADR-079)

## Acceptance (K3.1b)

- [x] CI always runs `./scripts/run-evals.sh --fixtures` (fail closed on schema fixtures)
- [x] PRs run `./scripts/run-evals.sh --changed` with history sufficient for `origin/main` diff
- [x] Full suite available on schedule / workflow_dispatch
- [x] No skill changes on PR → `--changed` may exit 0 (“No changed skills”) — allowed
