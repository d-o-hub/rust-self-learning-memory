# Validation Latest — 2026-10-06 (audit-backlog wave progress)

**Goal**: record the verified state of the audit-backlog wave before the next implementation wave — which slices
merged, which are in review, and the CI evidence attached to each head SHA (PR monitoring guardrail).

**Workspace**: `0.1.45` · **Branch**: `main` @ `0485bb66` · **Tag**: `v0.1.44`

## Method

- Merged slices verified by commit/SHA on `main` (`git log`) and by issue state (`gh issue view`): #1077, #1086,
  #1088 closed 2026-10-05; #1066 fix merged but issue still open (close with evidence).
- In-review PRs verified per head SHA via `gh pr checks` / the check-runs API; cancelled-vs-failed classified from
  the runs API — most 2026-10-05 jobs show `cancelled`, not `failed`.
- Local evidence: `./scripts/check-ignored-tests.sh` on the clean #1130 branch → `ignored_test_attrs=159`,
  `OK: Ignored test inventory valid and complete (159 items matched)` (2026-10-06).

## Result

| Slice | PR | Head SHA | Required rollup | Merge evidence |
|-------|----|----------|-----------------|----------------|
| W0 trackers | #1129 | `26b12fc8` | green | ✅ merged 2026-10-05 |
| W1 #1077 | #1131 | `874df209` | green | ✅ merged 2026-10-05 |
| W2 #1086 | #1135 | `75007b51` | green | ✅ merged 2026-10-05 |
| W3 #1088 | #1134 | `347b3296` | green | ✅ merged 2026-10-05 |
| W4 #1066 | #1139 | `f1c31699`, `0485bb66` | green | ✅ merged 2026-10-05 |
| W5 #1075 | #1138 | `6155e37b` | `CI / Required` ✅, Codacy ✅; non-required workflows cancelled at 2026-10-05 19:39 → rerun 2026-10-06 | ⏳ |
| W6 #1085 | #1140 | `4264f8aa` | aggregate failed closed on the cancelled set (correct behaviour); all cancelled workflows rerun 2026-10-06 | ⏳ |
| Q02 #1091 | #1130 → clean branch | `da0c867c` (rejected) | commitlint failed (body line >100); the tip also reverted W1–W4 | ⏳ clean branch prepared |

## Findings (2026-10-06)

| Finding | Evidence |
|---------|----------|
| #1130's PR tip had reverted main | `git diff 810db30b da0c867c` deletes `transaction_scope.rs`, `recommendation_index*.rs`, `ranking_guard_tests.rs`, monitoring tests, plans docs, `install-hooks.sh` — a stale-branch clobber pushed as commit `0401fa9c`; the only real work is `b9331ce3` (5 files) |
| #1130 shipped a silent-pass nightly job | `continue-on-error: true` + `\|\| true` on the isolated Turso job; removed on the clean branch — the job and `nightly-summary` are now fail-visible (issue #1091 acceptance: "native failures cannot silently pass as skipped/green") |
| #1138/#1140 non-required workflows cancelled mid-run | runs API shows `conclusion=cancelled` per job (not `failed`); `gh run rerun --failed` triggered 2026-10-06; dynamic CodeQL runs cannot be rerun via API — they re-analyze on the next push |
| Interactive rerun needed for the required aggregate | #1140's `CI / Required` correctly failed closed when the gated jobs were cancelled/abandoned (`ci-required-evaluate.sh` rejects unknown/`skipped`); rerunning the workflow restores a real aggregate |

## Live tracker state

`gh issue list --state open` = 21 (audit backlog minus #1077/#1086/#1088 closed, #1066 merged-awaiting-close; plus
#1109 code-complete/manual and #1137 release drift) · `gh pr list --state open` = 3 (#1130, #1138, #1140).
