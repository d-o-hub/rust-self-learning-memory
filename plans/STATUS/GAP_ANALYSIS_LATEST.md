# Gap Analysis — 2026-03-24 Audit (v0.1.22 Post-Release)

**Generated**: 2026-03-24 (GOAP audit reboot)
**Method**: Read-only repo inspection + ADR cross-check + CLI/MCP contract review
**Scope**: ADR-044 feature durability, documentation truth sources, CI/test coverage, disk hygiene

---

## Summary

The v0.1.22 sprint successfully shipped its 12 tracked issues, but the latest audit reveals new or resurfaced gaps. Documentation in `plans/` still states “all gaps resolved”, which no longer matches implementation reality. This report supersedes the 2026-03-20 gap analysis and feeds the new execution plan (`GOAP_EXECUTION_PLAN_v0.1.23.md`).

---

## Key Gaps (Prioritized)

### P0 — Implementation Integrity (ADR-044)

| Gap | Evidence | Impact | Linked WG |
|-----|----------|--------|-----------|
| ~~Checkpoint/handoff metadata dropped in storage round-trips~~ | ✅ Resolved 2026-03-24 via Turso `checkpoints` schema + serialization updates and `resume_from_handoff` storage-backed persistence | Restart and round-trip durability now validated by integration + targeted storage tests | WG-052 |
| ~~Batch MCP tools unresolved~~ | ✅ Resolved 2026-03-24 via explicit defer decision + parity/docs/plans alignment | MCP contract now truthfully documents deferred tool-level batch analytics names | WG-053 |

#### Remediation Progress (2026-03-24)

- ✅ **WG-051** — Durable recommendation attribution implemented via `do-memory-storage-turso/src/storage/recommendations.rs`, `do-memory-storage-redb/src/recommendations.rs`, and storage trait impls/resilient wrappers. Integration evidence: `tests/attribution_integration_test.rs` (`cargo nextest run --test attribution_integration`).
- ✅ **WG-052** — Durable checkpoints/handoffs implemented via Turso schema (`checkpoints` column), CRUD/query/batch checkpoint serialization, backward-compatible row conversion defaults, and storage-backed `resume_from_handoff` metadata persistence. Integration evidence: `cargo nextest run --test checkpoint_integration` and targeted Turso tests.

### P1 — Documentation & Contract Drift

| Gap | Evidence | Impact | Linked WG |
|-----|----------|--------|-----------|
| ~~API reference outdated (v0.1.13 + obsolete tools)~~ | ✅ Resolved in WG-054 via contract refresh from `do-memory-mcp/tests/tool_contract_parity.rs`; deferred batch tools explicitly marked absent | Contract index now matches runtime/parity tool list | WG-054 |
| ~~Playbook/checkpoint/feedback docs mention non-existent CLI commands~~ | ✅ Resolved in WG-054 via `do-memory-cli --help` aligned command updates (`episode`, `playbook`, `feedback`) | CLI onboarding docs now reflect live command names | WG-054 |
| ~~README + plans advertise secure code execution + “all gaps closed” despite disabled tool~~ | ✅ Resolved in WG-054 with conditional sandbox wording + status/roadmap truth updates | Reduced overclaiming in top-level docs/plans | WG-054 |
| ~~AGENTS.md/agent_docs/.agents/skills instructions lag behind script/CI reality~~ | ✅ Resolved 2026-03-24 via AGENTS + agent docs + skills parity refresh (script-first, coverage >=90, disk guidance) | Workflow guidance now matches current scripts and policies | WG-058 |

### P1 — Validation & Coverage Parity (ADR-033 / ADR-038)

