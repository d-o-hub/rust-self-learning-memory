# Gap Analysis — 2026-07-21

**Generated**: 2026-07-21  
**Audit commit**: `1ebab995` (`main`)  
**Workspace**: `0.1.36` unreleased · **Tag**: `v0.1.35` · **Unreleased commits**: 26  
**Full backlog**: [`../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`](../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md)

## Method

- Live tree inspection (Cargo version, production LOC, skills, ADR IDs)
- `rg` for `todo!` / `unimplemented!` / “not yet implemented” in production `src` → **0**
- GitHub: open PRs **#880**, **#877**; open issue **#879** (release drift)
- Prior gap register (2026-07-20) re-verified; completed rows closed with evidence

## Resolved since 2026-07-20 register

| Historical gap | Resolution |
|----------------|------------|
| G-P0-2 `provider_config.rs` >500 LOC | Split → 237 LOC (R-B1 / #878) |
| G-P0-3 ADR 025/054 collision | Alias registry in `plans/adr/README.md` (R-B5); files retained historically |
| G-P1-1 Incomplete skill routing | `skill-rules.json` expanded (#878) |
| G-P1-2 `ci-poll` missing evals | evals present (#878) |
| G-P1-3 Missing `.agents/SKILLS.md` | Generated + maintained |
| G-P1-4 F4 journal / provenance UX | `storage journal` CLI; MCP provenance already present |
| G-P1-5 TECH_DEBT stale | Refreshed with AGENTS (#878) |
| G-P1-6 Vision title v0.1.9+ | Updated (R-D7) |
| S1/W2/K3/F4 campaign items | Shipped 2026-07-16…20 |

## Open gaps (current)

### P0

| ID | Gap | Evidence | Track |
|----|-----|----------|-------|
| G-P0-1 | v0.1.36 unreleased (26 commits) | `git rev-list --count v0.1.35..origin/main` = 26; issue #879 | R-A1 |
| G-P0-4 | Release-prep PR CI must be CLEAN | PR #880 (docs); then ship via release-manager | R-A1 |
| G-P0-5 | Dependabot rust-major CI | PR #877 — sha2/lz4_flex/cargo_metadata compat fixes | deps |

### P1

| ID | Gap | Evidence | Track |
|----|-----|----------|-------|
| G-P1-7 | Medium-risk skills lack deeper behavioral evals | Beyond K3.2 core set | R-E2 |
| G-P1-8 | Historical ADR number reuse remains on disk | Filenames still dual 025/054; aliases documented | R-B5 residual |
| G-P1-9 | Dependabot security alerts on default branch | GitHub reports open Dependabot vulns (separate from #877) | security hygiene |

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
| Production `src` LOC >500 | **Closed** — remaining >500 files are `*_tests.rs` / test modules |
| `todo!` / `unimplemented!` in prod `src` | **0 matches** |

## Exit criteria for this gap register

- G-P0-1 closed by shipping `v0.1.36` and post-bump  
- G-P0-4 / G-P0-5 closed when #880 and #877 are green and merged as appropriate  
- P1 skill-depth rows scheduled or waived with owners  
- P2 rows remain spikes, not silent code stubs  
