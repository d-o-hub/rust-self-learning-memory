# Agent Coding Guidelines

**Skills Location**: Skills are at `.agents/skills/` (canonical path).

## Quick Reference
- **Build**: `./scripts/build-rust.sh dev|release|check|clean`
- **Quality**: `./scripts/code-quality.sh fmt|clippy|audit|check`
- **Tests**: `cargo nextest run --all` (doctests: `cargo test --doc`)
- **Quality Gates**: `./scripts/quality-gates.sh`
- **PR Readiness**: `./scripts/check-pr-readiness.sh [--fix] [PR_NUMBER]`
- **Disk Cleanup**: `./scripts/clean-artifacts.sh [quick|standard|full] [--node-modules]`

Memory system: Rust/Tokio + Turso + redb + embeddings (OpenAI/Cohere/Ollama/local)
Crates: do-memory-core, do-memory-storage-turso, do-memory-storage-redb, do-memory-mcp, do-memory-cli, do-memory-test-utils, benches

## Skill + CLI Pattern (CRITICAL)
Always use Skill + CLI first for high-frequency ops:
| Operation | Skill | CLI |
|-----------|-------|-----|
| Build | `build-rust` | `./scripts/build-rust.sh` |
| Format/Lint | `code-quality` | `./scripts/code-quality.sh` |
| Tests | `test-runner` | `cargo nextest run --all` + `cargo test --doc` |
| Debug | `debug-troubleshoot` | - |

Before task tool: skill? → script? → Skill+CLI? → task tool?

## Change Workflow
1. Identify owner crate + module
2. Read existing patterns
3. Add/update tests
4. `./scripts/code-quality.sh fmt`
5. `./scripts/code-quality.sh clippy --workspace`
6. `cargo nextest run -p <crate>`
7. `cargo nextest run --all`
8. `cargo test --doc`
9. `./scripts/quality-gates.sh` (coverage threshold is `QUALITY_GATE_COVERAGE_THRESHOLD`, default 90)
10. `git status` - verify all changes staged

## Core Invariants (Never Break)
- **Async**: Tokio everywhere. No blocking (use `spawn_blocking`)
- **Storage**: Parameterized SQL only. Short transactions. No locks across `.await`
- **Serialization**: Postcard required (not bincode)
- **Clippy**: Zero warnings (`-D warnings`). Fix, don't suppress
- **Files**: ≤500 LOC per source file
- **Tests**: ≥90% coverage. `#[tokio::test]` for async. AAA pattern
- **Docs**: URLs wrapped in `<...>`. New types re-exported from `lib.rs`

## Documentation Rules
- Wrap URLs in angle brackets, re-export new public types from `lib.rs`, and run `cargo doc --no-deps --document-private-items` before commit

## Common Pitfalls

- **Coderabbitai review loops**: Always `read_files` on the target file before acting on a finding. Trust current code, not conversation summaries or cached search results. Fix history may not match current tree.
- **Feature-gated imports in tests**: When adding `#[cfg(feature = "X")]` tests, also gate ALL imports, structs, and impls used exclusively by those tests. CI runs clippy without features and will reject ungated dead code. One ungated import in `tests/` blocks all PRs.
- Read patterns first; roadmap and status docs can lag real repo state.
- Verify release/package reality with `gh release view` and `cargo metadata` before editing version plans.
- Update `ROADMAP_ACTIVE.md`, `GOALS.md`, `ACTIONS.md`, `GOAP_STATE.md`, and `STATUS/CURRENT.md` together when sprint priorities change.
- For CPU/token work, use `goap-agent` first, then `agent-coordination`, then the implementation/validation skills.
- **Audit file accounting**: After opening an existing audit log, seed size tracking from file metadata (never reset to 0) or rotation will not fire until the new-write sum alone exceeds max size.
- **Skill / gate CI**: After changing skills or `GATE_CONTRACT.md`, run `./scripts/run-evals.sh --fixtures` and `./scripts/validate-gate-contract.sh --ci-parity` locally; Skill Evals workflow enforces both on PRs.

Before implementing: Read 3+ source files, check ADRs

