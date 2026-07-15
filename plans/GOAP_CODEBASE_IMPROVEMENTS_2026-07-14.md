# GOAP Codebase, Workflow, Documentation, and Skills Improvement Plan

- **Status**: Proposed
- **Date**: 2026-07-14
- **Audit commit**: `a8b7d6d6a350c3f431b5564332b8a5c1365aefb9`
- **Audit branch**: `main`
- **Scope**: Planning only; this audit changes no implementation, workflow, root documentation, or skill files
- **Related decisions**: ADR-022, ADR-033, ADR-039, ADR-042, ADR-059, ADR-072, ADR-073, ADR-074

## 1. Objective

Move the repository from contradictory claims and partially enforced controls to an evidence-backed state in which:

1. security and data-correctness boundaries are real rather than descriptive;
2. local and CI quality gates fail when their advertised conditions fail;
3. release and publishing automation is reproducible and has one authority;
4. `.agents/skills/` is discoverable, current, and evaluated by executable tests;
5. README and canonical planning documents describe the implementation that exists; and
6. new memory-system features are piloted only after their baselines and acceptance metrics exist.

This is a GOAP backlog, not a statement that any listed remediation is already implemented.

## 2. Evidence rules

Findings use three confidence classes:

- **Verified local**: observed in files or commands at the audit commit.
- **Remote-unverified**: GitHub release, PR, tag, CI-run, or crates.io state not queried during this audit. These facts must not be inferred from plans.
- **Proposal**: a new feature or design whose value must be established by a spike or benchmark.

Every future completion claim must record `{command, scope, commit, timestamp, result, artifact}`. Test annotation counts, prose estimates, and old CI links are not equivalent to a current test run.

## 3. Verified current state

`cargo metadata --format-version 1 --no-deps` reports nine workspace packages, all at `0.2.0`: `do-memory-core`, `do-memory-storage-redb`, `do-memory-storage-turso`, `do-memory-test-utils`, `do-memory-mcp`, `do-memory-cli`, `do-memory-benches`, `e2e-tests`, and `do-memory-examples`.

| Area | Verified evidence | Consequence |
|---|---|---|
| Version truth | `Cargo.toml` and Cargo metadata report `0.2.0`; `plans/STATUS/CURRENT.md`, `plans/ROADMAPS/ROADMAP_ACTIVE.md`, and `plans/GOAP_STATE.md` still report `0.1.33`/`0.1.34` | Canonical plans cannot currently establish release state |
| Planning governance | ADR-039 limits the active plan set and defines canonical roles, but numerous dated GOAP files remain active; ADR identifiers 025 and 054 are duplicated; referenced ADR-061 is absent | Cross-references and status propagation are ambiguous |
| Coverage | `scripts/quality-gates.sh` and `tests/quality_gates.rs` default to 70% and skip optional gates; `AGENTS.md` says 90%; Codecov uses other targets | “Coverage passed” has no single meaning |
| Security audit | `.github/workflows/ci.yml` uses `cargo audit || echo`; `security.yml` uses `cargo audit ... || true` | Vulnerabilities or tool failures can produce a green job |
| Quality orchestration | `quality-gates.sh` runs doctest/docs/LOC checks and `tests/quality_gates.rs`, but not the advertised full build, nextest, and strict clippy sequence; workflows do not invoke it | Local and CI gates with the same name cover different surfaces |
| Package wrappers | `scripts/build-rust.sh` rejects hyphens although real packages are `do-memory-*`; `tests/quality_gates.rs` invokes nonexistent `memory-core` and can fall back to a baseline | Intended checks can fail to execute yet appear successful |
| Release | `release.yml` preflight checks tag/version equality only; historical plans and release skills contain manual `gh release create` instructions despite the automated-only policy | Release prerequisites are not enforced consistently |
| Publishing | `publish-crates.yml` omits `--locked`, uses fixed `sleep 30`, makes semver informational, and has no CLI publish job, contradicting current AGENTS claims | Reproducibility and documented pipeline order are not trustworthy |
| Legacy code execution | `memory-mcp/src/sandbox/mod.rs` contains a Node executor that relies on regex/wrapper restrictions and does not call the existing `apply_isolation` helper; however, MCP does not register a working execution tool, and `handle_execute_code` returns that code execution is no longer available. README still advertises Wasmtime and a nonexistent `wasmtime-backend` feature | Fail-closed MCP behavior must be preserved; dead/experimental code and public contracts must be reconciled before any new backend is exposed |
| Retrieval cache | `retrieve_relevant_context` keys cache entries by query/domain/limit while ranking also uses language, framework, tags, complexity, mode, and embedding state | Context-distinct queries can reuse incorrect cached rankings |
| Async storage | Immediate step persistence and buffered flush hold `episodes_fallback.write()` across backend awaits | Slow storage can block unrelated readers/writers and complicate concurrency |
| Capacity | Completion removes evictions from memory but explicitly logs that backend deletion is unimplemented | Capacity, retention, and deletion are not durable across restart/backfill |
| Local embeddings | Real-model load failure silently installs mock vectors, and `is_available` returns true when any model is loaded | Health can report available while semantic quality is invalid |
| Retry controls | `retry_queue_timeout` is configurable but unused; limiter permits cover first attempts and backoff sleeps | Backpressure semantics differ from the public configuration |
| Source boundaries | Current production files above 500 LOC include `memory-core/src/retry/mod.rs` (577), `memory-cli/src/commands/embedding.rs` (539), `memory-core/src/memory/retrieval/context.rs` (528), `memory-core/src/embeddings/local.rs` (507), and `memory-core/src/memory/checkpoint/operations.rs` (505) | The stated hard source-size invariant currently fails |
| Skills evals | 20 eval files execute only `true`; `pr-readiness` uses `evals` while the runner reads `tests`; `ci-poll` has no eval; missing/zero tests exit successfully | Skill confidence is largely synthetic |
| Skill routing | `skill-rules.json` has four rules covering three unique skills out of 33 | Critical skills rely on accidental discovery and broad keyword collisions |
| Skill policy | Release skills contradict automated-only releases; several skills use stale commands, environment variables, plan paths, tool names, or unsafe mutation defaults | Following a skill can violate repository or agent policy |
| README contract | The episode `TaskContext` example is stale; Wasmtime feature claims do not match `memory-mcp/Cargo.toml`; storage/provider defaults are inconsistent | New users receive uncompilable or inaccurate guidance |

