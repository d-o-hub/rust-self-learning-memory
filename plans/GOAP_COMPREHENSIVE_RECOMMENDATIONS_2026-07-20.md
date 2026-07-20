# Comprehensive Codebase Recommendations — 2026-07-20

- **Status**: Active backlog (post–v0.1.35; workspace `0.1.36` unreleased)
- **Audit commit**: `2e0a2b89` (`main`)
- **Released tag**: `v0.1.35`
- **Open PRs / issues**: none
- **Coordinator**: goap-agent + agent-coordination
- **Supersedes**: archived `GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md` and dated 2026-06/07 execution plans
- **Archive**: `plans/archive/2026-07-consolidation/`

## 1. Executive summary

The 2026-07-14 → 2026-07-18 GOAP campaign closed the high-priority correctness, gate honesty, skill-eval, and F4 pilot packages. PRs **#840–#875** are merged; workspace is **`0.1.36`** with **18 unreleased commits** on top of `v0.1.35`.

This plan is the **single active recommendations register**. It covers:

1. **Code / product gaps** still worth shipping  
2. **Missing or partial implementations** (verified vs historical)  
3. **New feature proposals** (evidence-gated)  
4. **README / AGENTS / agent_docs / skills** hygiene  
5. **Plans folder hygiene** (ADR-039 re-applied this sprint)

### Confidence classes

| Class | Meaning |
|-------|---------|
| **Verified** | Observed on `main` at audit commit |
| **Partial** | Code or skill exists; contract incomplete |
| **Proposal** | Requires spike + go/no-go before implementation |

---

## 2. Verified current state (2026-07-20)

| Area | Evidence | Verdict |
|------|----------|---------|
| Version | `Cargo.toml` `0.1.36`; tag `v0.1.35` | Unreleased development |
| Main CI | Recent CI / Security / Storage Matrix / Skill Evals **success** | Green enough to plan release |
| Open work | `gh issue list` / `gh pr list` → empty open sets | No tracker debt |
| Production LOC >500 | `provider_config.rs` **511** | One invariant breach |
| `todo!` / `unimplemented!` / “not yet implemented” in prod `src` | 0 matches | Clean surface |
| Skills | 34 skill dirs; 33 eval files; **`ci-poll` missing evals** | Partial |
| Skill routes | `skill-rules.json` covers **16 / 34** skills | Partial (K3.3 remainder) |
| `.agents/SKILLS.md` | Generated 2026-07-20 from catalog | ✅ present; keep in sync |
| ADR IDs | Duplicate **025** and **054** filenames | Registry debt |
| F4 pilots | `provenance_api.rs`, `op_journal.rs`, model digests, skill contract compiler | Code present; operator UX incomplete |
| Batch MCP tools | Intentionally deferred / fail-closed | Documented in AGENTS |
| Code execution | Fail-closed; S1.1c **NO-GO** | Correct |

Prior campaign completion (do not re-open without regression):

- S1.1–S1.7, S1.1b/c, S1.2 remainder, S1.4b, S1.5/S1.5b  
- W2.1–W2.5 (incl. ci-parity, deny, workflow guards, benches)  
- K3.1/K3.2 (+ partial K3.3)  
- F4.1–F4.4 spikes GO; S1.1c NO-GO  
- Harness #861–#869, release-cadence-manager, release-guard path  

Archived source of truth for that work:  
`plans/archive/2026-07-consolidation/completed-sprints/`.

---

## 3. Recommendations by track

IDs use prefix **R** (recommendation). Priorities:

- **P0** — correctness, release safety, policy truth  
- **P1** — operator UX, skills/docs contract, maintainability  
- **P2** — product features and research epics  

### Track A — Release & version truth (P0)

| ID | Recommendation | Why | Acceptance |
|----|----------------|-----|------------|
| **R-A1** | Cut **v0.1.36** via `./scripts/release-manager.sh ship --execute` after CHANGELOG + Released Version docs | 18 commits since `v0.1.35`; release-drift gate already tracking | Tag = workspace version; `release.yml` GitHub Release; no `gh release create` |
| **R-A2** | Immediately bump workspace to **0.1.37** after tag (LESSON: equal version+tag blocks feat commits) | AGENTS post-release rule | Workspace > tag before next feat |
| **R-A3** | Re-run `./scripts/release-manager.sh status` and resolve any “main CI failed” stale signal | Status script reported 1 failed while recent runs success | Status green before ship |

### Track B — Code quality & invariants (P0/P1)