## Planning & Decisions
- **Use `goap-agent` skill** for complex tasks - decomposes into atomic goals
- **Use `agent-coordination`** when CPU/token or release/doc work can run in parallel
- **Check `plans/adr/`** for Architecture Decision Records before changes
- **Update `plans/ROADMAPS/ROADMAP_ACTIVE.md`** with progress
- **Keep `agent_docs/LESSONS.md` + `AGENTS.md` aligned** when recording non-obvious workflow learnings

## Tool Selection Enforcement

Target Bash:Grep ratio of 2:1 (current: 17:1)

**Use Grep for**:
- Finding files: `Grep pattern="*.rs"`
- Searching content: `Grep pattern="fn name"`
- Finding definitions: `Grep pattern="struct Name"`
- Checking usage: `Grep pattern="use crate"`

**Use Bash for**:
- File operations: `cp`, `mv`, `rm`
- Git commands: `git status`, `git diff`
- Running scripts: `./scripts/*.sh`
- Running workspace tests: `cargo nextest run --all`, `cargo test --doc`

**Before Bash**: Consider if Grep would be more efficient.

## Atomic Change Rules
1. **One change per commit** - message describes exactly what changed
2. **Workflow**: make change → test → quality check → verify → commit
3. **Format**: `feat(module): description`, `fix(module): description`
4. Never batch incomplete work

## Required Checks Before Commit
- [ ] `./scripts/code-quality.sh fmt`
- [ ] `./scripts/code-quality.sh clippy --workspace`
- [ ] `./scripts/build-rust.sh check`
- [ ] `cargo nextest run --all`
- [ ] `cargo test --doc`
- [ ] `cargo doc --no-deps --document-private-items` (catches bare URLs)
- [ ] `./scripts/quality-gates.sh` (coverage must be `>=90%`, unless threshold explicitly raised)
- [ ] `git status` - verify all changes staged

## Git Workflow
- **Branch Protection**: Direct pushes to `main` BLOCKED. Always work on a branch.
- See `agent_docs/git_workflow.md` for details.

## PR Health Check (MANDATORY before recommending merge)

**Skill**: `.agents/skills/pr-readiness/SKILL.md`  
**CLI**: `./scripts/check-pr-readiness.sh [--fix] [PR_NUMBER]`

When analyzing open PRs, reviewing a PR, or recommending merge, you MUST check **all** of the following — not just CI status:

1. Merge state (`mergeable`, `mergeStateStatus`)
2. CI / status checks (including CANCELLED)
3. **All PR comments and reviews** (human + bots) — and **address** actionable feedback

### Step 1: Full PR State Query
```bash
gh pr view {n} --json number,title,mergeable,mergeStateStatus,statusCheckRollup,headRefOid
# or all open:
gh pr list --state open --json number,title,mergeable,mergeStateStatus,statusCheckRollup
```

### Step 2: Interpret Merge State (CRITICAL)
| `mergeStateStatus` | `mergeable` | Meaning | Action Required |
|--------------------|-------------|---------|-----------------|
| `CLEAN` | `MERGEABLE` | Merge state OK | Still verify CI **and** comments |
| `BEHIND` | `MERGEABLE` | Branch behind main, no conflicts | Update branch: `gh api repos/{owner}/{repo}/pulls/{n}/update-branch -X PUT` |
| `BLOCKED` | `MERGEABLE` | Required checks still pending | Wait for CI to complete |
| `UNSTABLE` | `MERGEABLE` | Non-required checks failing | Check if failures are pre-existing/non-blocking |
| `DIRTY` | `CONFLICTING` | **Merge conflicts** | Checkout branch, merge main, resolve conflicts, push |
| `HAS_HOOKS` | varies | Pre-receive hooks blocking | Investigate hook failures |

### Step 3: CI Check Interpretation
- **`SUCCESS`** → ✅ Pass
- **`SKIPPED`** → ✅ Expected (e.g., Release workflow on PRs)
- **`CANCELLED`** → ⚠️ Investigate: may be dependent on a failed/cancelled prerequisite. Re-run if stale.
- **`FAILURE`** → ❌ Must fix before merge
- **`pending`** → ⏳ Wait for completion (do NOT recommend merge while pending)

### Step 4: Fetch and Address ALL PR Comments (MANDATORY)

**Never skip this step.** "No human reviews" does **not** mean nothing to do — bots (Codecov, Codacy, CodeRabbit, etc.) post actionable conversation comments.

