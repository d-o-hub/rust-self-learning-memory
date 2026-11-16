# GOAP State Snapshot

- **Last Updated**: 2026-11-15
- **Version**: workspace `0.1.51` (post-release bump; latest published tag `v0.1.50`,
  `0fe1b89d`, GitHub Release published 2026-11-16T08:45:18Z)
- **Branch**: post-release bump based on `main` @ `0fe1b89d`
- **Open PRs / issues**: 0 / 0 (`gh pr list --state open`, `gh issue list --state open`)
- **Active plan**: post-`v0.1.49` closeout. No code-side gap; the only open items are
  upstream-blocked Dependabot advisories (ACT-404) and maintainer-external ADR lifecycle
  acceptance (ADR-080/081).
- **Register**: [`STATUS/GAP_ANALYSIS_LATEST.md`](STATUS/GAP_ANALYSIS_LATEST.md)
- **Release**: `v0.1.49` shipped and reached crates.io (core, redb, turso, mcp); the
  released CLI was dogfooded. Next release is not scheduled.
- **Harness**: pin `v0.4.6`, measured green; both probes and the 31-case compat contract pass.
- **Archive**: `plans/archive/2026-11-consolidation/`

---

## Next queue

- **Release**: no tag planned. When cadence or a feature justifies it, run
  `release-guard` + `./scripts/release-manager.sh ship --execute`; released-CLI dogfood
  remains mandatory.
- **Dependabot (ACT-404)**: four upstream families, two lockfiles. Re-test only when the
  pinning chain moves; see `STATUS/GAP_ANALYSIS_LATEST.md`.
- **ADR lifecycle**: ADR-080/081 acceptance is maintainer-external, not a code blocker.
- **Product backlog**: R-F1…R-F7 are `NO-GO` for this cycle; promote only on a measured
  spike (`STATUS/spikes/R-F-product-backlog-2026-11-15.json`).
- **Plans hygiene**: keep dated root files ≤5; archive completed waves under
  `plans/archive/<year>-<month>-consolidation/`.

## Current truth flags

```text
release_current                    = true   (v0.1.50 tagged 2026-11-16 on 0fe1b89d; workspace 0.1.51)
crates_io_publish_truthful         = true   (0.1.50 published for core/redb/turso/mcp)
released_cli_dogfood               = true   (digest matched; --version 0.1.50; 3 steps, 2 patterns, stats parity)
required_ci_causal                 = true   (ruleset 9591004 requires Codacy + CI / Required; same-run aggregate)
audit_backlog_registered           = true   (all 22 findings closed; archived 2026-11-15)
ignored_test_inventory             = true   (159 entries matched; ceiling 200; isolated Turso suite)
storage_stats_pattern_count_truthful = true (#1225; 2 == 2 across processes)
context_subsignal_weights_preserved  = true (#1226; 5/5 Rust on top)
harness_pin_measured               = true   (v0.4.6; compat + doctrine + 31 contract cases)
plans_active_set_compact           = true   (2026-11-15; dated root files archived)
single_agent_contract              = true   (AGENTS.md; .jules/ and GEMINI.md removed)
product_backlog_gated              = true   (R-F1…R-F7 NO-GO; R-F8/R-F9/R-F10 shipped)
```

## Closed campaigns (pointer)

| Campaign | Result |
|----------|--------|
| v0.1.44…v0.1.50 release chain | ✅ shipped; publish chain verified from `0.1.46` onward; `0.1.50` dogfooded |
| Audit-backlog wave (2026-10-04) | ✅ all 22 findings closed; W1–W6 + Wave A merged |
| Release closeout (2026-10-16) | ✅ ACT-399…ACT-412 landed |
| ADR-079 CI trust | ✅ ruleset live; stage-4 merge-block proof 2026-10-30 (#1229) |
| Dogfood follow-ups (2026-10-30) | ✅ #1224…#1229 |
| Harness / plans hygiene (2026-11-15) | ✅ pin 0.4.6, compact plans, single agent contract |

Full history: `plans/archive/2026-11-consolidation/` and
`plans/archive/2026-07-consolidation/completed-sprints/`.
