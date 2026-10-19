# Agent Coding Guidelines

Entrypoint for coding agents. **Complete workflow:** [`agent_docs/coding_workflow.md`](agent_docs/coding_workflow.md) — **Docs index:** [`agent_docs/README.md`](agent_docs/README.md) — **Skills:** `.agents/skills/` (routed by `.agents/skills/skill-rules.json`; full inventory `.agents/SKILLS.md`).

Rust/Tokio episodic-memory workspace: `do-memory-core`, `do-memory-storage-turso`, `do-memory-storage-redb`, `do-memory-mcp`, `do-memory-cli`, `do-memory-test-utils`, `benches`. Storage: Turso + redb + embeddings (OpenAI/Cohere/Ollama/local).

## Quick Reference

| Task | Command |
|---|---|
| Build | `./scripts/build-rust.sh dev\|release\|check\|clean` |
| Format / lint | `./scripts/code-quality.sh fmt` · `./scripts/code-quality.sh clippy --workspace` |
| Tests | `cargo nextest run --all` + `cargo test --doc` |
| Quality gates | `./scripts/quality-gates.sh` |
| PR readiness | `./scripts/check-pr-readiness.sh [PR]` |
| Merge (gated) | `./scripts/merge-pr.sh <PR> [--accept-codecov-waiver] --execute` |
| Release | `./scripts/release-manager.sh ship --execute` |
| Release cadence | `./scripts/release-cadence-manager.sh` |
| Disk cleanup | `./scripts/clean-artifacts.sh [quick\|standard\|full]` |
| Harness | `do-harness verify --set verification --strict --record && do-harness status --set verification` · `do-harness doctor` |

## Skill + CLI Pattern (CRITICAL)

Route every operation: **skill? → script? → Skill + CLI? → task tool.**

| Operation | Skill | CLI |
|---|---|---|
| Build | `build-rust` | `./scripts/build-rust.sh` |
| Format / lint | `code-quality` | `./scripts/code-quality.sh` |
| Tests | `test-runner` | `cargo nextest run --all` + `cargo test --doc` |
| Debug failures | `debug-troubleshoot` | — |
| PR merge readiness | `pr-readiness` | `./scripts/check-pr-readiness.sh` |
| Wait for CI | `ci-poll` | `gh pr checks --watch` |
| Release | `release-guard` | `./scripts/release-manager.sh ship --execute` |
| Release cadence | `release-cadence-manager` | `./scripts/release-cadence-manager.sh` |
| Harness sensors | `harness` | `do-harness verify --set verification --strict --record && do-harness status --set verification` |
| Complex multi-step | `goap-agent` | — |
| Parallel workers | `agent-coordination` | — |