```bash
OWNER_REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
# Inline file/line review comments
gh api "repos/${OWNER_REPO}/pulls/{n}/comments" --paginate
# Submitted reviews (APPROVED / CHANGES_REQUESTED / COMMENTED)
gh api "repos/${OWNER_REPO}/pulls/{n}/reviews" --paginate
# Issue conversation (Codecov, Codacy, humans)
gh api "repos/${OWNER_REPO}/issues/{n}/comments" --paginate
```

| Feedback type | Action |
|---------------|--------|
| Human inline / `CHANGES_REQUESTED` | **Fix code** (or get re-approval after evidence waiver) |
| Codecov low patch / missing lines | Add tests for listed files; push; reply on PR |
| Codacy new issues | Fix cited lints/complexity |
| Stale / incorrect comment | Reply with commit SHA or counter-evidence |
| Informational bot only (e.g. bench dump) | Note in report; no code change required |

**Address** means: implement + test + push, then reply on the thread — not just acknowledge.

### Step 5: Fix Everything
A PR is NOT ready to merge unless ALL of:
1. `mergeable` = `MERGEABLE` (no conflicts)
2. `mergeStateStatus` = `CLEAN` (not BEHIND, DIRTY, BLOCKED, or UNSTABLE)
3. All **required** checks = `SUCCESS`
4. No `CANCELLED` checks that should have run (re-trigger if needed)
5. No pre-existing failures inherited from base branch
6. **All actionable PR comments addressed** (or waived with evidence on the thread)

### Step 6: Fix Procedures
| Problem | Fix |
|---------|-----|
| Branch behind (`BEHIND`) | `gh api repos/{owner}/{repo}/pulls/{n}/update-branch -X PUT -f update_method=merge` |
| Merge conflicts (`DIRTY`/`CONFLICTING`) | `gh pr checkout {n}` → `git merge origin/main` → resolve → `git push` |
| Cancelled CI | Re-run: `gh run rerun {run_id}` or push empty commit to re-trigger |
| Failed check | Diagnose with `gh run view {run_id} --log-failed`, fix code, push |
| Review / bot comments | See Step 4 — fix code, push, reply on PR |

### Common Mistake (NEVER DO THIS)
❌ "CI is green, ready to merge" — **WRONG** if `mergeStateStatus` ≠ `CLEAN` **or** comments remain unaddressed  
✅ "CI green, merge CLEAN, all review/bot feedback addressed → ready to merge" — **CORRECT**

CI checks run against the branch HEAD, not against the merge result. A PR can have green CI but be unmergeable due to conflicts with main. Codecov/Codacy conversation comments often require tests or code fixes even when required checks are green.

### Hard Blockers (NEVER BYPASS)

These rules have NO exceptions. Do not use `--admin`, `--force`, or any bypass mechanism:

1. **NEVER merge when `mergeStateStatus` is `UNSTABLE`** — Wait for all checks to complete. `UNSTABLE` means non-required checks are failing/pending. Investigate before merging.
2. **NEVER merge when checks are `pending`** — Wait for ALL checks to reach a terminal state (`SUCCESS`, `FAILURE`, `SKIPPED`).
3. **NEVER use `gh pr merge --admin` to bypass branch protection** — Branch protection exists for a reason. If checks fail, fix the code, don't bypass the gate.
4. **NEVER merge a PR you haven't personally verified** — Run the full PR readiness check (`gh pr view {n} --json mergeable,mergeStateStatus,statusCheckRollup`) and confirm ALL conditions before merging.
5. **NEVER skip the `pr-readiness` skill** — Load `.agents/skills/pr-readiness/SKILL.md` before any merge recommendation. The skill defines the exact verification procedure.
6. **NEVER ignore PR comments** — Fetch inline reviews, review bodies, and issue comments; address actionable feedback (including Codecov/Codacy) before declaring ready.

### Mandatory Pre-Merge Checklist

Before ANY merge action, ALL of these must be true:

```
□ Loaded pr-readiness skill
□ Ran: gh pr view {n} --json mergeable,mergeStateStatus,statusCheckRollup
□ mergeable = MERGEABLE
□ mergeStateStatus = CLEAN
□ All required checks = SUCCESS (not pending, not FAILURE)
□ No CANCELLED checks that should have run
□ No pre-existing failures from base branch
□ Fetched PR comments: pulls/{n}/comments + pulls/{n}/reviews + issues/{n}/comments
□ Actionable feedback addressed (code + tests pushed) or waived with evidence on thread
□ No open CHANGES_REQUESTED without re-approval
□ Verified locally: cargo nextest run --all passes
□ Verified locally: cargo clippy --workspace -- -D warnings passes
```