## 4. GOAP state model

### Initial state

```text
truth_reconciled = false
remote_release_state_attempted = false
remote_release_state_verified = false
sandbox_capability_boundary = false
retrieval_identity_complete = false
storage_awaits_lock_free = false
durable_eviction = false
embedding_health_truthful = false
retry_backpressure_effective = false
gates_match_policy = false
release_pipeline_enforces_preconditions = false
skill_evals_executable = false
skill_routes_complete = false
docs_match_code = false
plan_registry_unique = false
feature_pilots_have_baselines = false
```

### Goal state

```text
truth_reconciled = true
remote_release_state_verified = true
critical_security_and_correctness_actions = complete
all_required_gates_are_blocking_and_reproducible = true
release_and_publish_authority_is_singular = true
skill_contracts_are_routed_and_executable = true
docs_match_generated_or_verified_contracts = true
plan_and_adr_registry_is_unambiguous = true
feature_pilots_have_go_no_go_evidence = true
```

## 5. Priority and dependency strategy

- **P0**: prevent security-boundary misrepresentation, incorrect retrieval, write contention/data loss, false-green gates, and unsafe releases.
- **P1**: make provider health, retry/audit behavior, benchmarks, skills, and documentation trustworthy.
- **P2**: run evidence-driven feature pilots after P0/P1 instrumentation exists.

Execution pattern: Phase 0 is sequential. Phase 1 actions may run in parallel after their ADR/precondition is accepted. Phase 2 workflow actions may run in parallel but converge on one parity check. Phase 3 skill and documentation work may run in parallel after the authority matrix is accepted. Phase 4 pilots are independent, but none may be called shipped without measured exit criteria.

Critical path:

```text
T0.1 -> T0.2 -> {S1.1, S1.2, S1.3, W2.1, K3.1}
S1.1 -> S1.5
S1.2 -> F4.1
S1.3 -> S1.4 -> F4.2
W2.1 -> W2.2 -> W2.3 -> W2.4
K3.1 -> K3.2 -> K3.3
{S1.*, W2.*, K3.*, D3.*} -> V5.1
selected F4.* -> V5.1 (only when a pilot is promoted)
```

## 6. Work packages and atomic action backlog

The named S/W/K/D/F entries below are work packages. Only the atomic child actions in section 6.1 may be scheduled or marked complete independently; a work package closes only when every required child and its validation evidence pass.

### Phase 0 — Truth freeze and decision authority

#### T0.1 — Capture a reproducible baseline

- **Priority**: P0
- **Preconditions**: none
- **Actions**:
  - record Cargo metadata, active features, current LOC failures, ignored tests, coverage output, audit output, and skill-eval results at one commit;
  - query git tags, GitHub releases/actions, and crates.io separately; label unavailable remote facts unknown;
  - store command outputs as CI artifacts or a dated validation record.
- **Effects**: `remote_release_state_attempted = true`; `remote_release_state_verified = true` only when tag, release, Actions, and crates.io queries succeed. An unavailable remote is recorded as a blocker, not verification.
- **Acceptance**: every number in canonical status has command, scope, commit, timestamp, and artifact; remote classification cannot proceed while `remote_release_state_verified = false`.

#### T0.2 — Accept ADR-072 and reconcile canonical plans

