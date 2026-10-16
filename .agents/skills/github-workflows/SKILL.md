---
name: github-workflows
description: "Diagnose, fix, and optimize GitHub Actions workflows for Rust projects. Use when setting up CI/CD, troubleshooting workflow failures, optimizing build times, or ensuring best practices."
---

# GitHub Workflows

Diagnose, fix, and optimize GitHub Actions workflows for Rust projects.

## Quick Reference

- **[Caching Strategies](caching-strategies.md)** - Manual cache, rust-cache, sccache
- **[Troubleshooting](troubleshooting.md)** - Common issues, debugging, fixes
- **[Advanced Features](advanced-features.md)** - Coverage, security, benchmarking
- **[Release Management](release-management.md)** - Automated releases, versioning

## When to Use

- Setting up CI/CD for Rust projects
- Troubleshooting workflow failures
- Optimizing build times with caching
- Ensuring best practices for testing, linting, releases

## Before Making Changes

**ALWAYS verify current state first:**

```bash
# Get repo info
gh repo view --json nameWithOwner,owner,name

# List existing workflows
gh workflow list

# Check recent runs
gh run list --limit 10

# View workflow files
ls -la .github/workflows/
```

## Complete Rust CI Workflow

See the full workflow template with:
- Check job (format, clippy, check)
- Test job (unit, integration, doc tests)
- Coverage job (tarpaulin, codecov)
- Audit job (security, licenses)

See linked files for caching strategies, troubleshooting, and release management.

## Core Workflow Components

| Job | Purpose | Tools |
|-----|---------|-------|
| check | Code quality | rustfmt, clippy, cargo check |
| test | Verification | cargo test |
| coverage | Test metrics | cargo tarpaulin |
| audit | Security | cargo audit, deny |

## Common Patterns

- **Caching**: Dependencies, target directory, sccache
- **Matrix builds**: Multiple Rust versions, targets
- **Conditional jobs**: Skip on docs-only changes
- **Quality gates**: Block merge on failures

## Toolchain Pinning Guardrail

- Never float a toolchain in a workflow that gates work or runs on a schedule.
  `nightly` broke a transitive dependency (`branches 0.4.5`, `core::intrinsics`)
  with no repository change and failed every fuzz target as a startup failure;
  floating `stable` similarly adds deny-by-default lints between releases.
- Pin a dated channel (`nightly-2026-09-26`) in **both** the job env and the
  `dtolnay/rust-toolchain` action input, comment why, and validate the pin by
  building the affected crate with that toolchain before pushing (LESSON-040).

## Composite Input Truthiness Guardrail

- Composite-action (`action.yml`) inputs arrive as **strings**, even when the
  caller passes a rendered boolean, and GitHub coerces any non-empty string to
  `true` — so `!"false"` is `false`. `if: ${{ !inputs.dry-run }}` therefore skips
  the step on every real run while the job reports success (LESSON-053). Blast
  radius, measured from the crates.io index on 2026-10-16: `do-memory-core` /
  `-storage-redb` / `-storage-turso` were last published at **0.1.34** and
  `do-memory-mcp` at **0.1.31**, while tags ran to **v0.1.45** — i.e. roughly a
  dozen green releases published nothing, not one or two.
- Match the **wire type at the place where you compare**, and normalise when the
  value crosses a boundary: a `workflow_dispatch` input declared `type: boolean`
  is a real boolean in the calling workflow's `if:`, while a composite action input
  declared **without** `type:` (`.github/actions/publish-crate/action.yml:15`,
  `default: "false"`) arrives as a **string** — and any non-empty string is truthy
  in the `if:` expression language, so `!inputs.dry-run` is `!"false"` = `false` and
  the step silently never runs:

  ```diff
  # .github/actions/publish-crate/action.yml — the real publish step (line 197 at v0.1.45)
  - if: ${{ !inputs.dry-run && steps.check-version.outputs.exists != 'true' }}
  # after (#1179): negate the string form, like the two correct siblings already did
  + if: ${{ inputs.dry-run != 'true' && steps.check-version.outputs.exists != 'true' }}
  ```

  The workflow-level guard `(inputs.dry-run == true || github.ref_type == 'tag')`
  (`publish-crates.yml:47`) was **correct all along** and is not the bug — which is
  precisely why the chain stayed green: the outer guard passed, the inner step skipped.
  Tag-only authority + OIDC still applies (ADR-072/ADR-078, never a static token).

  Add the per-crate selector (`inputs.crate == '' || inputs.crate == '<name>'`)
  and keep **one** comparison form across all steps of the action: the broken file
  compared correctly on two steps and negated on the third, which is why the skip
  survived review. Prefer `fromJSON()` (or rendering the boolean once into a step
  output) over hand-matching stringified booleans.
