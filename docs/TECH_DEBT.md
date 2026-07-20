# Technical Debt Registry

**Last Updated**: 2026-07-20  
This document tracks known technical debt. Prefer code-verified claims over historical estimates.

## Architecture

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| ARCH-01 | Dual-backend synchronization complexity | Medium | Maintenance | Turso↔redb sync paths are operator-facing via `storage sync`. |
| ARCH-02 | MonitoringStorage wrapper unused | Low | Cleanup | SimpleMonitoringStorage used directly; wrapper reserved. |

## Storage

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| STOR-01 | Upstream libsql memory corruption in some tests | High | Stability | Many Turso integration tests remain `#[ignore]`; see ADR-027 and ignore ratchet. |
| STOR-02 | Prepared statement cache size limits | Medium | Performance | Fixed size of 100 may be tight for complex workloads. |

## Testing

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| TEST-01 | Ignored tests under ceiling | Medium | Coverage | Ceiling enforced by `./scripts/check-ignored-tests.sh` (current ceiling 200; count moves with tree). |
| TEST-02 | Non-deterministic pattern accuracy tests | Low | Flakiness | Floating-point variance; partially mitigated. |

## MCP & Integration

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| MCP-01 | `execute_agent_code` unavailable | Medium | Feature | **Intentional fail-closed** (not a temporary bug). S1.1c Wasmtime/WASI spike = NO-GO. Do not advertise a working sandbox. |
| MCP-02 | Batch tool handlers deferred | Low | Completeness | Batch tools intentionally absent/deferred; not a hidden partial impl. |

## Skills & agent harness

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| SKL-01 | Medium-risk skill behavioral evals incomplete | Medium | Agent safety | High-risk set covered (K3.2); expand per R-E2. |
| SKL-02 | Skill route collisions possible | Low | Routing | Full route set added 2026-07-20; watch negative-route fixtures. |

## Plans / docs

| ID | Issue | Priority | Impact | Notes |
|----|-------|----------|--------|-------|
| DOC-01 | ADR numbers 025 and 054 duplicated | Medium | Navigation | See `plans/adr/README.md` alias table; prefer full filenames. |
| DOC-02 | Vision roadmap title still “v0.1.9+” | Low | Clarity | `plans/ROADMAPS/ROADMAP_V030_VISION.md`. |

## Tracking

Open product recommendations: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`.
