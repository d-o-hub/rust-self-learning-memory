# Validation Latest — 2026-11-24 (post-`v0.1.50` features + 0.1.51 development)

**Workspace**: `0.1.51` (post-release development bump) · **Released tag**: `v0.1.50`
(`0fe1b89d`) · **Main baseline**: `eb076391` (after #1289, #1291, #1290)
**Scope**: `v0.1.50` release, crates.io publication, released-CLI dogfood, `0.1.51` bump,
the R-F2 tracing/lifecycle/learning-orchestrator merges, the adaptive source-discovery
example (#1281 + candidate window), and durable `EpisodeSummary` + `WG-075` retention
cleanup across core, storage wrappers, sync, CLI, and MCP.
**Previous full record**: [`archive/2026/VALIDATION_2026-11-01.md`](archive/2026/VALIDATION_2026-11-01.md)

## Episode summaries, retention cleanup CLI/MCP/wrapper parity & discovery candidate window — 2026-11-24

| Check | Command | Result |
|-------|---------|--------|
| Core summary & retention persistence | `cargo nextest run -p do-memory-core --test episode_summary_persistence` | ✅ passed |
| Turso + wrapper capabilities & cache eviction | `cargo nextest run -p do-memory-storage-turso --test capability_attribution_test` | ✅ 10 passed |
| redb summary & cleanup capabilities | `cargo nextest run -p do-memory-storage-redb --test capability_attribution_test` | ✅ 6 passed |
| CLI `episode cleanup` unit & policy tests | `cargo nextest run -p do-memory-cli --lib cleanup` | ✅ 3 passed |
| MCP contract parity & lifecycle (`get_episode` summary + `cleanup_episodes`) | `cargo nextest run -p do-memory-mcp --test tool_contract_parity --test episode_lifecycle_test` | ✅ 21 passed |
| Adaptive discovery candidate window | `cargo test -p do-memory-examples --example adaptive_source_discovery` | ✅ 23 passed |
| LOC & plan validation | `./scripts/check-loc.sh`; `./scripts/validate-plans.sh --all`; `./scripts/check-docs-integrity.sh` | ✅ clean |
| Full workspace tests | `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 cargo nextest run --all` | ✅ 4,474 passed; 149 skipped |
| Doctests | `cargo test --doc` | ✅ 230 passed; 5 ignored |
| Workspace Clippy | `./scripts/code-quality.sh clippy --workspace` | ✅ no warnings |
| Dependency gates | `do-harness verify --only lockfile-sync --strict`; `do-harness verify --only deny --strict` | ✅ both passed |
| CLI cleanup confirmation | `cleanup_is_preview_only_until_explicit_confirmation` in workspace suite | ✅ preview preserves episode; confirmation deletes it |
| Summary mirroring in single/batch synchronizer + CLI reconciliation | `cargo nextest run -p do-memory-core --test storage_sync`; `cargo nextest run -p do-memory-cli sync_storage_reconciles_episodes_between_backends` | ✅ 17 core sync tests and 2 CLI sync tests passed |
| End-to-end workflow dogfood | `cargo build -p do-memory-cli --features turso`; `./scripts/dogfood-coding-workflow.sh` | ✅ 25 passed; durable SQLite sibling and MCP handshake verified |
| MCP CRUD + direct SQLite inspection | MCP stdio `create_episode`/`get_episode`/`update_episode`/`delete_episode` against temporary SQLite DB; query `episodes` by id | ✅ create/read/update persisted across cold-cache restart; delete removed row |

## Adaptive source-discovery example (#1281) — 2026-11-17

| Check | Command | Result |
|-------|---------|--------|
| Lints (examples included) | `cargo clippy -p do-memory-examples --all-targets -- -D warnings` | ✅ clean |
| Example unit tests | `cargo test -p do-memory-examples --example adaptive_source_discovery` | ✅ 22 passed |
| End-to-end run | `./scripts/run-adaptive-discovery-example.sh` | ✅ 20 attempts seeded; expectations passed |
| Measured comparison | same runner, `report-recommend.json` | ✅ recall 0.33 → 0.87, precision 0.23 → 0.70, diversity 0.60 → 1.80, latency 3130 ms → 2560 ms, cost $0.0090 → $0.0140 |
| Restart check | `--phase verify --expect report-recommend.json` | ✅ fingerprint `56e59a953ba85e6f` identical across processes |
| Memory-harness replay | `replay-trace.sh` on an emitted trace | ✅ replayed through `do-memory-cli` |
| Fail-loud contract | rewrote a trace outcome to `in_progress` | ✅ replay rejected it |
| Post-merge dogfood | same runner from `main` @ `f02d81f9` | ✅ identical totals and fingerprint `56e59a953ba85e6f`; replay accepted, incomplete trace rejected |
| Formatting / size | `cargo fmt --all -- --check`; `./scripts/check-loc.sh` | ✅ clean; no file over 500 LOC |

Guide: [`../../docs/ADAPTIVE_SOURCE_DISCOVERY.md`](../../docs/ADAPTIVE_SOURCE_DISCOVERY.md).

## R-F2 opt-in OTLP tracing — 2026-11-17

| Check | Command | Result |
|-------|---------|--------|
| Feature compiles | `cargo check -p do-memory-core --features otel` | ✅ clean |
| Lints | `cargo clippy -p do-memory-core --all-targets --features otel-testing -- -D warnings` | ✅ clean |
| Zero-overhead / redaction / sampler / export | `cargo nextest run -p do-memory-core --features otel-testing -E 'test(observability)'` | ✅ 4 passed |
| Formatting | `cargo fmt --all -- --check` | ✅ clean |

Spike artifact: `plans/STATUS/spikes/R-F2-otel-2026-11-17.json`; decision record:
`plans/adr/ADR-084-Opt-In-OpenTelemetry-OTLP-Tracing.md`.

## ADR lifecycle closeout — 2026-11-16

| Check | Command | Result |
|-------|---------|--------|
| Attribution/ranking evidence | `cargo nextest run -p do-memory-core --test ranking_adaptation_e2e --test attribution_receipt_matrix --test attribution_feedback_restart --test attribution_property_tests` | ✅ 48 passed, 0 skipped |
| Turso capability truth | `cargo nextest run -p do-memory-storage-turso --test capability_attribution_test` | ✅ 8 passed |
| redb capability truth | `cargo nextest run -p do-memory-storage-redb --test capability_attribution_test` | ✅ 5 passed |
| ADR-060 suppression removal | `cargo check -p do-memory-mcp` | ✅ clean |
| Formatting | `cargo fmt --all -- --check` | ✅ clean |
| Plans / docs | `./scripts/validate-plans.sh --all`; `./scripts/check-docs-integrity.sh` | ✅ green; tracker drift 0 |
| ACT-404 revalidation | `cargo tree --locked -i rustls`; `cargo tree --locked -i lru@0.9.0`; `cargo tree --locked -i lru@0.12.5`; scratch `libsql 0.10.0` tree | ✅ no chain moved; no bump justified |

## Release evidence — v0.1.50

| Artifact | Evidence |
|----------|----------|
| Tag | `v0.1.50` → `0fe1b89d`, pushed by `release-manager.sh ship --execute` |
| Release workflow | run `43829111481` — preflight, plan, all five build targets, host, SBOMs, attestations and publish all succeeded |
| GitHub Release | published 2026-11-16T08:45:18Z, **37 assets**, not a draft, not a prerelease; body contains `## Release Notes` from `parse-changelog` |
| crates.io | `do-memory-core`, `do-memory-storage-redb`, `do-memory-storage-turso`, `do-memory-mcp` all report `0.1.50` in version membership |
| Publish workflow | run `43833847807` green; `Verify publish landed` in the publishing jobs |
| Released-CLI dogfood | asset `do-memory-cli-x86_64-unknown-linux-gnu.tar.xz` digest `ea2da8eddec492f04211cf598449a0dfdda64646f06880cb6495999344d4087d` matched `sha256sum`; `--version` = `0.1.50` |
| Dogfood database | fresh `/tmp/opencode/dogfood-0.1.50/memory.redb`; episode `e982decc-1618-4965-ba74-0e7b547bc975`, 3 successful steps, outcome `success` |
| Cross-process parity | fresh processes: `episode list` = 1, `pattern list` = 2, `pattern search "release dogfood"` = 2, `storage stats.patterns.total_count` = 2 → `2 == 2` |

No mismatch blocked closeout. The release notes were generated, not empty.

## Post-release bump checks

| Check | Command | Result |
|-------|---------|--------|
| Workspace version | `cargo metadata --format-version 1` | ✅ all 9 packages at `0.1.51` |
| Lockfiles | `./scripts/check-lockfile-sync.sh` | ✅ root and fuzz locks satisfy their manifests; `libsql` single-version holds in both |
| Release drift | `./scripts/check-release-drift.sh` | ✅ workspace `0.1.51` ahead of tag `v0.1.50`; `severity=clean` |
| Plan hygiene | `./scripts/validate-plans.sh --all` | ✅ `dated_root=0`, `cargo=0.1.51 tag=0.1.50` |
| Docs integrity | `./scripts/check-docs-integrity.sh` | ✅ links, scripts, version consistency, fail-closed contract |
| Changelog headings | `./scripts/check-changelog-headings.sh` | ✅ unique, ordered, one `Unreleased` section |
| Harness | `do-harness verify --set verification --strict --record` | ✅ 23/23 sensors; `status --set verification` green |

## Earlier verification (2026-11-15)

Plans compaction, the single agent contract, the harness `0.4.6` doctrine and the R-F
backlog truth were verified in that change: `validate-plans.sh --all`,
`check-docs-integrity.sh`, `validate-gate-contract.sh --ci-parity`, 31 harness contract
cases, 17 spike artifacts, and 23/23 verification sensors. Full history:
[`archive/2026/`](archive/2026/).

## Method

- Release claims come from the tag, the workflow run, the GitHub Release object and the
  crates.io version-membership API — not from the release job's green conclusion alone.
- Dogfood uses the published asset from this tag, never a local build.
- A version bump is not a release: `0.1.51` stays a development version until its own
  release-prep PR and `ship`.
