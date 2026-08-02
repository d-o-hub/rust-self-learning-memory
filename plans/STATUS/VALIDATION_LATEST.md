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
| ADR-078 status | Proposed; no implementation claim | ✅ truthful |
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
| P0 | PTA-A1 cascade capability truth | Implement typed unavailable/compile-time exclusion |
| P0 | PTA-A2 storage metric truth | Add provenance or omit unavailable fields |
| P1 | Gate contract/release/publish/fuzz truth | CIT-A3…A5 |
| P1 | PTA-A3 unsupported threshold surface | Remove/hide command |
| P1 | ADR-078 attribution capture | Maintainer decision, then RAT-A1…A7 |
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
