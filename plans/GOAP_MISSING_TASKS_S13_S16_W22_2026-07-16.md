# GOAP Missing Tasks Swarm — S1.3–S1.6 + W2.2 (2026-07-16)

**Status**: Code complete — PR pending  
**Coordinator**: goap-agent  
**Strategy**: Sequential implementation → quality gates → PR review  
**Branch**: `feat/goap-missing-tasks-s13-s16-w22-2026-07-16`  
**Source plan**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md`  
**Prior swarm**: `plans/GOAP_MISSING_TASKS_SWARM_2026-07-16.md` (PR #840 merged)

---

## Goal Hierarchy

```
G0: Land next deferred P0 correctness/gate tasks from improvements plan
├── G1: S1.3 short write locks; no backend await under episodes_fallback write
├── G2: S1.4 durable capacity eviction (delete_episode + embeddings)
├── G3: S1.5 embedding health (Real / DegradedMock / Unavailable)
├── G4: S1.6 retry queue timeout + first-attempt free + zero concurrency reject
├── G5: W2.2 remove soft-pass cargo audit (cargo deny blocking)
├── G6: Update plans/ + tests
└── G7: PR + CI green + review
```

## Work Packages

| ID | Package | Status |
|----|---------|--------|
| B1 | S1.3 episode log_step + flush_steps_internal | ✅ |
| B2 | S1.4 completion capacity eviction backend delete | ✅ |
| B3 | S1.5 EmbeddingHealth + LocalConfig.allow_mock_fallback | ✅ |
| B4 | S1.6 RetryError, queue timeout, first attempt free | ✅ |
| B5 | W2.2 ci.yml + security.yml audit hard gate | ✅ |
| B6 | Integration tests s13/s14 + retry tests | ✅ |
| B7 | Plans update | ✅ |
| B8 | PR + CI + review | 🟡 |

## Evidence

| Gate | Result |
|------|--------|
| `cargo check -p do-memory-core` | ✅ |
| Retry + local embedding nextest | ✅ |
| s13 concurrent steps + s14 durable eviction | ✅ |
| `code-quality.sh fmt` / clippy | (see PR CI) |

## Deferred (still open after this PR)

- S1.7 audit writer hardening
- W2.1 full gate contract unification / coverage authority matrix
- W2.3b quality_gates subprocess success asserts remainder
- W2.4/W2.5 release preconditions + benchmark signal
- K3.* skill eval contracts
- F4.* feature pilots
- S1.2 remainder (mode/provider/index generation provenance)
- Cut v0.1.35 via `release.yml` after merge when ready
