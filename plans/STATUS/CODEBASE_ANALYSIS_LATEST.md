# Codebase Analysis Latest — 2026-08-11 (closure PR refresh)

**Branch**: `fix/ci-attribution-truth-closure` (off `main` @ `5a943c98a98d3807fbcf7d644024c55451c7d702`)
**Workspace**: `0.1.40` · **Released tag**: `v0.1.39`
**Companion**: closure PR from branch `fix/ci-attribution-truth-closure` (PR number / head SHA recorded by the controller after creation)

## Architecture (as implemented)

| Crate | Role |
|-------|------|
| `do-memory-core` | Episodes, patterns, rewards, retrieval (CSM cascade), embeddings, F4 provenance/journal |
| `do-memory-storage-turso` | Durable libSQL / Turso |
| `do-memory-storage-redb` | Embedded cache |
| `do-memory-mcp` | MCP server, lazy tools, audit, fail-closed code exec |
| `do-memory-cli` | Operator CLI |
| `do-memory-test-utils` / benches / examples / e2e | Support |

**Stack**: Rust 2024, Tokio, Turso/libSQL, redb, postcard, optional embeddings, `csm` cascade.

## Health summary

| Check | Result |
|-------|--------|
| Open gaps | 0 open P0 code-side (remaining P0 = ADR-079 stage 4 external proof); G-P1-8/G-P1-9 monitor-only |
| Live merge rules | ruleset `9591004` requires `[Codacy Static Code Analysis, CI / Required]` (strict); aggregate is causally same-run |
| Fast-gate topology | same-run `commitlint` + `fast-gate` in `ci.yml`; `ci-required-evaluate.sh` accepts only `success`; waiter/anchor topology deleted |
| Production LOC >500 (non-test `src`) | **0** |
| Released tag | **v0.1.39** |
| Workspace advanced post-tag | **0.1.40** |
| Skills with evals / routes | 40/40 |
| Fail-closed code execution | Preserved (ADR-073) |
| Open issues | **0** (#913 closed 2026-08-02) |
| Open PRs | closure PR (`fix/ci-attribution-truth-closure`) — created by the controller after this task |
| ADR-071 | Accepted / Implemented (auto-checkpoint on Abstained) |
| ADR-072 | Accepted / Implemented (authority + governance) |
| ADR-073 | Accepted / Implemented (S1.1c NO-GO, fail-closed) |
| ADR-079 | Accepted — stage 3 live + stage 5 cleanup landed; stage 4 proof external |
| ADR-080/081 | Proposed (code evidence landed in closure PR; lifecycle awaits maintainer acceptance) |

## Strengths

1. Correctness campaign (locks, eviction, cache identity, embedding health).  
2. Broad automated coverage (deny, benchmarks, docs, storage matrices, skill evals).
3. Skill eval schema + high- and medium-risk behavioral fixtures (40 skills, all routed).  
4. Singular release path (`release-manager` + `release.yml`).  
5. Rich episodic/pattern/playbook MCP+CLI surface.
6. R-F8 CLI relationship box-drawing panel + R-F9 HNSW persistence/eviction (#893).

## Verified weaknesses / missing implementation

1. ~~Active ruleset `9591004` has no first-party required build/test aggregate; the standalone echo anchor is neither required nor a substantive gate~~ ✅ closed 2026-08-11 — ruleset requires `[Codacy Static Code Analysis, CI / Required]`; anchor + waiter topology deleted (ADR-079 stage 5).
2. ~~Five workflows poll only the format/Clippy job and accept cancelled, skipped, or missing outcomes~~ ✅ closed 2026-08-11 — same-run `commitlint` + `fast-gate` in `ci.yml`; `ci-required-evaluate.sh` accepts only `success`.
3. ~~Dependabot is excluded from core CI tests/builds, coverage, security, file-validation, semver, WASM, and benchmarks~~ ✅ closed CIT-A2 (2026-08-10) — actor parity under reduced permissions.
4. ~~Local and CI contracts drift; parity validator checks presence/keywords, not semantics~~ ✅ closed 2026-08-11 — `validate-gate-contract.sh --ci-parity` inspects commands/dependencies/evaluator/absence of waiters; `test-workflow-guards.sh --required-aggregate` fixtures.
5. ~~Release manual dispatch fails tag preflight; publish selection can be suppressed; fuzz failures can be continued~~ ✅ closed CIT-A4/A5 (2026-08-06).
6. ~~`CascadeRetriever::retrieve` returns a successful empty result without `csm`~~ ✅ PTA-A1 closed — returns `Err(CascadeError::CapabilityUnavailable)`.
7. ~~CLI storage output presents fixed-size estimates, completed-count "recent" values, and unavailable cache/connection data as observed telemetry~~ ✅ PTA-A2 closed — `MetricValue` provenance.
8. ~~`eval set-threshold` is public but has no persistence or reward-consumption path and always errors~~ ✅ PTA-A3 closed — command removed.
9. ~~Pattern recommendation does not create attribution sessions automatically; playbook recommendation records `Uuid::nil()` and skips persistence~~ ✅ closed 2026-08-11 — `AttributedPlaybookRequest` + `retrieve_playbooks_attributed` + `recommend_patterns_attributed` with validated episodes and checked receipts.
10. ~~Session/feedback persistence is warning-only and trait defaults can report success without writing; feedback integrity is not fully validated~~ ✅ closed 2026-08-11 — capability-gated checked receipts, integrity rules, cold-restart + receipt-matrix evidence.

## New feature decision

ADR-080 proposes optional, episode-bound attributed pattern/playbook operations.
Core derives a session from exact returned IDs and reports `persisted`,
`partially_persisted`, `memory_only`, or `persistence_failed`. Legacy calls stay
shape-compatible. The feature improves capture and feedback readiness, but does
not yet change recommendation ranking.

## Recommended focus order

1. ~~Accept ADR-079; implement and fault-inject a same-run first-party aggregate~~ ✅ Accepted; same-run aggregate live (ruleset `9591004`). Remaining: ADR-079 stage 4 live fault-injection merge-block proof — external maintainer evidence.
2. ~~Restore fail-closed cancellation/commitlint and Dependabot/fork parity, then stage the verified aggregate into the live ruleset with approval~~ ✅ done — same-run fast gate + actor parity; `CI / Required` required live.
3. ~~Reconcile gate scope and repair release/publish/fuzz false-success paths~~ ✅ CIT-A3/A4/A5 + closure-PR semantic fixtures.
4. ~~Accept and execute ADR-080 RAT-A1…A7~~ ✅ code-side executed (closure PR evidence); ADR-080/081 lifecycle stays `Proposed` until maintainer acceptance.
5. Design feedback-to-ranking adaptation only after capture integrity is proven — still deferred (no code changes ranking).
6. Residual: historical ADR filename collisions (025/054 aliased — docs only), transitive Dependabot advisories (monitor), R-F10 (OIDC) / R-F4 (SIMD) shipped.

Full prioritized plan: closure PR from branch `fix/ci-attribution-truth-closure` (PR number / head SHA recorded by the controller after creation).