`gh skill install cli/cli gh --scope user --agent <host>` adds the official cross-repo `gh` patterns (not a substitute for this repo's skills). Ship releases **only** through `release-guard` + `./scripts/release-manager.sh ship --execute`; **NEVER** `gh release create` for shipping.

## Complete Coding Workflow

0. **Prime** — route skills, read the trackers and `plans/adr/`.
1. **Scope** — acceptance criteria first; one atomic change per PR.
2. **Branch** — never on `main`; worktree per PR (`git worktree add -b <branch>`).
3. **Research** — reuse existing patterns; `xd://lsp` references before symbol changes; official docs for external APIs.
4. **Design** — boring > clever; ADR for architectural decisions.
5. **Implement** — small conventional commits; tests with the change.
6. **Verify** — run the changed path, then all gates in [Required Checks](#required-checks-before-commit).
7. **Document** — CHANGELOG, docs, ADR follow-ups, trackers together.
8. **PR** — `check-pr-readiness.sh`; address **all** comments (bots included).
9. **Roast** — independent review for risky changes; verify each finding with primary evidence.
10. **Merge** — `merge-pr.sh <PR> --execute` only (never `--admin`).
11. **Release** — see [Release Process](#release-process).
12. **Cleanup** — remove worktrees/scaffolds; record lessons.

Details, evidence requirements, blocked protocol, definition of done: [`agent_docs/coding_workflow.md`](agent_docs/coding_workflow.md).

## Core Invariants (Never Break)

- **Async**: Tokio everywhere. No blocking in async (use `spawn_blocking`).
- **Storage**: parameterized SQL only; short transactions; no locks across `.await`.
- **Serialization**: Postcard required (not bincode).
- **Clippy**: zero warnings (`-D warnings`). Fix, don't suppress.
- **Files**: ≤500 LOC per source file (enforced in CI).
- **Tests**: ≥70% coverage floor (90% target); `#[tokio::test]` for async; AAA pattern.
- **Docs**: URLs wrapped in `<...>`; new public types re-exported from `lib.rs`.
- **Fail-closed code execution**: `execute_agent_code` is unavailable and fails closed.

## Workflow Traps (2026-10, read before merging)

- **Bot/agent branches**: `mergeable=MERGEABLE` ≠ no lost work. Diff the tip against the merge base (`git diff <merge-base> <tip> --stat`) before merging a long-lived branch; PR #1130's tip silently reverted four merged PRs. The variant that actually bites: repeated `Merge branch 'main' into …` resolutions that keep "ours" roll back dependency bumps — PR #1176 (a one-line `query_range` clamp) carried `sysinfo = "0.38"` + the `objc2` tree deleted from `Cargo.lock` while merge base and main were at `0.39`, and a 3-way merge takes "theirs" for a line only the branch changed, so merging it would have downgraded **main**. On a non-deps branch, any `Cargo.toml`/`Cargo.lock` hunk is a red flag: `git checkout origin/main -- Cargo.lock '*/Cargo.toml'` and re-verify before merge.
- **Floating `stable`**: new deny-by-default lints appear between releases (clippy `assert_is_empty`, rustdoc `redundant_explicit_links` in 1.99). `rustup update stable` before diagnosing CI-only lint failures; fix the lint, never suppress.
- **Shared `CARGO_TARGET_DIR`**: concurrent worktree builds can replace test binaries and fake mass failures — re-run the failing test in isolation before believing it.
- **Strict up-to-date checks**: every merge invalidates the other PRs' required runs; merge as a serial pipeline (update the next branch immediately after each merge).
- **Machine-readable output**: never trust a job whose report file is empty (e.g. `--message-format libtest-json` needs `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`); assert the artifact exists.
- **Session traps (2026-10-11/12, full detail `agent_docs/LESSONS.md` LESSON-039…045)**: a missing `cargo fmt --all` reds the fast gate *and* `quality_gate_formatting` inside Storage Matrix + Coverage; floating toolchains break dependency *compilation* (pin dated nightly/stable); `gh pr checks` prints cancelled checks as `fail` (verify with the jobs API, and never cancel runs of a PR you intend to merge); same-file structural splits in parallel PRs (`backend.rs`) force manual rebuilds — land the structural PR first; a new defaulted `StorageBackend` method needs an overlay classification (`fail-closed` when it returns `CapabilityUnavailable` without a predicate); the drift check re-evaluates its original payload, so after the auto-label push or `gh pr update-branch`.
- **Advisory traps (2026-10-15, LESSON-046…048)**: RustSec is enforced by **two** blocking tools inside one Quality Gates step (`cargo deny check advisories` *and* `cargo audit --json`) — scope a fix from the union and write the waiver into both `deny.toml` and `.cargo/audit.toml`; a stale `~/.cargo/advisory-dbs/` clone gives deny a **false green** (cross-check with `cargo audit`, whose DB lives at `~/.cargo/advisory-db`); bump with `cargo update -p <pkg> --precise <min-patched>` from the advisory's `[versions] patched` range (not the newest), and re-run the WASM gate's exact command + `RUSTFLAGS` when a bump moves `getrandom` across a major.
- **Phantom CI failures (2026-10-15, LESSON-052)**: `statusCheckRollup` keeps *every* check run ever recorded for the head SHA, so a re-run leaves stale `FAILURE` rows behind. Both gates now reduce to the **latest run per check name**, fail-closed (an unfinished run outranks every finished one) — `check-pr-readiness.sh` and `merge-pr.sh` must change together. Never hand-count the raw array, and never delete workflow runs or bypass a gate to silence rows.
- **Self-suppressed CI (2026-10-17, LESSON-065)**: GitHub scans the **entire last commit message** for `[skip ci]`-style control tokens, and prose or backticks do not neutralize them — a message that merely *names* the token suppresses every `pull_request` run, so CodeQL + Codacy rows stay green while the required aggregate `CI / Required` never reports and `mergeStateStatus` is pinned `BLOCKED` with nothing pending. Squash-merge inherits that message, so the same wording would blind **push** CI on main. Probe with `gh api .../commits/<sha>/check-runs --jq .total_count` (0 = self-suppressed); write *skip-ci marker* in prose, and prove a message-only rewrite with `git rev-parse HEAD^{tree}` identical before `--force-with-lease`.
- **Bot-committer CI never runs (2026-10-18, LESSON-067)**: a PR whose commit is committed *and* triggered by `github-actions[bot]` (the changelog generator's `create-pull-request` PRs) parks every repository workflow run at `conclusion=action_required` with `jobs.total_count=0` until a maintainer approves, while CodeQL/Codacy still report `SUCCESS` — so the rollup shows zero failures and `mergeStateStatus` stays `BLOCKED` with nothing pending. An empty failing-row list is not green: check `gh run list --branch <head> --json name,conclusion` and `gh api repos/{owner}/{repo}/actions/runs/<id> --jq .conclusion`, and prefer generator designs that need no PR at all (#1190's skip guard) over blanket-approving a bot's runs.
- **Merge-gate traps (2026-10-15, LESSON-049…051)**: `main` is guarded by **ruleset `main-protection`**, not branch protection (`/branches/main/protection` 404s) — required contexts are only `CI / Required` + `Codacy Static Code Analysis`, read them from `gh api repos/{owner}/{repo}/rules/branches/main` (an array of rule objects) at `.parameters.required_status_checks`; `CI / Required` is an **aggregate job**, not a visible per-job row, so `mergeStateStatus` stays `BLOCKED` while every green row you can see has reported — wait for the slowest dependency; a `release-preparation` label applied by `github-actions[bot]` cannot re-fire the drift check (GitHub suppresses token-triggered runs and `gh run rerun` replays the stale payload) — toggle it as a human with `gh pr edit <n> --remove-label … && --add-label …`; `release-manager.sh ci-check` counts **schedule**-triggered nightly runs by head SHA, so an always-red nightly blocks shipping — advance main HEAD and ship inside the window before the next nightly rather than weakening the gate.
- **Publish-chain traps (2026-10-16, LESSON-053)**: composite-action inputs are **strings**, so a `!inputs.x` guard coerces `"false"` to true and skips the step while the job stays green — that made the crates.io chain false-green for v0.1.44/v0.1.45 (`Publish to crates.io` reported `conclusion=skipped`, crates.io still at 0.1.34/0.1.31). Compare inputs to the literal (`inputs.x != 'true'`), query crates.io version **membership** (`[(.versions // [])[].num] | index($v)`) instead of `.versions[0].num`, and only gate "wait for the upstream crate" on a step that actually published it; a workflow run executes the workflow **and** its composite actions from the tag SHA, so a superseded tag can never be republished — the fix (`01d1bd7e`, #1179) rides the next tag (`v0.1.46` — the first release expected on crates.io), and after any release verify each crate externally — `curl -sS -A ci-verify https://crates.io/api/v1/crates/<crate>/versions | jq -r --arg v "$TAG" '[(.versions // [])[].num] | index($v) != null'` must print `true`. crates.io **rejects a request with no `User-Agent`** (it returns an error body, so a bare `curl | jq` prints *empty*, which reads as "not published" and then as "fine" — assert the HTTP status, per the machine-readable-output rule above).
- **Advisory registers are three, not one (2026-10-16, LESSON-056)**: Dependabot (GHSA), `cargo audit` and `cargo deny` disagree by construction — the 12 open alerts are 6 advisories × 2 lockfiles, and `Cargo.toml:14` `exclude = ["fuzz"]` means no local gate ever reads `fuzz/Cargo.lock`. RustSec `informational` items land in audit's `.warnings.unsound` (measured: rc 0 in-repo vs rc 1 from `/tmp`, where `.cargo/audit.toml` ignores don't apply) and never in `.vulnerabilities`, the only field CI fails on. Classify by register and dedupe by `ghsa_id` before promising a fix — clearing a gate closes no alert.
- **A version bump does not make the repo shippable (2026-10-16, LESSON-057)**: `release-manager.sh` Phase 0 runs `./scripts/verify-release-state.sh --check-unreleased`, which validates `CHANGELOG.md` + the version-doc lines against `workspace_version` — run it before promising a tag date, and land the `## [X.Y.Z]` heading and the doc flip as one release-prep PR (the #1169 pattern). The drift gate files its own canonical `release-drift` issue in place (`release-drift.yml:98-149`, #1182); it is a symptom, so never close it by hand — the bump is the exit.
- **Nightly counts need their configuration (2026-10-16, LESSON-055)**: `nightly-tests.yml`'s cron `0 2 * * *` actually starts ~06:30Z and tag pushes trigger it too (`workflow_dispatch` is the only real clock); `[[test]] required-features` is a build-level gate and `--run-ignored only` force-runs `#[ignore]`d tests, so a red nightly can be a test-configuration artifact rather than a storage bug — 86 (`--run-ignored only`) + 20 (`--features turso_multi_dimension … --run-ignored all`) = 106 was the pre-fix total. Never quote a nightly result without the feature matrix and `--run-ignored` mode that produced it.
- **Skill evals punish the correct wording (2026-10-16, LESSON-058)**: 24 fixtures assert `"exec": "! rg -q 'gh release create' <skill>/SKILL.md"` — presence-only, so writing the *prohibition* ("**never** `gh release create`") reddens `Skill schema + evals`. Read a skill's `evals/evals.json` before adding forbidden-practice prose, and rewrite the fixture as canonical-command-required **plus** unguarded-mention-rejected: `rg -q 'release-manager\.sh ship --execute' SKILL.md && ! (rg -n 'gh release create' SKILL.md | rg -vi 'never|no t|do not|don.t|avoid|forbid|instead')`. `run-evals.sh --changed` diffs committed state only (`scripts/run-evals.sh:169-174`), so iterate with `./scripts/run-evals.sh <skill>` and re-run after the commit.
- **Bare `((i++))` aborts a `set -e` loop (2026-10-16, LESSON-059)**: post-increment evaluates to the *old* value, so the first increment from 0 returns exit status 1 and `set -euo pipefail` kills the script. `check-pr-readiness.sh` printed 2 of 9 open PRs, exited rc=1 with empty stderr — and because rc=1 is also its legitimate "issues found" exit, the truncated report read as complete (it hid PR #1171's 3 failing checks). Use `VAR=$((VAR + 1))` or `((++VAR))`, and assert a sweep's output length equals the work it was supposed to cover.
- **Two triage sources that lie (2026-10-16, LESSON-060…062)**: (a) **clippy** — the authoritative gate is `cargo clippy --workspace --tests -- -D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used` (`scripts/code-quality.sh:144,158`); a `-p <crate>` run misreports crate-root `#![expect(...)]` gates as `unfulfilled lint expectation` because feature unification differs, so never "fix" an unrelated `lib.rs` from a scoped run (and `-A` on the command line cannot override a crate-level attribute). (b) **patch coverage** — a package-scoped `cargo llvm-cov -p` report under-reports hits that other crates' tests contribute in CI's whole-workspace run, so it manufactures fake waivers; pull the `coverage-report` artifact (`coverage.yml:151-154`, `gh run download <id> --name coverage-report`) and intersect `DA:<line>,0` with `git diff -U0` patch hunks instead. Check `#[cfg(not(feature = "…"))]` polarity before waiving (that arm *is* built), and try destructive setup (`DROP TABLE` after a successful write on a `new_local` DB) before calling an error arm unreachable — direct-connection mode hands out a fresh prepared-cache id per checkout (`prepared/cache.rs:102-108`), so races really cannot be built there while schema invalidation can (LESSON-062).
- **Documented affordances that do not exist (2026-10-17, LESSON-063)**: four guides told agents to run `cargo install parse-changelog-cli` — that crate 404s; the real one is `parse-changelog`. `verify-release-state.sh --help` also advertised `--fix  Auto-fix version references`, but its `FIX_MODE` was read nowhere, so `--fix` exited 0 having repaired nothing. Before writing an install command or a flag into a guide, verify it: `curl -sS -A "<ua>" https://crates.io/api/v1/crates/<name>` (the `-A` is mandatory, LESSON-053) and grep the script for whether the flag's variable is ever read. Delete a dead flag rather than leaving a documented lie.

## Dev Harness (do-harness)

Pinned compatible harness version: `v0.1.0`. Sensors live in `do-harness.toml` and map to guides in `HARNESS.md`.
`do-harness.toml` defines two explicit signal sets:
- **`feedback`** (`do-harness verify --set feedback`): fast local edit loop (`fmt`, `loc`, `check`).
- **`verification`** (`do-harness verify --set verification --strict --record && do-harness status --set verification`): full pre-completion contract covering formatting, compilation, clippy, LOC, dependency deny, doctests/doc-build (`docs`), normal tests (`test`), CSM feature tests (`csm-feature`), ignored tests, lint suppressions, frontmatter, storage backend capabilities, and quality gates.

`do-harness verify --set verification --strict --record` runs the full verification suite and persists beats; `verify --only <sensor>` re-runs one; `do-harness task done <id>` refuses until its sensor passed. Feedback set is an edit-loop tool and is **not** completion evidence. Sensor fired? Fix that sensor first, then commit.

**Do NOT run `do-harness hook install`** — `.pre-commit-config.yaml` owns the git hooks, installed via `./scripts/install-hooks.sh`. A fresh clone has none installed (`.git/hooks` holds only `.sample` files), so every declared sensor is inert until that script runs; `./scripts/install-hooks.sh --verify` reports which are missing. The commit stage is the cheap subset (fmt, loc, file hygiene, shellcheck, yamllint) — clippy is `stages: [manual]` (`pre-commit run --hook-stage manual --all-files`), and `commitlint` needs the **commit-msg** hook type, which plain `pre-commit install` does not create. Worktrees share the common hooks dir, so one install covers them all. `do-harness init` must never inject the skill-creator scripts or ignore `.agents/events/`. Steering loop, metrics events and the fired-sensor runbook: [`.agents/skills/harness/SKILL.md`](.agents/skills/harness/SKILL.md).

## Required Checks Before Commit

- [ ] `do-harness verify --set feedback` (cheap edit-loop check during coding)
- [ ] `do-harness verify --set verification --strict --record` (pre-completion contract: fmt, check, clippy, loc, deny, docs, test, csm-feature, ignored-tests, lint-suppressions, yaml-frontmatter, storage-backend-capabilities, quality-gates)
- [ ] `do-harness status --set verification` (confirm green local evidence boundary)
- [ ] `git status` — only intended changes staged

## PR Health Check

Before recommending or performing a merge, verify **all** of: `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`, every required check terminal and green, and every conversation/review thread (human **and** bot — Codacy/Codecov included) addressed or waived with evidence on the thread. Use the `pr-readiness` skill + `./scripts/check-pr-readiness.sh`; merge only via `./scripts/merge-pr.sh <PR> --execute`. **Never** `--admin`, `--force`, or any bypass.

## Release Process

One path only: skill `release-guard`, CLI `./scripts/release-manager.sh ship --execute` (tag `v` + workspace version → `.github/workflows/release.yml` publishes draft-first with attestations). **NEVER** manual `gh release create`, tagging off `main`, or `--admin` merges. Version + CHANGELOG + `Released Version` docs land on `main` (green CI) first; post-release, bump the workspace in a follow-up PR.

## Release Cadence Management

Skill `release-cadence-manager`, CLI `./scripts/release-cadence-manager.sh`: `detect` → `resolve --pr <n>` → `validate`. Critical reasons: `version_not_advanced`, `tag_not_ancestor`, `invalid_next_version`, `commit_limit` (≥30 unreleased commits), `age_limit` (≥14 days), `no_release_tag`.

## Tool Selection Enforcement

Search with `grep`/`glob`/`find`, read with `read`; `bash` is for binaries and short pipelines, never for file edits or paged output. Target Bash:Grep ratio 2:1 — think "would a search tool answer this better?" first. Details: [`agent_docs/token_efficiency.md`](agent_docs/token_efficiency.md), [`agent_docs/common_friction_points.md`](agent_docs/common_friction_points.md).

## Monitoring

MCP observability (`get_metrics`, `health_check`) and MCP tool contracts: [`docs/API_REFERENCE.md`](docs/API_REFERENCE.md) + [`.agents/skills/do-memory-mcp/SKILL.md`](.agents/skills/do-memory-mcp/SKILL.md).

## Cross-References

| Topic | Document |
|---|---|
| Complete workflow | [`agent_docs/coding_workflow.md`](agent_docs/coding_workflow.md) |
| Build | `agent_docs/building_the_project.md` |
| Tests / coverage | `agent_docs/running_tests.md` |
| Code style, security, perf budgets | `agent_docs/code_conventions.md` |
| Git workflow | `agent_docs/git_workflow.md` |
| CI guidance | `agent_docs/ci_guidance.md` |
| GH Actions patterns, CI optimization, publish | `agent_docs/github_actions_patterns.md` |
| Dependencies | `agent_docs/dependency_upgrades.md` |
| Architecture | `agent_docs/service_architecture.md` |
| Database | `agent_docs/database_schema.md` |
| Friction points | `agent_docs/common_friction_points.md` |
| CSM cascade retrieval | `agent_docs/csm_integration.md` |
| Coverage waivers | `.agents/skills/coverage-waivers/SKILL.md` |
| Token efficiency | `agent_docs/token_efficiency.md` |
| Disk hygiene | `agent_docs/disk_hygiene.md` |
| Lessons log | `agent_docs/LESSONS.md` |
| Planning | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| GOAP state | `plans/GOAP_STATE.md` |
| ADRs | `plans/adr/` |
