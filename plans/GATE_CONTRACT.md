# Quality Gate Contract (W2.1)

- **Status**: Accepted baseline (W2.1a) + CI parity (W2.1b) + skill-evals CI (K3.1b)
- **Date**: 2026-07-18
- **Related**: ADR-042, AGENTS.md, `scripts/quality-gates.sh`, Codecov config
- **Validator**: `./scripts/validate-gate-contract.sh` (+ `--ci-parity`)

## Purpose

One matrix that maps every **advertised** quality gate to:

1. **Measured** value (what we observe when tools run),
2. **Blocking floor** (what fails CI / local commit gate today),
3. **Aspirational target** (where we are ratcheting),
4. **Authoritative command / CI job**.

Claims such as “coverage ≥90%” without a matching blocking check are **documentation debt**, not green status.

## Gate matrix

| Gate | Measured (how) | Blocking floor (local) | Blocking floor (CI) | Aspirational target | Authoritative surface |
|------|----------------|------------------------|---------------------|---------------------|------------------------|
| Format | `cargo fmt --check` | required | Quick PR Check | 100% formatted | `./scripts/code-quality.sh fmt` / Quick Check |
| Clippy | `cargo clippy -D warnings` | required | Quick PR Check | 0 warnings workspace | `./scripts/code-quality.sh clippy --workspace` |
| Build check | `cargo check` / `./scripts/build-rust.sh check` | recommended | CI Tests / MCP Build | always clean | `./scripts/build-rust.sh check` |
| Unit + integration | `cargo nextest run --all` | required before commit (AGENTS) | CI Tests job | all pass | `cargo nextest run --all` |
| Doctests | `cargo test --doc` | required before commit (AGENTS) | CI / quality-gates path | all pass | `cargo test --doc` |
| Docs links | `cargo doc --no-deps` | required before commit | CI docs where configured | 0 broken | `cargo doc --no-deps --document-private-items` |
| LOC ≤500 | quality-gates / file structure | required in quality-gates | File Structure / quality | 0 prod files >500 | `./scripts/quality-gates.sh` LOC check |
| Coverage | `cargo llvm-cov` | **default floor 70%** via `QUALITY_GATE_COVERAGE_THRESHOLD` (quality-gates.sh); AGENTS text still says 90% | Codecov / Coverage job (project + patch targets) | **90%** (AGENTS + ADR-042 ratchet) | `QUALITY_GATE_COVERAGE_THRESHOLD` + Codecov |
| Security advisories | `cargo deny check advisories` | blocking (W2.2) | Cargo Deny / Supply Chain | clean advisories | `cargo deny` (not soft-pass audit) |
| Cargo audit | `cargo audit` | informational if deny is blocking | may still run | no ignored vulns without justification | prefer deny for gating |
| Semver | cargo-semver-checks | CI Semver Check | Semver Check job | no accidental breaks | CI Semver Check |
| Skill evals | `./scripts/run-evals.sh` | `./scripts/run-evals.sh --fixtures` recommended before skill PRs | Skill Evals workflow: fixtures always; `--changed` on PR; full suite on schedule / dispatch | all skills strict schema | `.github/workflows/skill-evals.yml` + `./scripts/run-evals.sh --fixtures` |
| Release cadence | `./scripts/check-release-drift.sh` | warning@20 / critical@30 | Release Drift Check | tag before hard limit | release-drift workflow |
| Gate contract integrity | `./scripts/validate-gate-contract.sh` | required when editing gates | Skill Evals job runs default + `--ci-parity` | matrix ↔ scripts ↔ workflows aligned | `./scripts/validate-gate-contract.sh --ci-parity` |

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
| tests | `cargo nextest run --all` | CI Tests (`ci.yml`) |
| quality bundle | `./scripts/quality-gates.sh` | Quality Gates job (subset may differ) |
| deny advisories | `cargo deny check` | Cargo Deny / Supply Chain (`security.yml` / `supply-chain.yml`) |
| skill schema | `./scripts/run-evals.sh --fixtures` | Skill Evals workflow (`skill-evals.yml`) always |
| changed skill evals | `./scripts/run-evals.sh --changed` | Skill Evals on `pull_request` |
| full skill suite | `./scripts/run-evals.sh` | Skill Evals on `schedule` / `workflow_dispatch` |
| release drift | `./scripts/check-release-drift.sh` | `release-drift.yml` |
| gate contract | `./scripts/validate-gate-contract.sh` | Skill Evals job (default + `--ci-parity`) |

`./scripts/validate-gate-contract.sh` fails if this matrix file is missing required sections or if default coverage floor in `quality-gates.sh` disagrees with the **Blocking floor (local)** cell above.

`./scripts/validate-gate-contract.sh --ci-parity` additionally requires the authoritative workflow files and scripts listed above (including `skill-evals.yml` wiring for fixtures + gate-contract checks).

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

## Acceptance (K3.1b)

- [x] CI always runs `./scripts/run-evals.sh --fixtures` (fail closed on schema fixtures)
- [x] PRs run `./scripts/run-evals.sh --changed` with history sufficient for `origin/main` diff
- [x] Full suite available on schedule / workflow_dispatch
- [x] No skill changes on PR → `--changed` may exit 0 (“No changed skills”) — allowed
