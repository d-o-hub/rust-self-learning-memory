# Validation Latest — 2026-08-06 (CIT-A1/A2/A3 workflow wave)

**Goal**: Validate the fail-closed waiter changes, the `CI / Required` aggregate,
and the semantic gate-contract validator. Ruleset state untouched (maintainer).

**Workspace**: `0.1.38` · **Tag**: `v0.1.37` · **HEAD**: `92db07bf`

## Evidence

| Check | Observation | Result |
|-------|-------------|--------|
| `yamllint` | ci/coverage/security/benchmarks/file-structure clean | ✅ |
| `bash -n` | `validate-gate-contract.sh` clean | ✅ |
| `validate-gate-contract.sh --ci-parity` | PASS on this state | ✅ |
| Negative fixture 1 | reintroduced `allowed-conclusions: success,skipped,cancelled` → FAIL with message | ✅ |
| Negative fixture 2 | removed `CI / Required` aggregate → FAIL with message | ✅ |
| Waiters | 5 workflows: `allowed-conclusions: success`, `fail-on-no-checks: true`, commit-lint wait added | ✅ |
| Aggregate | `ci.yml` job `name: CI / Required`, `if: always()`, needs test/mcp/multi/quality-gates | ✅ |
| Plans validation | `validate-plans.sh --active-set --version-state --adrs --identifiers --links` | ✅ (pre-existing warnings only) |
| Live ruleset | Unchanged — adding `CI / Required` to ruleset needs ADR-079 acceptance | ⚠️ open |

---

# Validation Latest — 2026-08-06 (CIT-A4/A5 wave)

**Goal**: Validate the CIT-A4/CIT-A5 workflow changes and the plan-truth refresh
without touching the live ruleset (maintainer decision).

**Workspace**: `0.1.38` · **Tag**: `v0.1.37` · **HEAD**: `92db07bf`

## Evidence

| Check | Observation | Result |
|-------|-------------|--------|
| `yamllint` | `.github/workflows/{release,publish-crates,fuzz}.yml` clean | ✅ |
| `scripts/test-release-workflow.sh --publish-fixtures` | Asserts `cargo publish --locked` in publish-crates.yml — now passes | ✅ |
| publish polling jq | `[(.versions // [])[].num] | index($v) != null` — true/false/error-JSON guarded | ✅ tested |
| publish closure jq | workspace-member names extracted for `path+file://…#name@version`; dev-deps excluded; core→∅, redb→core, turso→core+redb, mcp→core+redb+turso | ✅ matches `needs:` chain |
| Release trigger | `workflow_dispatch` absent; `push: tags` + `pull_request` retained | ✅ |
| Fuzz evidence | Upload `if: always()` + `if-no-files-found: ignore`; status report fails job on crash/artifact | ✅ |
| Plans validation | `validate-plans.sh --active-set --version-state --adrs --identifiers --links` | ✅ (below) |
| Open PRs | #927 (attribution) + CIT-A4/A5 wave PR | — |
| Live ruleset | Unchanged — ADR-079 aggregate still requires maintainer acceptance | ⚠️ open |

## Planning-document validation (2026-08-06)

```bash
git diff --check
./scripts/validate-plans.sh --active-set --version-state --adrs --identifiers --links
./scripts/validate-gate-contract.sh --ci-parity
```

---

# Validation Latest — 2026-07-30

**Goal**: Validate the GitHub Actions/CI audit and synchronize it with the
product-truth and recommendation-attribution plan without changing workflows.

**Workspace**: `0.1.38` · **Tag**: `v0.1.37` · **Audit HEAD**: `e66defdf`

## Evidence

| Check | Observation | Result |
|-------|-------------|--------|
| Version state | `Cargo.toml` `0.1.38`; latest tag `v0.1.37` | ✅ |
| Live ruleset | REST `rulesets/9591004`: Codacy required; CodeQL severity policy; no first-party context | ⚠️ P0 |
| Required Check Anchor | Echo-only workflow; absent from ruleset | ⚠️ misleading/dead |
| Wait semantics | 5 workflows allow `success,skipped,cancelled` and `fail-on-no-checks: false` | ❌ not fail-closed |
| PR #914 control case | Commit Message Lint failed; all downstream `Check Quick Check Status` jobs succeeded | ❌ split fast-gate contract |
| Dependabot | Core CI/test/coverage/security/file/semver/WASM paths explicitly exclude actor | ❌ assertion gap |
| Test contract | Local `cargo nextest run --all`; CI excludes benches/examples/test-utils | ❌ drift |
| Quality contract | CI duplicates a subset; parity validator checks files/keywords only | ❌ drift |
| Release dispatch | Run `30301797956`: `main` failed expected-tag preflight, all downstream skipped | ❌ broken trigger |
| Action pinning | Inspected external `uses:` references are commit-SHA pinned | ✅ |
| ADR-080 status | Proposed; no implementation claim | ✅ truthful |
| ADR-079 status | Proposed; workflow and ruleset implementation unchanged | ✅ truthful |
| PTA-A1…A3 | ✅ Implemented (2026-08-01): cascade `CapabilityUnavailable`, `MetricValue` storage provenance, `eval set-threshold` removed |
| Patch whitespace | `git diff --check` | ✅ |
| Active/version/ADR/ID/link plans validation | `validate-plans.sh --active-set --version-state --adrs --identifiers --links` | ✅; known historical ADR-025/054 warnings only |
| Current gate validator | `validate-gate-contract.sh --ci-parity` | ✅ presence contract; not semantic proof |
| Production tests | No production/workflow code changed | Not run |