- **Priority**: P0
- **Depends on**: T0.1
- **Actions**:
  - adopt the authority matrix and automated-only release path;
  - decide whether `0.2.0` is unreleased development, a release candidate, or already released based on remote evidence;
  - update CURRENT, ROADMAP, GOALS, ACTIONS, GOAP_STATE, navigation, validation, and gap analysis together;
  - archive superseded dated plans per ADR-039;
  - create an ADR/WG registry without rewriting historical records; register duplicate 025/054 aliases and remove invalid ADR-061 claims.
- **Effects**: `truth_reconciled = true`; `plan_registry_unique = true`.
- **Acceptance**: one current version state, one release procedure, no unresolved ADR link, no contradictory active status.

### Phase 1 — Security and correctness

#### S1.1 — Preserve fail-closed execution and evaluate a capability boundary

- **Priority**: P0 security/contract accuracy
- **Depends on**: T0.2; ADR-073 accepted before implementing a new backend
- **Actions**:
  - keep `execute_agent_code` absent/unavailable and direct calls rejected while no approved backend exists;
  - correct README/tool metadata and either remove or quarantine the disconnected legacy Node executor as trusted-development-only code;
  - run a bounded Wasmtime/WASI feasibility spike; expose no new tool if the spike is rejected;
  - if approved, implement runtime-enforced network/filesystem/subprocess denial, resource limits, startup self-test, and pinned compiler/runtime artifacts.
- **Acceptance A — no backend**: tool discovery omits `execute_agent_code`, direct calls fail closed, and docs state unavailable.
- **Acceptance B — approved backend**: obfuscated network/filesystem/process probes fail at runtime; heap/CPU/output bombs are bounded; tool discovery reports backend and capabilities accurately.
- **Rollback**: unregister the new backend and return to the already fail-closed state; memory/query functionality remains available.

#### S1.2 — Make retrieval cache identity complete

- **Priority**: P0 correctness
- **Depends on**: T0.2; ADR-074 accepted
- **Actions**:
  - introduce typed `RetrievalRequestIdentity` including normalized context, mode, ranking config, provider/model, index generation, and limit;
  - use the full identity as the cache key and emit a versioned provenance fingerprint;
  - normalize tag order and invalidate/bump generation on relevant mutation.
- **Acceptance**: same query/domain with Rust/Axum and Python/Django contexts yields independent correct results in either request order; provider/index changes cannot reuse stale entries.

#### S1.3 — Remove lock guards from backend await paths

- **Priority**: P0 reliability
- **Depends on**: T0.2
- **Actions**:
  - snapshot/version episode state under a short lock, release it, perform I/O, then merge/commit with conflict detection;
  - define per-episode sequencing so moving awaits cannot lose concurrent steps;
  - add blocked-backend concurrency tests.
- **Acceptance**: while one backend store is held on a barrier, unrelated reads/writes complete; 100 concurrent unique steps persist exactly once; `await_holding_lock` remains denied.

#### S1.4 — Make capacity and deletion durable

- **Priority**: P0 data lifecycle
- **Depends on**: S1.3
- **Actions**:
  - delete evicted episode, summary, embeddings, relationships, and indexes idempotently across cache/durable backends;
  - return explicit partial-failure state and add reconciliation rather than logging success;
  - decide whether a lightweight operation journal is required (feature F4.2).
- **Acceptance**: with capacity one, an evicted episode stays absent after restart/backfill; injected backend failure is observable and repairable.

#### S1.5 — Make embedding availability and artifacts truthful

- **Priority**: P1 correctness/supply chain
- **Depends on**: S1.1 for shared artifact policy
- **Actions**:
  - expose `real`, `degraded-mock`, and `unavailable` states;
  - fail closed in production unless mock fallback is explicitly enabled for test/dev;
  - pin model revision/files/dimensions/digests and enforce download size limits.
- **Acceptance**: corrupt/missing/wrong-digest/oversized artifacts fail or report degraded; mock never reports production-ready; output dimension matches the manifest.

#### S1.6 — Implement retry backpressure semantics

- **Priority**: P1 reliability
- **Depends on**: T0.2
- **Actions**:
  - reject zero concurrency;
  - apply queue timeout to retry attempts, not initial calls;
  - release permits during backoff and return a typed queue-timeout error;
  - document local versus process-wide budgets.
- **Acceptance**: first attempts proceed under saturated retry permits; queued retry times out within bound; simultaneous retry attempts never exceed the configured maximum.

#### S1.7 — Harden asynchronous audit logging

- **Priority**: P1 security/reliability
- **Depends on**: T0.2
- **Actions**:
  - recursively redact nested objects/arrays and case variants;
  - initialize rotation state from existing file size;
  - move blocking writes to a bounded writer task or `spawn_blocking` with drop metrics.
- **Acceptance**: nested secrets are absent, oversized existing logs rotate on first write, slow storage does not stall request workers, overflow behavior is measured.

### Phase 2 — Trustworthy engineering and release gates

#### W2.1 — Define one gate contract

