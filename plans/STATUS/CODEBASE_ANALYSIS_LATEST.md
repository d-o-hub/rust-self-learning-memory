# Codebase Analysis Latest — 2026-07-30

**Branch**: `main` @ `e66defdf`
**Workspace**: `0.1.38` · **Released tag**: `v0.1.37`
**Companion**: `plans/GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`

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
| Open gaps | 4 P0 (2 CI, 2 product), 6+ P1 contract/feature/automation gaps |
| Live merge rules | Codacy required + CodeQL severity policy; no first-party build/test aggregate |
| Cross-workflow waiters | 5; accept `success,skipped,cancelled` and missing checks |
| Production LOC >500 (non-test `src`) | **0** |
| Released tag | **v0.1.37** |
| Workspace advanced post-tag | **0.1.38** |
| Skills with evals / routes | 40/40 |
| Fail-closed code execution | Preserved (ADR-073) |
| Open issues | **1** (#913) |
| Open PRs | **2** (#914, #915) |
| ADR-071 | Accepted / Implemented (auto-checkpoint on Abstained) |
| ADR-072 | Accepted / Implemented (authority + governance) |
| ADR-073 | Accepted / Implemented (S1.1c NO-GO, fail-closed) |

## Strengths

1. Correctness campaign (locks, eviction, cache identity, embedding health).  
2. Broad automated coverage (deny, benchmarks, docs, storage matrices, skill evals).
3. Skill eval schema + high- and medium-risk behavioral fixtures (40 skills, all routed).  
4. Singular release path (`release-manager` + `release.yml`).  
5. Rich episodic/pattern/playbook MCP+CLI surface.
6. R-F8 CLI relationship box-drawing panel + R-F9 HNSW persistence/eviction (#893).

## Verified weaknesses / missing implementation

1. Active ruleset `9591004` has no first-party required build/test aggregate;
   the standalone echo anchor is neither required nor a substantive gate.
2. Five workflows poll only the format/Clippy job and accept cancelled, skipped,
   or missing outcomes. Commit lint is outside that observed result; PR #914 had
   failed commit lint while downstream waiters passed.
3. Dependabot is excluded from core CI tests/builds, coverage, security,
   file-validation, semver, WASM, and benchmarks rather than receiving the same
   assertions under reduced permissions.
4. Local and CI contracts drift: CI excludes workspace packages from the test
   surface, uses copied Clippy allowances, and implements only part of the local
   quality bundle. The parity validator checks presence/keywords, not semantics.
5. Release manual dispatch fails tag preflight by construction. Publish selection
   can be suppressed by skipped dependencies, and fuzz failures can be continued
   without triggering crash-artifact upload.
6. ~~`CascadeRetriever::retrieve` returns a successful empty result without `csm`~~ ✅ PTA-A1 closed — returns `Err(CascadeError::CapabilityUnavailable)`.
7. ~~CLI storage output presents fixed-size estimates, completed-count "recent" values, and unavailable cache/connection data as observed telemetry~~ ✅ PTA-A2 closed — `MetricValue` provenance.
8. ~~`eval set-threshold` is public but has no persistence or reward-consumption path and always errors~~ ✅ PTA-A3 closed — command removed.
9. Pattern recommendation does not create attribution sessions automatically;
   playbook recommendation records `Uuid::nil()` and skips persistence.
10. Session/feedback persistence is warning-only and trait defaults can report
    success without writing; feedback integrity is not fully validated.

## New feature decision

ADR-078 proposes optional, episode-bound attributed pattern/playbook operations.
Core derives a session from exact returned IDs and reports `persisted`,
`partially_persisted`, `memory_only`, or `persistence_failed`. Legacy calls stay
shape-compatible. The feature improves capture and feedback readiness, but does
not yet change recommendation ranking.

## Recommended focus order

1. Accept ADR-079; implement and fault-inject a same-run first-party aggregate.
2. Restore fail-closed cancellation/commitlint and Dependabot/fork parity, then
   stage the verified aggregate into the live ruleset with approval.
3. Reconcile gate scope and repair release/publish/fuzz false-success paths.
4. Accept and execute ADR-078 RAT-A1…A7.
5. Design feedback-to-ranking adaptation only after capture integrity is proven.
6. Residual: historical ADR filename collisions (025/054 aliased — docs only),
   transitive Dependabot advisories (monitor), R-F10 (OIDC) / R-F4 (SIMD) in progress.

Full prioritized plan: `plans/GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`.