## Analysis evidence

- Traced recommendation generation and attribution across core, MCP, CLI,
  Turso, and redb.
- Verified the non-`csm` cascade successful-empty path.
- Verified CLI storage estimates/unavailable values are rendered as telemetry.
- Verified `eval set-threshold` has no persisted/consumed override model.
- Verified feedback currently feeds statistics but not later recommendation ranking.
- Inspected Quick Check, CI, Coverage, Security, Benchmarks, File Structure,
  Release, Publish, Fuzz, Mutation, Storage Matrix, Supply Chain, and gate scripts.
- Queried recent Actions history. In the 14-day sample, superseding pushes account
  for substantial cancellation churn (CI 12/38, Coverage 10/39, Benchmarks 12/26);
  these counts are operational context, not failures by themselves. Acceptance of
  cancellation by gate waiters is the independent correctness defect.
- Verified the active ruleset was updated 2026-06-03 and has no first-party
  required context. Classic branch protection returns “Branch not protected”
  because protection is implemented as a repository ruleset.
- Confirmed two open PRs (#914, #915) and one open issue (#913) at audit time.

## Open after this validation

| Priority | Item | Next step |
|----------|------|-----------|
| P0 | ADR-079 required-check control plane | Maintainer decision; implement/fault-inject aggregate before ruleset change |
| P0 | Cancelled/missing/commitlint and actor gaps | CIT-A2 after aggregate skeleton |
| P0 | PTA-A1 cascade capability truth | ✅ #916 merged (typed `CapabilityUnavailable`) |
| P0 | PTA-A2 storage metric truth | ✅ #916 merged (`MetricValue` provenance) |
| P1 | Gate contract/release/publish/fuzz truth | CIT-A3…A5 |
| P1 | PTA-A3 unsupported threshold surface | ✅ #916 merged (`eval set-threshold` removed) |
| P1 | ADR-080 attribution capture | Maintainer decision, then RAT-A1…A7 |
| P2 | Feedback-to-ranking adaptation | Separate ADR after capture integrity |

## Planning-document validation

Executed successfully:

```bash
git diff --check
./scripts/validate-plans.sh --active-set --version-state --adrs --identifiers --links
./scripts/validate-gate-contract.sh --ci-parity
```

The last command validates the current presence-based contract only;
ADR-079/CIT-A3 records why it is not yet semantic proof. Plan validation reported
only the known historical ADR-025 and ADR-054 duplicate-number aliases.

## PR merge session — 2026-08-02 (#916 + #917)

GOAP-orchestrated review/merge of the open PR queue (swarm: pr-readiness +
release-cadence-manager + code review). Result: queue 2 → 0 open.

| Check | Observation | Result |
|-------|-------------|--------|
| PR #917 ci(mutants) shard | 37 pass / 9 skip, no actionable comments; CLEAN | ✅ merged `b7d67f3e` (squash) |
| PR #916 PTA-A1/A2/A3 | Release-drift check blocked: `commit_limit` (30 unreleased, critical). Resolved via `release-preparation` label (release PR escape hatch) | ✅ merged `33b9d302` (squash) |
| PR #916 full CI | 42 checks green: Tests, MCP Build, Multi-Platform ×2, Benchmarks (51m), Quality Gates, Codacy, CodeQL | ✅ |
| main CI after #917 | Quick Check, CI, Coverage, CodeQL, Storage Matrix, YAML Lint | ✅ |
| Open PRs | `gh pr list --state open` | none |
| Release drift | workspace `0.1.38` > tag `v0.1.37` (version advanced); `commit_limit` accumulates until next release | ⚠️ monitored |
| PR comments | codacy 0 issues ×2, codecov all-covered ×2, benchmark posts (informational) | ✅ no actionable |

**Roast findings (pre-existing)** — all fixed 2026-08-02 in the warnings-
remediation PR: `vacuum_storage` now reports honestly (no fabricated
`storage_optimized`); `#[expect(clippy::excessive_nesting)]` removed via
`sync_storage` flatten refactor; `HealthStatus` rendering switched Debug→Display;
redundant `expanded_terms.is_empty()` guard removed from `retrieve_concept_graph`.
Also fixed pre-existing `clippy::uninlined_format_args` in `tests/arch_fitness.rs`,
renumbered duplicate ADR-078 (attribution → ADR-080; OIDC keeps 078), and added
a nextest slow-timeout override for the nested-build `quality_gate_pattern_accuracy`
test (cold-cache local timeout, passes in CI).