- **Priority**: P0
- **Depends on**: T0.2
- **Actions**:
  - distinguish measured coverage, current blocking floor, and aspirational target;
  - use ADR-042’s ratchet only after a fresh baseline; do not claim 90% without evidence;
  - choose one authoritative command surface for fmt, clippy, build, nextest, doctest, docs, LOC, audit, coverage, semver, and performance;
  - make CI call the same scripts or generate both from one definition.
- **Acceptance**: a matrix maps every advertised gate to exactly one blocking implementation and required CI check.

#### W2.2 — Eliminate false-green checks

- **Priority**: P0
- **Depends on**: W2.1
- **Actions**:
  - preserve `cargo audit`, coverage, benchmark, doctest, and subprocess exit status;
  - structurally parse audit/coverage output and reject missing/malformed reports;
  - remove dummy benchmark results from gating runs;
  - do not treat expected required-check cancellation as success.
- **Acceptance**: fixtures for vulnerability, malformed output, failed benchmark, failed doctest, and cancelled prerequisite all produce nonzero/blocking results.

#### W2.3 — Repair package and wrapper correctness

- **Priority**: P0
- **Depends on**: W2.1
- **Actions**:
  - derive package names/exclusions from Cargo metadata;
  - accept real hyphenated names in `build-rust.sh`;
  - assert subprocess success before parsing pattern/complexity/performance results;
  - add wrapper smoke tests.
- **Acceptance**: targeted checks run `do-memory-*` packages; nonexistent packages and ineffective exclusions fail immediately.

#### W2.4 — Enforce release/publish preconditions

- **Priority**: P0 supply chain
- **Depends on**: W2.2, W2.3
- **Actions**:
  - require successful validation for the exact release SHA before artifact creation;
  - run release-state and metadata verification;
  - use `cargo publish --locked`, blocking semver policy, exact-version sparse-index polling, and valid dependency conditions;
  - decide and document whether CLI is publishable; implementation and docs must agree;
  - prohibit manual release creation in plans and skills outside `release.yml` internals.
- **Acceptance**: bad version, dirty metadata, failed required gate, unavailable dependency version, or semver violation blocks publication; dry-run and single-crate dispatch are tested.

#### W2.5 — Repair benchmark/nightly signal

- **Priority**: P1
- **Depends on**: W2.2
- **Actions**:
  - fail benchmark jobs on execution failure and compare against a real baseline with the documented regression budget;
  - include CLI benchmarks in path discovery;
  - upload nightly reports before cleanup, honor slow-test input, and remove duplicate suites;
  - ratchet ignored-test ceilings.
- **Acceptance**: synthetic >10% regression blocks; missing Criterion output blocks; nightly artifact/trend contains nonzero real values.

#### W2.6 — Restore source boundaries and module reachability

- **Priority**: P1 maintainability
- **Depends on**: W2.3
- **Actions**:
  - split all five current production files over 500 LOC along type/helper/test boundaries;
  - consolidate or remove uncompiled duplicate effectiveness, validation, and compatibility modules;
  - add source-module reachability checking with explicit generated/bin exceptions;
  - run the LOC gate in required CI.
- **Acceptance**: zero production files over 500 LOC and zero unexplained orphan `.rs` files.

### Phase 3 — Skills, workflow guidance, and public documentation

#### K3.1 — Define and enforce a skill contract

- **Priority**: P0 agent safety
- **Depends on**: T0.2
- **Actions**:
  - define one eval schema; fail on missing file, unknown top-level key, zero tests, missing `exec`, or literal no-op;
  - repair the eval runner’s failure aggregation;
  - add a fast CI job for changed skills and a periodic full run.
- **Acceptance**: the current `pr-readiness` schema and `exec: true` fixtures fail validation; missing `ci-poll` eval fails rather than passes.

#### K3.2 — Replace synthetic evals with behavioral fixtures

- **Priority**: P1
- **Depends on**: K3.1
- **Actions**:
  - prioritize release-guard, pr-readiness, commit, ci-fix, code-quality, test-runner, goap-agent, and web-doc-resolver;
  - test negative safety cases: no unsolicited push/force/manual release, no ignored required cancellation, no lint suppression, no lockfile deletion;
  - test documented command/path/environment examples.
- **Acceptance**: each high-risk skill has at least one positive and one refusal/error-path test that can fail for the intended reason.

#### K3.3 — Generate inventory, routing, and overlap boundaries

- **Priority**: P1
- **Depends on**: K3.1
- **Actions**:
  - generate a canonical 33-skill inventory from frontmatter;
  - add intent/path routing for high-risk/high-frequency skills;
  - remove broad collisions and document “use X instead when” boundaries for CI, release, testing, and planning clusters;
  - lint broken links, unknown related skills, stale tool names, unbalanced fences, commands, and environment variables.
- **Acceptance**: every canonical skill is inventoried; route coverage and negative-routing fixtures pass; no broken local skill links remain.

#### D3.1 — Align release and mutation guidance