## Release Process (MANDATORY — one path only)

**Skill:** `.agents/skills/release-guard/SKILL.md`  
**CLI:** `./scripts/release-manager.sh ship --execute`  
**GitHub Release:** tag push → `.github/workflows/release.yml` (never `gh release create`)

```bash
# After version + CHANGELOG + Released Version docs are on main and main CI is green:
git checkout main && git pull --ff-only
./scripts/release-manager.sh status
./scripts/release-manager.sh ship --execute
./scripts/release-manager.sh wait-release   # optional poll
```

**NEVER** manually `gh release create`, tag off main, or `--admin` merge.  
**Tag format:** `v` + workspace `Cargo.toml` version (must match).

Feature flags: `openai`, `local-embeddings`, `turso`, `redb`, `embeddings-full`, `full`, `csm`

## CSM Integration

Enable CPU-local cascading retrieval with the `csm` feature flag:
```bash
cargo build --features csm
```

**Available types when enabled:**
- `Bm25Index` - First-tier keyword search (no API calls)
- `HVec10240` - 10,240-bit HDC vectors for similarity
- `ConceptGraph` - Ontology expansion for synonym matching
- `CascadeRetriever` - Tier escalation orchestration

**Docs**: `agent_docs/csm_integration.md` for full cascade pipeline (WG-128 through WG-131).

## Release Process

Same as **Release Process (MANDATORY — one path only)** above: skill `release-guard`, CLI `./scripts/release-manager.sh ship --execute`, GitHub `release.yml` on tag push.

**Future (2026)**: Migrate to Trusted Publishing (OIDC) to eliminate `CARGO_REGISTRY_TOKEN` secret.
See <https://crates.io/docs/trusted-publishing> for setup.

## Security
- Use env vars (never hardcode)
- Parameterized SQL
- **OAuth/JWT**: Always use `jsonwebtoken` with signature verification. Mandatory `MCP_OAUTH_TOKEN_SECRET` for production HMAC verification.

Environment variables: `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`, `OPENAI_API_KEY`, `RUST_LOG`, `MCP_OAUTH_TOKEN_SECRET`
Local dev: set `TURSO_DATABASE_URL="http://127.0.0.1:8080"` and leave `TURSO_AUTH_TOKEN` empty when using `turso dev`.

## Performance Targets
- Episode Creation: < 50ms | Step Logging: < 20ms
- Episode Completion: < 500ms | Memory Retrieval: < 100ms

## CI Optimization (2026-04-28)

PR CI time reduced from ~50+ min to ~15-18 min via paths-based benchmark triggering.

| Job | Time | Trigger |
|-----|------|---------|
| Quick Check | ~7 min | All PRs |
| Tests | ~12 min | All PRs |
| MCP Build | ~10 min | All PRs |
| Multi-Platform | ~12-15 min | All PRs |
| Run Benchmarks | ~54 min | **Only perf-critical paths** |

**Perf-critical paths** (trigger benchmarks):
- `memory-core/src/**/*.rs`
- `memory-storage-turso/src/**/*.rs`
- `memory-storage-redb/src/**/*.rs`
- `memory-mcp/src/**/*.rs`
- `benches/**`
- `Cargo.toml`, `Cargo.lock`
- `.github/workflows/benchmarks.yml`

**Skip benchmarks manually**: Add `skip-benchmarks` label to PR.

**Manual trigger**: Use `workflow_dispatch` in Actions UI.

**Main branch**: Benchmarks always run with regression detection.

**Key insight**: GitHub Actions doesn't support `paths` + `paths-ignore` at same trigger level - use `paths` only.

**Related skills**:
- `.agents/skills/github-workflows/SKILL.md` - Workflow patterns and troubleshooting
- `.agents/skills/ci-fix/SKILL.md` - CI failure diagnosis

See `plans/GOAP_CI_OPTIMIZATION_2026-04-28.md` for full plan.

### Publish Pipeline (2026-07-08)

