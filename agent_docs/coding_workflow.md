# Complete Coding Workflow

The end-to-end, best-practice workflow for any change in this repository —
feature, fix, refactor, docs, CI. `AGENTS.md` is the entrypoint; this file is the
detail. Each step states the command, the evidence it must produce, and the
skill that owns it.

> Scope: the workflow below is the *how*. The *must never break* rules are in
> [`AGENTS.md#core-invariants-never-break`](../AGENTS.md#core-invariants-never-break);
> the command inventory is in [`AGENTS.md#quick-reference`](../AGENTS.md#quick-reference).

---

## 0. Prime

1. Read `AGENTS.md` (auto-loaded) and route the work through
   [`.agents/skills/skill-rules.json`](../.agents/skills/skill-rules.json); when a
   skill matches, read its `SKILL.md` **before** acting.
2. Read the live trackers: [`plans/ROADMAPS/ROADMAP_ACTIVE.md`](../plans/ROADMAPS/ROADMAP_ACTIVE.md),
   [`plans/STATUS/CURRENT.md`](../plans/STATUS/CURRENT.md).
3. Check [`plans/adr/`](../plans/adr/) for decisions that constrain the change.
4. If the harness is unfamiliar or stale: `do-harness doctor` (see
   [`AGENTS.md#dev-harness-do-harness`](../AGENTS.md#dev-harness-do-harness)).

**Evidence:** the skill(s) you will use, the ADR(s) that apply, the issue/ask
that scopes the work.

## 1. Scope and acceptance criteria

- Turn the ask into 2–5 **verifiable acceptance criteria** (observable
  behaviour, not "code changed").
- One atomic change per PR; split when the change has independent parts.
- 3+ steps or cross-crate work → plan with the `goap-agent` skill; parallelizable
  slices → `agent-coordination`.

**Evidence:** acceptance criteria written down (issue/PR body), plan or GOAP
decomposition for non-trivial work.

## 2. Branch and worktree hygiene

- `main` is protected: **never** commit or push to it; always a branch, and a
  dedicated worktree for parallel work:
  `git worktree add -b <branch> ../worktrees/<name> origin/main`.
- Keep local `main` fast-forwarded (`git pull --ff-only`) before branching.
- Remove worktrees and stale branches once the PR merges.

**Evidence:** branch name in conventional style (`feat/…`, `fix/…`, `docs/…`,
`ci/…`, `chore/…`).

## 3. Research before editing

- Read 3+ existing files in the touched area and **reuse the existing pattern**;
  do not introduce a second convention.
- Changing an exported symbol → run `xd://lsp` **references** first.
- External APIs / GitHub features / crates → confirm against **official docs**
  (and record the source in the PR body); never rely on memory.
- Tooling: `find` for unknown locations, `grep` for known symbols, `glob` for
  paths. See [`AGENTS.md#tool-selection-enforcement`](../AGENTS.md#tool-selection-enforcement).

**Evidence:** the pattern you copied, the references you checked, the docs/source
you cite.

## 4. Design

- Boring over clever; no speculative abstraction; delete dead weight.
- Compiled code: no avoidable allocation, copying, or computation.
- Files ≤ 500 LOC; split modules rather than grow them (this repo enforces the
  ceiling in CI).
- Architectural decisions → ADR in `plans/adr/` (before or with the change).
- Post-release and API contracts → update the owning ADR's follow-up section.

**Evidence:** the decision record (ADR or PR body section) and the invariants the
design preserves.

## 5. Implement

- Small, atomic, conventional commits (`feat(module): …`, `fix(module): …`) —
  one logical change per commit, message describes exactly what changed.
- Tests follow [`agent_docs/running_tests.md`](running_tests.md): `#[tokio::test]`
  for async, AAA structure, deterministic and isolated, full-suite safe.
- Test only what carries risk: behaviour, boundaries, invariants, transitions,
  precedence, error paths. No wiring/copy/echo tests, no tautologies.
- A bug fix starts with a reproducer that fails before the fix.

**Evidence:** the diff, the new/updated tests, and the reproducer for fixes.

## 6. Verify locally (do not skip)

- Run the **specific** path you changed (`cargo nextest run -p <crate>`,
  `--test <target>`, the binary/CLI/web surface) and observe the result — tests
  alone are not proof.
- Then the full gate set in [`AGENTS.md#required-checks-before-commit`](../AGENTS.md#required-checks-before-commit):

  ```bash
  ./scripts/code-quality.sh fmt
  ./scripts/code-quality.sh clippy --workspace
  ./scripts/build-rust.sh check
  cargo nextest run --all
  cargo test --doc
  cargo doc --no-deps --document-private-items
  ./scripts/quality-gates.sh
  do-harness verify --record          # computational sensors + state; --only <sensor> to iterate
  git status                          # everything intended is staged
  ```

- CI is the second pair of eyes, not the first: reproduce CI's exact command
  (toolchain, feature set) when a gate disagrees with local results.
- Changing a skill or `plans/GATE_CONTRACT.md`? Also run
  `./scripts/run-evals.sh --fixtures` and
  `./scripts/validate-gate-contract.sh --ci-parity` locally — the Skill Evals
  workflow enforces both on PRs.

**Evidence:** command + observed output (counts, exit codes) recorded in the PR
body. "Should pass" is not evidence.

## 7. Document

- User-visible behaviour → `CHANGELOG.md` (Keep a Changelog sections, issue
  references).
- Docs that name behaviour → update the docs (`agent_docs/`, `docs/`, ADRs).
- Sprint/priority changes → the trackers together:
  `plans/ROADMAPS/ROADMAP_ACTIVE.md`, `plans/STATUS/CURRENT.md`,
  `plans/GOAP_STATE.md`, `GOALS.md`, `ACTIONS.md`.
- New public types → re-export from `lib.rs`; URLs in docs wrapped in `<…>`.

**Evidence:** docs diff; `./scripts/check-docs-integrity.sh` passes.

## 8. Open the PR and validate the health check

- Follow [`agent_docs/git_workflow.md`](git_workflow.md) for commit/branch rules.
- Run the readiness gate before declaring anything ready:

  ```bash
  ./scripts/check-pr-readiness.sh <PR>       # merge state + CI + comments
  gh pr checks <PR> --required --watch       # `ci-poll` skill: block until terminal
  ```

- The checks that matter: `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`,
  required checks terminal and green, **every** comment (human and bot —
  Codacy/Codecov included) addressed or waived with evidence on the thread.
- Coverage finding you are waiving? Classify it with the `coverage-waivers` skill,
  reply on the thread, and pass `--accept-codecov-waiver` to the merge command.
- Skill: `.agents/skills/pr-readiness/SKILL.md` (supersedes the short summary in
  [`AGENTS.md#pr-health-check`](../AGENTS.md#pr-health-check)).

**Evidence:** readiness output + a reply on each addressed thread.

## 9. Adversarial review of risky changes

For security-relevant, concurrency, release-pipeline, storage, or otherwise
risky changes, get an **independent** opinion before merge:

- `reviewer` / `security-reviewer` subagents on the final diff, or the
  `analysis-swarm` skill for competing designs.
- Treat agent findings as claims: verify each with primary evidence (source,
  empirical command) before acting; refuting a finding with evidence is a valid
  outcome, and the evidence goes in the PR.
- Apply accepted findings, re-run the gates, then merge.

**Evidence:** findings list (severity, location), disposition per finding, and
the re-run gate output.

## 10. Merge (gated)

```bash
./scripts/merge-pr.sh <PR> --execute     # readiness gate + independent re-check
```

- Never `--admin`, never force, never bypass a red check.
- `BEHIND` → `gh api repos/{owner}/{repo}/pulls/{n}/update-branch -X PUT -f update_method=merge`,
  wait for the new head's checks, then merge.
- After merge: delete the branch/worktree, sync local `main`.

**Evidence:** the merge commit SHA and a clean `check-pr-readiness.sh` run on the
merged head.

## 11. Release (when drift or intent demands it)

- One path only: `.agents/skills/release-guard/SKILL.md` +
  `./scripts/release-manager.sh ship --execute`; the tag push builds and
  publishes the GitHub Release (`release.yml`, draft-first + attested).
- Drift detected (`version_not_advanced`, `commit_limit`, `age_limit`) →
  `.agents/skills/release-cadence-manager/SKILL.md` +
  `./scripts/release-cadence-manager.sh`.
- After the tag: post-release workspace bump PR, then verify the release notes
  and artifacts (`gh release view`).

**Evidence:** shipped tag, release URL/notes, drift issue state, bump PR.

## 12. Cleanup and record

- Remove scaffolds, throwaway scripts, temporary files; prefer
  `./scripts/clean-artifacts.sh` for disk (see [`disk_hygiene.md`](disk_hygiene.md)).
- Record non-obvious friction as a LESSON in [`LESSONS.md`](LESSONS.md) and, when
  it is a recurring class, extend the owning skill or harness sensor (protocol:
  [`.agents/skills/harness/SKILL.md` § Steering Loop](../.agents/skills/harness/SKILL.md)).
- Update the status trackers if the sprint scope changed.

**Evidence:** clean `git status`, tracker/LESSONS diff where applicable.

---

## When blocked

Stop and report — do not silently narrow the task:

1. State the exact missing input or failing precondition, and what you already
   tried (commands + observed output).
2. If it can be unblocked by another party, say precisely what they must do.
3. Keep everything reachable complete and committed; never leave half-finished
   scaffolds behind.

## Definition of done

- [ ] Acceptance criteria demonstrably met (evidence, not assertions).
- [ ] All gates in `AGENTS.md#required-checks-before-commit` pass locally.
- [ ] CI green on the PR head; all review/bot comments addressed or waived with evidence.
- [ ] Docs/CHANGELOG/trackers updated where behaviour or priority changed.
- [ ] Merged via `merge-pr.sh`; branch and worktree cleaned up.
- [ ] For releases: shipped via `release-guard`, release verified, drift resolved.

## References

- `AGENTS.md` — invariants, gates, routing, release rules.
- `agent_docs/running_tests.md` — test/validation commands and coverage.
- `agent_docs/git_workflow.md` — commits, branches, PRs.
- `agent_docs/ci_guidance.md` — CI parity and troubleshooting.
- `agent_docs/github_actions_patterns.md` — workflow patterns, CI optimization,
  publish pipeline.
- `agent_docs/common_friction_points.md`, `agent_docs/LESSONS.md` — failure
  patterns to avoid.
- `.agents/skills/` — one skill per recurring operation (`build-rust`,
  `code-quality`, `test-runner`, `pr-readiness`, `release-guard`, `harness`,
  `goap-agent`, `agent-coordination`, …).
