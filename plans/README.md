# Plans Directory

**Workspace**: `0.1.36` (unreleased) · **Released tag**: `v0.1.35`  
**Active plan**: [GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md](GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md)  
**Last Updated**: 2026-07-20  
**Policy**: ADR-039 (canonical active set) + ADR-072 (authority / release path)

## Quick Navigation

| Document | Purpose |
|----------|---------|
| [STATUS/CURRENT.md](STATUS/CURRENT.md) | Live project status and metrics |
| [STATUS/VALIDATION_LATEST.md](STATUS/VALIDATION_LATEST.md) | Latest validation slice |
| [STATUS/GAP_ANALYSIS_LATEST.md](STATUS/GAP_ANALYSIS_LATEST.md) | Current gap register |
| [STATUS/CODEBASE_ANALYSIS_LATEST.md](STATUS/CODEBASE_ANALYSIS_LATEST.md) | Latest codebase analysis summary |
| [ROADMAPS/ROADMAP_ACTIVE.md](ROADMAPS/ROADMAP_ACTIVE.md) | Active development roadmap (forward only) |
| [GOALS.md](GOALS.md) | GOAP goal index |
| [ACTIONS.md](ACTIONS.md) | Action backlog |
| [GOAP_STATE.md](GOAP_STATE.md) | GOAP phase snapshot |
| [GATE_CONTRACT.md](GATE_CONTRACT.md) | Local/CI quality gate matrix |
| [GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md](GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md) | **Active**: full recommendations backlog |

## Architecture

| Document | Purpose |
|----------|---------|
| [ARCHITECTURE/ARCHITECTURE_CORE.md](ARCHITECTURE/ARCHITECTURE_CORE.md) | Core system architecture |
| [ARCHITECTURE/ARCHITECTURE_PATTERNS.md](ARCHITECTURE/ARCHITECTURE_PATTERNS.md) | Design patterns |
| [ARCHITECTURE/ARCHITECTURE_INTEGRATION.md](ARCHITECTURE/ARCHITECTURE_INTEGRATION.md) | Integration architecture |
| [ARCHITECTURE/API_DOCUMENTATION.md](ARCHITECTURE/API_DOCUMENTATION.md) | API reference (plans-local) |
| [adr/](adr/) | Architecture Decision Records (ADR-022+) |

## Spikes & evidence

| Path | Purpose |
|------|---------|
| [spikes/](spikes/) | Spike config TOMLs (F4.x, S1.1c, …) |
| [STATUS/spikes/](STATUS/spikes/) | Spike decision artifacts (GO / NO-GO) |

## Long-Term Vision

| Document | Purpose |
|----------|---------|
| [ROADMAPS/ROADMAP_V030_VISION.md](ROADMAPS/ROADMAP_V030_VISION.md) | Multi-instance, multi-tenancy, observability vision |

## Archive

Historical and completed documents live under `archive/`:

| Path | Contents |
|------|----------|
| `archive/2025-deprecated/` | Deprecated 2025 documents |
| `archive/2026-01-completed/` | January 2026 completed work |
| `archive/2026-02-completed/` | February 2026 completed work |
| `archive/2026-03-consolidation/` | March 2026 ADR-039 consolidation |
| **`archive/2026-07-consolidation/`** | **July 2026 post-v0.1.35 cleanup** (completed sprints, analyses, CI plans, research WGs, stale status) |

### Active-set rule (ADR-039)

Only forward-looking / living documents stay unarchived:

1. Navigation (`README.md`)
2. Status + validation + gap + latest analysis
3. Roadmap + GOAP trio (GOALS / ACTIONS / GOAP_STATE)
4. Gate contract
5. One active recommendations / execution plan
6. ADRs, architecture, spike artifacts

Dated execution plans archive when the sprint merges. Do not keep parallel status files.

### Validation

```bash
./scripts/validate-plans.sh --active-set --version-state --identifiers --links
```
