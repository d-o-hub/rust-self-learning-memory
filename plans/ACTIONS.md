# GOAP Actions Backlog

- **Last Updated**: 2026-11-17
- **Active plan**: post-`v0.1.50` closeout. The release, publish chain and released-CLI
  dogfood are green; the workspace is `0.1.51`. The last open product issue (#1281,
  external-consumer example) is implemented and measured. **No code-side action is open.**
- **Remaining**: upstream-blocked advisories (ACT-404).
- **Archive**: `plans/archive/2026-11-consolidation/`
- **Completed pointer**: `plans/STATUS/archive/2026/` and the archived registers above.

## Active actions

| ID | Action | Owner | Status |
|----|--------|-------|--------|
| ACT-404 | Keep the four upstream-blocked advisory families classified with their pinning chains (`rustls`/`rustls-webpki` via libsql/hyper, `lru` via augurs/agentfs, `libsql-sqlite3-parser` with no patch). Re-test only when a chain moves; never open a no-op `libsql 0.10` bump. Revalidated 2026-11-16: `libsql 0.10.0` still resolves `hyper-rustls 0.25 → rustls 0.22.4`; `augurs-changepoint 0.10.3 → rv 0.16.5 → lru 0.9.0`; `agentfs-sdk 0.6.4 → lru 0.12.5` | Dependabot | ⏸ upstream |
| ACT-424 | Schedule the next release only when cadence or a feature justifies it; run `release-guard` + `release-manager.sh ship --execute` and dogfood the exact artifact | release | ✅ `v0.1.50` shipped 2026-11-16; next not scheduled |

## Completed actions (2026-11-16 — v0.1.50 release)

| ID | Action | Status |
|----|--------|--------|
| ACT-426 | Release `v0.1.50`: release-prep PR #1275, gated `ship`, release workflow #43829111481, publish chain #43833847807, crates.io verification and released-CLI dogfood | ✅ tag `v0.1.50` on `0fe1b89d`; all four crates at `0.1.50`; dogfood record in `VALIDATION_LATEST.md` |

## Completed actions (2026-11-17 — external-consumer example, #1281)

| ID | Action | Status |
|----|--------|--------|
| ACT-429 | Add a fixture-driven external-consumer example for adaptive web/source discovery: record attempts as episodes, retrieve by query family, bound the plan switch, redact endpoints, and self-check the run | ✅ `examples/adaptive_source_discovery.rs`; 22 unit tests; measured recall 0.33 → 0.87, precision 0.23 → 0.70; identical decision fingerprint across processes; trace replay + fail-loud rejection |

## Completed actions (2026-11-17 — R-F2 opt-in OTLP tracing)

| ID | Action | Status |
|----|--------|--------|
| ACT-428 | Promote R-F2 from the gated backlog with a measured spike and implement opt-in OpenTelemetry OTLP tracing | ✅ `R-F2-otel-2026-11-17.json` passes every threshold; ADR-084; `otel` feature with redaction, sampler clamp, and 4 tests |

## Completed actions (2026-11-16 — ADR lifecycle closeout)

| ID | Action | Status |
|----|--------|--------|
| ACT-425 | Accept ADR-080 / ADR-081 / ADR-082 with recorded test evidence; reconcile stale ADR-028 / ADR-042 / ADR-043 statuses; retire ADR-060 | ✅ `ADR-080/081/082 Accepted`; `ADR-028/042/043` rebaselined; `ADR-060` Accepted with the stale `#[allow(deprecated)]` removed |
| ACT-427 | Implement the `LearningOrchestrator` follow-up (reward + reflection + pattern dispatch in one coordinator) | ✅ `memory-core/src/learning/orchestrator.rs`; 1503 core lib tests pass |

## Completed actions (2026-11-16 — coding-workflow dogfood)

| ID | Action | Status |
|----|--------|--------|
| ACT-426 | Dogfood the coding workflow with the product itself (CLI lifecycle, cross-process patterns, durable SQLite parity, tags/journal, MCP stdio, memory-harness trace replay) and fix the drift it exposed | ✅ `scripts/dogfood-coding-workflow.sh`; 23/23 checks; skill evals green |

## Completed actions (2026-11-15 — harness + plans hygiene)

Every row is backed by a command in `STATUS/VALIDATION_LATEST.md`.

