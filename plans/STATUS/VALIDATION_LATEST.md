# Validation Latest — 2026-10-09 (audit-backlog wave A in review)

**Goal**: record the verified state of the audit-backlog wave — which slices merged, which are in review, and the
CI evidence attached to each head SHA (PR monitoring guardrail).

**Workspace**: `0.1.45` · **Branch**: `main` @ `5054c15a` · **Tag**: `v0.1.44`

## Method

- Merged slices verified by commit/SHA on `main` (`git log`) and by issue state (`gh issue view`).
- In-review PRs verified per head SHA via `gh pr checks` / the check-runs API; every failure was reproduced locally
  with the canonical commands where possible (local clippy is 1.98; CI's floating stable is 1.99, so 1.99-only lints
  were fixed from the CI log evidence).
- Local evidence for new checks: `./scripts/check-ignored-tests.sh` (159 attrs, inventory complete),
  `./scripts/check-lint-suppressions.sh` (317 → 0 after #1153), the storage-backend capability checker + its 6 fixtures
  from PR #1158 (all OK), `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 cargo nextest … --message-format libtest-json` emits JSON.

## Merged (2026-10-05…08)

| Slice | PR | Commit | Evidence |
|-------|----|--------|----------|
| W0 trackers + wave doc | #1129 | `26b12fc8` | plans validation + tracker-drift 0 mismatches |
| W1 #1077 ranking guard | #1131 | `874df209` | contention test; guard dropped before `.await` |
| W2 #1086 sub-second latency | #1135 | `75007b51` | non-zero ms regression test |
| W3 #1088 tag-tx rollback | #1134 | `347b3296` | failure-injection test |
| W4 #1066 redb index order | #1139 | `f1c31699`, `0485bb66` | 6 rank/repair tests; issue #1066 closed with evidence |
| Q02 #1091 ignored tests | #1130 | `8a363505` | inventory 159 + 14 un-ignored tests green; isolated job fail-visible |
| #1065 recommendation precision | #1142 | `5054c15a` | ms ordering + UUID tie-break + backfill tests; issue closed |
| #1092 lint slice 1 | #1143 | `cdb70594` | turso denies `unsafe_code`; ratchet checker + fixtures |
| #1079 provenance | #1148 | `9fdbc765` | 314 lib retrieval tests; single lookup |

## In review (head SHA at 2026-10-09)

| Slice | PR | Head | Local evidence |
|-------|----|------|----------------|
| W5 #1075 lexical fallback | #1138 | `b7b71493`+ | 17/17 `pattern_search` tests; clippy clean |
| W6 #1085 health probes | #1140 | `c35e3e39`+ | 7/7 unit + 4/4 integration; fmt fixed; clippy clean |
| #1067 watermark | #1149 | `88a1d047` | 14/14 `storage_sync`; monotonic/backfill/durable-watermark tests |
| #1064 scoped checkout | #1147 | `74401715` | 13/13 pool integration; nested-checkout BLOCKER fixed |
| #1063 adaptive pool | #1146 | `409258f4` | 33/33 adaptive; E4 test replaced |
| #1070 capacity eviction | #1150 | `95021e52` | 12/12 capacity tests incl. failure injection |
| #1071 strict row decode | #1145 | `8035b12f` | doctest fixed; 12/12 raw-query/builder tests |
| #1073 embedding adapter | #1151 | `61b82a94` | 147 lib embeddings + 67 MCP; LOC decomposed |
| #1078 ranking index | #1154 | `213c28a8` | 100k: 222 ms full rebuild → 0.84 µs incremental |
| #1087 capability truth | #1144/#1152/#1158 | — | workspace clippy; checker + 6 fixtures OK |
| #1092 lint slice 2 | #1153 | `655f4dca` | crate-root allows 317 → 0; workspace clippy green |
| #1155 perf lowercase Cow | #1155 | `181d363f` | scoring/types diffs reviewed; tests added |
| #1157 query_range clamp | #1157 | `9dac415a` | 70/70 spatiotemporal; clamp pinned by test |
| #1159 nightly JSON flag | #1159 | — | flag verified locally |

## Findings (2026-10-06…09)

| Finding | Evidence |
|---------|----------|
| #1130's PR tip had reverted main | `git diff 810db30b da0c867c` deleted W1–W4 files, plans and scripts; rescued to one clean commit before merge |
| #1091's isolated nightly never ran its tests | `--message-format libtest-json` needs `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`; both report files were empty; fixed in #1159 |
| #1140 shipped an unformatted test line | rustfmt diff in `health_probe_test.rs`; fixed and pushed |
| #1145's module doctest did not compile | `filter(EpisodeColumn::Domain, "=", …)` vs typed `FilterOp`; fixed |
| #1150 new tests tripped clippy 1.99 `assert_is_empty` | `assert!(x.is_empty())` → `assert_eq!(x.len(), 0)` |
| #1152 tripped rustdoc 1.99 `redundant_explicit_links` | explicit `](crate::Error::CapabilityUnavailable)` on a resolvable label; fixed |
| #1153 unfulfilled expectations | crate-root `expect` for a lint the canonical gate `-A`-allows; scoped/removed with evidence |
| Release-drift check label race | the enforcement step reads the trigger payload's labels; the workflow's own label is not visible to it (GITHUB_TOKEN events don't re-trigger), so a dispatch run (or a push after the label) is the documented way to clear the stale failure |
| Strict up-to-date policy ⇒ serial merges | every merge invalidates the other PRs' required-check runs; 17 PRs need one update+CI cycle each |

## Live tracker state

`gh issue list --state open` = 16 · `gh pr list --state open` = 17 (wave A + plans #1141).
