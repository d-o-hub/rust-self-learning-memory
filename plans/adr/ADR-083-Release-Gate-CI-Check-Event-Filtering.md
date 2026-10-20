# ADR-083: Release Gate CI Check Event Filtering

- **Status**: Accepted — implemented in PR #1204
- **Date**: 2026-10-18
- **Deciders**: Project maintainers
- **Related**: ADR-027, ADR-034, ADR-072, ADR-079, LESSON-051
- **Plan**: `plans/ACTIONS.md` (ACT-405)
- **Issue**: #1200
- **Code evidence**: `scripts/release-manager.sh`, `scripts/lib/release-gate-events.sh`,
  `scripts/test-release-workflow.sh`

---

## Context

The canonical release command `./scripts/release-manager.sh ship --execute` executes `do_ci_check()` to verify that CI workflow runs on `origin/main` HEAD are complete and green before creating and pushing a release tag (`vX.Y.Z`).

Previously, `do_ci_check()` queried workflow runs via `gh run list --branch main --commit <head>` without inspecting the `.event` field. As recorded in LESSON-051 and ACT-405, scheduled runs (`schedule`, e.g. `nightly-tests.yml` running daily at `0 2 * * *`) and manual dispatches (`workflow_dispatch`) attach to whatever commit is HEAD on `main` at trigger time.

When an out-of-band or scheduled nightly run failed on `main` HEAD, `do_ci_check()` counted that failure as a release blocker—even when all standard integration PR/push workflow runs on `main` HEAD (such as `ci.yml` / `CI / Required`, `security.yml`, etc.) were completely green. Conversely, if an out-of-band nightly was red for days, maintainers were forced to wait for cron windows, push dummy commits, or risk using `--skip-ci-check` at release time.

Release engineering governance requires explicit, deterministic gates ("decide, never weaken at ship time"). The gate must strictly prove that the integration CI suite for the release commit is green, while out-of-band / scheduled runs must be clearly classified.

## Decision

### 1. Classification of Workflow Run Events

Workflow runs attached to `origin/main` HEAD are classified into three categories:

1. **Gating Runs** — triggered by `push` or `pull_request`. These constitute the required CI
   control plane for code landing on `main` and gate releases. Every gating run on
   `origin/main` HEAD must be completed with `success`, `skipped`, or `neutral`. Any pending or
   failed gating run strictly blocks `release-manager.sh ship`.
2. **Advisory Runs** — triggered by `schedule` or `workflow_dispatch`. These represent
   out-of-band, periodic (nightly), or manual executions (e.g. long-running ignored test suites
   or stress runs). Advisory runs are informational for release gating: their failure or pending
   status does **not** block `release-manager.sh ship`.
3. **Neither** — runs triggered by any other event (`issues`, `dynamic`, …). `dynamic` runs are
   GitHub CodeQL's own analyses of the repository and `issues` runs react to issue events; neither
   is evidence about *this commit's* CI, and each has its own workflow gate. They are listed in
   the `ci-check` output for visibility but never decide the verdict.

The rule is an **allowlist** (`push`/`pull_request` gate), not a denylist of advisory events: a
denylist would silently promote any newly-introduced event type (a `release`-triggered run, a
`repository_dispatch`) into the gate.

**Tag-triggered runs remain gating.** `nightly-tests.yml` also fires on `push` of `v*.*.*` tags.
That is a `push` event, so the tag nightly is a gating run and a red tag nightly blocks a
release. This ADR relaxes only *scheduled* and *manually dispatched* runs.

### 2. Machine-Enforced `.event` Filtering in `release-manager.sh`

The classification lives in a **sourceable library**, `scripts/lib/release-gate-events.sh`, so the
gate and its test share one implementation. A test that re-implements the filter inline cannot
fail when the gate is broken.

The library holds the event names **once**, in `RELEASE_GATE_GATING_EVENTS` and
`RELEASE_GATE_ADVISORY_EVENTS`, and exposes `release_gate_select_runs` /
`release_gate_other_runs` to select by class. The gate calls those helpers; it does **not**
write the event names into a jq expression of its own. A second copy of the event list is a
second source of truth, and nothing in the test suite can catch the two disagreeing — the
fixture would happily pass while the gate implemented a different rule. (This was not
hypothetical: with the filter inlined in `release-manager.sh`, dropping `pull_request` from
the gate's jq left the fixture green. The library now owns the names, the fixture asserts
the gate uses the shared selector, and both mutations fail the fixture.)

