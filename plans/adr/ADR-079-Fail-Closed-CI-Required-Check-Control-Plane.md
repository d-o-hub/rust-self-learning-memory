# ADR-079: Fail-Closed CI Required-Check Control Plane

- **Status**: Accepted — stage 3 implemented 2026-08-10 (ruleset `9591004` requires `CI / Required`; CIT-A2 actor parity + fail-closed fixtures live); stages 4–5 (fault-inject merge-block proof, echo-anchor/waiter cleanup) pending
- **Date**: 2026-07-30
- **Deciders**: Project maintainers
- **Related**: ADR-029, ADR-030, ADR-038, ADR-039, ADR-042, ADR-072
- **Plan**: `plans/GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`

## Context

The checked-in workflows run substantial validation, but the merge control plane
does not require it. The active `main-protection` ruleset requires only Codacy
status and high-severity CodeQL policy. Quick Check, tests, coverage, security,
storage matrices, and the `Required Check Anchor` are not required contexts.
The anchor itself only echoes a message and is not causally linked to validation.

Five workflows independently poll one Quick Check job. Their waiters accept
`cancelled` and `skipped`, permit a missing check, and ignore the separate commit
lint job. PR #914 demonstrates the split contract: commit lint failed while all
downstream `Check Quick Check Status` jobs passed. Dependabot is explicitly
excluded from most CI, test, coverage, security, file-validation, semver, and
WASM jobs. This means workflow presence and green status cannot be treated as a
merge gate.

There is also contract drift:

- `plans/GATE_CONTRACT.md` advertises `cargo nextest run --all`, while required
  CI excludes benches, examples, and test-utils;
- the CI “Quality Gates” job does not execute the local quality-gate bundle;
- CI Clippy has a broad copied allow-list that differs from the local command;
- the gate-contract validator checks that files and keywords exist, not that
  cancellation, dependency, command, and ruleset semantics are aligned;
- Release exposes a manual dispatch that necessarily fails its tag preflight;
- manual single-crate publish selections are constrained by a dependency chain
  that can turn skipped prerequisites into skipped requested jobs; and
- fuzz targets use `continue-on-error`, while crash upload is guarded by
  `failure()`, allowing crashes to be green and lose their evidence.

## Decision

### 1. One causally complete required context

Create one stable aggregate context, provisionally `CI / Required`, from a single
PR orchestrator. Required jobs and their applicability decisions run in the same
workflow invocation. The aggregate uses `if: always()` and evaluates every
declared dependency result.

The aggregate succeeds only when each applicable gate succeeded. `failure`,
`cancelled`, missing/unknown state, and timeout fail closed. `skipped` is accepted
only when an in-workflow path/event classifier explicitly marked that gate not
applicable. A workflow-level path filter must not make the required context
disappear.

Cross-workflow polling is not part of the required control plane. Reusable
workflows or composite actions may share implementation, but required outcomes
must return to the orchestrator as same-run job dependencies.

### 2. Aggregate the whole fast gate

Formatting, Clippy, doctests/docs, ignored-test ceiling, YAML/frontmatter checks,
and commit-message policy form the fast gate. A failure in any component fails the
aggregate. A separate successful format/Clippy job cannot mask failed commit lint.
Cancelled or absent fast checks never authorize expensive or required work.

### 3. Actor parity with least privilege

Dependabot and fork PRs receive the same code-quality and test assertions as
maintainer PRs. Trust differences are handled with read-only permissions, no
secrets, no cache writes, and explicit secret-dependent job applicability—not by
omitting tests, semver, WASM, security, or structural checks wholesale.

### 4. One executable gate contract

Each gate has one canonical command or shared script used by local and CI entry
points. `GATE_CONTRACT.md` records three distinct facts:

1. command/scope implemented by the workflow;
2. whether the job is merge-required by the live ruleset; and
3. measured versus required versus target thresholds.

The parity validator must inspect command arguments, aggregate dependencies,
fail-closed result handling, actor conditions, and the expected ruleset context.
Presence-only keyword checks are insufficient. Test exclusions must either be
removed from the required Linux test surface or be documented as an explicit
policy with a separately enforced owner; prose may not call the subset `--all`.