| ID | Recommendation | Why | Acceptance |
|----|----------------|-----|------------|
| **R-B1** | Split `memory-core/src/embeddings/config/provider_config.rs` (511 LOC) | Only production file over 500 LOC | `./scripts/quality-gates.sh --loc-only` clean |
| **R-B2** | Wire F4 operator surfaces: CLI/MCP diagnostics for retrieval provenance + op-journal health/repair | Pilots are library-level; users cannot exercise them | Documented commands; at least one integration test each |
| **R-B3** | Reconcile `docs/TECH_DEBT.md` with reality (ignored-test counts, batch tools, WASM wording) | Tech debt registry still claims WASM “disabled with issues” and stale ignore counts | TECH_DEBT matches fail-closed docs + current ignore inventory |
| **R-B4** | Audit remaining ignored tests; keep ratchet (`check-ignored-tests.sh`) and only lower ceiling with evidence | Historical ~100+ ignores; media STOR-01 libsql | Ratchet green; no unevidenced ceiling raise |
| **R-B5** | ADR registry hygiene: alias duplicates **025** and **054**; ban new collisions | Two ADR numbers reused | `./scripts/validate-plans.sh --identifiers` green; index in `plans/adr/README.md` or registry note |

### Track C — Missing / partial implementations (P1)

| ID | Surface | Status | Recommendation |
|----|---------|--------|----------------|
| **R-C1** | Skill routing (K3.3 remainder) | 16/34 skills in `skill-rules.json` | Add routes for 18 unrouted skills; expand negative-routing fixtures; `./scripts/validate-skill-routes.sh` green |
| **R-C2** | `ci-poll` skill evals | No `evals/evals.json` | Add positive + timeout/cancel fixtures; include in Skill Evals |
| **R-C3** | `.agents/SKILLS.md` inventory | ✅ Generated 2026-07-20; keep synced when skills change | Regenerate via catalog compiler; wire `--check` in CI if not already |
| **R-C4** | F4 provenance in MCP tool responses | Partial library API | Optional fields on `query_memory` / pattern search with redaction policy (ADR-074) |
| **R-C5** | Operation journal repair CLI | Core module exists | `do-memory-cli storage journal status|repair` (or equivalent) |
| **R-C6** | Model digest enforcement path in default local embedding load | S1.5b code path | Document config keys; e2e fixture for wrong-digest reject |
| **R-C7** | Batch MCP tools | Explicitly deferred | Keep deferred **or** schedule ADR to remove dead metadata if still advertised anywhere |

### Track D — Documentation contracts (P1)

| ID | Recommendation | Files | Acceptance |
|----|----------------|-------|------------|
| **R-D1** | Keep README examples compilable (`TaskContext`, playbook, local mode) | `README.md` | `cargo test --doc` + optional `validate-readme-contracts.sh` |
| **R-D2** | Align README quality claims with GATE_CONTRACT (coverage floor vs target vs Codecov) | `README.md`, `plans/GATE_CONTRACT.md` | One matrix; no conflicting % |
| **R-D3** | AGENTS.md skill quick-ref: add `release-guard`, `pr-readiness`, `ci-poll`, `memory-harness` | `AGENTS.md` | Table lists high-frequency ops only; full inventory in SKILLS.md |
| **R-D4** | Fix HARNESS.md link to missing `.agents/SKILLS.md` | `HARNESS.md` | Link resolves |
| **R-D5** | Refresh `agent_docs/` for post-v0.1.35: release path, gate contract, fail-closed exec | `agent_docs/*.md` | No manual `gh release create`; no wasmtime-backend claim |
| **R-D6** | Regenerate or verify `docs/CLI_COMMANDS.md` / `docs/API_REFERENCE.md` against Clap + tool registry | `docs/` | Diff-check or generator `--check` |
| **R-D7** | Update `plans/ROADMAPS/ROADMAP_V030_VISION.md` title/dates (still “v0.1.9+”) | vision doc | Honest future framing for 0.2/1.0 |

### Track E — Skills & harness (P1)

| ID | Recommendation | Why |
|----|----------------|-----|
| **R-E1** | Complete K3.3: inventory + routes + link lint for all 34 skills | Partial completion from #873 |
| **R-E2** | Behavioral evals for remaining medium-risk skills: `agent-coordination`, `analysis-swarm`, `storage-sync`, `memory-harness`, `loop-agent` | K3.2 covered high-risk set only |
| **R-E3** | `github-release-best-practices` skill: ensure it only references release-guard (no manual release drift) | Historical contradiction risk |
| **R-E4** | Skill frontmatter: verify every skill has name, description, allowed tools, related skills | Contract compiler F4.4 GO — enforce in CI |
| **R-E5** | Remove or gitignore `__pycache__` under `web-doc-resolver` | Noise in skill tree |

### Track F — New features (P2, spike-gated)

Do **not** implement without spike artifacts under `plans/STATUS/spikes/`.

