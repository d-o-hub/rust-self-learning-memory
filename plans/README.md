# Plans Directory

**Workspace**: `v0.1.48` · **Released tag**: `v0.1.47` (`854f6f8d`, GitHub Release 2026-10-28T20:41:48Z, all four registered crates published) · **Main baseline**: `e12c7101`
**Active plan**: **post-`v0.1.47` hardening** — [ACTIONS.md](ACTIONS.md) ACT-413...ACT-419 (released-CLI dogfood fixes #1225/#1226, advisory #1224, generated-lockfile alert #1227, version restore #1228, toolchain pin #1222, ADR-079 stage 4 probe #1229), all closed. The audit-backlog wave [GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md](GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md) and the release closeout [GOAP_RELEASE_CLOSEOUT_2026-10-16.md](GOAP_RELEASE_CLOSEOUT_2026-10-16.md) are closed. Per-issue register: [STATUS/GAP_ANALYSIS_LATEST.md](STATUS/GAP_ANALYSIS_LATEST.md).
**Completed previous wave**: v0.1.47 shipped 2026-10-28 (`854f6f8d`) and was dogfooded with the released CLI; v0.1.44/0.1.45/0.1.46 shipped earlier in the same chain (release pipeline C1–C7 in #1121/#1123/#1125)
**Last Updated**: 2026-10-30
**Open PRs**: run `gh pr list --state open` (**1** on 2026-10-30 — #1230, this refresh) · **Open issues**: run `gh issue list --state open` (**0** on 2026-10-30)
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
| [GOAP_PR_REVIEW_CI_FIX_WAVE_2026-08-07.md](archive/2026-09-consolidation/superseded-goap/GOAP_PR_REVIEW_CI_FIX_WAVE_2026-08-07.md) | **Historical**: PR review & CI fix wave (2026-08-07), superseded by merged #947 (2026-08-12) |
| [GOAP_CIT_A1_A2_A3_WORKFLOW_WAVE_2026-08-06.md](archive/2026-09-consolidation/superseded-goap/GOAP_CIT_A1_A2_A3_WORKFLOW_WAVE_2026-08-06.md) | **Historical**: CIT-A1/A2/A3 workflow-side wave (2026-08-06), superseded by merged #947 (2026-08-12) |
| [GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md](archive/2026-09-consolidation/superseded-goap/GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md) | **Historical**: CIT-A4/A5 implementation + plan truth (2026-08-06), superseded by merged #947 (2026-08-12) |
| [GOAP_ADR081_CAPABILITY_TRUTH_2026-08-10.md](archive/2026-09-consolidation/superseded-goap/GOAP_ADR081_CAPABILITY_TRUTH_2026-08-10.md) | **Historical**: ADR-081 §2 capability truth (2026-08-10), superseded by merged #947 (2026-08-12) |
| [GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md](archive/2026-09-consolidation/superseded-goap/GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md) | **Historical upstream**: CI trust + product-truth fixes + ADR-080 attribution capture |
| [GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md](GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md) | Reference recommendations backlog |
| [ANALYSIS_GH_CLI_SKILLS_AND_BEST_PRACTICES_2026-07-20.md](archive/2026-09-consolidation/analyses/ANALYSIS_GH_CLI_SKILLS_AND_BEST_PRACTICES_2026-07-20.md) | Official `gh` / `gh skill` skills + manual vs repo policy |

## Architecture

| Document | Purpose |
|----------|---------|
| [ARCHITECTURE/ARCHITECTURE_CORE.md](ARCHITECTURE/ARCHITECTURE_CORE.md) | Core system architecture |
| [ARCHITECTURE/ARCHITECTURE_PATTERNS.md](ARCHITECTURE/ARCHITECTURE_PATTERNS.md) | Design patterns |
| [ARCHITECTURE/ARCHITECTURE_INTEGRATION.md](ARCHITECTURE/ARCHITECTURE_INTEGRATION.md) | Integration architecture |
| [ARCHITECTURE/API_DOCUMENTATION.md](ARCHITECTURE/API_DOCUMENTATION.md) | API surface |

## ADRs

See [adr/README.md](adr/README.md) for the ADR index (including number-collision aliases).

Current proposals: [ADR-080 automatic recommendation attribution](adr/ADR-080-Automatic-Recommendation-Attribution.md)
and [ADR-081 attribution capability truth & feedback resolution](adr/ADR-081-Attribution-Capability-Truth-And-Feedback-Resolution.md)
(both `Proposed` pending maintainer acceptance; code evidence merged in #947, 2026-08-12).
[ADR-079 fail-closed CI control plane](adr/ADR-079-Fail-Closed-CI-Required-Check-Control-Plane.md) is **Accepted** — stage 3 live
(ruleset `9591004` requires `CI / Required`), stage 5 cleanup merged in #947 (2026-08-12),
stage 4 live fault-injection proof remains external maintainer evidence.
[ADR-083 release-gate CI event filtering](adr/ADR-083-Release-Gate-CI-Check-Event-Filtering.md) is **Accepted** —
only `push`/`pull_request` runs gate a release; `schedule`/`workflow_dispatch` runs are advisory
(LESSON-051's durable fix, ACT-405).

## Spikes

| Path | Role |
|------|------|
| [spikes/](spikes/) | Spike configs (TOML) |
| [STATUS/spikes/](STATUS/spikes/) | Decision artifacts (JSON) |

## Archive

Superseded plans live under [archive/](archive/) (do not treat as active backlog).  
Latest consolidation: [archive/2026-07-consolidation/](archive/2026-07-consolidation/).