| ID | Action | Status |
|----|--------|--------|
| ACT-420 | do-harness `0.4.6` pin and doctrine contract (landed in #1273); re-verified both probes and the 31-case contract in this pass | ✅ measured green |
| ACT-421 | Compact `plans/`: archive the four completed dated GOAP registers, the 2026-11-01 gap/validation records and the prior tracker snapshots; rewrite the active set forward-only | ✅ `validate-plans.sh --all` |
| ACT-422 | Replace the unmeasured `forced_go` R-F artifacts with an authoritative `NO-GO` backlog artifact; correct the false portable-SIMD premise and record the OTel 0.33.x / trusted-publishing facts | ✅ all spike artifacts schema-valid |
| ACT-423 | Remove `.jules/` and the one-line `GEMINI.md` shim; drop `GEMINI.md` from the file-structure workflow and root-doc allowlist | ✅ `AGENTS.md` is the single contract |

## Earlier completed (pointer)

| Series | Outcome |
|--------|---------|
| Post-`v0.1.49` closeout | #1265 runner-pin fix, #1267 `0.1.50` bump, #1268 libsql pin policy, #1269 sensor inputs, #1270…#1272 dependency/action bumps |
| Post-`v0.1.47` dogfood | #1224 advisory, #1225 pattern counts, #1226 context budget, #1227 generated lockfile, #1228 version restore, #1229 ADR-079 probe |
| Release closeout ACT-399…ACT-412 | publish guard, readiness ordering, schema sensor, release-prep and tags `v0.1.46`…`v0.1.49` |
| Audit-backlog wave ACT-370…ACT-398 | W1–W6 + Wave A merged (22 findings closed) |
| Release pipeline #1109 C1–C7 | OIDC publishing, draft-first release, attestations, verify page (#1121/#1123/#1125) |
| Earlier campaigns (ACT-190…ACT-369) | Archived under `plans/archive/2026-07-consolidation/completed-sprints/` |

## Prevention permanently (do not regress)

- Never `#[serde(tag=)]` on postcard types.
- StorageBackend new methods → all backends.
- CLI path flags → set `redb_path`.
- Cross-process storage features → e2e CLI test.
- No manual `gh release create`; use release-manager + `release.yml`.
- No soft-pass on cargo deny / required cancelled checks.
- Required status must be a causal same-run aggregate; an echo anchor is not a gate.
- Dependabot/fork trust changes permissions and secret access, not test assertions.
- Fail-closed `execute_agent_code` unless an approved capability backend exists.
- sha2 digests: use portable hex encode (not `format!("{:x}", finalize())` on 0.11+).
- Docs integrity: do not re-check `plans/archive/**` link rot as a ship blocker.
- After tag `vX.Y.Z`, immediately bump workspace to the next patch before more feat/fix commits.
- Commit bodies must stay ≤100 chars; repair mechanically and verify with commitlint.
- No-op `chore(ci): re-trigger` commits are lint noise — drop them via rebase.
- Fix release drift at the root (ship or bump), never with a waiver; `release-preparation`
  only re-evaluates when re-applied from outside.
- Codecov patch coverage: dedupe duplicated rendering and add targeted tests for new paths.
- Bot/agent branches can silently revert main: diff the tip against its merge base, not
  the moving `main` tip.
- Machine-readable test output can be empty: require `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`
  and assert artifacts are non-empty.
- A version bump alone does not make the repo shippable: run
  `verify-release-state.sh --check-unreleased` before promising a tag date.
- Dependabot (GHSA), `cargo audit` and `cargo deny` (RUSTSEC) are different registers.
- An advisory can be invisible to both local gates (informational / GHSA-only); cite the
  ignore list and `.warnings` count when using a green audit as evidence.
- Never plan a dependency upgrade from a changelog claim: resolve the candidate in a
  scratch crate and `cargo tree` it.
- Record an unfixable advisory as upstream-blocked with the pinning chain named.
- Floating `stable` adds deny-by-default lints between releases: fix, never suppress.
- Strict up-to-date checks make merges serial; merge as a pipeline and update the next branch.
- Shared `CARGO_TARGET_DIR` across worktrees fakes mass failures; re-run in isolation.
- Composite-action inputs are strings: compare literally (`inputs.dry-run != 'true'`).
- A green job is not evidence of work: assert the effect, not the conclusion.
- `statusCheckRollup` keeps superseded runs: reduce to the latest run per name.
- `#[ignore]` + `--run-ignored only` cannot express feature gating; use
  `[[test]] required-features` and assert the run count.
- Never assert `result.is_ok()` without printing the error.
- Hand-written SQL must match the DDL constants (columns *and* units).
- `get_connection_with_id()` is direct-connection only; pooled call sites use `with_connection`.
- A sensor pin is only real when the binary, tracked file, prose and CI matrix agree.
