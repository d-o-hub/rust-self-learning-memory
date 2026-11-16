# Active Development Roadmap

**Last Updated**: 2026-11-16
**Released Version**: v0.1.50 (`0fe1b89d`, GitHub Release 2026-11-16T08:45:18Z; all four
registered crates verified at `0.1.50` on crates.io)
**Workspace Version**: 0.1.51 (post-release development bump)
**Branch**: post-release bump based on `main` @ `0fe1b89d`
**Open PRs / issues**: `gh pr list` / `gh issue list` are the live sources; Dependabot group
PRs may be open

## Active forward work

| Priority | Area | Item | Acceptance | Status |
|----------|------|------|------------|--------|
| P1 | Security | Track the four upstream-blocked advisory families; re-test only when a pinning chain moves | `cargo tree` on both graphs proves the chain moved; no no-op bump PR | ⏸ upstream (ACT-404) |
| P2 | Governance | ADR-080 / ADR-081 lifecycle acceptance | maintainer flips `Proposed` → `Accepted` with recorded evidence | ⏸ external |
| P2 | Harness | Keep the pin, prose, gate contract and skill-evals matrix on one measured version | both probes green against the published binary | ✅ `v0.4.6` |
| P3 | Release | `v0.1.50` shipped and dogfooded; bump the workspace to `0.1.51` in the post-release PR | release + publish chains green; four crates at `0.1.50`; dogfood recorded | ✅ 2026-11-16 |
| P3 | Plans | Keep this active set compact; archive completed waves | `validate-plans.sh --all` green; dated root files ≤5 | ✅ 2026-11-15 |

## Gated product backlog (R-F)

The 2026-07-20 register is archived; its completed tracks are evidence, not backlog.
The authoritative gate is
`plans/STATUS/spikes/R-F-product-backlog-2026-11-15.json` (`NO-GO` for this cycle).
An item moves back into "active forward work" only after a measured spike passes its
pre-approved thresholds and names an owner, a release slot, and (where it changes
storage or transport) an ADR.

| ID | Epic | Status | Gate / next evidence |
|----|------|--------|----------------------|
| R-F1 | Distributed sync via Turso embedded replicas | ⏸ `NO-GO` | needs R-F3 tenant partition + transport ADR |
| R-F2 | OpenTelemetry / OTLP tracing | ⏸ `NO-GO` | pin `opentelemetry` 0.33.x; traces are still beta; needs `otel` feature + redaction tests |
| R-F3 | Multi-tenancy (`tenant_id`, `CacheKey`) | ⏸ `NO-GO` | schema + cache-key ADR; four-PR estimate in the archived register |
| R-F4 | Portable SIMD cosine | ⏸ `NO-GO` | `std::simd` is still nightly-only; keep the 8-way unrolled auto-vectorized loop |
| R-F5 | Episode versioning / retention | ⏸ `NO-GO` | schema migration + `StorageBackend` extension + MCP/CLI surface |
| R-F6 | Mixture-of-Experts embedding routing | ⏸ `NO-GO` | ADR above ADR-077 exact-provider model; no silent substitution |
| R-F7 | Federated HDC | ⏸ `NO-GO` | depends on R-F1 + R-F3; highest complexity |
| R-F8 | CLI relationship panel | ✅ shipped | #893 |
| R-F9 | HNSW persistence + capacity eviction | ✅ shipped | #893 |
| R-F10 | crates.io trusted publishing (OIDC) | ✅ shipped | ADR-078; four publishers, `v*` environment, no token |

## Standing product decisions (do not reopen casually)

| Topic | Decision |
|-------|----------|
| Agent code execution | **Fail-closed**; Wasmtime/WASI **NO-GO** |
| Batch MCP tools | Explicitly deferred |
| Release creation | Automated only: `release-manager.sh ship` → tag → `release.yml` |
| Serialization | Postcard required; no internally-tagged enums on stored types |
| ADR-075 durable complete | All-or-nothing; backend failures are hard errors |
| ADR-076 pattern list | Empty diagnostics in human mode; JSON/YAML machine-stable |
| Spike promotion | A `GO` artifact is approval to implement, not evidence that code exists |

## Next release checklist (when scheduled)

1. Confirm cadence and a clean tracker: `./scripts/check-release-drift.sh`,
   `./scripts/check-release-blockers.sh`.
2. Land the release-prep PR: CHANGELOG `## [0.1.51]`, `Released Version` docs, version
   references.
3. `./scripts/release-manager.sh ship --execute` (never `gh release create`).
4. Dogfood the exact released CLI artifact and record the digest/version/parity in
   `STATUS/VALIDATION_LATEST.md`.
5. Bump the workspace to `0.1.52` in a follow-up PR.

## History

Completed sprint tables were archived on 2026-11-15 and are not re-expanded here
(ADR-039). See `plans/archive/2026-11-consolidation/` and `plans/STATUS/archive/2026/`.
The long-term vision remains `ROADMAP_V030_VISION.md`; it is aspirational and does not
commit sprint work.