`do_ci_check()` and `do_status()` in `scripts/release-manager.sh` request the `event` field
(`--json status,conclusion,name,event`) and query gating and advisory runs **separately**,
server-side:

- gating: `gh run list --branch main --commit <head> --limit 100 --json …`, then
  `release_gate_select_runs "$runs_json" gating`
- advisory: the same query with `--event schedule --event workflow_dispatch`, then
  `release_gate_select_runs "$advisory_runs_json" advisory`

Querying separately is deliberate. A single query followed by a client-side filter would let
advisory noise consume the `--limit` window and silently drop the real `CI` run — a fail-open
gate that reports green without ever seeing the run that matters.

Gating assertions (`total > 0`, `pending == 0`, `failed == 0`) operate strictly on the gating set.
Advisory runs are logged separately with an explicit label (`schedule/workflow_dispatch —
non-blocking`), and `do_status` prints an `advisory runs:` count, so nothing is hidden by the
filter.

### 3. Hardening Invariant: No Gate Weakening at Ship Time

At ship time, maintainers and agents must **never** edit or bypass `do_ci_check` to get a release
out. The `.event` filtering logic is codified permanently in script logic and verified by
automated fixtures (`scripts/test-release-workflow.sh`) that drive the **real** `do_ci_check`
against a stubbed `gh` and assert the verdict in both directions: a red `schedule` run must not
block, and a red or pending `push` run must block.

## Consequences

### Positive

- Releases on `origin/main` HEAD are no longer blocked by out-of-band scheduled/nightly run failures when all `push`/`pull_request` integration runs are green.
- Maintainers do not need to race against daily cron schedules or push dummy commits to detach stale nightly runs.
- Advisory runs remain fully visible in `release-manager.sh status` and `ci-check` output.
- Deterministic behavior is enforced and verified with automated test fixtures that exercise the shipped code path.

### Negative and Trade-offs

- Maintainers must still independently monitor nightly workflow results (`gh run list --workflow=nightly-tests.yml`) to address long-term test suite debt.
- A failure in a non-`push`/`pull_request` workflow (e.g. a CodeQL `dynamic` analysis) no longer blocks a release through this gate. Those workflows must be green on their own terms before the code reaches `main`.

## Alternatives Considered

1. **Require Nightly to Gate Releases**: Rejecting releases whenever `schedule` runs fail. Rejected: Scheduled nightlies re-attach to the same SHA daily and include force-run ignored tests; an un-cleared nightly debt would lock the release pipeline indefinitely despite green push CI.
2. **Silently Omit Advisory Runs**: Hiding `schedule` runs entirely from `ci-check`. Rejected: Maintainers need visibility into all runs on HEAD; logging advisory runs separately provides transparency without blocking ship.
3. **Denylist Filtering (the first implementation of this ADR)**: gate = "not `schedule` and not `workflow_dispatch`". Rejected: it makes every *other* event type gating by default, including `issues` and `dynamic` (CodeQL) runs, which are not evidence about the release commit. The allowlist in §1 is what the code now implements.
4. **Single Query + Client-Side Filter**: one `gh run list` call, filtered by `jq` afterwards. Rejected: the `--limit` window is consumed by whatever runs sort first, so advisory noise can push the real `CI` run out of the result set — the gate would pass without having seen it.
5. **Event names inlined in the gate's jq expression**: the classification in a library, but `release-manager.sh` selecting runs with its own `select(.event == "push" or …)`. Rejected: the event names then exist twice, and the fixture exercises the library while the gate implements something else — a disagreement no test in the suite can see. Measured: dropping `pull_request` from the gate's inline filter left every fixture green. The gate now calls `release_gate_select_runs`, and the fixture asserts it.

## Acceptance Criteria

- `plans/adr/ADR-083-Release-Gate-CI-Check-Event-Filtering.md` records this decision.
- `do_ci_check` in `scripts/release-manager.sh` filters by `.event` so `schedule` and `workflow_dispatch` runs do not block releases, and `push`/`pull_request` runs still do.
- The classification lives in `scripts/lib/release-gate-events.sh` and is exercised by `scripts/test-release-workflow.sh` through the real `do_ci_check` (not a re-implementation).
- The event names are written down **once**, in the library's arrays; the gate selects runs through `release_gate_select_runs` and the fixture fails if the gate inlines them.
- `AGENTS.md` and the `release-guard` skill document the event filtering rule, and `agent_docs/LESSONS.md` LESSON-051 plus `.agents/skills/ci-fix/SKILL.md` no longer assert the superseded rule.
