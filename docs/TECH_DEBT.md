# Technical Debt Registry

**Last Updated**: 2026-11-25 (rebaselined against the ADR-085 audit of `main`)
This document tracks known technical debt. Prefer code-verified claims over historical estimates.
The 2026-07-20 baseline was stale: DOC-02 was already resolved and the ignore-ratchet
numbers predated the `WG-075` wave. Verification status per row is dated inline.

## Architecture

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| ARCH-01 | Dual-backend synchronization complexity | Medium | Maintenance | Turso↔redb sync paths are operator-facing via `storage sync`; `WG-075` extended them to summaries/retention (verified 2026-11-25). |
| ARCH-02 | MonitoringStorage wrapper unused | Low | Cleanup | Confirmed 2026-11-25: `SimpleMonitoringStorage` is the production path (init.rs); the wrapper is retained by design per its doc comment (cites ADR-040). Decide use-or-delete in a follow-up wave. |

## Storage

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| STOR-01 | Upstream libsql memory corruption in some tests | High | Stability | Many Turso integration tests remain `#[ignore]` under the ADR-027 ratchet: 159 entries vs ceiling 200 (2026-11-25). |
| STOR-02 | Prepared statement cache size limits | **Resolved 2026-11-26** | Performance | Capacity is configurable via `TursoConfig::prepared_cache_config` (ADR-085 HY-04); `PreparedStatementCache::default()` now derives from `PreparedCacheConfig::default()` instead of a duplicated `100` literal. Default remains 100. |

## Testing

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| TEST-01 | Ignored tests under ceiling | Medium | Coverage | 159 entries, ceiling 200 (2026-11-25); enforced by `./scripts/check-ignored-tests.sh`. |
| TEST-02 | Non-deterministic pattern accuracy tests | Low | Flakiness | Floating-point variance; partially mitigated (not re-verified 2026-11-25). |

## MCP & Integration

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| MCP-01 | `execute_agent_code` unavailable | Medium | Feature | **Intentional fail-closed** (not a temporary bug). S1.1c Wasmtime/WASI spike = NO-GO. Do not advertise a working sandbox. |
| MCP-02 | Batch tool handlers deferred | Low | Completeness | Batch tools intentionally absent/deferred; not a hidden partial impl. Candidate ADR-085 F-3. |

## Skills & agent harness

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| SKL-01 | Medium-risk skill behavioral evals incomplete | Medium | Agent safety | High-risk set covered (K3.2); expand per R-E2 (not re-verified 2026-11-25). |
| SKL-02 | Skill route collisions possible | Low | Routing | Full route set added 2026-07-20; watch negative-route fixtures (not re-verified 2026-11-25). |

## Plans / docs

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| DOC-01 | ADR numbers 025 and 054 duplicated | Medium | Navigation | Alias (historical) files live in `_aliases/`; prefer canonical filenames via `plans/adr/README.md` (still true 2026-11-25). |
| DOC-02 | Vision roadmap title still "v0.1.9+" | ✅ Resolved | Clarity | Title is now "Long-Term Vision (0.2 → 1.0)"; row retained for history. |

## Tracking

2026-11-25 audit, tracker-truth reconciliations and the candidate backlog
(HY-01…HY-07 hygiene, F-1…F-6 gated features): see
[`plans/adr/ADR-085-Comprehensive-Analysis-v0.1.51.md`](../plans/adr/ADR-085-Comprehensive-Analysis-v0.1.51.md).
Product gate: `plans/STATUS/spikes/R-F-product-backlog-2026-11-15.json`.