Publish improvements (PR #789):
- `cargo publish --locked` for reproducibility
- Sparse-index polling (max 5 min) replaces `sleep 30`
- Explicit `needs` chain: core → redb → turso → cli
- Semver check output surfaced in `$GITHUB_STEP_SUMMARY`

## Cross-References
| Topic | Document |
|-------|----------|
| Build | `agent_docs/building_the_project.md` |
| Tests | `agent_docs/running_tests.md` |
| Code style | `agent_docs/code_conventions.md` |
| Git workflow | `agent_docs/git_workflow.md` |
| CI guidance | `agent_docs/ci_guidance.md` |
| Dependencies | `agent_docs/dependency_upgrades.md` |
| GH Actions | `agent_docs/github_actions_patterns.md` |
| Architecture | `agent_docs/service_architecture.md` |
| Database | `agent_docs/database_schema.md` |
| Patterns | `agent_docs/service_communication_patterns.md` |
| Friction points | `agent_docs/common_friction_points.md` |
| Disk hygiene | `agent_docs/disk_hygiene.md` |
| Token efficiency | `agent_docs/token_efficiency.md` |
| Lessons log | `agent_docs/LESSONS.md` |
| Planning | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| GOAP state | `plans/GOAP_STATE.md` |
| ADRs | `plans/adr/` |
| Trusted Publishing | `plans/adr/` (future ADR) |

## Disk Space
- **No Temporary Files in Root**: Never create temporary files, logs, trial outputs, or one-off scripts (`.py`, `.sh`, etc.) in the repository root. Use `plans/` for design-related notes, `target/` for build/test artifacts, or `scripts/` for reusable tooling.
- Dev profile: `debug = "line-tables-only"`, deps `debug = false`
- Default artifact path: `target/` (or `$CARGO_TARGET_DIR` when set)
- For external disk/offload, set `CARGO_TARGET_DIR` (for example: `CARGO_TARGET_DIR=/mnt/fastssd/rslm-target`)
- Use `./scripts/clean-artifacts.sh standard` for routine cleanup
- Use `./scripts/clean-artifacts.sh standard --node-modules` only when JS dependencies are not needed locally

## MCP Server Interaction Patterns
- The MCP server implements lazy loading of tools (ADR-024) to optimize initialization.
- The server exposes a suite of tools defined in `docs/API_REFERENCE.md`, including:
  - **Core and Monitoring**: `query_memory`, `analyze_patterns`, `health_check`, `get_metrics`
  - **Pattern / Recommendation / Explainability**: `advanced_pattern_analysis`, `quality_metrics`, `search_patterns`, `recommend_patterns`, `recommend_playbook`, `explain_pattern`
  - **Recommendation Attribution / Feedback**: `record_recommendation_session`, `record_recommendation_feedback`, `get_recommendation_stats`
  - **Playbook / Checkpoint / Handoff**: `checkpoint_episode`, `get_handoff_pack`, `resume_from_handoff`
  - **Episode Lifecycle**: `bulk_episodes`, `create_episode`, `add_episode_step`, `complete_episode`, `get_episode`, `delete_episode`, `update_episode`, `get_episode_timeline`
  - **Episode Tags**: `add_episode_tags`, `remove_episode_tags`, `set_episode_tags`, `get_episode_tags`, `search_episodes_by_tags`
  - **Episode Relationships**: `add_episode_relationship`, `remove_episode_relationship`, `get_episode_relationships`, `find_related_episodes`, `check_relationship_exists`, `get_dependency_graph`, `validate_no_cycles`, `get_topological_order`
  - **Embeddings**: `configure_embeddings`, `query_semantic_memory`, `test_embeddings`, `generate_embedding`, `search_by_embedding`, `embedding_provider_status`
  - **Unavailable / fail-closed**: `execute_agent_code` (WASM sandbox removed; calls fail closed and the tool is not a working execution backend).
- Note: Batch tools (`batch_query_episodes`, `batch_pattern_analysis`, `batch_compare_episodes`) are intentionally absent/deferred and will not resolve.

## Storage Optimization (Batch Eviction)
- Capacity eviction in Turso uses batch 'DELETE' with 'IN (...)' clauses for episodes and embeddings to avoid N+1 query overhead.
- Multi-dimensional embeddings must be cleared via 'delete_embeddings_batch_dimension_aware' to ensure all sharded tables are purged.
