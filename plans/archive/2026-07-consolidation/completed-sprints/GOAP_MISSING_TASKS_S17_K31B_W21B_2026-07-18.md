# GOAP Missing Tasks Swarm — S1.7 + K3.1b + W2.1b (2026-07-18)

**Status**: Code complete — PR pending  
**Coordinator**: goap-agent  
**Strategy**: Hybrid — parallel swarm for code/CI packages, sequential quality + PR  
**Branch**: `feat/goap-s17-k31b-w21b-2026-07-18`  
**Source plan**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md`  
**Prior**: K3.1a/W2.1a merged (#851); S1.3–S1.6/W2.2 on main; v0.1.35 released  

---

## Goal Hierarchy

```
G0: Land next deferred P0/P1 packages from improvements plan
├── G1: S1.7 audit writer hardening (recursive redaction, size init, non-blocking write)
├── G2: K3.1b skill evals in CI (fixtures always; --changed on PR)
├── G3: W2.1b gate contract CI parity (validate-gate-contract --ci-parity)
├── G4: Update plans/ + learnings
└── G5: PR + all CI green + review
```

## Work Packages

| ID | Package | Plan ref | Owner type | Status |
|----|---------|----------|------------|--------|
| C1 | S1.7a recursive redaction + rotation size from existing file | S1.7a | feature-implement | ✅ |
| C2 | S1.7b bounded writer / spawn_blocking + drop metrics | S1.7b | feature-implement | ✅ |
| C3 | K3.1b CI: fixtures + run-evals --changed | K3.1b | ci-fix | ✅ |
| C4 | W2.1b expand --ci-parity + wire job; GATE_CONTRACT update | W2.1b | ci-fix | ✅ |
| C5 | Tests for audit + script smoke | — | test-runner | ✅ |
| C6 | Plans + LESSONS + CHANGELOG | — | docs | ✅ |
| C7 | PR + CI green | — | pr-readiness | 🟡 |

## Atomic acceptance (from improvements ledger)

| Child | Validation |
|-------|------------|
| S1.7a | `cargo nextest run -p do-memory-mcp audit` — nested secret + existing-size fixtures |
| S1.7b | `cargo nextest run -p do-memory-mcp audit_writer` — slow-writer/overflow drop metrics |
| K3.1b | `./scripts/run-evals.sh --fixtures`; CI runs fixtures + `--changed` |
| W2.1b | `./scripts/validate-gate-contract.sh --ci-parity` |

## Deferred (next PRs)

- W2.4 / W2.5 release preconditions + benchmark signal  
- K3.2 behavioral high-risk skill fixtures  
- S1.2 remainder (mode/provider/index generation provenance)  
- F4 pilots  

## Evidence

| Gate | Result |
|------|--------|
| `cargo nextest run -p do-memory-mcp audit` | ✅ 33 passed |
| `./scripts/run-evals.sh --fixtures` | ✅ |
| `./scripts/validate-gate-contract.sh --ci-parity` | ✅ |
| `cargo clippy -p do-memory-mcp -- -D warnings` | ✅ |
| PR CI | 🟡 |

## Learnings

- Seed audit rotation size from file metadata at open (LESSON-018).
- Cheap skill/gate CI without cargo keeps K3.1/W2.1 checks on every PR (LESSON-019).
