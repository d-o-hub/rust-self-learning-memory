# Agent Coding Guidelines

Entrypoint for coding agents. **Complete workflow:** [`agent_docs/coding_workflow.md`](agent_docs/coding_workflow.md) — **Docs index:** [`agent_docs/README.md`](agent_docs/README.md) — **Skills:** `.agents/skills/` (routed by `.agents/skills/skill-rules.json`; full inventory `.agents/SKILLS.md`).

Rust/Tokio episodic-memory workspace: `do-memory-core`, `do-memory-storage-turso`, `do-memory-storage-redb`, `do-memory-mcp`, `do-memory-cli`, `do-memory-test-utils`, `benches`. Storage: Turso + redb + embeddings (OpenAI/Cohere/Ollama/local).

**Workspace version**: `v0.1.48` (development) · **Latest released tag**: `v0.1.47` (`854f6f8d`, 2026-10-28). A workspace version ahead of the latest tag is a *development bump*, not a release — see **Workflow Traps** → "A version bump does not make the repo shippable" (LESSON-057).

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
- **Serialization**: Postcard required (not bincode). Any type that reaches postcard storage must stay **externally tagged**: postcard cannot deserialize `#[serde(tag = "...")]` internally-tagged enums, which made stored patterns unreadable (issue #831, LESSON-017). Internally tagged enums are fine for JSON-only wire types that never hit storage.
- **Clippy**: zero warnings (`-D warnings`). Fix, don't suppress.
- **Files**: ≤500 LOC per source file (enforced in CI).
- **Tests**: ≥70% coverage floor (90% target); `#[tokio::test]` for async; AAA pattern.
- **Docs**: URLs wrapped in `<...>`; new public types re-exported from `lib.rs`.
- **Fail-closed code execution**: `execute_agent_code` is unavailable and fails closed.
- **Dependency alignment**: version comments in `Cargo.toml` are constraints (Dependabot does not read them). `libsql` must resolve to one version per graph — `chaotic_semantic_memory` pins 0.9.x and two libsql versions duplicate its exported C symbols under `csm`. Any requirement change also re-resolves `fuzz/Cargo.lock` (`cd fuzz && cargo metadata --format-version 1 >/dev/null`), else the audit gate fails with *0 vulnerabilities*. Sensor: `lockfile-sync`. See `agent_docs/dependency_upgrades.md`, LESSON-077.
- **Commit messages**: never write a GitHub CI skip-control token in a commit message or PR body, not even inside prose or backticks — the matcher scans the whole head message, so the branch gets zero runs and the merge stays `BLOCKED` with nothing pending; a squash-merge also copies the body onto main, blinding the commit the release gate counts. Describe it as a *skip-ci marker*. Sensor + commit-msg hook: `commit-ci-tokens`. See LESSON-065.
- **Build isolation**: do not share one `CARGO_TARGET_DIR` across worktrees while tests run — a sibling build replaces the test binaries under a running suite and produces fake mass failures. Re-run in an isolated target dir before believing any mass test failure, and keep per-worktree dirs (`agent_docs/disk_hygiene.md`, LESSON-038).

## Workflow Traps (2026-10, full evidence in `agent_docs/LESSONS.md`)

Before merges or releases, review LESSON-035 and LESSON-039…080. These cover
stale-tip/dependency loss, toolchain/test/CI-rollup failures, release event
filtering, advisory databases, version state, publish-chain traps, Dependabot
group bumps carrying a member that must be dropped (LESSON-077), self-suppressed
CI from a commit-message token (LESSON-065), concluding "CI never ran" from one
Actions query a minute after a push (LESSON-080), and routing docs that assert
checks which do not exist (LESSON-063, LESSON-079).

## Dev Harness (do-harness)

Pinned compatible harness version: `v0.1.0`. Sensors live in `do-harness.toml` and map to guides in `HARNESS.md`.
`do-harness.toml` defines two explicit signal sets:
- **`feedback`** (`do-harness verify --set feedback`): fast local edit loop (`fmt`, `loc`, `check`, `commit-ci-tokens`, `agents-md-ids`).
- **`verification`** (`do-harness verify --set verification --strict --record && do-harness status --set verification`): full pre-completion contract covering formatting, compilation, clippy, LOC, dependency deny, lockfile freshness of both graphs and the `libsql` single-version alignment (`lockfile-sync`), changelog heading integrity (`changelog-headings`), shell counter ratchet (`shell-counters`), lesson-routing ratchet (`lesson-distillation`), commit-message CI tokens (`commit-ci-tokens`), AGENTS.md numeric-run ceiling (`agents-md-ids`), doctests/doc-build (`docs`), normal tests (`test`), CSM feature tests (`csm-feature`), ignored tests, lint suppressions, frontmatter, storage backend capabilities, and quality gates.

`do-harness verify --set verification --strict --record` runs the full verification suite and persists beats; `verify --only <sensor>` re-runs one; `do-harness task done <id>` refuses until its sensor passed. Feedback set is an edit-loop tool and is **not** completion evidence. Sensor fired? Fix that sensor first, then commit.

**Do NOT run `do-harness hook install`** — `.pre-commit-config.yaml` owns the git hooks, installed via `./scripts/install-hooks.sh`. A fresh clone has none installed (`.git/hooks` holds only `.sample` files), so every declared sensor is inert until that script runs; `./scripts/install-hooks.sh --verify` reports which are missing. The commit stage is the cheap subset (fmt, loc, file hygiene, shellcheck, yamllint) — clippy is `stages: [manual]` (`pre-commit run --hook-stage manual --all-files`), and `commitlint` needs the **commit-msg** hook type, which plain `pre-commit install` does not create. Worktrees share the common hooks dir, so one install covers them all. `do-harness init` must never inject the skill-creator scripts or ignore `.agents/events/`. Steering loop, metrics events and the fired-sensor runbook: [`.agents/skills/harness/SKILL.md`](.agents/skills/harness/SKILL.md).

## Required Checks Before Commit

- [ ] `do-harness verify --set feedback` (cheap edit-loop check during coding)
- [ ] `do-harness verify --set verification --strict --record` (pre-completion contract: fmt, check, clippy, loc, deny, lockfile-sync, changelog-headings, shell-counters, lesson-distillation, commit-ci-tokens, agents-md-ids, docs, test, csm-feature, ignored-tests, lint-suppressions, yaml-frontmatter, storage-backend-capabilities, quality-gates)
- [ ] `do-harness status --set verification` (confirm green local evidence boundary)
- [ ] `git status` — only intended changes staged

## PR Health Check

Before recommending or performing a merge, verify **all** of: `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`, every required check terminal and green, and every conversation/review thread (human **and** bot — Codacy/Codecov included) addressed or waived with evidence on the thread. Use the `pr-readiness` skill + `./scripts/check-pr-readiness.sh`; merge only via `./scripts/merge-pr.sh <PR> --execute`. **Never** `--admin`, `--force`, or any bypass.

On a bot or long-lived branch, `MERGEABLE` says nothing about lost work. Diff the
head against its **merge base**, not the moving `main` tip
(`MB=$(git merge-base origin/main HEAD); git diff --name-only "$MB" HEAD`), and
treat any `Cargo.toml` / `Cargo.lock` hunk you did not author as a revert: a
`Merge branch 'main'` resolution on a bot branch rolled back a dependency bump
that CI then re-shipped (LESSON-064, LESSON-035, LESSON-074).

## Release Process

One path only: skill `release-guard`, CLI `./scripts/release-manager.sh ship --execute` (tag `v` + workspace version → `.github/workflows/release.yml` publishes draft-first with attestations). **NEVER** manual `gh release create`, tagging off `main`, or `--admin` merges. Version + CHANGELOG + `Released Version` docs land on `main` (green CI) first; post-release, bump the workspace in a follow-up PR — drift treats workspace == latest-tag with unreleased commits as `version_not_advanced` (LESSON-020).

## Release Cadence Management

Skill `release-cadence-manager`, CLI `./scripts/release-cadence-manager.sh`: `detect` → `resolve --pr <n>` → `validate`. Critical reasons: `version_not_advanced`, `tag_not_ancestor`, `invalid_next_version`, `commit_limit` (≥30 unreleased commits), `age_limit` (≥14 days), `no_release_tag`.

## Tool Selection Enforcement

Search with `grep`/`glob`/`find`, read with `read`; `bash` is for binaries and short pipelines, never for file edits or paged output. Target Bash:Grep ratio 2:1 — think "would a search tool answer this better?" first. Details: [`agent_docs/token_efficiency.md`](agent_docs/token_efficiency.md), [`agent_docs/common_friction_points.md`](agent_docs/common_friction_points.md).

## Monitoring

MCP observability (`get_metrics`, `health_check`) and MCP tool contracts (`query_memory`, `analyze_patterns`, `advanced_pattern_analysis`): [`docs/API_REFERENCE.md`](docs/API_REFERENCE.md) + [`.agents/skills/do-memory-mcp/SKILL.md`](.agents/skills/do-memory-mcp/SKILL.md).

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
