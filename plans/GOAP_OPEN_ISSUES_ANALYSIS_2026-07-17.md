# GOAP: Open GitHub Issues vs Codebase — 2026-07-17

- **Status**: Active planning (analysis complete; implementation queued)
- **Date**: 2026-07-17
- **Analyst**: GOAP agent (codebase-verified)
- **Workspace**: `0.1.35` on `main` (tag `v0.1.34` still latest release)
- **Open issues**: 4 (`#845`, `#846`, `#847`, `#849`)
- **Related ADRs**: ADR-075, ADR-076, ADR-058 / release-drift plan for #843→#849
- **Prior related closed**: `#831` (patterns), `#829` (config), `#830` (db-path), `#832` (storage_mode), `#843` (drift automation)

---

## Executive Summary

| Issue | Title (short) | Labels | Codebase verdict | Priority |
|-------|---------------|--------|------------------|----------|
| [#849](https://github.com/d-o-hub/rust-self-learning-memory/issues/849) | Release due: 27 commits / 6 days since v0.1.34 | `release-drift` | **Action required** — cut `v0.1.35` via `release.yml` | **P0** |
| [#847](https://github.com/d-o-hub/rust-self-learning-memory/issues/847) | `episode complete … failure` no-op for stuck `in_progress` | `bug` | **Open defect on main** — silent durability / operator path | **P0** |
| [#845](https://github.com/d-o-hub/rust-self-learning-memory/issues/845) | Pattern list empty after create→complete ingest | `bug` | **Mostly fixed on main (#831)**; residual UX/docs + release | **P1** |
| [#846](https://github.com/d-o-hub/rust-self-learning-memory/issues/846) | Config format undocumented / hard to discover | `bug` | **Fixed on main (#829)**; needs release + minor doc polish | **P2** |

All four issues cite **v0.1.34** binaries. Workspace `main` already carries the v0.1.35 CLI UX patch and subsequent GOAP sprints, but **no `v0.1.35` tag exists**. Shipping the release unblocks #845/#846 for users and clears #849; #847 still needs code work on top of that.

---

## Issue-by-Issue Codebase Analysis

### #849 — Release drift (27 commits / 6 days)

**Evidence**

- Workspace `Cargo.toml` version: `0.1.35`
- Latest tag: `v0.1.34`
- Unreleased commits on `main`: **27** (matches issue body)
- Drift automation remodeled under `plans/GOAP_RELEASE_DRIFT_ISSUE_843_2026-07-17.md` (idempotent upsert, ADR-058 thresholds)

**Verdict**

- Not a product bug; **release engineering debt**.
- Cadence is in **warning** band (20–29 commits or ≥10 days). Hard fail at 30 commits / 14 days.
- CHANGELOG already has `## [0.1.35] - 2026-07-15` for CLI UX fixes.

**GOAP actions**

| ID | Action | Owner skill | Depends |
|----|--------|-------------|---------|
| R1 | Finalize STATUS/ROADMAP notes for release | docs | — |
| R2 | Ensure all required checks green on `main` | `pr-readiness` / CI | — |
| R3 | Tag `v0.1.35` via `./scripts/release-manager.sh full --execute` only (never manual `gh release create`) | `release-guard` | R1–R2 |
| R4 | Confirm `#849` auto-closes / upsert closes when tag matches | release-drift workflow | R3 |

---

### #847 — `episode complete <id> failure` appears as no-op

**Reporter**: stuck bot-loop episodes, zero steps, remain `in_progress` after CLI complete with exit 0.

**Code paths**

| Layer | Path | Finding |
|-------|------|---------|
| CLI complete | `memory-cli/src/commands/episode/core/complete.rs` | Calls `get_episode` then `complete_episode`; maps errors; **always prints success** if core returns `Ok` |
| Core complete | `memory-core/src/memory/completion.rs` | Loads from `episodes_fallback` only (after prior hydrate); `episode.complete(outcome)`; quality gate; **store errors are `warn!` only** (lines ~318–327); still returns `Ok(())` |
| Episode status | `Episode::is_complete` | `end_time.is_some() && outcome.is_some()` |
| Quality gate | CLI `MemoryConfig.quality_threshold = 0.0` | Zero-step episodes **not** rejected by quality (CLI) |
| Steps requirement | Pattern extractors | Empty steps → **no patterns**; does **not** block completion |
| Operator force-fail | CLI | **No** `episode fail` subcommand |

**Root-cause hypotheses (ordered by likelihood)**

1. **Silent durable write failure**  
   `store_episode` failures are logged as warnings; CLI still exits 0 and prints “Status: completed”. A fresh CLI process lists from redb and still shows `in_progress`.

2. **DB path mismatch (v0.1.34)**  
   Issue #830 fixed on main: `--db-path` / `MEMORY_DB_PATH` previously ignored for redb. Complete and list could hit different stores. Partially mitigated on `main` after #830.

3. **Reporter expectation vs design**  
   Zero-step complete is allowed on CLI (`quality_threshold: 0.0`). There is no lifecycle guard rejecting empty episodes; the failure mode is durability/UX, not a “must have steps” gate.

4. **No operator escape hatch**  
   No dedicated force-finalize path for abandoned synthetic rows.

**Not the cause (verified)**

- Quality threshold 0.7 rejecting empty episodes: CLI forces `0.0`.
- Explicit “must have steps” guard in `complete_episode`: none.
- Missing Failure outcome enum: CLI maps `TaskOutcome::Failure` correctly.

**Decision** → **ADR-075**

**GOAP actions**

| ID | Action | Priority |
|----|--------|----------|
| C1 | Make backend store failures **fatal** (or partial-success with non-zero exit) on `complete_episode` when any configured backend fails | P0 |
| C2 | After complete, re-load episode and assert `is_complete()`; fail CLI if not durable | P0 |
| C3 | CLI: emit clear error when get/complete fails; never print “completed” unless verified | P0 |
| C4 | Optional: `episode fail <id>` (or `complete --force`) for operator cleanup of abandoned rows | P1 |
| C5 | Tests: zero-step complete persists across process; store-failure surfaces non-zero exit; force-fail path | P0 |
| C6 | Document: complete requires durable write; empty steps complete but extract no tool patterns | P2 |

---

### #845 — Pattern extraction yields 0 patterns after ingest

**Reporter**: bulk blog ingest (create→complete), 72 episodes, `pattern list` empty; `storage sync` no-op.

**Code paths**

| Layer | Finding |
|-------|---------|
| Pattern durability (v0.1.34 bug) | Internally tagged `Pattern` broke postcard; `get_all_patterns` did not hydrate → **fixed #831** in `58bed23f` |
| redb | `get_all_patterns` + trait path tests (`memory-storage-redb/src/patterns.rs`) |
| Core types | Postcard roundtrip test on `Pattern` (`memory-core/src/patterns/types.rs`) |
| E2E | `tests/e2e/cli_workflows.rs` asserts cross-process `pattern list` after complete |
| Tool-sequence extractor | **Requires ≥1 step** (`extraction/extractors/mod.rs`) — zero-step completes → 0 patterns **by design** |
| `storage sync` | Turso↔redb only (`memory-cli/src/commands/storage/commands.rs`); **does not extract patterns**; needs both backends |
| `pattern extract` | No first-class re-extract CLI for “re-derive from all completed episodes” |

**Verdict**

| Aspect | Status |
|--------|--------|
| Cross-process pattern list after successful extraction | ✅ Fixed on main (#831) |
| User on v0.1.34 binary | ⏳ Needs **v0.1.35 release** (#849) |
| Create→complete with **no steps** | ⚠️ Still 0 patterns (expected); needs docs + better CLI messaging |
| `storage sync` as extraction trigger | ❌ Wrong tool; should error/explain when only local redb or when no Turso |
| Explicit extract / empty-result reason | ❌ Missing residual UX |

**Decision** → **ADR-076** (discoverability + empty-result semantics; not re-litigating postcard)

**GOAP actions**

| ID | Action | Priority |
|----|--------|----------|
| P1 | Ship v0.1.35 so #831 reaches users | P0 (via #849) |
| P2 | On `pattern list` empty: print hint (no patterns stored / try complete with steps / same `--db-path`) | P1 |
| P3 | `storage sync`: clear message when local-only; never imply extraction | P1 |
| P4 | Docs: min steps for tool-sequence; complete vs extract; release notes for #831 | P1 |
| P5 | Optional: `pattern extract [--episode-id]` re-run extraction for completed episodes | P2 |
| P6 | After release: comment on #845 + close if repro fixed; keep open only for residual UX if needed | P1 |

---

### #846 — Config file format undocumented

**Code paths (main)**

| Artifact | Status |
|----------|--------|
| `do-memory-cli config init` / `show-template` | ✅ `memory-cli/src/commands/config_template.rs` |
| Example TOML | ✅ `memory-cli/config/do-memory-cli.example.toml` |
| Root README `## Configuration` + TOML sample | ✅ Present |
| `memory-cli/CONFIGURATION_GUIDE.md` | ✅ Expanded (#829) |
| Precedence chain (flag → env → `--config` → CWD → default) | ⚠️ Partially documented; issue’s explicit chain can be tightened |

**Verdict**: Fixed for v0.1.35 users; issue remains open until release + optional precedence table polish.

**GOAP actions**

| ID | Action | Priority |
|----|--------|----------|
| D1 | Ship v0.1.35 | P0 (via #849) |
| D2 | Add explicit 5-level precedence table to README + CONFIGURATION_GUIDE | P2 |
| D3 | Close #846 after release + comment with `config init` / example path | P1 |

---

## Cross-Cutting Dependency Graph

```text
                    ┌─────────────────┐
                    │  #849 Release   │  P0
                    │  tag v0.1.35    │
                    └────────┬────────┘
           ┌─────────────────┼─────────────────┐
           ▼                 ▼                 ▼
    #845 user fix     #846 user fix     CHANGELOG already
    (#831 on main)    (#829 on main)    prepared
           │
           ▼
    residual #845 UX ──► ADR-076 (docs/messages/extract)
           
    #847 code fix ──► ADR-075 (independent of release, can land before or after tag)
```

**Recommended order**

1. **#847 fix** on a short branch (correctness) — can merge before or after tag  
2. **#849 cut v0.1.35** if merge window allows either (a) tag current main then #847 as 0.1.36, or (b) land #847 then tag 0.1.35  
3. Close/verify **#845/#846** against released binary  
4. Residual pattern UX (ADR-076 P1/P2) if still needed  

**Release recommendation**: Prefer **(b)** if #847 can land quickly (silent complete failure is user-visible correctness). If release is urgent to stop drift hard-limit, ship **(a)** immediately then 0.1.36 for #847.

---

## Goal Hierarchy (GOAP)

### G0 — Ship truthful, durable CLI episode lifecycle and clear learning UX

**Success criteria**

- [ ] `v0.1.35` (or `0.1.36` if #847 lands after) published via `release.yml`
- [ ] `#849` resolved
- [ ] `#847` fixed with tests; no silent complete success without durable write
- [ ] `#845` verified against released binary or residual tickets split
- [ ] `#846` closed with docs + release pointer
- [ ] ADRs 075–076 accepted; GOAP_STATE / GOALS / ACTIONS / ROADMAP / STATUS updated

### Subgoals

| Goal | Issues | Strategy |
|------|--------|----------|
| G1 Release gate | #849 | Sequential (release-guard) |
| G2 Complete durability | #847 | Sequential feature-implement + tests |
| G3 Pattern empty-result UX | #845 residual | Parallel docs + small CLI messages after G1 |
| G4 Config discoverability closeout | #846 | Parallel docs polish + close after G1 |

---

## Work Packages

### WP-A — Release v0.1.35 (#849)

1. Align `plans/STATUS/CURRENT.md` + `ROADMAP_ACTIVE.md` release section  
2. `release-guard` checklist  
3. `./scripts/release-manager.sh full --execute` from clean main  
4. Verify GitHub Release + issue #849 closed  

### WP-B — Episode complete durability (#847 / ADR-075)

1. Fail complete when configured backend store fails  
2. Post-complete verify `is_complete` from durable read  
3. CLI non-zero exit + actionable message  
4. Optional `episode fail` / `--force`  
5. Unit + CLI e2e tests  

### WP-C — Pattern residual UX (#845 / ADR-076)

1. Empty `pattern list` / `search` hints  
2. `storage sync` local-only messaging  
3. Docs for step requirement + release note pointer to #831  
4. Optional `pattern extract`  

### WP-D — Config closeout (#846)

1. Precedence table  
2. Close issue after release comment  

---

## Mapping to Existing Plans

| Existing plan | Relation |
|---------------|----------|
| `GOAP_CLI_UX_PATCH_0.1.35_2026-07-15.md` | Delivered #828–#832; #845/#846 are user-facing residual of that release gap |
| `GOAP_RELEASE_DRIFT_ISSUE_843_2026-07-17.md` | Automation; #849 is the live cadence ticket |
| `GOAP_MISSING_TASKS_S13_S16_W22_2026-07-16.md` | Unrelated correctness sprint already on main; ships in same unreleased window |
| `GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md` | Backlog; no conflict |

---

## Out of Scope (closed / not open)

Older backlog issues (#770, #753, #749, #746, #743, #800, #799, etc.) are **not currently open** on GitHub as of this analysis. Do not re-open them in this sprint unless recreated.

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Open issues after sprint | 0 (or only deferred enhancement with new issue) |
| Unreleased commits after tag | 0 relative to tagged version |
| `episode complete` false-success | Impossible when backends configured |
| Pattern list after create+step+complete | ≥1 pattern across process (existing e2e) |
| Config discovery | `config init` / `show-template` + README table |

---

## Next Immediate Actions

1. Accept ADR-075 and ADR-076  
2. Choose release order (a) tag-now vs (b) #847-then-tag  
3. Implement WP-B (#847)  
4. Execute WP-A (#849)  
5. WP-C / WP-D closeout comments on GitHub  
