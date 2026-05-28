# GOAP Plan: Missing Implementation Remediation Sprint (v0.1.32)

- **Created**: 2026-05-21
- **Last Verified**: 2026-05-26 (post-release `rg` audit; v0.1.32 published 2026-05-24; 12/15 functional WGs landed, 3 deferred to v0.1.33)
- **ADR**: [ADR-055](adr/ADR-055-Missing-Implementation-Remediation-v0.1.32.md)
- **Sprint Target**: `v0.1.32` ✅ Released (GitHub tag `v0.1.32`, 2026-05-24)
- **Carry-over to v0.1.33**: WG-158 (`episode_success_rate`), WG-160 (Turso cache `query_hits`/`evictions`), WG-162 (`generate_simple_embedding` in prod path)
- **Branch base**: `main` (clean)
- **Audit method**: `rg -i "not yet implemented|placeholder|stub\b|fallback"` across
  `memory-core/src`, `memory-mcp/src`, `memory-cli/src`, `memory-storage-redb/src`,
  `memory-storage-turso/src` on 2026-05-21; re-verified on 2026-05-22.

## 2026-05-26 Final Snapshot (post-release)

| Phase | Done | Open (→ v0.1.33) | Notes |
|-------|------|------------------|-------|
| P1 — User contract | WG-150, WG-151, WG-152 (typed-error), WG-153 (typed-error), WG-154, WG-155 | — | WG-154 closed by Mistral bit-unpacking impl (progress_log 2026-05-22) |
| P2 — Telemetry | WG-156, WG-157, WG-159 | **WG-158, WG-160** | `pattern_match_score` + `memory_usage_mb` now compute real values; success-rate + Turso cache counters still placeholders |
| P3 — Internal debt | WG-161 (resolved by removal), WG-163, WG-164 | **WG-162** | `generate_simple_embedding` still called from `retrieval/context.rs:393` |
| P4 — Validation + release | WG-165..WG-167, WG-169, WG-170 | WG-168 (partial — 3 residuals) | `v0.1.32` GitHub release published 2026-05-24 |

Live evidence is recorded in [`GOAP_STATE.md`](GOAP_STATE.md). Re-run the sprint-exit
audit before claiming Phase 4 readiness:

```
rg -in 'not yet implemented|// *Placeholder' memory-core/src memory-mcp/src \
  memory-cli/src memory-storage-redb/src memory-storage-turso/src
```

---

## Goal (GOAP root)

> Eliminate all advertised-but-unimplemented surfaces (CLI commands, MCP tools,
> embedding providers, telemetry variables) so the v0.1.32 public contract matches
> actual behavior. Either implement the feature, remove it, or feature-gate it.

### World-state preconditions
- `workspace.version = "0.1.31"` ✅
- `main` clean, all v0.1.31 WGs ✅
- Coverage ≥ 90%, clippy clean, fmt clean ✅
- No `TODO`/`FIXME` markers in `memory-*/src` ✅

### World-state goals (postconditions for sprint exit)
- `rg -i "not yet implemented" memory-*/src/` → 0 matches outside tests
- `rg -i "placeholder" memory-*/src/` → 0 matches outside SQL `?` builders & tests
- 0 hard-coded telemetry constants in `monitoring/`, `time_series.rs`, `health.rs`
- `workspace.version = "0.1.32"`, CHANGELOG updated, tag pushed

---

## GOAP Action Graph

```diagram
                       ╭───────────────────────────╮
                       │ Sprint Goal: v0.1.32      │
                       │ honest public contract    │
                       ╰─────────────┬─────────────╯
                                     │
            ╭────────────────────────┼────────────────────────╮
            ▼                        ▼                        ▼
   ╭────────────────╮       ╭────────────────╮       ╭────────────────╮
   │ Phase 1 (P1)   │       │ Phase 2 (P2)   │       │ Phase 3 (P3)   │
   │ User contract  │       │ Telemetry      │       │ Internal debt  │
   │ G1..G6         │       │ G7..G11        │       │ G12..G15       │
   ╰───────┬────────╯       ╰───────┬────────╯       ╰───────┬────────╯
           │ sequential per crate    │ parallel (read-only    │ parallel
           │ (storage→cli, core→mcp) │ counters/metrics)      │ (independent files)
           ▼                         ▼                         ▼
   ╭────────────────────────────────────────────────────────────────╮
   │ Phase 4: Validate + version bump + release                     │
   │ fmt → clippy → nextest --all → doctest → quality-gates → tag   │
   ╰────────────────────────────────────────────────────────────────╯
```