- **Priority**: P0
- **Depends on**: T0.2, K3.1
- **Actions**:
  - make release-guard the safety authority and remove all manual-release fallbacks;
  - gate commit/push/worktree force operations on explicit user authorization;
  - remove lockfile deletion and blanket lint-suppression advice;
  - use current Kiro/platform-neutral capability names.
- **Acceptance**: repository search finds no active agent instruction that bypasses the release workflow or performs destructive/remote mutation without confirmation.

#### D3.2 — Correct README and generated contracts

- **Priority**: P1 user contract
- **Depends on**: T0.2, S1.1, W2.1
- **Actions**:
  - fix the `TaskContext` example and compile it as a doctest/example;
  - describe actual sandbox availability/backend and remove nonexistent feature flags;
  - scope storage defaults and embedding providers accurately;
  - replace unsupported performance/coverage/test claims with reproducible measurements;
  - generate CLI/MCP command summaries from Clap/tool registry where practical.
- **Acceptance**: README examples compile; every feature flag exists in the owning manifest; measured claims contain provenance.

#### D3.3 — Re-establish canonical plan hygiene

- **Priority**: P1 governance
- **Depends on**: T0.2
- **Actions**:
  - keep CURRENT concise and current, ROADMAP future-only, and GOAP_STATE limited to active execution state;
  - archive superseded dated plans after synthesis;
  - make plan integrity checks blocking for changed active plans.
- **Acceptance**: active tree complies with ADR-039 as amended by ADR-072; duplicate/missing IDs and stale canonical timestamps are CI failures.

### Phase 4 — Evidence-driven feature proposals

- **Priority**: P2 for every F4 work package and child.

These are proposals, not commitments. Each begins with a bounded spike and can be rejected.

#### F4.1 — Retrieval provenance and explainability envelope

- **Depends on**: S1.2
- **Proposal**: expose the ADR-074 request fingerprint, retrieval tier/backend, model/index generation, candidate counts, score components, cache status, and latency to CLI/MCP diagnostics without leaking raw sensitive queries.
- **Go criteria**: no retrieval-quality regression beyond the preapproved metric tolerance, <2% P95 latency overhead, and incident-fixture diagnosis time improves by the preapproved threshold; otherwise record `NO-GO`.

#### F4.2 — Multi-backend reconciliation journal

- **Depends on**: S1.3, S1.4
- **Proposal**: persist idempotent operation intent/outcome for episode completion, eviction, embedding cleanup, and relationship deletion; add health/repair commands.
- **Go criteria**: the predeclared fault matrix converges after restart with zero resurrection and duplicate steps, while storage and compaction metrics remain within preapproved numeric limits; otherwise record `NO-GO`.

#### F4.3 — Verifiable local-model registry

- **Depends on**: S1.5
- **Proposal**: signed/pinned model manifests with source revision, file digests, dimensions, license metadata, maximum size, and health state.
- **Go criteria**: valid offline fixtures reproduce the declared digest/dimensions, every tamper fixture rejects, and security/legal owners approve the recorded trust root, license, and rotation policy; otherwise record `NO-GO`.

#### F4.4 — Agent skill contract compiler

- **Depends on**: K3.3
- **Proposal**: generate inventory, route map, reference checks, and eval matrix from skill frontmatter plus a versioned schema.
- **Go criteria**: all baseline schema/no-op/link/routing fixtures are detected, generated catalog duplication count is zero, and changed-skill CI completes within the preapproved runtime (target <30 seconds); otherwise record `NO-GO`.

### Phase 5 — Convergence

#### V5.1 — Prove the integrated goal state

- **Priority**: P0 release blocker
- **Depends on**: every required T/S/W/K/D child. All are required except S1.1d, which is required only when S1.1c records `GO`; a `NO-GO` requires the fail-closed S1.1a/b state. F4 children are optional until maintainers select a pilot, after which that child is required.
- **Actions**: run the canonical workspace, security, fault-injection, documentation, plan-integrity, and skill-eval suites at one commit; independently review evidence.
- **Acceptance**: every goal-state flag is supported by a passing command/run artifact at the same integration SHA; any skipped or unavailable required check blocks completion. Conditional and pilot selections are read from validated decision artifacts, never chosen ad hoc at V5.1.

### 6.1 Atomic child and validation ledger

Commands marked `new` are deliverables of that child and must have their own fixtures. Each child inherits its parent work package priority and external dependencies. The required internal edges are:

```text
T0.1a -> T0.1b -> T0.1c -> T0.2a -> T0.2b -> T0.2c -> T0.2d -> {T0.2e,T0.2f,T0.2g}
T0.2e -> S1.1c -> S1.1d[only if GO]; S1.1a -> S1.1b; T0.2f -> S1.2a -> S1.2b -> S1.2c
S1.3a -> S1.3b -> S1.4a -> S1.4b; S1.5a -> S1.5b; S1.6a -> S1.6b; S1.7a -> S1.7b
W2.1a -> W2.1b -> W2.2a -> W2.2b -> W2.3a -> W2.3b -> {W2.4a,W2.4b,W2.5a,W2.5b,W2.6a,W2.6b}
K3.1a -> K3.1b -> K3.2a -> {K3.3a,K3.3b,K3.3c}; D3.1a -> D3.1b; D3.2a -> D3.2b -> D3.2c; D3.3a -> V5.1
```

