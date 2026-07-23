# Gap Analysis — 2026-07-22 (post v0.1.36)

**Generated**: 2026-07-22  
**Workspace**: `0.1.37` · **Tag**: `v0.1.36`  
**Full backlog**: [`../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`](../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md)

## Method

- Live tree + GitHub release/tag verification  
- Production `todo!` / `unimplemented!` → **0**  
- Recommendations register re-verified  

## Closed this wave

| Gap | Resolution |
|-----|------------|
| G-P0-1 v0.1.36 unreleased | ✅ Tagged `v0.1.36` via release-manager |
| G-P0-4 / G-P0-5 open CI PRs | ✅ #880 / #877 merged earlier |
| G-P1-7 medium-risk eval depth | ✅ R-E2 #883 |
| Docs integrity ship blocker | ✅ #885 |

## Open gaps (current)

### P0

*None.*

### P1

| ID | Gap | Track |
|----|-----|-------|
| G-P1-8 | Historical ADR number reuse on disk (aliased) | residual docs |
| G-P1-9 | Transitive Dependabot advisories | upstream |

### P2

| ID | Gap | Track |
|----|-----|-------|
| G-P2-1…6 | Research / vision epics (R-F*) | ⏸ DEFER |

## Explicit non-gaps

| Claim | Verdict |
|-------|---------|
| Working `execute_agent_code` backend | Intentional fail-closed |
| Batch MCP tools | Deferred product decision |
| Production LOC >500 | Closed |
| Medium-risk skill presence-only evals | Closed |
