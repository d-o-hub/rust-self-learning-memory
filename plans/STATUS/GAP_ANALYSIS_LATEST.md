# Gap Analysis — 2026-07-20

**Generated**: 2026-07-20  
**Audit commit**: `2e0a2b89` (`main`)  
**Workspace**: `0.1.36` unreleased · **Tag**: `v0.1.35`  
**Full backlog**: [`../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`](../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md)

## Method

- Live tree inspection (Cargo version, LOC, skills, ADR IDs, F4 modules)
- `rg` for `todo!` / `unimplemented!` / “not yet implemented” in production `src` → **0**
- GitHub: no open PRs or issues
- Prior gap register (2026-04/06) treated as historical; re-verified claims only

## Resolved since prior gap registers

| Historical gap | Resolution |
|----------------|------------|
| S1.1–S1.7 correctness package | Shipped 2026-07-16…18 |
| Soft-pass cargo audit / false-green gates | W2.2–W2.5 |
| Synthetic skill evals (`exec: true`) | K3.1/K3.2 |
| Pattern list empty after complete (#831) | v0.1.35 |
| `--db-path` ignored (#830) | v0.1.35 |
| Episode complete no-op on store failure (#847) | ADR-075 |
| Release drift tooling | #843 + release-cadence-manager |
| F4 pilots without spikes | Spikes GO in `STATUS/spikes/` |
| Plans bloat / contradictory status | Archived → `archive/2026-07-consolidation/` |

## Open gaps (current)

### P0

| ID | Gap | Evidence | Track |
|----|-----|----------|-------|
| G-P0-1 | v0.1.36 unreleased (18 commits) | `git rev-list --count v0.1.35..HEAD` = 18 | R-A1 |
| G-P0-2 | Production file >500 LOC | `provider_config.rs` 511 | R-B1 |
| G-P0-3 | Duplicate ADR numbers 025, 054 | `plans/adr/` filename collision | R-B5 |

### P1

| ID | Gap | Evidence | Track |
|----|-----|----------|-------|
| G-P1-1 | Incomplete skill routing | 16/34 skills in `skill-rules.json` | R-C1 |
| G-P1-2 | `ci-poll` has no evals | Missing `evals/evals.json` | R-C2 |
| G-P1-3 | ~~Missing `.agents/SKILLS.md`~~ | Generated 2026-07-20; keep in sync with catalog | R-C3 partial (routing still open) |
| G-P1-4 | F4 pilots not fully exposed via CLI/MCP | Core modules present; limited operator surface | R-B2 / R-C4–C5 |
| G-P1-5 | TECH_DEBT.md stale (WASM / ignore counts) | `docs/TECH_DEBT.md` | R-B3 |
| G-P1-6 | Vision roadmap still titled v0.1.9+ | `ROADMAP_V030_VISION.md` | R-D7 |
| G-P1-7 | Medium-risk skills lack behavioral evals | Beyond K3.2 set | R-E2 |

### P2 (product / research)

| ID | Gap | Notes | Track |
|----|-----|-------|-------|
| G-P2-1 | WG-108 version-retained persistence | Backlog epic | R-F5 |
| G-P2-2 | WG-110 SIMD similarity | Bench-gated | R-F4 |
| G-P2-3 | WG-125 MoE routing eval | Research only | R-F6 |
| G-P2-4 | WG-135 federated HDC | Evaluation archived | R-F7 |
| G-P2-5 | Distributed sync / multi-tenancy / OTel | Vision | R-F1–F3 |
| G-P2-6 | Trusted Publishing OIDC | crates.io | R-F10 |

## Explicit non-gaps

| Claim | Verdict |
|-------|---------|
| `execute_agent_code` working backend | **Not a gap** — intentional fail-closed |
| Batch MCP tools | Deferred by product decision; document only |
| 2026-05 telemetry stubs WG-156–162 | Prior campaign treated as remediated; re-open only if `rg` finds constants again |

## Exit criteria for this gap register

- All P0 rows closed or waived with evidence on the thread  
- P1 skill/docs rows closed or scheduled with owners  
- P2 rows remain spikes, not silent code stubs  