All decision/spike artifacts use schema `{id, commit, timestamp, owner, commands, metrics, preapproved_thresholds, result, decision, reviewers}` and are produced and checked by `new: ./scripts/run-feature-spike.sh` and `new: ./scripts/validate-feature-spike.sh`. Missing fields, post-hoc thresholds, or unmet `GO` thresholds fail validation. Work-package prose above supplies design context; this ledger is the completion contract.

| Child | Single outcome | Executable validation and evidence |
|---|---|---|
| T0.1a | Capture local package/gate/LOC/skill baseline | `cargo metadata --format-version 1 --no-deps`; canonical LOC/eval commands; archive outputs with SHA |
| T0.1b | Verify remote tag/release/CI/crates state for exact version/SHA | derive `VERSION`, `TAG`, and tag commit; `gh release view "$TAG" --json tagName,targetCommitish,isDraft,isPrerelease,publishedAt,url`; `gh run list --commit "$SHA" --json workflowName,headSha,status,conclusion,url`; exact package/version crates.io queries; `new: ./scripts/validate-release-evidence.sh` must prove all SHAs/versions match or leave a blocker |
| T0.1c | Validate metric provenance schema | `new: ./scripts/validate-plan-evidence.sh --fixtures`; missing SHA/time/scope/artifact fixture must fail |
| T0.2a | Accept and register ADR-072 | `new: ./scripts/validate-plans.sh --adrs`; status/registry fixture passes |
| T0.2b | Classify version/release state from T0.1 evidence | `new: ./scripts/validate-plans.sh --version-state`; contradictory canonical fixture fails |
| T0.2c | Reconcile and archive canonical plans | `new: ./scripts/validate-plans.sh --active-set`; superseded active dated-plan fixture fails |
| T0.2d | Register duplicate aliases and repair ADR-061 links | `new: ./scripts/validate-plans.sh --identifiers --links`; duplicate-new-ID and missing-link fixtures fail |
| T0.2e | Record ADR-073 accept/reject decision | `./scripts/validate-plans.sh --adr-decision ADR-073`; requires deciders/date and enforces NO-GO or S1.1d branch |
| T0.2f | Record ADR-074 accept/reject decision | `./scripts/validate-plans.sh --adr-decision ADR-074`; requires deciders/date and blocks V5.1 after rejection until an alternative cache-correctness ADR passes |
| T0.2g | Add ADR-072 supersession notes to ADR-039/034/045/058 | `new: ./scripts/validate-plans.sh --supersession`; each affected ADR must link ADR-072 and identify changed clauses |
| S1.1a | Preserve unavailable MCP execution and correct contract | `cargo nextest run -p do-memory-mcp execute_agent_code`; list omits tool and direct call rejects |
| S1.1b | Remove or quarantine legacy Node executor | `new: ./scripts/check-source-reachability.sh --deny memory-mcp/src/sandbox/mod.rs`; MCP path test proves no production-safe path reaches Node |
| S1.1c | Decide Wasmtime/WASI feasibility | `./scripts/run-feature-spike.sh S1.1c --config plans/spikes/S1.1c.toml --output plans/STATUS/spikes/S1.1c.json`; preapproved controls/platforms/P95/binary-size thresholds and threat review determine GO/NO-GO |
| S1.1d | If GO, implement and approve capability backend | `cargo nextest run -p do-memory-mcp sandbox`; runtime bypass/resource fixtures pass; `new: ./scripts/validate-security-review.sh plans/STATUS/SECURITY_REVIEW_CODE_EXECUTION.md "$SHA"` requires independent `APPROVE` before registration |
| S1.2a | Add typed/versioned retrieval identity | `cargo nextest run -p do-memory-core retrieval_identity`; canonical postcard fixtures pass |
| S1.2b | Wire cache key and mutation generation | `cargo nextest run -p do-memory-core retrieval_cache`; context/provider/index-order fixtures pass |
| S1.2c | Add redacted retrieval provenance | `cargo nextest run -p do-memory-core retrieval_provenance`; `./scripts/run-feature-spike.sh S1.2c --config plans/spikes/S1.2c.toml --output plans/STATUS/spikes/S1.2c.json`; `./scripts/validate-feature-spike.sh plans/STATUS/spikes/S1.2c.json` rejects leak, quality tolerance, or preapproved P95 breach |
| S1.3a | Implement versioned lock-free backend persistence | `cargo nextest run -p do-memory-core episode_persistence`; lock-barrier test passes |
| S1.3b | Prove concurrent step preservation | `cargo nextest run -p do-memory-core concurrent_step_preservation`; 100-step stress fixture passes repeatedly with exactly-once IDs |
| S1.4a | Implement idempotent cross-backend eviction | `cargo nextest run -p do-memory-core durable_capacity_eviction`; restart/backfill capacity-one fixture passes |
| S1.4b | Expose and repair partial eviction failure | `cargo nextest run -p do-memory-core eviction_reconciliation`; injected partial state is typed and converges after repair/restart |
| S1.5a | Expose real/degraded/unavailable provider state | `cargo nextest run -p do-memory-core local_embedding_state`; mock is never production-ready |
| S1.5b | Verify model manifests/artifacts | `cargo nextest run -p do-memory-core local_model_integrity`; wrong digest/size/revision/file/dimension reject and valid offline fixture passes |
| S1.6a | Implement retry queue/concurrency semantics | `cargo nextest run -p do-memory-core retry`; zero, timeout, first-attempt, permit-release fixtures pass |
| S1.6b | Document local versus process-wide retry budgets | `cargo test --doc -p do-memory-core retry`; API examples and scope assertions compile/pass |
| S1.7a | Implement recursive redaction and correct rotation | `cargo nextest run -p do-memory-mcp audit`; nested-secret and existing-size fixtures pass |
| S1.7b | Move audit writes off async workers | `cargo nextest run -p do-memory-mcp audit_writer_backpressure`; slow-writer/overflow fixture proves bounded latency and drop metrics |
| W2.1a | Publish measured/required/target gate matrix | `new: ./scripts/validate-gate-contract.sh`; conflicting-threshold fixture fails |
| W2.1b | Establish one canonical local/CI command surface | `./scripts/validate-gate-contract.sh --ci-parity`; required jobs/scripts at the same SHA must match |
| W2.2a | Preserve audit/coverage/benchmark/doctest exit status | `./scripts/validate-gate-contract.sh --failure-fixtures`; injected failures and malformed reports return nonzero |
| W2.2b | Reject cancelled required prerequisites | `new: ./scripts/test-workflow-guards.sh --cancelled-required`; expected cancellation blocks |
| W2.3a | Derive package names/exclusions from metadata | `cargo nextest run --test quality_gates package_names`; accepts `do-memory-core`, rejects nonexistent names/exclusions |
| W2.3b | Require subprocess success before metric parsing | `cargo nextest run --test quality_gates subprocess_failure`; failed/missing pattern, complexity, and performance commands cannot use fallback values |
| W2.4a | Gate release on exact validated SHA | `new: ./scripts/test-release-workflow.sh --fixtures`; wrong tag/SHA/metadata/required check blocks |
| W2.4b | Make publishing reproducible and dependency-aware | `./scripts/test-release-workflow.sh --publish-fixtures`; verifies `--locked`, blocking semver, exact polling, full dry-run, and each single-crate dispatch/dependency path |
| W2.4c | Decide CLI publication policy | `./scripts/validate-plans.sh --package-policy do-memory-cli`; manifest/workflow/docs must have one answer |
| W2.4d | Remove active manual-release guidance | `new: ./scripts/validate-plans.sh --release-policy` rejects active direct-release instructions |
| W2.5a | Enforce real benchmark regression data including CLI paths | `new: ./scripts/test-benchmark-workflow.sh --fixtures`; CLI path fixture runs, >10% regression and missing Criterion output block |
| W2.5b | Preserve nightly reports, honor inputs once, and ratchet ignores | `new: ./scripts/test-nightly-workflow.sh --fixtures`; `new interface: ./scripts/check-ignored-tests.sh --fixture ratchet`; validates upload-before-cleanup, one selected run, and no unevidenced ceiling increase |
| W2.6a | Split five oversized production files | `new interface: ./scripts/quality-gates.sh --loc-only`; reports zero production files above 500 |
| W2.6b | Remove unexplained orphan source modules | `new: ./scripts/check-source-reachability.sh --fixtures`; orphan fixture fails |
| K3.1a | Enforce one strict skill-eval schema | `./scripts/run-evals.sh` rejects `evals`, zero/missing tests, missing exec, and `exec: true` |
| K3.1b | Run changed/full skill evals in CI | `new interface: ./scripts/run-evals.sh --changed`; `./scripts/run-evals.sh`; interface fixture blocks and full artifact enumerates 33 skills |
| K3.2a | Add positive/negative high-risk skill fixtures | `for s in release-guard pr-readiness commit ci-fix code-quality test-runner goap-agent web-doc-resolver; do ./scripts/run-evals.sh "$s" || exit 1; done`; refusal regressions fail |
| K3.3a | Generate complete skill inventory | `new: ./scripts/generate-skill-inventory.sh --check`; output contains all 33 valid skills |
| K3.3b | Define routes and overlap boundaries | `new: ./scripts/validate-skill-routes.sh --fixtures`; positive/negative cluster routes pass |
| K3.3c | Lint skill links/tools/commands/env/fences | `new: ./scripts/validate-skills.sh --fixtures`; each known stale pattern fixture fails |
| D3.1a | Establish release-guard as sole skill authority | `./scripts/validate-skills.sh --release-policy`; no manual fallback and negative eval passes |
| D3.1b | Confirmation-gate remote/destructive mutations | `for s in commit git-worktree-manager pr-readiness; do ./scripts/run-evals.sh "$s" || exit 1; done`; unrequested push/force rejects and lockfile remains |
| D3.2a | Correct and compile README contracts | `cargo test --doc` plus `new: ./scripts/validate-readme-contracts.sh`; examples/features/defaults/providers pass |
| D3.2b | Generate CLI/MCP summaries | `new: ./scripts/generate-contract-docs.sh --check`; checked docs match Clap/tool registry |
| D3.2c | Attach reproducible provenance to public metrics | `new: ./scripts/validate-doc-metrics.sh README.md --evidence plans/STATUS/VALIDATION_LATEST.md`; stale/missing command, scope, SHA, time, or artifact fixture fails |
| D3.3a | Restore canonical active-plan roles | `./scripts/validate-plans.sh --active-set --version-state --identifiers --links`; passes after archival |
| F4.1 | Decide provenance-envelope pilot | `./scripts/run-feature-spike.sh F4.1 --config plans/spikes/F4.1.toml --output plans/STATUS/spikes/F4.1.json`; validator requires no quality regression and preapproved diagnosis-time/P95 thresholds (target P95 overhead <2%) |
| F4.2 | Decide reconciliation-journal pilot | same producer/validator with restart fault matrix, zero resurrection/duplication, and preapproved storage/compaction limits |
| F4.3 | Decide verifiable-model-registry pilot | same producer/validator with tamper/offline/reproducibility fixtures and approved trust-root/rotation review |
| F4.4 | Decide skill-compiler pilot | same producer/validator must catch all baseline defect fixtures and meet preapproved changed-skill runtime (target <30s) and generated-file duplication count (zero) |
| V5.1 | Certify integrated goal state | `./scripts/quality-gates.sh`; targeted fault suites; `./scripts/validate-plans.sh`; `./scripts/run-evals.sh`; decision/security-review validators; independent review all pass at one SHA |