- Assert the guard's type with a fixture, not with a re-read: extend
  `scripts/test-release-workflow.sh --publish-fixtures` (it already fails the chain
  for a missing bounded poll or verification step) with a check for the exact
  comparison form, and let `scripts/validate-gate-contract.sh` reject any `!inputs.`
  negation in the publish files. A guard nobody can grep is a guard that regresses.
- **Green ≠ did**: a job concludes `success` while the step that performs the side
  effect is `skipped`. Read the step, not the rollup —
  `gh run view <id> --json jobs -q '.jobs[].steps[] | select(.name == "Publish to crates.io") | .conclusion'`
  and `duration_ms=0` on that step is a failed release even when CI is green.
- Put the proof **inside** the workflow: a final `Verify publish landed` step polls
  each crate's versions endpoint with the membership test below and `::error::` +
  `exit 1`s when the published version is missing. Keep the post-release external
  assert too (`curl -sS -A ci-verify https://crates.io/api/v1/crates/<crate>/versions`),
  never a `sleep`-then-assume.
- Publish acceptance is **exactly** the four crates the chain owns
  (core → redb → turso → mcp). `do-memory-cli` answering HTTP 404 is by design (it
  ships as a GitHub Release asset) and `do-memory-test-utils` is `publish = false` —
  neither is a publish failure, and both must be excluded from the expected set
  rather than "fixed" by widening the gate. Always send a `User-Agent`: crates.io
  rejects the default curl UA with `FETCH_FAILED`-shaped noise that looks like an
  outage.
- A run executes the workflow **and** the composite actions from its own SHA: a fix
  only applies to runs dispatched from refs that contain it (dispatching on a tag
  uses the tag's copy, so a superseded tag keeps the old behaviour forever).
- Prove side effects, do not assume them: any step that claims an external change
  (publish, push, register) must poll the provider with a bounded loop
  (`for i in $(seq 1 20)`, `sleep 15`) and `::error::` out when the object is
  missing. A green job is not evidence that an artifact exists.
- When checking whether an API already has an object, test **membership** of the
  full list (`[(.versions // [])[].num] | index($v) != null`) rather than
  comparing the newest entry — `[0]` equality misreads a superseded-but-present
  version as absent and flips a retry into a hard failure.

## PR Check Attachment Guardrail

- After pushing fixes, validate checks are attached to the PR head SHA:
  - `gh pr view --json statusCheckRollup,mergeStateStatus`
  - `gh run list --commit <head_sha>`
- If rollup is empty for required checks, treat as blocked and investigate trigger/path conditions.

## Trigger And Clock Guardrail

- Before explaining *why* a run happened, enumerate everything that can start it.
  Read the whole `on:` block — a `push` filter includes **tag** patterns, so
  `nightly-tests.yml` (`cron: 0 2 * * *` **plus** `push: tags: ['v*.*.*']` plus
  `workflow_dispatch`) also runs on every release tag. Name the real cause with
  `gh run view <id> --json event,triggerActor,runStartedAt`, never by assuming the
  schedule.
- Cron is not a clock. Queue delay pushed an `0 2 * * *` start to 06:30Z and later,
  and a scheduled run re-attaches to whatever SHA is current (LESSON-051). Reason
  from `runStartedAt`/`updatedTime`, never from the cron expression.
- `workflow_dispatch` is the only reliable manual clock: to claim "the nightly ran",
  dispatch it and cite the run id —
  `gh workflow run nightly-tests.yml -f run_slow_tests=true` then
  `gh run list --workflow nightly-tests.yml --event workflow_dispatch --limit 1`.
- The trigger **list** sets the price of an edit. `release-drift.yml` uses
  `[opened, synchronize, reopened, labeled, unlabeled]`: `edited` is not a trigger,
  so PR body/title edits are free, while every branch push (`synchronize`) restarts
  the entire required suite. Batch docs/skills edits into one push instead of one
  push per file (LESSON-052/LESSON-051 churn).
- A label applied with `GITHUB_TOKEN` does not re-trigger a run. When a check only
  fires on `labeled`/`synchronize`, force it on a **fresh head SHA**
  (`git commit --allow-empty -m "chore: re-trigger CI" && git push`) — never by
  deleting runs, dropping a trigger, or weakening the gate to make it quiet.

## Workflow Lint Guardrail

- Lint before pushing, on the same surface CI uses:
  `yamllint -c .yamllint.yml .github/`, `actionlint .github/workflows/*.yml`, and
  `shellcheck` on every `run:` block you touched.
- `yaml-lint.yml` invokes actionlint with `actionlint_flags: -shellcheck=` — pointing
  the shellcheck path at an empty value **disables** actionlint's embedded shell
  analysis, so shell bugs in workflow `run:` blocks pass CI green. Run shellcheck
  yourself; the fix is never to delete the lint job or silence rules.