| Gap | Evidence | Impact | Linked WG |
|-----|----------|--------|-----------|
| ~~Required PR CI only runs three `--lib` subsets~~ | ✅ Resolved in WG-055: `ci.yml` now runs workspace nextest scope for required test jobs | Wider CI gate now exercises integration surfaces beyond lib-only smoke | WG-055 |
| ~~Coverage script never enforces ≥90% target~~ | ✅ Resolved in WG-056: `scripts/check-coverage.sh` parses TOTAL coverage and fails below threshold (default 90); `tests/quality_gates.rs` defaults/parsing updated | Coverage gate now provides true failure semantics | WG-056 |
| ~~Benchmark workflow runs 4/14 benches~~ | ✅ Resolved in WG-055: `benchmarks.yml` dynamically discovers/runs full bench set from `benches/Cargo.toml` | Performance regression coverage expanded to declared bench surface | WG-055 |

### P2 — Disk / Developer Experience (ADR-032)

| Gap | Evidence | Impact | Linked WG |
|-----|----------|--------|-----------|
| `target/` back to 32G locally | `du -sh target` (2026-03-24 audit) | Addressed with stronger cleanup automation (`scripts/clean-artifacts.sh`) and `CARGO_TARGET_DIR` guidance | WG-057 |
| `node_modules/` present (130M) despite ADR claim of removal | `du -sh node_modules` | Addressed with explicit optional cleanup mode (`--node-modules`) and documentation of expected local variance | WG-057 |
| ~~Mold linker removal undocumented in plans/skills~~ | ✅ Resolved 2026-03-24 by removing mold-first guidance from active docs/skills | Guidance now reflects CI-compatible linker defaults | WG-058 |

---

## Actions Tracked in GOAP v0.1.23

| WG | Description | Initial Actions |
|----|-------------|-----------------|
| WG-051 | Durable recommendation attribution | storage trait + schema design doc, integration tests, CLI verification |
| WG-052 | Durable checkpoint/handoff persistence | Turso row serialization updates, resume pipeline tests |
| WG-053 | MCP contract integrity | ✅ Complete — keep tool-level batch analytics deferred; parity tests + plans/README/API docs aligned |
| WG-054 | Docs + CLI/API truth source refresh | ✅ Complete — API reference, README, `docs/PLAYBOOKS_AND_CHECKPOINTS.md`, and plans status files refreshed against parity/CLI help |
| WG-055 | CI/test surface expansion | ✅ Complete — `ci.yml` test jobs now run workspace nextest scopes; `mcp-build` test scope expanded; `benchmarks.yml` dynamically runs full bench list from `benches/Cargo.toml` |
| WG-056 | Coverage enforcement | ✅ Complete — `scripts/check-coverage.sh` now parses TOTAL coverage and exits non-zero below threshold (default 90); `tests/quality_gates.rs` threshold/parsing updated with tests |
| WG-057 | Disk hygiene automation | ✅ Complete — `scripts/clean-artifacts.sh` now supports `--help`, `--node-modules`, `--target-dir`, `--dry-run`, and expanded coverage artifact cleanup |
| WG-058 | Agent guidance parity | ✅ Complete — AGENTS.md, relevant agent_docs, and relevant `.agents/skills/` aligned to script-first + coverage/disk guidance |

See `plans/GOAP_EXECUTION_PLAN_v0.1.23.md` for phase sequencing and quality gates.

---

## Previously Closed Items (Reference Only)

The v0.1.22 sprint closure data remains accessible in git history (commit `15bc3ab3`). Completed tables for WG-040—WG-053 remain accurate for that sprint but are no longer considered “current state”. Historical metrics are preserved below for comparison only.

### Historical Snapshot — v0.1.22 Close-out (2026-03-20)

| Metric | Value | Target |
|--------|-------|--------|
| Tests | 2,841/2,841 passing | — |
| Ignored tests | 124 | ≤125 |
| `#[allow(dead_code)]` | 31 | ≤40 |
| Snapshot tests | 80 | ≥80 |
| Property test files | 16 | ≥13 |

*(All other historical tables from the previous analysis were removed from the “current” view to avoid confusion.)*

---

## Cross-References

- `plans/STATUS/VALIDATION_LATEST.md`
- `plans/GOAP_EXECUTION_PLAN_v0.1.23.md`
- `plans/ROADMAPS/ROADMAP_ACTIVE.md`
- ADR-022, ADR-032, ADR-033, ADR-038, ADR-044