## 7. Validation matrix

| Outcome | Required evidence |
|---|---|
| Code correctness | Targeted crate tests plus concurrency/fault-injection tests for changed behavior |
| Workspace health | Canonical fmt, strict clippy, build/check, nextest, doctest, and docs commands all pass at one SHA |
| Security | Blocking cargo-audit/cargo-deny; sandbox runtime bypass suite; nested-redaction tests |
| Coverage | Fresh llvm-cov artifact; blocking floor and target reported separately |
| Performance | Criterion baseline comparison; no fabricated fallback data |
| Release | Exact-SHA preflight, metadata verification, dry-run, artifact checksums/signatures, automated release only |
| Skills | Strict schema validator, changed-skill evals, periodic full eval, routing/link lint |
| Documentation | README examples compile; feature/command lists generated or manifest-verified; plan integrity gate passes |

## 8. Exit criteria

The plan is complete only when all P0 and P1 actions have evidence at the same integration commit and:

- no untrusted code-execution tool is advertised without a capability-enforced backend;
- context-distinct retrieval requests cannot collide in cache;
- no memory lock is held across backend I/O and durable eviction survives restart;
- required audit, coverage, benchmark, doctest, and subprocess failures cannot be converted to green;
- release creation has one automated authority and validates the exact SHA;
- all high-risk skills have executable positive/negative evals and current routes;
- README examples/contracts match manifests and code;
- canonical plans agree on version/state and ADR/WG references are unique; and
- every shipped feature pilot has its recorded go/no-go evidence.

## 9. Risks and rollback

- **Gate activation may reveal extensive existing debt**: introduce truthful reporting first, record baseline, then make only explicitly selected floors blocking; never hide failures with `|| true`.
- **Concurrency refactor may lose updates**: require versioned merge and stress tests before replacing the current path.
- **Sandbox migration may temporarily remove a tool**: prefer unavailable over insecure; core memory operations are unaffected.
- **Durable reconciliation adds state**: start with idempotent delete plus observable partial failures; adopt a journal only if fault tests justify it.
- **Plan cleanup can break links**: use redirects/index entries and validate active links before archival.
- **Skill routing can over-trigger**: add negative-routing fixtures and path/intent precedence before broad rollout.

## 10. Explicitly out of scope for this planning change

- Editing Rust, workflow, script, README, AGENTS, or skill files.
- Claiming current GitHub releases, CI runs, PRs, tags, or crates.io versions without querying them.
- Accepting the proposed ADRs on behalf of maintainers.
- Changing production infrastructure, publishing crates, creating tags/releases, or committing/pushing changes.
