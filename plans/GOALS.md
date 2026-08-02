# GOAP Goals Index

- **Last Updated**: 2026-07-30
- **Status**: CI trust, product-truth remediation, and ADR-078 attribution capture proposed
- **Workspace**: `0.1.38` · **Tag**: `v0.1.37`
- **Plan**: `plans/GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`
- **Archive**: `plans/archive/2026-07-consolidation/`

## Active goals (2026-07-30)

| Goal | Rec IDs | Priority | Status |
|------|---------|----------|--------|
| Make first-party validation causally merge-required | CIT-A1 / ADR-079 | P0 | Proposed |
| Fail closed across cancellation, missing checks, commit lint, Dependabot, and forks | CIT-A2 | P0 | Planned |
| Reconcile local/CI gate scope and semantic drift validation | CIT-A3 | P1 | Planned |
| Make release/publish/fuzz automation truthful and observable | CIT-A4/A5 | P1 | Planned |
| Make disabled cascade capability truthful | PTA-A1 | P0 | ✅ Implemented |
| Make storage metrics provenance-truthful | PTA-A2 | P0 | ✅ Implemented |
| Remove unsupported threshold command from CLI help | PTA-A3 | P1 | ✅ Implemented |
| Capture episode-bound recommendation attribution automatically | RAT-A1…A7 / ADR-078 | P1 | Proposed |
| Design idempotent feedback-to-ranking updates | Follow-up ADR | P2 | Deferred |
| R-F10 OIDC trusted publishing (publish-crates.yml) | R-F10 | P2 | 🔄 In progress |
| R-F4 SIMD cosine acceleration + benchmark variants | R-F4 | P2 | 🔄 In progress |
| Optional research/product spikes (R-F1…R-F3, R-F5…R-F7) | R-F* | P3 | ⏸ DEFER |

The ranking-learning loop remains open: ADR-078 captures trustworthy evidence but
does not yet apply feedback to recommendation scores. First-party CI is also not
currently merge-required; green workflow runs must not be described as branch
protection until ADR-079's staged ruleset migration completes. Active campaign:
R-F10 (ACT-325) and R-F4 (ACT-326) in progress; PTA-A1/A2/A3 implemented.

## Closed this wave (2026-07-20…25)

| Goal | Status |
|------|--------|
| Ship v0.1.36 (R-A1) | ✅ |
| Post-release bump 0.1.37 (R-A2) | ✅ #886 |
| R-E2 medium-risk skill evals | ✅ #883 |
| Docs integrity ship gate | ✅ #885 |
| Recommendations R-B/C/D/E/G/H | ✅ #878 |
| Plans truth refresh | ✅ #889 |
| Changelog hygiene | ✅ #887 |
| Cosine perf (8-way unrolled) | ✅ #888 |
| Gap tasks (ADR-074 docs, G-P1-12 pattern extract) | ✅ #891 |
| R-F8 CLI relationship show polish + R-F9 HNSW persistence | ✅ #893 |
| 6 new domain skills (40 total, all routed) | ✅ #894 |
| ADR-077 runtime embedding activation A1-A5 | ✅ main (`9ef4b742`, `e0f7f712`) |
| ADR-077 A6 validate / document / gate | ✅ #897 merged |
| Full gap audit — 0 P0/P1 code gaps | ✅ 2026-07-25 |

## Completed goal series (pointer only)

| Series | Outcome | Archive |
|--------|---------|---------|
| Post-v0.1.36 sprint (all tasks) | ✅ Complete 2026-07-24 | — |
| Recommendations #878 | Merged | — |
| 2026-07-14 improvements S1/W2/K3/F4 | Implemented; S1.1c NO-GO | `archive/2026-07-consolidation/completed-sprints/` |
| v0.1.35 CLI UX + ADR-075/076 | Released | same |
| Harness + release-cadence-manager | Merged | same |
| v0.1.36 release campaign | Shipped 2026-07-22 | — |

Do not re-list completed WG tables here.
