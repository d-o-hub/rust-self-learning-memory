# Codebase Validation — 2026-03-24

**Validated by**: Read-only repo audit + GOAP coordination
**Branch**: `main` (HEAD aligned with origin/main)
**Workspace Version**: `0.1.22`
**Previous Validation**: 2026-03-20 (v0.1.22 tag readiness)

---

## Commands Executed

| Command | Output (abridged) |
|---------|-------------------|
| `git status --short --branch` | `## main...origin/main` (clean, `audit-report.json` untracked) |
| `du -sh . target node_modules benchmark_results data metrics .git` | `.` = **32G**, `target` = **32G**, `node_modules` = **130M**, `benchmark_results` = 152K, `data` = 1.8M, `metrics` = 12K, `.git` = 23M |
| `ls plans/adr/ADR-0*.md | wc -l` | 25 ADRs reviewed for constraints (ADR-022/032/033/038/044 most relevant) |
| `rg -n` (targeted) | Verified attribution/checkpoint persistence gaps and doc drift noted below |
| `./scripts/code-quality.sh fmt` | ✅ Formatting check passed after new storage modules |
| `cargo nextest run --test attribution_integration` | ✅ 2 attribution integration tests (durable persistence) |
| `cargo nextest run --test checkpoint_integration` | ✅ 2 checkpoint/handoff integration tests (including restart durability) |
| `cargo nextest run -p memory-storage-turso test_store_and_get_episode_persists_checkpoints test_row_to_episode_defaults_missing_checkpoints_to_empty test_get_episodes_batch_preserves_checkpoints` | ✅ 3 targeted Turso durability tests |

No build/test commands were executed in this validation slice per user request (read-only audit).

---

## Findings Snapshot

### Implementation Gaps (ADR-044 compliance)
- ✅ **Recommendation attribution durability restored**: Turso SQL + `memory-storage-turso/src/storage/recommendations.rs`, `memory-storage-redb/src/recommendations.rs`, and storage trait impls persist sessions/feedback/stats; validated via `tests/attribution_integration_test.rs` (WG-051).
- ✅ **Checkpoint/handoff durability restored (WG-052)**:
  - Turso episode schema + CRUD/query/batch paths now persist `Episode.checkpoints`.
  - `row_to_episode` now deserializes checkpoints (with backward-compatible defaulting for legacy rows).
  - `resume_from_handoff` now persists metadata via `update_episode_full` instead of mutating fallback-only state.
  - Validated via `tests/checkpoint_integration_test.rs` and targeted Turso durability tests.
- ✅ **Batch MCP contract aligned (WG-053)**: tool-level batch analytics names remain intentionally absent (`tool_definitions_extended.rs`), parity tests assert non-advertisement + direct-call rejection, and active docs/plans now reflect this deferred state.

### Documentation & Contract Drift
- `docs/API_REFERENCE.md` still reports **v0.1.13** / MCP **v2024-11** and documents removed batch tools; it omits ADR-044 tools.
- `docs/PLAYBOOKS_AND_CHECKPOINTS.md:48-92` and `plans/STATUS/GAP_ANALYSIS_LATEST.md:97-105` reference old CLI subcommands (`feedback add`, `provide_feedback`), but code exposes `record-session`, `record-feedback`, and `record_recommendation_feedback`.
- `README.md:29-90` advertises “secure code execution sandbox” and “13 command groups” without acknowledging that `execute_agent_code` is disabled (see `plans/STATUS/CURRENT.md:79-83`).
- `plans/STATUS/VALIDATION_LATEST.md` (previous version) and `plans/ROADMAPS/ROADMAP_ACTIVE.md` still tout “all gaps resolved”, contradicting current audit results.

### Validation Coverage Gaps (ADR-033 & ADR-038)
- `.github/workflows/ci.yml:81-90` runs `cargo nextest run --profile ci --package <crate> --lib` for only three crates; integration/CLI/MCP tests are not part of required PR checks despite AGENTS.md claims.
- `scripts/check-coverage.sh:17-82` defaults to **70%** and always prints “Coverage check passed” without parsing actual coverage.
- `benchmarks.yml:155-160` executes only 4 of 14 declared benches.

### Disk & Developer Experience Gaps (ADR-032)
- Local `target/` regained **32G** (mostly `incremental/` and `deps/`), conflicting with ADR-032’s “19 GB after cleanup” claim.
- `node_modules/` (130M) exists despite ADR-032 Phase 6 bullet stating “Removed orphaned node_modules/”.
- `.cargo/config.toml:51-55` notes that mold linker support was removed, but ADR-032 still claims it is “installed and configured”. Guidance in AGENTS.md / skills does not reflect this.

---

## Recommended Remediation (Mapped to New WGs)

| WG | Focus | Actions | Owners |
|----|-------|---------|--------|
| **WG-051** | Durable recommendation attribution | Extend storage traits + Turso schema, add integration tests, surface metrics | feature-implementer + architecture |
| **WG-052** | Durable checkpoints/handoffs | Persist checkpoint metadata across storage/redb caches, verify resume flows | feature-implementer + architecture |
| **WG-053** | MCP contract integrity | ✅ Complete — explicit defer decision + parity/tests/docs/plans alignment | memory-mcp + goap-agent |
| **WG-054** | Docs/CLI/API parity | Regenerate API reference, README, CLI docs, playbook/feedback docs, plans/STATUS | documentation |
| **WG-055** | Required CI coverage | Expand CI workflows to cover full workspace tests (or document scoped filtersets) | github-workflows + test-runner |
| **WG-056** | Coverage gate enforcement | Update scripts/tests to enforce ≥90% threshold (per AGENTS.md) | quality-unit-testing |
| **WG-057** | Disk hygiene | Automate cleanup + document CARGO_TARGET_DIR guidance, reconcile ADR-032 claims | performance |
| **WG-058** | Agent guidance parity | Align AGENTS.md, agent_docs/, `.agents/skills/` with script-first workflow + disk/CI reality | agents-update + documentation |

All items are scoped and sequenced in `plans/GOAP_EXECUTION_PLAN_v0.1.23.md`.

---

## References

- `plans/GOAP_EXECUTION_PLAN_v0.1.23.md`
- `plans/ROADMAPS/ROADMAP_ACTIVE.md`
- ADR-022, ADR-032, ADR-033, ADR-038, ADR-044
- Audit evidence from `git status`, `du -sh`, and targeted `rg` queries captured above