---

## Phase 1 — User Contract (P1) · Sequential within crate, parallel across crates

| WG | Gap | Crate(s) | Skill | Strategy | Est PR LOC |
|----|-----|----------|-------|----------|------------|
| WG-150 | G1: `relationship show <id>` implementation | `memory-cli`, `memory-core`, `memory-storage-{turso,redb}` | `feature-implement` | Add `get_relationship_by_id` in storage trait → CLI calls it | ~150 |
| WG-151 | G2: global cycle validation | `memory-cli`, `memory-core` | `feature-implement` | Iterate all episodes + DFS, OR remove `--global` flag | ~120 |
| WG-152 | G3: `eval --custom-thresholds` | `memory-cli`, `memory-core` (`reward/adaptive`) | `feature-implement` | Plumb overrides into `AdaptiveThresholds::with_overrides` | ~100 |
| WG-153 | G4: Cohere provider decision | `memory-core`, `memory-mcp` | `analysis-swarm` → `feature-implement` | Either real `reqwest` client behind `cohere` feature OR return typed error (no silent fallback) | ~250 or ~30 |
| WG-154 | G5: Mistral binary dequantization | `memory-core/src/embeddings/mistral` | `feature-implement` | Implement per Mistral cookbook OR remove `binary` from `Encoding` enum | ~80 or ~20 |
| WG-155 | G6: AgentFS SDK probe | `memory-mcp/src/server/tools/external_signals` | `external-signal-provider` | Wire real SDK call OR put behind `agentfs` feature flag (currently always-on stub) | ~150 |

**Dependencies**: WG-150 must land before WG-151 (shares storage trait change).
WG-153/WG-154 require a decision call first (implement vs remove) — use `analysis-swarm`.

---

## Phase 2 — Telemetry Truthfulness (P2) · Parallel

| WG | Gap | File | Skill | Action |
|----|-----|------|-------|--------|
| WG-156 | G7: `pattern_match_score` | `time_series.rs:55` | `feature-implement` | Compute from `Episode::patterns` overlap with template set |
| WG-157 | G8: `memory_usage_mb` | `time_series.rs:59` | `feature-implement` | Use `sysinfo::Pid::memory()` OR remove enum variant |
| WG-158 | G9: `episode_success_rate` | `monitoring/types.rs:363` | `feature-implement` | Track `episode_failures` atomic counter; rate = `1 - failures/total` |
| WG-159 | G10: `uptime_seconds` | `memory-cli/.../health.rs:307` | `feature-implement` | `OnceLock<Instant>` for process start, return elapsed |
| WG-160 | G11: Turso cache `query_*` / `evictions` / `expirations` | `cache/wrapper.rs:142` | `feature-implement` | Add `AtomicU64` to `CacheStats`, increment in hot paths |

All Phase-2 WGs touch independent files → safe to run in parallel via `agent-coordination`.

---

## Phase 3 — Internal Debt (P3) · Parallel

| WG | Gap | File | Skill | Action |
|----|-----|------|-------|--------|
| WG-161 | G12: cascade `analyze_query` | `retrieval/cascade/mod.rs:446` | `feature-implement` | Implement query-class heuristic OR drop method |
| WG-162 | G13: `generate_simple_embedding` | `memory/retrieval/helpers.rs:59` | `code-quality` | Mark `#[cfg(test)]` OR remove (find prod callers first) |
| WG-163 | G14: WG-149 lifecycle emit wiring | `memory/types.rs:185`, `memory/core/struct_priv.rs:90` | `feature-implement` | Call `emit_event` inside `create_episode` / `complete_episode` — closes 3× `#[allow(dead_code)]` |
| WG-164 | G15: stale extraction comment | `extraction/tests.rs:49` | `code-quality` | Delete misleading comment; assert real extraction output |

