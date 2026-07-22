# Comprehensive Codebase Recommendations — 2026-07-20

- **Status**: Active backlog (post–v0.1.35; workspace `0.1.36` unreleased)
- **Audit commit**: refreshed 2026-07-22
- **Released tag**: `v0.1.35`
- **Open PRs**: R-E2 skill evals / plans truth (this wave); #877/#880/#881 merged
- **Open issues**: none (#879 resolved with release-docs path)
- **Coordinator**: goap-agent + agent-coordination
- **Supersedes**: archived `GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md` and dated 2026-06/07 execution plans
- **Archive**: `plans/archive/2026-07-consolidation/`

## 1. Executive summary

The 2026-07-14 → 2026-07-22 GOAP campaign closed the high-priority correctness, gate honesty, skill-eval, F4 pilot, and recommendations packages. PRs **#840–#881** are merged (including release docs #880 and rust-major #877); workspace is **`0.1.36`**. Remaining P0 is **ship v0.1.36** via `./scripts/release-manager.sh ship --execute` when main is green.

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
| Version | `Cargo.toml` `0.1.36`; tag `v0.1.35`; 26 unreleased commits | Unreleased development |
| Main CI | Recent CI green on main HEAD | Green enough to plan release |
| Open work | Release ship only | #877/#880/#881 merged |
| Production LOC >500 (non-test `src`) | No production offenders (`provider_config.rs` 237) | ✅ |
| `todo!` / `unimplemented!` / “not yet implemented” in prod `src` | 0 matches | Clean surface |
| Skills | Catalog + `ci-poll` evals present; routes complete | ✅ |
| `.agents/SKILLS.md` | Generated / maintained | ✅ keep in sync |
| ADR IDs | Duplicate **025** / **054** filenames | Aliased in `plans/adr/README.md` |
| F4 pilots | provenance, `storage journal`, model digests | ✅ operator surfaces landed |
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
| **R-A1** | Cut **v0.1.36** via `./scripts/release-manager.sh ship --execute` after CHANGELOG + Released Version docs | #880 merged; main green | 🟡 ship |
| **R-A2** | Immediately bump workspace to **0.1.37** after tag | AGENTS post-release rule | 🟡 After R-A1 |
| **R-A3** | Re-run `./scripts/release-manager.sh status` before ship | Status green before ship | 🟡 With R-A1 |

### Track B — Code quality & invariants (P0/P1)

| ID | Recommendation | Why | Acceptance |
|----|----------------|-----|------------|
| **R-B1** | Split provider configs ≤500 LOC | ✅ Done |
| **R-B2** | F4 CLI journal + MCP `with_provenance` | ✅ CLI journal; MCP already had provenance |
| **R-B3** | TECH_DEBT refresh | ✅ Done |
| **R-B4** | Ignored-test ratchet | ✅ 173/200 ceiling; keep ratchet |
| **R-B5** | ADR 025/054 aliases | ✅ `plans/adr/README.md` |

### Track C — Missing / partial implementations (P1)

| ID | Surface | Status | Recommendation |
|----|---------|--------|----------------|
| **R-C1** | Skill routing | ✅ 34/34 |
| **R-C2** | `ci-poll` evals | ✅ + `pr checks --watch` |
| **R-C3** | `.agents/SKILLS.md` | ✅ |
| **R-C4** | MCP `with_provenance` | ✅ Already on main/PR; docs updated |
| **R-C5** | `storage journal` CLI | ✅ |
| **R-C6** | Model digest docs + unit tests | ✅ EMBEDDINGS guide + `verify_model_artifact` tests |
| **R-C7** | Batch tools deferred | ✅ API_REFERENCE explicit deferred |

### Track D — Documentation contracts (P1)

| ID | Recommendation | Files | Acceptance |
|----|----------------|-------|------------|
| **R-D1** | README examples | Keep under doctest/CI; no known wasmtime claim |
| **R-D2** | README ↔ GATE_CONTRACT coverage | ✅ |
| **R-D3** | AGENTS skill quick-ref + gh bootstrap | ✅ |
| **R-D4** | `.agents/SKILLS.md` | ✅ |
| **R-D5** | agent_docs release + fail-closed | ✅ git_workflow + architecture |
| **R-D6** | CLI_COMMANDS journal + API provenance | ✅ |
| **R-D7** | Vision title | ✅ |

### Track E — Skills & harness (P1)

| ID | Recommendation | Why |
|----|----------------|-----|
| **R-E1** | K3.3 routes + inventory | ✅ |
| **R-E2** | Medium-risk skill evals expanded | ✅ Second wave (behavioral fixtures for thin skills) |
| **R-E3** | github-release-best-practices → release-guard only | ✅ Already |
| **R-E4** | Frontmatter name/description evals | ✅ medium set + compiler |
| **R-E5** | pycache gitignore | ✅ skill + skills/ + root |

### Track F — New features (P2, spike-gated)

Do **not** implement without spike artifacts under `plans/STATUS/spikes/`.

**Status 2026-07-20:** All R-F* **DEFER** — decision artifact  
`plans/STATUS/spikes/R-F-product-backlog-2026-07-20.json` (`decision: DEFER`).  
No product/research implementation this sprint.

| ID | Feature | Status |
|----|---------|--------|
| **R-F1**…**R-F10** | Distributed sync, OTel, multi-tenancy, SIMD, WG-108/125/135, relationship polish, ANN, OIDC | ⏸ DEFER (spike-gated) |

### Track G — Plans governance (P1 — this sprint)

| ID | Recommendation | Status |
|----|----------------|--------|
| **R-G1** | Archive superseded dated GOAP / analysis / CI plans | ✅ Done 2026-07-20 → `archive/2026-07-consolidation/` |
| **R-G2** | Keep root `plans/*.md` to canonical set only | ✅ Reduced; see `plans/README.md` |
| **R-G3** | Refresh CURRENT / GOALS / ACTIONS / GOAP_STATE / ROADMAP / GAP / VALIDATION | ✅ This document + companion updates |
| **R-G4** | Warn excess dated plans root files | ✅ `validate-plans.sh --active-set` |
| **R-G5** | One active analysis + gh analysis doc | ✅ |

### Track H — GitHub CLI skills & best practices (P1)

**Analysis**: `plans/ANALYSIS_GH_CLI_SKILLS_AND_BEST_PRACTICES_2026-07-20.md`

| ID | Recommendation | Status |
|----|----------------|--------|
| **R-H1** | Bootstrap official `gh` skill docs in AGENTS | ✅ |
| **R-H2** | `gh-skill` install docs | ✅ |
| **R-H3** | ci-poll → `gh pr checks --watch` / `gh run watch` | ✅ |
| **R-H4** | `gh pr update-branch` in pr-readiness | ✅ |
| **R-H5** | `release.yml` tag on main ancestry preflight | ✅ |
| **R-H6** | Ban manual `gh release create` for ship | ✅ |
| **R-H7** | Dual skill layout documented | ✅ |

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