### 5. Stage ruleset migration

Repository rules are changed only after the new aggregate has emitted the stable
context successfully on representative normal, Dependabot, fork, docs-only, and
code PR fixtures. Migration order is:

1. add the aggregate without changing protection;
2. observe and fault-inject failure/cancellation/missing/applicability cases;
3. add `CI / Required` to `main-protection` while retaining current external
   Codacy and CodeQL controls;
4. verify a deliberately failed aggregate blocks merge; and
5. remove the unused echo anchor and obsolete polling waiters.

Ruleset mutation is a shared-infrastructure action and requires explicit
maintainer approval at implementation time. No bypass flag is used.

### 6. Keep release and publish triggers truthful

Release creation remains tag-only under ADR-072. Remove the broken Release
`workflow_dispatch` surface rather than inventing a second release path.
Publishing gets an explicit package plan/dependency closure, exact version
preflight, `cargo publish --locked`, bounded sparse-index propagation polling,
and a separate safe validation/dry-run mode. A requested package must publish,
validate as already present, or fail with a reason; it must not disappear because
an unrelated prerequisite job was skipped.

### 7. Preserve evidence for informational gates

Fuzz and mutation checks may remain non-merge-blocking while baselines mature,
but crashes, surviving mutants, startup failures, and timeouts must be visible in
the job summary and uploaded with `if: always()`. A top-level green conclusion
must not be the only durable signal. Threshold promotion requires a measured
baseline and a separate ratchet change.

## Consequences

### Positive

- Branch protection becomes causally linked to first-party validation.
- Cancellation, missing checks, commit lint, and actor differences cannot
  silently produce approval.
- Local and CI acceptance criteria become reviewable and machine-checked.
- Required checks retain one stable context while internal jobs evolve.
- Release, publish, fuzz, and mutation outcomes become truthful and actionable.

### Negative

- Consolidating orchestration touches several workflows and requires staged
  ruleset coordination.
- Dependabot/fork parity may increase CI minutes until duplication is removed.
- A stable aggregate adds explicit applicability and result-mapping logic that
  needs fixture tests.

### Neutral

- Coverage and performance need not become merge-blocking merely because they
  are visible; their required status is an explicit policy choice.
- External Codacy and CodeQL controls remain in place during and after migration.

## Alternatives considered

1. **Require every current job directly**: rejected because path-conditional jobs
   can disappear and job names/topology become ruleset API.
2. **Make the echo anchor query other workflows**: rejected because it repeats
   the current cross-workflow race, missing-check, and cancellation problems.
3. **Use `workflow_run` as the aggregate**: rejected because its status attaches
   to another workflow run rather than providing a simple same-PR dependency DAG.
4. **Keep only external analysis required**: rejected because it does not prove
   Rust compilation, tests, documentation, or repository-specific contracts.
5. **Skip untrusted actors**: rejected because dependency changes are precisely
   where build, test, semver, and security validation are needed.

## Acceptance criteria

- The live ruleset requires `CI / Required` after staged verification. ✅ Done 2026-08-10 — ruleset `9591004` `required_status_checks` = `[Codacy Static Code Analysis, CI / Required]` (strict policy); enforced by `validate-gate-contract.sh --ci-parity` live-ruleset fixture.
- Failed, cancelled, timed-out, or missing applicable jobs fail the aggregate.
- Commit lint failure fails the aggregate.
- Explicitly inapplicable path/secret jobs do not block and cannot masquerade as
  applicable successes.
- Normal, Dependabot, fork, docs-only, and code PR fixtures emit the same stable
  aggregate context.
- Required test and Clippy commands match the documented contract exactly.
- A gate-contract fixture fails when cancellation is allowed, Dependabot is
  excluded, a dependency is removed, or the ruleset context is absent.
- Release manual dispatch is absent and tag release remains successful.
- Publish selection/dependency fixtures cannot silently skip requested work.
- Fuzz crashes upload evidence and produce a visible non-green signal even while
  the workflow remains non-required.