---

## Phase 4 — Validation & Release · Sequential

| WG | Step | Command | Gate |
|----|------|---------|------|
| WG-165 | Workspace nextest | `cargo nextest run --all` | 0 failures |
| WG-166 | Doctests | `cargo test --doc` | 0 failures |
| WG-167 | Coverage | `./scripts/quality-gates.sh` | ≥ 90% |
| WG-168 | Sprint-exit audit | `rg -i "not yet implemented\|placeholder" memory-*/src \| grep -v test \| grep -v '?"'` | 0 matches |
| WG-169 | Version bump | edit `Cargo.toml` workspace → `0.1.32`; cascade to publishable crates | `cargo metadata` agrees |
| WG-170 | CHANGELOG + release | `git-cliff` → CHANGELOG, `gh release create v0.1.32` | release-guard skill |

---

## Skill stack

| Layer | Skill |
|-------|-------|
| Orchestration | `goap-agent`, `agent-coordination` |
| Investigation | `codebase-analyzer`, `plan-gap-analysis` |
| Implementation | `feature-implement`, `external-signal-provider`, `analysis-swarm` (for build/remove calls) |
| Quality | `code-quality`, `test-runner`, `test-patterns`, `test-fix` |
| Release | `release-guard`, `github-release-best-practices` |

---

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Cohere/AgentFS implementations balloon scope | Medium | High | Default to **remove or feature-gate** unless explicit demand |
| `sysinfo` adds platform-specific code paths | Low | Medium | Use `sysinfo` `apple-app-store` / cross-platform feature set; gate per-OS in `cfg` |
| Storage trait change (G1) cascades into Turso + redb + mocks | Medium | Medium | Default impl returning `Err(NotImplemented)`, override per backend |
| Global cycle validation (G2) is O(N²) | Low | Low | Cap at 10k episodes or stream + early-exit |
| Coverage drops below 90% after telemetry refactor | Low | High | Add unit tests for each new counter in same PR |

---

## Execution strategy (per `agent-coordination`)

**Hybrid**: Phase 1 sequential-per-crate, Phase 2 fully parallel, Phase 3 fully parallel,
Phase 4 sequential. Cap concurrent code-writing subagents at **3** to keep write
targets disjoint (one per crate).

```
T+0   ──▶ Decision swarm: WG-153 (Cohere), WG-154 (Mistral), WG-155 (AgentFS)
T+1   ──▶ Parallel kick-off:
              [Subagent A] WG-150 → WG-151 → WG-152      (memory-cli + storage)
              [Subagent B] WG-156, WG-157, WG-158, WG-159, WG-160   (telemetry)
              [Subagent C] WG-161, WG-162, WG-163, WG-164           (internal)
T+2   ──▶ Integration: implement decisions from T+0 swarm
T+3   ──▶ Phase 4 validation, version bump, release
```

---

## Tracking

Update on each PR merge:
- `plans/STATUS/CURRENT.md` → flip WG status, bump test counts
- `plans/GOAP_STATE.md` → add Sprint v0.1.32 row
- `plans/ROADMAPS/ROADMAP_ACTIVE.md` → record sprint progress
- `plans/STATUS/GAP_ANALYSIS_LATEST.md` → mark gap rows resolved

---

## Cross-references

- [ADR-055](adr/ADR-055-Missing-Implementation-Remediation-v0.1.32.md) — decision record
- [STATUS/CURRENT.md](STATUS/CURRENT.md) — live metrics
- [STATUS/GAP_ANALYSIS_LATEST.md](STATUS/GAP_ANALYSIS_LATEST.md) — preceding gap analysis (2026-04-22)
- [ROADMAPS/ROADMAP_ACTIVE.md](ROADMAPS/ROADMAP_ACTIVE.md) — to be updated with sprint
- [GOAP_STATE.md](GOAP_STATE.md) — to be updated with sprint row
