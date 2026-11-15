# GOAP Goals Index

- **Last Updated**: 2026-11-15
- **Workspace**: `0.1.50` (release preparation) · **Latest published tag**: `v0.1.49` (`16d92d22`)
- **Status**: post-`v0.1.49` closeout. Every code-side goal in the audited backlogs is
  closed; no PR or issue is open. Remaining: upstream-blocked advisories (ACT-404) and
  maintainer-external ADR lifecycle acceptance.
- **Plan**: `ACTIONS.md` + `ROADMAPS/ROADMAP_ACTIVE.md`
- **Archive**: `plans/archive/2026-11-consolidation/`

## Active goals (2026-11-15)

| Goal | Priority | Acceptance | Status |
|------|----------|------------|--------|
| G-C1 one measured harness version across binary, pin, prose, gate contract and CI matrix | P2 | both probes green against published 0.4.6 | ✅ 2026-11-15 |
| G-C2 compact active plans; archive completed registers and stale trackers | P3 | `validate-plans.sh --all` green; dated root files ≤5 | ✅ 2026-11-15 |
| G-C3 single agent contract | P3 | `.jules/` and `GEMINI.md` removed; CI no longer lists them | ✅ 2026-11-15 |
| G-C4 keep the product backlog honest | P2 | authoritative R-F artifact is schema-valid and `NO-GO` | ✅ 2026-11-15 |
| G-C5 clear upstream dependency advisories when (and only when) the chains move | P1 | `cargo tree` proves the move; no no-op bump | ⏸ upstream |
| G-C6 accept ADR-080 / ADR-081 | P2 | maintainer lifecycle decision | ⏸ external |

## Completed goals (pointer)

All earlier G-A / G-B / Wave A goals are recorded in the archived registers and in
`plans/STATUS/archive/2026/`. The high-level outcome:

| Series | Outcome |
|--------|---------|
| Audit-backlog wave (2026-10-04) | 22 findings closed; W1–W6 + Wave A merged |
| Release closeout (2026-10-16) | publish guard, readiness ordering, schema sensor, ADR-079 stage 4 |
| Dogfood follow-ups (2026-10-30) | `storage stats` pattern count, context sub-signal budget, lockfile hygiene, toolchain pin |
| Release chain | `v0.1.44` … `v0.1.49` shipped; crates.io publish verified for the registered crates |
| Feature work | R-F8/R-F9/R-F10 shipped; R-F1…R-F7 gated `NO-GO` |

Do not re-list completed W/G tables here. Add a new goal only with an owner, an
acceptance command, and a place in the roadmap.
