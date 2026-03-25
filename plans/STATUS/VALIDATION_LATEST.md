# Codebase Validation — 2026-03-24 (WG-054 through WG-058)

**Validated by**: GOAP multi-workstream remediation + local command verification
**Branch**: `remediation/wg051-wg053-durability-contract` (working tree dirty during remediation)
**Workspace Version**: `0.1.22`
**Previous Validation**: 2026-03-20 (v0.1.22 tag readiness)

---

## Commands Executed

| Command | Output (abridged) |
|---------|-------------------|
| `git status --short --branch` | `## remediation/wg051-wg053-durability-contract...origin/remediation/wg051-wg053-durability-contract` + modified files for WG-054..WG-058 |
| `du -sh . target node_modules benchmark_results data metrics .git` | `.` = **32G**, `target` = **32G**, `node_modules` = **130M**, `benchmark_results` = 152K, `data` = 1.8M, `metrics` = 12K, `.git` = 23M |
| `ls plans/adr/ADR-0*.md | wc -l` | 25 ADRs reviewed for constraints (ADR-022/032/033/038/044 most relevant) |
| `rg -n` (targeted) | Verified attribution/checkpoint persistence gaps and doc drift noted below |
| `cargo run -p memory-cli -- --help` | ✅ Command list confirms current top-level groups: `episode`, `pattern`, `storage`, `config`, `health`, `backup`, `monitor`, `logs`, `eval`, `embedding`, `completion`, `tag`, `relationship`, `playbook`, `feedback` |
| `./scripts/check-docs-integrity.sh` | ⚠️ Reports many broken links in archived `plans/archive/**` and `plans/STATUS/archive/**`; active docs remain aligned and archived cleanup remains non-blocking |
| `./scripts/clean-artifacts.sh --help` | ✅ Usage text now documents `quick|standard|full` modes plus `--node-modules`, `--target-dir`, and `--dry-run` |
| `./scripts/code-quality.sh fmt` | ✅ Passed |
| `cargo nextest run --test quality_gates` | ✅ Passed (`12 passed, 1 skipped`) |
| `cargo nextest run --test attribution_integration test_recommendation_persistence_with_storage` | ✅ Passed (Turso-backed recommendation session/feedback reload durability) |
| `cargo nextest run --test checkpoint_integration test_resume_handoff_metadata_persists_across_storage_reload` | ✅ Passed (Turso-backed checkpoint/handoff metadata durability across reload) |
| `./scripts/check-coverage.sh --threshold 90 --summary-mode` | ✅ Enforcement works: command exits non-zero with `Measured: 62.00%`, `Coverage check failed: 62.00% < 90%` |

Validation covered docs/contract truth, CI-workflow surface expansion, coverage gate enforcement logic, disk hygiene automation, and Turso durability evidence.

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
- ✅ `docs/API_REFERENCE.md` now reflects current MCP tool contract from `memory-mcp/tests/tool_contract_parity.rs` and explicitly marks deferred batch tools absent.
- ✅ `docs/PLAYBOOKS_AND_CHECKPOINTS.md` now uses current CLI command families (`memory-cli episode ...`, `memory-cli feedback record-session`, `memory-cli feedback record-feedback`) and current MCP feedback tool naming (`record_recommendation_feedback`).
- ✅ `README.md` CLI/docs references refreshed to current command groups and less overclaiming wording for conditional sandbox capability.
- ✅ WG-054 status propagated to GOALS/ACTIONS/GOAP_STATE/ROADMAP/CURRENT/GAP_ANALYSIS/README plan docs.

### Validation Coverage & CI Parity (ADR-033 & ADR-038)
- ✅ `.github/workflows/ci.yml` now runs workspace nextest scope in required test jobs (instead of three `--lib` slices), and `mcp-build` test scope no longer uses `--lib`-only execution.
- ✅ `.github/workflows/benchmarks.yml` now discovers bench names dynamically from `benches/Cargo.toml` and executes the full declared bench surface (timeout-governed).
- ✅ `scripts/check-coverage.sh` now enforces threshold by parsing TOTAL coverage and failing below target (default 90).
- ✅ `tests/quality_gates.rs` now defaults coverage threshold to 90 and includes parsing robustness tests for TOTAL-row coverage extraction.

### Disk & Developer Experience Gaps (ADR-032)
- ✅ `scripts/clean-artifacts.sh` now supports practical cleanup automation:
  - mode help output (`--help`)
  - optional JS cleanup (`--node-modules`)
  - explicit target override (`--target-dir`) and `CARGO_TARGET_DIR` awareness
  - coverage artifact cleanup (`coverage/`, `coverage-html/`, `*.profraw`, `*.profdata`, `lcov.info`, `cobertura.xml`)
- ✅ AGENTS.md + relevant agent docs/skills now document current disk hygiene reality and remove stale mold-first guidance.

---

## Recommended Remediation (Mapped to New WGs)

| WG | Focus | Actions | Owners |
|----|-------|---------|--------|
| **WG-051** | Durable recommendation attribution | ✅ Complete — Turso/redb storage traits, integration tests, metrics surfaced | feature-implementer + architecture |
| **WG-052** | Durable checkpoints/handoffs | ✅ Complete — checkpoint metadata persisted across storage/redb, resume flows verified | feature-implementer + architecture |
| **WG-053** | MCP contract integrity | ✅ Complete — explicit defer decision + parity/tests/docs/plans alignment | memory-mcp + goap-agent |
| **WG-054** | Docs/CLI/API parity | ✅ Complete — API/README/playbook + plans truth-source refresh verified against parity test and CLI help | documentation |
| **WG-055** | Required CI coverage | ✅ Complete — CI test scope expanded to workspace nextest + benchmark workflow now covers full bench declarations | github-workflows + test-runner |
| **WG-056** | Coverage gate enforcement | ✅ Complete — script/test coverage thresholds and parsing now enforce <90% failures correctly | quality-unit-testing |
| **WG-057** | Disk hygiene | ✅ Complete — cleanup script automation + `CARGO_TARGET_DIR` + optional node_modules mode + coverage artifact cleanup | performance |
| **WG-058** | Agent guidance parity | ✅ Complete — AGENTS.md, relevant agent docs, and relevant skills aligned to script-first, coverage >=90 policy, and disk/linker reality | agents-update + documentation |

All items are scoped and sequenced in `plans/GOAP_EXECUTION_PLAN_v0.1.23.md`.

---

## References

- `plans/GOAP_EXECUTION_PLAN_v0.1.23.md`
- `plans/ROADMAPS/ROADMAP_ACTIVE.md`
- ADR-022, ADR-032, ADR-033, ADR-038, ADR-044
- Audit evidence from `git status`, `du -sh`, and targeted `rg` queries captured above
