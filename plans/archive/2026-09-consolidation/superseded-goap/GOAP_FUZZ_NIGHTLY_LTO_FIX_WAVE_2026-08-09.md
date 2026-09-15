# GOAP: Fuzz nightly + LTO-off fix wave (2026-08-09)

- **Date**: 2026-08-09
- **Branch**: `fix/ci-fuzz-nightly-gitleaks-version` · **PR**: #934
- **Goal**: Make the weekly Fuzzing workflow green; land PR #934 (fuzz fix + gitleaksignore + v0.1.39 bump + plans sync)
- **Orchestrator**: GOAP hybrid — swarm verification + sequential controller edits
- **ADR context**: ADR-079 (fail-closed informational CI, CIT-A5) — fuzz evidence must be durable and non-green runs must signal; ADR-022 (GOAP agent system)

## Goal hierarchy

| Goal | Success criteria |
|------|------------------|
| G1 fuzz_workflow_green | `gh workflow run fuzz.yml -f duration=60` on branch → conclusion `success`; log has `No fuzz crashes...` and no `Z`-option or `__sancov_gen_` errors |
| G2 pr_934_mergeable | PR statusCheckRollup all required `SUCCESS`, mergeStateStatus `CLEAN`, no unaddressed comments |

## Root-cause (verified this session)

1. fuzz.yml pinned **stable** toolchain; cargo-fuzz injects `-Zsanitizer=address` → rejected. Fix: `toolchain: nightly` + `RUSTUP_TOOLCHAIN: nightly` job env (already committed in `b207fe3a`).
2. After nightly: link failure `undefined symbol: __sancov_gen_*` (all 3 targets). Root cause: repo-root `.cargo/config.toml` has `[profile.release] lto = "fat"`; cargo **config** profiles apply directory-wide to every build in the tree, including the `exclude`d standalone `fuzz/` crate (cargo-fuzz issue #384: sanitizer-coverage counters can't resolve under fat LTO).
3. A/B proof (local, same machine/toolchain): default build → FAILED; `CARGO_PROFILE_RELEASE_LTO=false cargo fuzz build fuzz_mcp_jsonrpc` → **Finished release profile, no link error**.

## Tasks

### Phase 1 — SWARM (parallel, independent)
| Task | Agent | Output |
|------|-------|--------|
| T1 Build `fuzz_postcard_roundtrip` + `fuzz_search_matchers` locally with LTO off | worker | pass/fail per target + tail of each build |
| T2 Independent root-cause review: confirm LTO mechanism + fix choice | reviewer | verdict: env var vs fuzz-local config, and whether anything else must change |
| T3 PR #934 status snapshot (checks, merge state, comments) | scout | statusCheckRollup + mergeStateStatus + pending comments |

### Phase 2 — SEQUENTIAL (controller)
- T4 Edit `.github/workflows/fuzz.yml` job env: add `CARGO_PROFILE_RELEASE_LTO: false`
- T5 Commit (amend fuzz commit), force-push
- T6 Re-trigger `gh workflow run fuzz.yml --ref <branch> -f duration=60`

### Phase 3 — SWARM (parallel)
| Task | Agent | Output |
|------|-------|--------|
| T7 Poll fuzz run to terminal state; verify green + evidence lines | worker | final conclusion + matched log lines |
| T8 PR #934 full pr-readiness (checks, comments, drift) | reviewer | ready-to-merge verdict |

### Phase 4 — SEQUENTIAL (controller)
- T9 Merge PR #934 (squash), verify main CI aggregate green

## Cross-task contracts
- **Ownership**: only the controller edits `.github/workflows/fuzz.yml` and pushes. Swarm workers are read-only w.r.t. the repo except their own cargo build artifacts in `fuzz/target/`.
- **No validation**: workers skip project-wide build/lint/test suites; T1's `cargo fuzz build` IS the task, not a gate.
- **Evidence**: workers return exact command output lines (conclusion, log greps), not summaries.
