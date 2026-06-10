# GOAP Plan — Comprehensive Analysis Remediation (2026-06-06)

- **ADR**: [ADR-055](adr/ADR-055-Comprehensive-Analysis-v0.1.32.md)
- **Trigger**: Fresh `main` analysis (ff to `47a8609c`) found doc/reality drift +
  4 genuine implementation gaps after v0.1.32 shipped (2026-05-24).
- **Primary Goal**: Reconcile plan docs with released reality, eliminate the 4
  remaining production placeholders, and prepare a clean `v0.1.33`.
- **Complexity**: Moderate (single-agent feasible; gaps are module-isolated).
- **Strategy**: Hybrid — Phase 0 sequential (docs), Phase 1 parallel-safe (code),
  Phase 2 sequential (validate + release).

---

## GOAP Skill Stack

- **Planning**: `goap-agent`, `agent-coordination`
- **Docs**: `agents-update`, `architecture-validation`
- **Code**: `feature-implement`, `code-quality`, `debug-troubleshoot`
- **Validation**: `test-runner`, `code-quality`, `release-guard`

---

## Phase 0 — Documentation Truth Reconciliation (P0, sequential)

| WG | Task | Owner Skill | Status | Evidence |
|----|------|-------------|--------|----------|
| WG-175 | Mark v0.1.32 missing-impl sprint **Released** in GOAP_STATE / ROADMAP_ACTIVE / CURRENT | `agents-update` | ✅ Complete | This sprint's doc edits |
| WG-176 | Repoint dangling `ADR-055-Missing-Implementation` + `GOAP_MISSING_IMPLEMENTATION_2026-05-21` refs to ADR-055 (this analysis) + this GOAP plan | `agents-update` | ✅ Complete | `rg ADR-055\|GOAP_MISSING` across plans |
| WG-177 | Correct `STATUS/CURRENT.md` metrics: release v0.1.32, `#[allow(dead_code)]` 0→15 | `agents-update` | ✅ Complete | `rg -c allow\(dead_code\) memory-*/src` = 15 |

## Phase 1 — Close the 4 Genuine Gaps (P1, parallel-safe)

| WG | Gap | Location | Owner Skill | Status |
|----|-----|----------|-------------|--------|
| WG-171 | `episode_success_rate=99.0` placeholder → real failure counter | `memory-mcp/src/monitoring/types.rs:363` | `feature-implement` | 🔴 Open |
| WG-172 | Turso cache `query_hits/query_misses/evictions/expirations` stubbed at 0 | `memory-storage-turso/src/cache/wrapper.rs:142` | `feature-implement` | 🔴 Open |
| WG-173 | Cascade `estimate_api_call_probability` returns constant `0.5` (csm-only) — heuristic OR remove | `memory-core/src/retrieval/cascade/mod.rs:446` | `feature-implement` | 🔴 Open |
| WG-174 | `generate_simple_embedding` "placeholder" with a **production caller** — document/promote OR replace call site (NOT cfg-gate) | `memory-core/src/memory/retrieval/helpers.rs:59` + `context.rs:393` | `analysis-swarm` → `feature-implement` | 🔴 Open |

## Phase 2 — Validation & v0.1.33 Release (P2, sequential)

| WG | Step | Status |
|----|------|--------|
| WG-178 | `./scripts/code-quality.sh fmt && clippy --workspace` | 🟡 Queued |
| WG-179 | `cargo nextest run --all` + `cargo test --doc` | 🟡 Queued |
| WG-180 | `cargo nextest run -p do-memory-core --features csm` (covers WG-173) | 🟡 Queued |
| WG-181 | `./scripts/quality-gates.sh` (≥90%) | 🟡 Queued |
| WG-182 | Sprint-exit `rg` audit returns 0 genuine placeholders | 🟡 Queued |
| WG-183 | Bump workspace to `0.1.33` + CHANGELOG | 🟡 Queued |
| WG-184 | `gh release create v0.1.33` via `release-guard` | 🟡 Queued |

---

## Sprint-Exit Gate

```bash
rg -ni 'not yet implemented|// *placeholder' memory-*/src \
  | grep -vi 'placeholders for IN\|placeholders.join\|let placeholders\|? placeholder'
# expect: 0 matches
```

## Notes / Decisions Carried Forward

- WG-156 (`pattern_match_score`) and WG-157 (`memory_usage_mb`) from the prior audit
  are **already resolved** in the released tree — excluded from this sprint.
- "Resolved-by-removal" or "resolved-by-documentation" are acceptable outcomes for
  WG-173/WG-174, consistent with the ADR-055-era typed-error decisions (WG-152/153).