| ID | Feature | Rationale | Spike seed |
|----|---------|-----------|------------|
| **R-F1** | Distributed multi-instance sync (CRDT / vector clocks) | Vision roadmap; multi-agent deployments | F5.1 |
| **R-F2** | Prometheus + OpenTelemetry export for episode/retrieval/embedding SLIs | Ops readiness | F5.2 |
| **R-F3** | Multi-tenancy / RBAC for MCP | SaaS path | F5.3 |
| **R-F4** | SIMD similarity (WG-110) | Only if bench shows clear win | F5.4 / WG-110 |
| **R-F5** | Version-retained persistence / concept drift (WG-108) | Long-horizon learning | F5.5 / WG-108 |
| **R-F6** | Routing-free MoE eval (WG-125) | Research | keep evaluation only until GO |
| **R-F7** | Federated HDC multi-agent (WG-135) | Research evaluation exists | promote only with GO |
| **R-F8** | CLI `relationship show` + global cycle validation if still hidden | Historical gap register; re-verify before work | product polish |
| **R-F9** | ANN semantic episode retrieval hardening (post #775 lineage) | Performance path for large stores | bench vs BM25/HDC cascade |
| **R-F10** | Trusted Publishing (OIDC) for crates.io | AGENTS future note | release engineering |

### Track G — Plans governance (P1 — this sprint)

| ID | Recommendation | Status |
|----|----------------|--------|
| **R-G1** | Archive superseded dated GOAP / analysis / CI plans | ✅ Done 2026-07-20 → `archive/2026-07-consolidation/` |
| **R-G2** | Keep root `plans/*.md` to canonical set only | ✅ Reduced; see `plans/README.md` |
| **R-G3** | Refresh CURRENT / GOALS / ACTIONS / GOAP_STATE / ROADMAP / GAP / VALIDATION | ✅ This document + companion updates |
| **R-G4** | Make `validate-plans.sh --active-set` optionally warn on excess dated root files | Proposed follow-up script enhancement |
| **R-G5** | One active analysis file: this recommendations doc | ✅ |

---

## 4. Suggested execution waves

### Wave 0 — Docs & plans truth (this change)

- Archive + recommendations + canonical plan refresh  
- No release; no code behavior change  

### Wave 1 — Release v0.1.36

1. CHANGELOG Unreleased → 0.1.36 notes  
2. Align Released Version docs  
3. `release-manager.sh ship --execute`  
4. Bump to 0.1.37  

### Wave 2 — Invariants & skills completeness

- R-B1 LOC split  
- R-C1/C2/C3 + R-E1/E2  
- R-B5 ADR aliases  
- R-D3/D4/D5  

### Wave 3 — F4 productization

- R-B2, R-C4, R-C5, R-C6  
- Operator docs in README + CLI help  

### Wave 4 — Product / research (optional)

- Spike R-F* items independently; promote only on GO  

---

## 5. Explicit non-goals (for now)

- Re-opening Wasmtime/WASI (S1.1c NO-GO stands)  
- Re-implementing closed harness issues #861–#869  
- Re-running completed S1/W2 packages without regression evidence  
- Manual GitHub releases  
- Raising coverage floor without a dedicated measurement sprint  

---

## 6. Validation commands (for implementers)

```bash
./scripts/validate-plans.sh --active-set --version-state --identifiers --links
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy --workspace
./scripts/build-rust.sh check
cargo nextest run --all
cargo test --doc
./scripts/quality-gates.sh
./scripts/run-evals.sh --fixtures
./scripts/validate-skill-routes.sh
./scripts/validate-gate-contract.sh --ci-parity
./scripts/release-manager.sh status
```

---

## 7. Priority scorecard (next 2 sprints)

| Priority | IDs | Outcome |
|----------|-----|---------|
| P0 | R-A1–A3, R-B1, R-B5 | Released 0.1.36; LOC + ADR registry clean |
| P1 | R-C1–C6, R-D*, R-E*, R-B2–B4, R-G4 | Skills/docs/operator contract complete |
| P2 | R-F* | Optional product/research after spikes |

---

## 8. Cross-references

| Doc | Role |
|-----|------|
| `plans/STATUS/CURRENT.md` | Live status |
| `plans/ROADMAPS/ROADMAP_ACTIVE.md` | Forward roadmap |
| `plans/GOALS.md` / `ACTIONS.md` / `GOAP_STATE.md` | GOAP execution |
| `plans/GATE_CONTRACT.md` | Gate matrix |
| `plans/STATUS/GAP_ANALYSIS_LATEST.md` | Gap register |
| `plans/STATUS/VALIDATION_LATEST.md` | Latest validation |
| `plans/archive/2026-07-consolidation/` | Historical plans |
| ADR-039, ADR-072, ADR-073, ADR-074, ADR-075, ADR-076 | Governance / correctness ADRs |
