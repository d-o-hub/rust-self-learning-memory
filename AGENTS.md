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
| Harness | `do-harness verify --record` · `do-harness doctor` |

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
| Harness sensors | `harness` | `do-harness verify --record` |
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

## Dev Harness (do-harness)

Sensors live in `do-harness.toml` (fmt, check, clippy, test, deny, loc) and map to guides in `HARNESS.md`. `do-harness verify --record` runs the suite and persists beats; `verify --only <sensor>` re-runs one; `do-harness task done <id>` refuses until its sensor passed. Sensor fired? Fix that sensor first, then commit.

**Do NOT run `do-harness hook install`** — `.pre-commit-config.yaml` owns `.git/hooks/pre-commit` (cheap sensor subset: fmt + loc). `do-harness init` must never inject the skill-creator scripts or ignore `.agents/events/`. Steering loop, metrics events and the fired-sensor runbook: [`.agents/skills/harness/SKILL.md`](.agents/skills/harness/SKILL.md).

## Required Checks Before Commit

- [ ] `./scripts/code-quality.sh fmt`
- [ ] `./scripts/code-quality.sh clippy --workspace`
- [ ] `./scripts/build-rust.sh check`
- [ ] `cargo nextest run --all`
- [ ] `cargo test --doc`
- [ ] `cargo doc --no-deps --document-private-items`
- [ ] `./scripts/quality-gates.sh`
- [ ] `do-harness verify --record`
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
