# GOAP Missing Tasks Swarm — 2026-07-16

**Status**: Code complete — PR #840 open (CI running)  
**Coordinator**: goap-agent + agent-coordination  
**Strategy**: Hybrid (parallel swarm → sequential quality → PR review)  
**Branch**: `feat/goap-missing-tasks-swarm-2026-07-16`  
**PR**: https://github.com/d-o-hub/rust-self-learning-memory/pull/840  
**Source plan**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md`  
**Related ADRs**: ADR-072 (Proposed), ADR-074 (S1.2 partial Implemented), ADR-039  
**Open issues**: #837 (docs — fixed in this PR), #838 (release drift — cut via release.yml after merge)

---

## Goal Hierarchy

```
G0: Land high-value missing plan tasks in one PR with evidence
├── G1: #837 restore fuzzy_match public rustdoc
├── G2: S1.2 retrieval cache identity includes TaskContext fields (ADR-074)
├── G3: W2.3 build-rust accepts do-memory-* hyphenated package names
├── G4: W2.6 split production source files >500 LOC
├── G5: S1.1a/D3.2 correct fail-closed execute_agent_code + wasmtime docs
├── G6: Update plans/ + ADR status notes
└── G7: PR + review readiness
```

## World State (start)

| Fact | Value |
|------|-------|
| Branch | main → feat/goap-missing-tasks-swarm-2026-07-16 |
| Workspace version | 0.1.35 (unreleased; latest tag v0.1.34) |
| Open PRs | 0 |
| Open issues | #837, #838 |
| CacheKey fields | query, domain, task_type, time_*, limit — **missing** language/framework/complexity/tags |
| build-rust crate regex | `^[a-z0-9_]+$` — rejects hyphens |
| Production >500 LOC | retry/mod 577, embedding 539, context 528, storage 507, local 507, checkpoint/ops 505 |
| ADR-072/073/074 | Proposed |

## Work Packages (atomic)

| ID | Package | Owner agent | Depends | Status |
|----|---------|-------------|---------|--------|
| A1 | #837 fuzzy rustdoc | feature-implementer | — | ✅ |
| A2 | S1.2 CacheKey + wire + tests | feature-implementer | — | ✅ |
| A3 | W2.3 build-rust hyphen + help text | feature-implementer | — | ✅ |
| A4 | W2.6 LOC splits (6 files) | refactorer + orchestrator | A2 for context.rs | ✅ |
| A5 | Docs contract: execute_agent_code / wasmtime | feature-implementer | — | ✅ |
| A6 | Plans/ADR progress | orchestrator | A1–A5 | ✅ |
| A7 | Quality gates + PR + review | orchestrator | A1–A6 | 🟡 PR #840; CI pending |

## Swarm Phases

```
Phase A (parallel): A1, A2, A3, A5
Phase B (parallel after A2): A4 (LOC splits including context if still over after A2)
Phase C (sequential): quality → A6 plans → commit/push → PR → review
```

## Success Criteria

1. `pub fn fuzzy_match` carries full rustdoc + doctests; helper has short `//` comment only
2. Same query/domain/limit with different language/framework/tags → distinct cache keys; tests pass
3. `./scripts/build-rust.sh check do-memory-core` succeeds
4. Zero production source files >500 LOC under quality-gates source gate
5. Docs state code execution unavailable/fail-closed; no `wasmtime-backend` feature claim
6. plans/GOAP_STATE, ACTIONS, GOALS, ROADMAP, CURRENT updated
7. PR open; review completed in this session

## Evidence Rules (ADR-072)

Every completion claim records command scope and result in the swarm log below.

---

## Swarm Log

### Phase A — parallel (2026-07-16)

| Agent | Result | Evidence |
|-------|--------|----------|
| A1+A3+A5 feature-implementer | fuzzy rustdoc, build-rust hyphens, fail-closed docs | `cargo test --doc -p do-memory-core -- fuzzy` 2 pass; `./scripts/build-rust.sh check do-memory-core` OK |
| A2 feature-implementer | CacheKey + with_task_context + 8 tests | `cargo nextest run -p do-memory-core` cache identity tests 9+ pass; clippy clean |
| A4 refactorer | LOC splits for retry, embedding, storage, local, checkpoint | All production ≤500; cargo check core+cli OK |

### Phase B — context.rs LOC (orchestrator)

- Extracted `cache_episodes_if_eligible` helper; `context.rs` 529 → ≤492.
- Production oversized: **NONE** under quality-gates source gate.

### Phase C — quality + plans

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check` | ✅ |
| `./scripts/code-quality.sh clippy --workspace` | ✅ 0 warnings |
| `./scripts/quality-gates.sh` | ✅ All gates PASSED |
| Targeted cache/fuzzy tests | ✅ 22/22 |
| Fuzzy doctests | ✅ 2/2 |
| CLI full suite | 411 pass; 1 pre-existing timeout (`test_relationship_full_cycle` 120s) — unrelated to this PR |

---

## Deferred (next PR)

- S1.3 lock-free backend awaits, S1.4 durable eviction, S1.5 embedding health states
- S1.6 full retry queue semantics, S1.7 audit writer
- W2.1/W2.2 gate contract unification, W2.4 release preconditions
- K3.* skill eval contracts, F4.* feature pilots
- Cut v0.1.35 via `release.yml` (never manual `gh release create`) after this PR merges if ready
