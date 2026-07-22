# Gap Analysis — 2026-07-22

**Generated**: 2026-07-22  
**Audit commit**: `main` HEAD at PR time  
**Workspace**: `0.1.36` unreleased · **Tag**: `v0.1.35`  
**Full backlog**: [`../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`](../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md)

## Method

- Live tree inspection (Cargo version, production LOC, skills, ADR IDs)
- `rg` for `todo!` / `unimplemented!` / “not yet implemented” in production `src` → **0**
- GitHub: open PRs/issues checked at audit time
- Prior gap register re-verified; completed rows closed with evidence
- Skill eval depth audit: medium-risk skills expanded to behavioral fixtures (R-E2)

## Resolved since 2026-07-21 register

| Historical gap | Resolution |
|----------------|------------|
| G-P0-4 Release docs CI | #880 merged |
| G-P0-5 Dependabot rust-major CI | #877 merged |
| G-P1-7 Medium-risk skill eval depth | R-E2 second wave (this PR) |
| Stale open-PR tracker rows | #881 + this refresh |

## Open gaps (current)

### P0

| ID | Gap | Evidence | Track |
|----|-----|----------|-------|
| G-P0-1 | v0.1.36 unreleased | tag still `v0.1.35`; workspace `0.1.36` | R-A1 |

### P1

| ID | Gap | Evidence | Track |
|----|-----|----------|-------|
| G-P1-8 | Historical ADR number reuse remains on disk | Filenames still dual 025/054; aliases documented | R-B5 residual (docs only) |
| G-P1-9 | Transitive Dependabot advisories | Upstream chains (libsql/openssl/webpki); not direct product surface | security hygiene |

### P2 (product / research)

| ID | Gap | Notes | Track |
|----|-----|-------|--------|
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
| Production `src` LOC >500 | **Closed** — remaining >500 files are test modules |
| `todo!` / `unimplemented!` in prod `src` | **0 matches** |
| MCP `with_provenance` | **Closed** — present + tested |
| Skill routes / SKILLS.md / ci-poll evals | **Closed** |
| Medium-risk skill eval presence-only fixtures | **Closed** by R-E2 second wave |

## Exit criteria for this gap register

- G-P0-1 closed by shipping `v0.1.36` and post-bump to `0.1.37`  
- P2 rows remain spikes, not silent code stubs  
