# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **F4.1** retrieval provenance API: `retrieve_relevant_context_with_provenance`
  returns redacted `RetrievalProvenance` + latency (no raw query leakage).
- **F4.2** operation journal for capacity-eviction multi-backend intents with
  `operation_journal_snapshot` / `operation_journal_pending`.
- **S1.5b / F4.3** local model integrity pins: `expected_sha256`,
  `max_artifact_bytes`, `model_revision` on `LocalConfig` +
  `verify_model_artifact`.
- **F4.4** skill contract compiler: `scripts/compile-skill-contracts.sh` and
  generated `.agents/skills/skill-catalog.generated.json`.

### Changed

- Workspace version advanced to **0.1.36** after shipping `v0.1.35` so
  release-drift gates treat new commits as normal development
  (`version_not_advanced` no longer applies).
- **CHANGELOG uniqueness** (from PR #858): rename historical duplicate
  `## [0.1.11]` / `## [0.1.10]` headings so `parse-changelog` / cargo-dist can
  emit GitHub Release notes. `verify-release-state.sh` rejects duplicate
  version headings.
- **S1.1b**: Node code sandbox module gated behind optional `sandbox-dev`
  feature (default off). Production MCP remains fail-closed for
  `execute_agent_code`.

### Added

- **S1.2 remainder (ADR-074)**: retrieval `CacheKey` identity now includes
  retrieval mode, provider/model/dimension identity, ranking config version,
  and monotonic index generation; redacted `RetrievalProvenance` envelope;
  generation bumps on cache invalidation.
- **S1.4b**: typed capacity-eviction partial failures with
  `pending_eviction_failures` / `reconcile_pending_evictions()`.
- **K3.2**: positive/negative eval fixtures for high-risk skills
  (release-guard, pr-readiness, commit, ci-fix, code-quality, test-runner,
  goap-agent, web-doc-resolver).
- **W2 tooling**: `scripts/check-source-reachability.sh`,
  `scripts/test-workflow-guards.sh`, `scripts/test-release-workflow.sh`,
  `scripts/generate-skill-inventory.sh`, `scripts/test-benchmark-workflow.sh`,
  `scripts/check-ignored-tests.sh`, `scripts/validate-skill-routes.sh`,
  `scripts/validate-plans.sh`.
- **W2.3b**: quality_gates refuse metric parse on failed subprocess; package
  name guards reject legacy `memory-core`.
- **W2.5**: benchmarks workflow fails on missing Criterion output (no dummy
  soft-pass); `fail-on-alert: true` for >10% regression.
- **K3.3**: expanded `skill-rules.json` routes for high-frequency skills.
- **S1.7 audit hardening**: recursive nested/array redaction (case-insensitive
  field match); rotation size initialized from existing file metadata; bounded
  non-blocking file writer thread with `dropped_writes()` overflow metrics.
- **K3.1b skill-eval CI**: `.github/workflows/skill-evals.yml` runs schema
  fixtures always, `run-evals.sh --changed` on PRs, full suite on weekly
  schedule / workflow_dispatch.
- **W2.1b gate contract CI parity**: `validate-gate-contract.sh --ci-parity`
  verifies authoritative workflows (quick-check, ci, release-drift,
  security/supply-chain, skill-evals); Skill Evals job runs the validator.

### Fixed

- Audit logger no longer reset in-memory size to `0` after opening an existing
  log file (oversized logs now rotate on first write when rotation is enabled).

## [0.1.35] - 2026-07-17

### Added

- **K3.1 skill eval contract**: `scripts/run-evals.sh` enforces a strict
  `tests[]` schema (rejects legacy `evals` key, zero tests, missing `exec`, and
  noop `true`). Schema fixtures under `scripts/fixtures/skill-evals/`. Skill
  evals migrated to non-noop contract checks; `pr-readiness` validates
  merge-state and comment-fetch requirements.
- **W2.1 gate contract**: `plans/GATE_CONTRACT.md` documents measured / blocking
  floor / aspirational target / authority for each advertised gate (explicit
  coverage 70% floor vs 90% target). Validator:
  `./scripts/validate-gate-contract.sh` (optional `--ci-parity`).
- **#845 / ADR-076 empty pattern diagnostics**: when `pattern list` / `pattern
  search` return zero results in **Human** format, print a short diagnostic
  footer (patterns come from episode complete with ≥1 step for tool sequences;
  not from `storage sync`; confirm `--db-path` / `config show`). JSON/YAML empty
  arrays stay machine-stable.
- **#846 config precedence docs**: document the 5-level precedence table
  (CLI flags → env → `--config` → auto-discovered CWD config → defaults) in
  root `README.md` and `memory-cli/CONFIGURATION_GUIDE.md`.
- **ADR-076 `storage sync` messaging**: when only local redb / missing dual
  backends, error text states sync is Turso↔redb reconciliation, **not** pattern
  extraction.
- **#847 / ADR-075 operator `episode fail`**: force-fail abandoned in-progress
  episodes with `TaskOutcome::Failure` using the same verify-after-write path as
  `episode complete`.

### Fixed

- **#847 / ADR-075 durable episode complete**: configured backend `store_episode`
  failures on complete no longer soft-warn and return success. Any cache/Turso
  store failure returns `Error::Storage` and skips in-memory finalize / pattern
  extract. CLI re-fetches and asserts `is_complete()` before printing success
  (no false-green complete banner).
- **S1.3 lock-free step persistence**: `log_step` (immediate path) and
  `flush_steps_internal` no longer hold `episodes_fallback` write locks across
  backend `store_episode` awaits; concurrent unique steps persist exactly once.
- **S1.4 durable capacity eviction**: capacity-constrained completion now deletes
  evicted episodes (and embeddings) from cache and durable backends instead of
  only removing them from the in-memory map.
- **S1.5 embedding health**: introduce `EmbeddingHealth` (`Real` /
  `DegradedMock` / `Unavailable`). Mock fallback is opt-in via
  `LocalConfig::allow_mock_fallback`; mock providers are never
  `is_available()` / production-ready.
- **S1.6 retry queue semantics**: first attempts do not consume concurrency
  permits; permits are released before backoff; `retry_queue_timeout` returns
  `RetryError::QueueTimeout` / `Error::RetryQueueTimeout`; zero
  `max_concurrent_retries` is rejected.
- **W2.2 advisory gates**: CI/security no longer soft-pass `cargo audit`;
  `cargo deny check advisories` is the blocking gate.
- **#837 fuzzy_match rustdoc**: public `///` docs and doctests restored on
  `pub fn fuzzy_match` (they had been attached to the private lowercased helper
  after the PR #836 performance split).
- **S1.2 / ADR-074 (partial) retrieval cache identity**: `CacheKey` now includes
  language, framework, complexity, and normalized tags from `TaskContext`, so
  context-distinct queries no longer share incorrect cache entries. Wired in
  `retrieve_relevant_context` via `with_task_context`.
- **W2.3 build-rust package names**: `./scripts/build-rust.sh` accepts hyphenated
  `do-memory-*` crate names (was limited to `[a-z0-9_]+`).
- **W2.6 source LOC gate**: split production files over 500 LOC (`retry`, CLI
  embedding commands, `storage` backend trait, local embeddings tests, checkpoint
  op tests, retrieval context cache helper).
- **S1.1a / D3.2 docs contract**: document `execute_agent_code` as unavailable /
  fail-closed; remove false `wasmtime-backend` feature claims; fix README
  `TaskContext` example fields.
- **#831 Pattern retrieval across processes**: `Pattern` enum was internally
  tagged (`#[serde(tag = "type")]`), which Postcard cannot deserialize. Patterns
  were written to storage but never read back, so `pattern list` / `pattern search`
  returned 0 after completing episodes in a fresh CLI process. Changed `Pattern`
  to an externally-tagged (Postcard-compatible) representation and added a
  `postcard` round-trip regression test. Also added `StorageBackend::get_all_patterns`
  (redb + Turso) and lazy-loaded `queries::get_all_patterns` so list/search hydrate
  from durable storage. Bumped redb `SCHEMA_VERSION` to 4 so stale caches are
  cleared on upgrade; undecodable pattern rows are skipped rather than failing
  the whole list. **Note:** JSON shape of `Pattern` also changes (externally
  tagged); Turso stores a separate JSON DTO and is unaffected.

- **#830 `--db-path` / `MEMORY_DB_PATH` ignored for redb**: always override
  `redb_path` (never only-when-None — `Config::default()` pre-fills XDG).
  Default **redb-only** builds open redb at the exact user path. Builds with
  the `turso` feature use sibling files (`memory.db` → redb `memory.redb`) so
  SQLite and redb never share a file. Unset `storage_mode` defaults to
  `local` when no Turso URL is configured. Unit tests in
  `config/cli_overrides.rs`.

- **#829 Undocumented config file format**: added `config init [--path]` and
  `config show-template` commands that emit a valid, populated `Config` TOML, and
  fixed documentation of real env vars (`TURSO_URL`/`TURSO_TOKEN`/`REDB_PATH`),
  the `[database].storage_mode` location, and removed non-existent sections
  (`[sandbox]`, `[backup]`, `[logging]`) and the unimplemented `import =` syntax.

- **#828 Release drift**: relabeled the `0.2.0` bump back to `0.1.35` (patch on the
  `0.1.x` line) across all workspace manifests and `Cargo.lock`.

- **#832 Config discoverability**: `config init` / `show-template` surface the
  canonical config shape. `storage_mode` belongs under `[database]`; an alias
  under `[storage]` is accepted and normalized. Partial TOML files work via
  `#[serde(default)]` on all config sections. Added
  `memory-cli/config/do-memory-cli.example.toml`.

## [0.1.34] - 2026-07-11



### Added

- Implement missing CLI/MCP surfaces and replace telemetry placeholders ([2d97328](https://github.com/d-o-hub/rust-self-learning-memory/commit/2d9732881387983bdda77b84e72b3825aca30df7))

- implement missing stubs WG-158/160/161/162 ([6a43dea](https://github.com/d-o-hub/rust-self-learning-memory/commit/6a43deae1256c2bdfc17131e938c93fda728cf39))

- bump v0.1.33, fix flaky benchmarks, resolve release drift (#623) (#624) ([07e4d0c](https://github.com/d-o-hub/rust-self-learning-memory/commit/07e4d0c03f922ae9d36dbad4ba619461fdf45d09))

- LOC boundary splits (WG-185) + llms.txt generator (WG-183, closes #652) (#692) ([61c2f9f](https://github.com/d-o-hub/rust-self-learning-memory/commit/61c2f9fea0b7baabf9530aae354c6d63f93b9ded))

- add AbstentionPattern extractor + abstention_score to RewardScore (#718) ([3972024](https://github.com/d-o-hub/rust-self-learning-memory/commit/3972024195ad403a7a21f1a6e4f047ed01a77224))

- adopt VERSION file as co-canonical version source (ADR-061) ([1ff45b6](https://github.com/d-o-hub/rust-self-learning-memory/commit/1ff45b6473328d561e052c9176345fd6a1eef433))

- auto-checkpoint on TaskOutcome::Abstained (#738) ([776648f](https://github.com/d-o-hub/rust-self-learning-memory/commit/776648f3ef2842e5ea09fa07f4234416c7f3ae86))

- add OpenTelemetry / OTLP export for tracing and monitoring (#755) ([a97727d](https://github.com/d-o-hub/rust-self-learning-memory/commit/a97727de2b42504907335d83af04cbd2b645cfc4))

- normalize reward scores and add temporal decay ([f9c78c0](https://github.com/d-o-hub/rust-self-learning-memory/commit/f9c78c0b07944e2f0417aefd1f34d9e65570b5ce))

- normalize reward scores and add temporal decay ([11ca894](https://github.com/d-o-hub/rust-self-learning-memory/commit/11ca894356d00ad6f7daf2d67fe40582739a038b))

- normalize reward scores and add temporal decay ([9cf40df](https://github.com/d-o-hub/rust-self-learning-memory/commit/9cf40dfad68d97716ab0a547994a6196ae9b2b51))

- normalize reward scores and add temporal decay ([58b9df9](https://github.com/d-o-hub/rust-self-learning-memory/commit/58b9df9ce9ffd2f33a333ac98df448b36905ae54))

- normalize reward scores and add temporal decay ([91aa936](https://github.com/d-o-hub/rust-self-learning-memory/commit/91aa936c8a12af71d2e9495ccc7535a52f02e5d8))

- normalize reward scores and add temporal decay ([9d42350](https://github.com/d-o-hub/rust-self-learning-memory/commit/9d42350727c38def2ef5839667ec8b3c7c96ac66))

- normalize reward scores and add temporal decay ([986e48f](https://github.com/d-o-hub/rust-self-learning-memory/commit/986e48fe8db989331154a8d1ad515cbb4f14fab3))

- implement statistical normalization and temporal decay ([c479ae8](https://github.com/d-o-hub/rust-self-learning-memory/commit/c479ae81a8a3e11490c9b2b019e36ddd4d2f1c35))

- implement statistical normalization and temporal decay ([ae4bc63](https://github.com/d-o-hub/rust-self-learning-memory/commit/ae4bc63baf9722a0be4b55b3c8f72965bfd7dfd5))

- implement statistical normalization and temporal decay ([0015e52](https://github.com/d-o-hub/rust-self-learning-memory/commit/0015e52f9159257285a903d0a9805f9bdcf4901f))

- add ANN-backed semantic episode retrieval (#775) ([13fdcf0](https://github.com/d-o-hub/rust-self-learning-memory/commit/13fdcf050bc848480c025fa607741234587b76bb))

- add focused mutation testing workflow for core modules (#793) ([0413b57](https://github.com/d-o-hub/rust-self-learning-memory/commit/0413b57eaf9ff18af8a9833fe217fc0b753323a4))

- v0.1.34 — CI fixes, all open issues resolved ([5fd3ca5](https://github.com/d-o-hub/rust-self-learning-memory/commit/5fd3ca5c0719273cc1c32abfd881b5c9c4914486))


### CI/CD

- split fast PR checks from heavy coverage and nightly workflows ([3e9386f](https://github.com/d-o-hub/rust-self-learning-memory/commit/3e9386f583fc64f014df092c376ba69e7cdb3718))

- bump the actions-all group with 2 updates (#612) ([b1496a7](https://github.com/d-o-hub/rust-self-learning-memory/commit/b1496a7a91b2f15e3598e5a0d54fa2ab3561f33c))

- bump lewagon/wait-on-check-action in the actions-all group (#628) ([fe9d2fe](https://github.com/d-o-hub/rust-self-learning-memory/commit/fe9d2fe1878c64570522831ab383989fe009812c))

- bump actions/checkout from 6 to 7 in the actions-all group ([a6c20fc](https://github.com/d-o-hub/rust-self-learning-memory/commit/a6c20fc455d26bd80b0eccc5b54527bda5fc9530))

- bump the actions-all group across 1 directory with 13 updates (#681) ([f19daef](https://github.com/d-o-hub/rust-self-learning-memory/commit/f19daefa9ae3fd5619c388f371df455e7a423ef4))

- Move audit-report.json, benchmark_results/, and metrics/ to CI artifacts ([41db5ff](https://github.com/d-o-hub/rust-self-learning-memory/commit/41db5ff500de7cde8be16db8311207c71cb26d4c))

- Move artifacts to CI and pin actions to SHAs ([82a340a](https://github.com/d-o-hub/rust-self-learning-memory/commit/82a340a400364facf561db5cbb91156ce561f1d7))

- Move artifacts to CI and pin actions to repo-consistent SHAs ([b27c59e](https://github.com/d-o-hub/rust-self-learning-memory/commit/b27c59ed6991c16c3e89ef1ae9f1a135f49bd632))

- Fix nightly-tests.yml linting and maintain SHA consistency ([87b5695](https://github.com/d-o-hub/rust-self-learning-memory/commit/87b569519e6b6442c7503674bd3cb2d845768fbf))

- bump the actions-all group with 4 updates (#758) ([a4a1c6b](https://github.com/d-o-hub/rust-self-learning-memory/commit/a4a1c6b796d15b417af6de6976214e4e90d77cd1))

- improve crates.io publish pipeline (#789) ([731d0a1](https://github.com/d-o-hub/rust-self-learning-memory/commit/731d0a1c5685eb527c7b480087aa00a48a60dbbe))


### Changed

- apply rustfmt to relationship_command_tests ([0f6b19f](https://github.com/d-o-hub/rust-self-learning-memory/commit/0f6b19f090bf7b164bab7d9f5ac7025ccd7b61c5))

- centralize JSON-RPC error construction in handlers (#705) ([9404242](https://github.com/d-o-hub/rust-self-learning-memory/commit/9404242ac51c87e3332ba7bac336f971d6005cbe))

- remove deprecated marker from handle_shutdown (ADR-060 Option 1) ([eef4d01](https://github.com/d-o-hub/rust-self-learning-memory/commit/eef4d012ddaaa0c694e9914b57c8f3f77cca2bcd))

- replace sleep-poll with bounded mpsc channel (#698) (#739) ([4218eaa](https://github.com/d-o-hub/rust-self-learning-memory/commit/4218eaaaa32b0de41e5b1364de48ef97ebf92d45))


### Documentation

- attribute v0.1.32 features under correct version header ([84bd3a7](https://github.com/d-o-hub/rust-self-learning-memory/commit/84bd3a71a04408a9e82fe0a7d3211ed1f64e87af))

- finalize FIX_CI_AND_RELEASE_STATE.md - all tasks complete (#585) ([9aee7e0](https://github.com/d-o-hub/rust-self-learning-memory/commit/9aee7e0467cc8f5dde1bcdfcc486c5db7d817ab1))

- update architecture, crate names, and feature docs (#607) ([f98ec58](https://github.com/d-o-hub/rust-self-learning-memory/commit/f98ec58d09a4ae0d101bf0eecfc52d1ddcdc9d04))

- Add remote repository analysis and maintenance documentation ([2edbca9](https://github.com/d-o-hub/rust-self-learning-memory/commit/2edbca92f44886772e64cd51d49ea52f8a58f778))

- add Codecov patch coverage gap workflow to ci-fix skill ([0777da4](https://github.com/d-o-hub/rust-self-learning-memory/commit/0777da4fc3a7b298ac0b6493dac30d218774aeb9))

- add CI remediation analysis, code changes spec, and ADR-057 ([0cf671e](https://github.com/d-o-hub/rust-self-learning-memory/commit/0cf671e9dbc019598a8a2d706aa39f210a80442a))

- update ROADMAP_ACTIVE and CURRENT to reflect v0.1.32 release status ([8139a90](https://github.com/d-o-hub/rust-self-learning-memory/commit/8139a903a796dcff6cc9adbaaa918c62efb50471))

- fix remaining stale v0.1.31 refs in roadmap and status ([d2d4013](https://github.com/d-o-hub/rust-self-learning-memory/commit/d2d40131b7377bbc003aaa3cf9b1ae77aa151162))

- Comprehensive update to README and documentation (#662) ([48f3b12](https://github.com/d-o-hub/rust-self-learning-memory/commit/48f3b12c1e73414147efb70488fd17a4900a36a6))

- update progress — WG-176..182 complete, dependency maintenance done (#687) ([a9fb3d7](https://github.com/d-o-hub/rust-self-learning-memory/commit/a9fb3d7321242f9ee01b0d5f7eb823d4475f4b38))

- add GOAP analysis 2026-06-30 and ADR-058/059 (#690) ([81ee08e](https://github.com/d-o-hub/rust-self-learning-memory/commit/81ee08e82f70c1db3434aeac5ebd306e922ec024))

- align documentation with current architecture and capabilities ([e3bec17](https://github.com/d-o-hub/rust-self-learning-memory/commit/e3bec17a37e54eeca85fe4a9e5fbc0e50b77d1ae))

- align documentation with current architecture and capabilities ([fa9f50b](https://github.com/d-o-hub/rust-self-learning-memory/commit/fa9f50b3f5de0041adf3ec80f55beb4365c00180))

- add missing abstention_score field to RewardScore doctest example (#727) ([2bfe09d](https://github.com/d-o-hub/rust-self-learning-memory/commit/2bfe09d6dddc352cb73c34a9b18331a459535c60))

- mark PR remediation campaign complete + fix stale metrics (#735) ([05119e4](https://github.com/d-o-hub/rust-self-learning-memory/commit/05119e40a80827d7e841004142557d6d9a90d6ff))

- clarify module scope and links to sibling modules (#742) ([17506dc](https://github.com/d-o-hub/rust-self-learning-memory/commit/17506dc6b457cdbe36dbfd6e113f9c11434a9a84))

- add usage guide and executable examples ([5f80682](https://github.com/d-o-hub/rust-self-learning-memory/commit/5f806827553b574c62843752e35b994a787d20a6))

- add usage guide and executable examples ([67d483b](https://github.com/d-o-hub/rust-self-learning-memory/commit/67d483b3b1b4c26dd37156197d10e2eb2dd476f9))

- add usage guide and executable examples ([5169ca1](https://github.com/d-o-hub/rust-self-learning-memory/commit/5169ca1605af7bdf932854a7e2cf0cf6d25be611))

- add usage guide and executable examples ([905dc92](https://github.com/d-o-hub/rust-self-learning-memory/commit/905dc92cd1fb41461a3f21c5537ee34f504d5b70))

- add build dependencies section (#788) ([4cf342d](https://github.com/d-o-hub/rust-self-learning-memory/commit/4cf342dbf7899f3ae18c8510e6cadc70642adf8a))

- update status for v0.1.34 sprint (#790) ([52830e1](https://github.com/d-o-hub/rust-self-learning-memory/commit/52830e1de7cefcb6363d6a84c8a7f3b21f85faad))

- add lessons #013-#014, update release/CI references (#791) ([da3e7fd](https://github.com/d-o-hub/rust-self-learning-memory/commit/da3e7fd2ef3032af3c0476ac55d24c76451cba6b))

- update CHANGELOG and STATUS for v0.1.34 ([152968a](https://github.com/d-o-hub/rust-self-learning-memory/commit/152968ad72a000f371369f7a8c6b010253db664d))

- enforce CI wait before merge, never use --admin ([dafa72c](https://github.com/d-o-hub/rust-self-learning-memory/commit/dafa72cbd2f469576b19bb2573323b2714e52beb))

- enforce ALWAYS use release.yml workflow, never bypass ([7bfd766](https://github.com/d-o-hub/rust-self-learning-memory/commit/7bfd7662d64bb156f048fb5a2eda88029111b19b))


### Fixed

- add base input to changelog workflow for tag-triggered runs ([870421d](https://github.com/d-o-hub/rust-self-learning-memory/commit/870421dd872567a819980d77f0a17b67ed5f2cef))

- transaction leak, IN clause chunking, coverage tests ([e405a77](https://github.com/d-o-hub/rust-self-learning-memory/commit/e405a779ab2ed59f2be4501cad7f8487bfc094a5))

- resolve formatting and clippy failures in Quick Check ([3adde56](https://github.com/d-o-hub/rust-self-learning-memory/commit/3adde568482c47d50be13d7c089b1abc3df780da))

- harden advanced analysis and metrics inputs (#609) ([47a8609](https://github.com/d-o-hub/rust-self-learning-memory/commit/47a8609c4200264711fdc66db1b9296f7ce3dcf9))

- apply rustfmt to monitoring types.rs ([16c4e09](https://github.com/d-o-hub/rust-self-learning-memory/commit/16c4e09f2ebccebad9f9e43954b7934d5a238b46))

- resolve pre-existing SIGSEGV test + storage.rs LOC violation ([3cd15d7](https://github.com/d-o-hub/rust-self-learning-memory/commit/3cd15d70515c289e8e64fb66ff6315008e694f07))

- bump ignored-test ceiling 160→165 to match documented limit ([8607bad](https://github.com/d-o-hub/rust-self-learning-memory/commit/8607bad6ed0c889cb30d535c57b77f46d32da6a3))

- increase MCP Build timeout 15→30min, remove duplicate codecov config ([21d26a0](https://github.com/d-o-hub/rust-self-learning-memory/commit/21d26a0b7057a988ebb39a85232f94e7d1ab20ad))

- resolve gitleaks false positives, bound slow test, close WG-156–162 ([d5a25dd](https://github.com/d-o-hub/rust-self-learning-memory/commit/d5a25dd1ff45cab1ca0cfd7e7e54a228d2836a61))

- fix doctest compilation and mark flaky turso test as ignored ([9d0cbee](https://github.com/d-o-hub/rust-self-learning-memory/commit/9d0cbee2fde0bf653a9d281c0ed8ea5c97b6997b))

- add input bounds to pattern search tools ([7d4cf76](https://github.com/d-o-hub/rust-self-learning-memory/commit/7d4cf76c7359e6d7caaa3dde62a6741c2747d4b3))

- add input bounds to pattern search tools ([3c85e96](https://github.com/d-o-hub/rust-self-learning-memory/commit/3c85e960896c9de800b761ab558a4c545ba2e85e))

- add input bounds to pattern search tools ([1cfbff8](https://github.com/d-o-hub/rust-self-learning-memory/commit/1cfbff821f82c358ea6e79eaceb5fa9b98dee919))

- add input bounds and tests for pattern search tools ([01c6259](https://github.com/d-o-hub/rust-self-learning-memory/commit/01c6259687d5888cadd8486560531a5fc8ffa2fb))

- add input bounds and tests for pattern search tools ([c9bdb48](https://github.com/d-o-hub/rust-self-learning-memory/commit/c9bdb482d8b9398f475112de54141890a0761a50))

- improve test coverage for truncate_safe ([c548d67](https://github.com/d-o-hub/rust-self-learning-memory/commit/c548d675d2bb8d776ff8019705db86702cbbcba8))

- add input bounds and full test coverage ([bfef921](https://github.com/d-o-hub/rust-self-learning-memory/commit/bfef92137913766d9c35714f84cbdc86b1490807))

- add input bounds and tests for pattern search tools ([498be7a](https://github.com/d-o-hub/rust-self-learning-memory/commit/498be7a53623152986a8ffbe82820efe6e23cc39))

- add input bounds to search patterns tool ([c4955f2](https://github.com/d-o-hub/rust-self-learning-memory/commit/c4955f20edc2974fafc0f5e876455bc8f25dfb24))

- add missing input bounds to public tool parameters (#665) ([ac3ad08](https://github.com/d-o-hub/rust-self-learning-memory/commit/ac3ad08200afa262d4c84faf1a7bee5ddd3c316c))

- use stable @v1 tags for Pages actions (#669) ([d828cc9](https://github.com/d-o-hub/rust-self-learning-memory/commit/d828cc90447e6cfcf11d1fa6cf55eb34c52560f7))

- resolve Codacy script injection in mutants.yml ([8c7961e](https://github.com/d-o-hub/rust-self-learning-memory/commit/8c7961ec3aca4991bf8cd829aff60ce0933c6db6))

- resolve Codacy script injection in mutants.yml ([19a1725](https://github.com/d-o-hub/rust-self-learning-memory/commit/19a172573fa267838fdd4d31d5b4573de2ca4453))

- WG-176..182 CI health + code quality (#675) ([cdfac7f](https://github.com/d-o-hub/rust-self-learning-memory/commit/cdfac7fecba0aa446a55aad8713d1fc4d828b59a))

- remove stale advisory ignores and update anyhow to 1.0.103 (#682) ([4c6cfec](https://github.com/d-o-hub/rust-self-learning-memory/commit/4c6cfecfaf9d7ca8af6e261a9374e688c2d02732))

- add allow(deprecated) for handle_shutdown in bin ([cf1c848](https://github.com/d-o-hub/rust-self-learning-memory/commit/cf1c84843b7890940012c8dfb1633cb48e2a6d95))

- unify sysinfo to 0.39 across workspace, dedupe Cargo.lock (#719) ([b9190ed](https://github.com/d-o-hub/rust-self-learning-memory/commit/b9190ed13382f18c3256c2951ceff4259e289373))

- harden sandbox against injection and prevent stack trace leakage ([3bcff11](https://github.com/d-o-hub/rust-self-learning-memory/commit/3bcff110a658e636f611aad9c9a5aa76f75d0146))

- harden sandbox against injection and prevent stack trace leakage ([653f78b](https://github.com/d-o-hub/rust-self-learning-memory/commit/653f78bd003081f4d023d01092f38f1530be14f0))

- update sandbox syntax test for newer JS runtime + wire up types_tests (#729) ([aaf1c6e](https://github.com/d-o-hub/rust-self-learning-memory/commit/aaf1c6e0a6729c97708873357ad9f4ca99651f86))

- allow clippy::too_many_arguments on update_incremental ([451f34c](https://github.com/d-o-hub/rust-self-learning-memory/commit/451f34c3509c10ab6df693b7eed4de43cc495d3e))

- make reconciliation cache backfill synchronous instead of fire-and-forget (#781) ([33003a3](https://github.com/d-o-hub/rust-self-learning-memory/commit/33003a303119e9f508de91aeec928497bbd3fa1d))

- add SQLite error to accepted patterns in CLI coverage tests (#782) ([9987779](https://github.com/d-o-hub/rust-self-learning-memory/commit/99877796ef705143180a05a0f410a2e1ebf0456a))

- ignore slow preset_configs_always_valid proptest (#783) ([2970de0](https://github.com/d-o-hub/rust-self-learning-memory/commit/2970de06d3a4e59c6a01fa61a0c25f2bd7e9fceb))

- persist episodes in local storage mode (#787) ([d589557](https://github.com/d-o-hub/rust-self-learning-memory/commit/d589557facebe688ba04acb53127db0cae9098c5))

- add file-level lint allows + build maintenance script + PR readiness tooling ([ed2c606](https://github.com/d-o-hub/rust-self-learning-memory/commit/ed2c606acfad061c261680055d97038c9342db8b))

- add CI-aware sample sizing to prevent 600s timeout ([4c77cd5](https://github.com/d-o-hub/rust-self-learning-memory/commit/4c77cd5d8c91fc2a1a92b6c33ea42a790cb9f033))

- use .values() instead of .iter() for clippy for-kv-map lint ([11d28fd](https://github.com/d-o-hub/rust-self-learning-memory/commit/11d28fd5fc95482ec1435718882a13994d3b7a8f))

- use assert_eq! instead of assert!(==) for clippy lint ([f6b57c4](https://github.com/d-o-hub/rust-self-learning-memory/commit/f6b57c49b9e689c2100c8f55ea0cafd5580e06a6))

- use assert_eq! for clippy manual_assert_eq lint ([51af30c](https://github.com/d-o-hub/rust-self-learning-memory/commit/51af30c3b5206f15ccc09564a951252077070918))

- replace assert!(a == b) with assert_eq! across codebase ([d7c8409](https://github.com/d-o-hub/rust-self-learning-memory/commit/d7c8409db0b490d44d96101ac4d96b7c90ebbaa6))

- excessive_nesting in retry tests, pool module naming, budget test assertion ([fa48c43](https://github.com/d-o-hub/rust-self-learning-memory/commit/fa48c438f7afe6e1939dc20d8eadb3f52600b1ae))

- eliminate sql injection in relationships module ([fb0e2d0](https://github.com/d-o-hub/rust-self-learning-memory/commit/fb0e2d07f64e58e0dee354d8d5d81a5aa8f0202c))

- resolve clippy warnings in relationships tests ([9fc4f71](https://github.com/d-o-hub/rust-self-learning-memory/commit/9fc4f7147bbbee32ce99c70dece554a8e79e1d75))

- broken intra-doc link in limiter.rs ([4dacd66](https://github.com/d-o-hub/rust-self-learning-memory/commit/4dacd66d6f5e3ea4eb0e20d099fdc0626ec9798b))

- serialize env var tests, fix nightly workflow config ([c03ca46](https://github.com/d-o-hub/rust-self-learning-memory/commit/c03ca461a7e24a358278ba7da34cd75092a5c2cc))


### Maintenance

- prepare v0.1.32 - CI infrastructure fixes (#584) ([7ecb135](https://github.com/d-o-hub/rust-self-learning-memory/commit/7ecb13597089881b2db880829856eacd68b8b85d))

- bump the rust-patch-minor group with 7 updates (#586) ([fb63b81](https://github.com/d-o-hub/rust-self-learning-memory/commit/fb63b81ff9fde950684d188e50b72b125c4af7c4))

- Add Antigravity CLI project context file ([6fe5cd2](https://github.com/d-o-hub/rust-self-learning-memory/commit/6fe5cd29b484b8598045beb3db4f7f652084c0e8))

- bump the rust-patch-minor group with 3 updates (#601) ([8f3fb2f](https://github.com/d-o-hub/rust-self-learning-memory/commit/8f3fb2fa439c9b2d0a2e2e0ae2c4457beb233219))

- bump sysinfo from 0.38.4 to 0.39.3 in the rust-major group ([e04f8fa](https://github.com/d-o-hub/rust-self-learning-memory/commit/e04f8fa42dbab6fa0bff23d7b6b344e8f756e562))

- bump sysinfo from 0.38.4 to 0.39.3 in the rust-major group (#614) ([b587746](https://github.com/d-o-hub/rust-self-learning-memory/commit/b5877463f9712b923006424f8b222b855e5926c4))

- bump the rust-patch-minor group with 4 updates (#613) ([28bcc51](https://github.com/d-o-hub/rust-self-learning-memory/commit/28bcc512843dbf19a51a26c2bf6233e1a7128cb5))

- bump the rust-patch-minor group with 3 updates (#629) ([6c05f45](https://github.com/d-o-hub/rust-self-learning-memory/commit/6c05f45c274c1f77cee551aa20737efd2fe823b7))

- resolve release drift for v0.1.33 (#632) ([1395e53](https://github.com/d-o-hub/rust-self-learning-memory/commit/1395e536b96ed0712418cc675c11a506fe0a4339))

- resolve release drift for v0.1.33 (#635) ([b820349](https://github.com/d-o-hub/rust-self-learning-memory/commit/b820349f589d307fb0a5108d5d9a0a84d6fbc6e0))

- resolve release drift for v0.1.33 ([860811e](https://github.com/d-o-hub/rust-self-learning-memory/commit/860811e65a16e1296079cdaca2992127d99f0b0a))

- bump sysinfo from 0.38.4 to 0.39.5 in the rust-major group ([ef8663a](https://github.com/d-o-hub/rust-self-learning-memory/commit/ef8663aa760712aba8addc3e52029e9e6c6972e4))

- Enforce no scripts in root and remove one-off Python files ([03cba95](https://github.com/d-o-hub/rust-self-learning-memory/commit/03cba95af971aceed235de505891f21af64e3d1d))

- remove one-off .py scripts from root ([850bf69](https://github.com/d-o-hub/rust-self-learning-memory/commit/850bf69d15c4ca960c299c5170f61bc5c8e58d9c))

- bump the rust-patch-minor group across 1 directory with 2 updates (#684) ([8b2e81d](https://github.com/d-o-hub/rust-self-learning-memory/commit/8b2e81d63d577ac1171892fa81dafd1464d6f678))

- bump sysinfo in the rust-major group across 1 directory (#678) ([4459bb8](https://github.com/d-o-hub/rust-self-learning-memory/commit/4459bb8477c4d19ac6b002734eacb43d9cc164e6))

- remove .trigger-ci.txt and add workflow_dispatch ([7ce1f55](https://github.com/d-o-hub/rust-self-learning-memory/commit/7ce1f556d864167b215cbbd7833a5ec8c4803d30))

- bump the rust-patch-minor group with 3 updates (#759) ([ff4dda3](https://github.com/d-o-hub/rust-self-learning-memory/commit/ff4dda345270398dfb304a9914e846084ad5c8ea))

- bump the rust-major group with 2 updates (#760) ([9c04f7a](https://github.com/d-o-hub/rust-self-learning-memory/commit/9c04f7a51c63d0f2f7ccd1d6447a8b75adff78f8))

- update CHANGELOG for v0.1.33 release (#778) ([c5d2de9](https://github.com/d-o-hub/rust-self-learning-memory/commit/c5d2de9ad87313b66931d96ed4120d5ec97aff3e))

- bump to v0.1.34 ([0e17de8](https://github.com/d-o-hub/rust-self-learning-memory/commit/0e17de897ba6745dc7568f6bb33a24a215da0fce))


### Performance

- optimize cosine similarity calculation (#606) ([1f1ca84](https://github.com/d-o-hub/rust-self-learning-memory/commit/1f1ca8417010eb944d3572696f803aa217f12b47))

- optimize cosine similarity with SIMD-friendly accumulators and reduced sqrt calls ([11d1d1e](https://github.com/d-o-hub/rust-self-learning-memory/commit/11d1d1ed215bee6c02ee777c45211beb2ac3d4c5))

- optimize cosine similarity and fix flaky storage tests ([0a35e93](https://github.com/d-o-hub/rust-self-learning-memory/commit/0a35e9343a33a9df8ac10213d74183bb9a6eaa85))

- optimize cosine similarity and fix mcp build/test flakiness ([2a7dd7c](https://github.com/d-o-hub/rust-self-learning-memory/commit/2a7dd7cb7148c8bd930c044937cc2c15536060d1))

- optimize cosine similarity calculation ([93c5fad](https://github.com/d-o-hub/rust-self-learning-memory/commit/93c5fadba73b29f7e3f369acc4dc749907204698))

- optimize cosine similarity and stabilize storage tests ([ce16897](https://github.com/d-o-hub/rust-self-learning-memory/commit/ce168970511bd65594081ff4ed8c97db0230958b))

- optimize cosine similarity and stabilize storage tests ([e0fff78](https://github.com/d-o-hub/rust-self-learning-memory/commit/e0fff7862860dbf63bae8a7e2bd54c058839e1cd))

- optimize cosine similarity and stabilize mcp tests ([2e38c60](https://github.com/d-o-hub/rust-self-learning-memory/commit/2e38c60bbb654083de411a1a3e0325b8ebd90ad0))

- optimize cosine similarity with SIMD autovectorization ([f4223ad](https://github.com/d-o-hub/rust-self-learning-memory/commit/f4223ad88c0fda1587c24372eeff99063cb01a86))

- optimize cosine similarity with SIMD autovectorization ([07af5a8](https://github.com/d-o-hub/rust-self-learning-memory/commit/07af5a893c157c168c6aab3fa79d74156f192740))

- optimize cosine similarity with SIMD autovectorization ([0e90364](https://github.com/d-o-hub/rust-self-learning-memory/commit/0e90364fa8b21ef5e25e10b020fe67bd528045c4))

- optimize pattern ranking by replacing HashSet with Vec scans (#636) ([ea355b0](https://github.com/d-o-hub/rust-self-learning-memory/commit/ea355b017ec58cbe4f6a8b49892ec10da85ae62c))

- optimize edit distance space complexity (#659) ([14173bb](https://github.com/d-o-hub/rust-self-learning-memory/commit/14173bbd5cf3acf6dac328212a63f664cae2211d))

- optimize edit distance space complexity (#710) ([51327f6](https://github.com/d-o-hub/rust-self-learning-memory/commit/51327f6e04f2defc55dcea0236f54bd7427ce9ba))

- optimize edit distance space complexity ([3a0610c](https://github.com/d-o-hub/rust-self-learning-memory/commit/3a0610c32b8d087d0f4009ff99a18e89d0520b0f))

- optimize edit distance row transitions with swap ([c2e226f](https://github.com/d-o-hub/rust-self-learning-memory/commit/c2e226fa75ed464f5ae3eee3520426ae0d00e8c2))

- optimize edit distance and stabilize CI workflows ([2379711](https://github.com/d-o-hub/rust-self-learning-memory/commit/2379711940e6851f67a8bc11e11d1e0d03268aaf))


### Testing

- add relationship core and validation coverage ([330c11d](https://github.com/d-o-hub/rust-self-learning-memory/commit/330c11d0e6ce0f5a90c81febb67b900f9bbe7793))

- add relationship coverage and fix formatting ([ca3b491](https://github.com/d-o-hub/rust-self-learning-memory/commit/ca3b491e254b224bf3768b59d96a38168e6933c9))

- cover relationships core and validation command paths ([065a0af](https://github.com/d-o-hub/rust-self-learning-memory/commit/065a0afaa4797de994f0896597ab70d1497dbdf7))

- cover relationships core and validation command paths ([198f6c1](https://github.com/d-o-hub/rust-self-learning-memory/commit/198f6c1305db325862ac9630c858f0b320ab2620))

- add coverage for local/in-memory constructors, storage initializers, and memory mod (#615) ([957efdd](https://github.com/d-o-hub/rust-self-learning-memory/commit/957efddd5590b765bf48c4e149577553915fb17d))

- add tests for Codecov patch gaps in 6 files ([086186f](https://github.com/d-o-hub/rust-self-learning-memory/commit/086186f3c400133ae3c61cd8eb0d8f62a6207ba5))

- fix memory leak false positive in performance tests ([97c17ac](https://github.com/d-o-hub/rust-self-learning-memory/commit/97c17ac4307596c90f640eb9119840bbcbd877fe))

- add end-to-end shutdown json-rpc integration test (ADR-060) ([7098d46](https://github.com/d-o-hub/rust-self-learning-memory/commit/7098d465336cc2ee95751f3f53b22da423becfff))

- add arbitrary support for core types and align dependencies ([3882ed2](https://github.com/d-o-hub/rust-self-learning-memory/commit/3882ed2e3215b061dcd83dbdb95378bde3fc5468))

- add end-to-end tests for Turso + redb hybrid consistency and recovery (#769) ([7468fa6](https://github.com/d-o-hub/rust-self-learning-memory/commit/7468fa6e17e28ff9a124858c59f7f76c07775485))

- expand coverage for relationships module ([1ead1f0](https://github.com/d-o-hub/rust-self-learning-memory/commit/1ead1f081ff56e3a1b1f5c11cdc3a12856e3c6b2))


### security

- add #![deny(unsafe_code)] to 4 crates (#701) (#740) ([7fa8baf](https://github.com/d-o-hub/rust-self-learning-memory/commit/7fa8baf3fb98ee4e1995886a5e536c4e59183f47))


## [0.1.32] - 2026-05-24



### Added

- implement WG-120, WG-121, WG-122 research-inspired retrieval (#505) ([ffe9fcc](https://github.com/d-o-hub/rust-self-learning-memory/commit/ffe9fcc4b587e029a95f28c931c1ecf54c4653f1))

- CloudEvents EventEmitter, ConceptGraph ontology, planning evaluations (WG-131/132/133/149) (#544) ([12b459a](https://github.com/d-o-hub/rust-self-learning-memory/commit/12b459a6af25817c629882aa92326e4e67874c6d))

- implement SIMD-accelerated cosine similarity (#565) ([8befbe4](https://github.com/d-o-hub/rust-self-learning-memory/commit/8befbe43c0b2f2603064bc52da74423bfd1b31f6))

- implement agentfs-sdk integration ([d9b64d6](https://github.com/d-o-hub/rust-self-learning-memory/commit/d9b64d62781fde1004880ac3567d96d0e21e2d7d))

- implement agentfs-sdk integration ([f02caf9](https://github.com/d-o-hub/rust-self-learning-memory/commit/f02caf96caf8b3cfd8cb9d3d31bd5f7c039c261b))

- implement agentfs-sdk integration ([4831a6d](https://github.com/d-o-hub/rust-self-learning-memory/commit/4831a6dc27df60aaa76dd7c1d444004034e0f026))

- implement ProceduralMemory type (WG-124) ([da1b453](https://github.com/d-o-hub/rust-self-learning-memory/commit/da1b453bd7103fd46283f373b779eed452b067be))


### CI/CD

- bump benchmark-action/github-action-benchmark (#535) ([8fb74ab](https://github.com/d-o-hub/rust-self-learning-memory/commit/8fb74ab8fb46e068020aa19ad6f09d730e1df047))

- bump peter-evans/create-pull-request in the actions-all group (#574) ([2ec00cd](https://github.com/d-o-hub/rust-self-learning-memory/commit/2ec00cd537a235b61783117df09b1de5ecc4280a))


### Changed

- extract tests from oversized source files ([a70d17d](https://github.com/d-o-hub/rust-self-learning-memory/commit/a70d17de799ab13f904c132eb206f2292a62d59f))


### Documentation

- update release docs for v0.1.31 ([cf711d8](https://github.com/d-o-hub/rust-self-learning-memory/commit/cf711d83198603a2279a8d45a18ca8a53605954e))

- update GOAP_STATE.md and CURRENT.md for v0.1.31 audit ([a78f7b5](https://github.com/d-o-hub/rust-self-learning-memory/commit/a78f7b5bdb24b60b50cead763992ee742c38f5cd))

- update CI optimization documentation and lessons (#492) ([b485c67](https://github.com/d-o-hub/rust-self-learning-memory/commit/b485c67d5f4c85d07c5db99497c1d60e5c0ba902))

- update STATUS and GOAP_STATE with security/CI work (#497) ([b508d41](https://github.com/d-o-hub/rust-self-learning-memory/commit/b508d411ec4be17f1e4138075a53a59d2f1f5ab7))

- align coverage targets with ADR-042 phased approach ([b2cc3e6](https://github.com/d-o-hub/rust-self-learning-memory/commit/b2cc3e64c45dabf389faa609a1bbaaa9b3d55b6c))

- audit and update readme and docs for accuracy (#498) ([515c342](https://github.com/d-o-hub/rust-self-learning-memory/commit/515c342eaff72c5fe75336d9c0f4ddac535e2bf9))

- add LESSON-007 for cosine_similarity normalization ([9e2596e](https://github.com/d-o-hub/rust-self-learning-memory/commit/9e2596e58b593a877e91c86900061e63a22b828e))

- audit and synchronize docs with technical state ([2f30221](https://github.com/d-o-hub/rust-self-learning-memory/commit/2f302217e1dfd52c3a1a4daae464fd82c6c19f92))

- add coverage plan and GOAP analysis status documents ([c2505a5](https://github.com/d-o-hub/rust-self-learning-memory/commit/c2505a5590db678a22dc721487735ba0103c3016))

- verify WG-115 and WG-116 complete ([3918594](https://github.com/d-o-hub/rust-self-learning-memory/commit/39185947371209e756a5a8d9d49f07b835eaf4db))

- verify WG-115 and WG-116 complete ([2fb0cd4](https://github.com/d-o-hub/rust-self-learning-memory/commit/2fb0cd42fb89c4f71275fb79e02d73ef6f752d16))

- sync WG-119 status to Complete ([857c4ca](https://github.com/d-o-hub/rust-self-learning-memory/commit/857c4ca7e04f9fe6a7e55a46fd3765c9a1f669b2))

- clarify CSM feature flag and Bash:Grep ratio ([418df07](https://github.com/d-o-hub/rust-self-learning-memory/commit/418df0702056b24a31890d0081c90f1923285dc8))

- mark WG-109 complete and reduce clippy suppressions (#506) ([d972021](https://github.com/d-o-hub/rust-self-learning-memory/commit/d9720211cdaaaf31847979101a3de119c9280ee3))

- Synchronize README.md and documentation directories with implementation (#526) ([bbc25eb](https://github.com/d-o-hub/rust-self-learning-memory/commit/bbc25eb95db82bbe362ae3033a61ef4ab0273220))

- align architecture, diagrams and features with current state (#540) ([0c7513f](https://github.com/d-o-hub/rust-self-learning-memory/commit/0c7513f7c53921b02a454edd44c3c0407ec615be))

- sync CURRENT.md and GOAP_STATE.md to 2026-05-16 ([53070e2](https://github.com/d-o-hub/rust-self-learning-memory/commit/53070e2aab3fcbb1eec4ca108f6a3479bcb8c3ba))

- add ADR-055 evaluating Routing-Free MoE for DyMoE replacement (#563) ([159c490](https://github.com/d-o-hub/rust-self-learning-memory/commit/159c4903f9a24055dbd783808117a36010bb604f))

- generate impact analysis for MAP paradigm (arXiv:2605.13037) (#573) ([ea57d0d](https://github.com/d-o-hub/rust-self-learning-memory/commit/ea57d0d7c523db8cb821a696561da6a805d6890d))

- mark WG-123/126 Complete, WG-124 In Review, split schema files ([f0f3688](https://github.com/d-o-hub/rust-self-learning-memory/commit/f0f36882b30b6d3f349bf44a8aea24221ed8e53a))

- mark WG-124 Complete after PR #569 merge ([57f0252](https://github.com/d-o-hub/rust-self-learning-memory/commit/57f0252195fb3a4f1f66a02c634919e08bab064e))

- update README and docs to reflect current capabilities, remove marketing fluff (#581) ([1380fe4](https://github.com/d-o-hub/rust-self-learning-memory/commit/1380fe44dfb7c5afd8c3cecb7495175932398224))


### Fixed

- resolve sql injection and implement parameter bounds ([a6ab979](https://github.com/d-o-hub/rust-self-learning-memory/commit/a6ab97947c01ce71f6905aff58f9ee7baff4f944))

- resolve sql injection and implement parameter bounds ([3c14129](https://github.com/d-o-hub/rust-self-learning-memory/commit/3c14129a7807c2ef85ff0af93c58aed26fad9892))

- resolve sql injection and implement parameter bounds ([6d56644](https://github.com/d-o-hub/rust-self-learning-memory/commit/6d56644dafdc33cc0e91d9c9f23f83f9c8e171db))

- resolve sql injection and implement parameter bounds ([49c15bf](https://github.com/d-o-hub/rust-self-learning-memory/commit/49c15bf17540c9a54042e535945350f2944b97e7))

- resolve sql injection and implement parameter bounds ([2c1b760](https://github.com/d-o-hub/rust-self-learning-memory/commit/2c1b76025afd654c63c36cf8aca2bf6e17a94b64))

- resolve sql injection and implement parameter bounds ([45a5ab7](https://github.com/d-o-hub/rust-self-learning-memory/commit/45a5ab7c1e5a1692c36ec2b0df6eaf9ddcb2f8b9))

- make ExecutionStep parameters postcard-compatible ([9ce831d](https://github.com/d-o-hub/rust-self-learning-memory/commit/9ce831d950942373b8c9e28c4ec7ef7a8f3f0c70))

- align thresholds with ADR-042 Phase 1 target ([2c7cbff](https://github.com/d-o-hub/rust-self-learning-memory/commit/2c7cbff76a09cce40eef4e3be5fcabbceb9e7917))

- suppress unused warnings in compression tests ([f282f39](https://github.com/d-o-hub/rust-self-learning-memory/commit/f282f390e01421a8a83a4a70ea2550841c195153))

- correct cosine_similarity assertions and add ADR-027 ignores ([c2d3b3b](https://github.com/d-o-hub/rust-self-learning-memory/commit/c2d3b3b4ed79726a927eae2a20ff8fb044b4199b))

- enforce jwt signature verification (#504) ([bc7e24f](https://github.com/d-o-hub/rust-self-learning-memory/commit/bc7e24fc977834056dcec7db79e45863e1d603e2))

- add backticks to identifiers in doc comments ([dea3270](https://github.com/d-o-hub/rust-self-learning-memory/commit/dea32703a208729c83b666f025796805907e95ca))

- add allow attributes to redb test files ([04168b3](https://github.com/d-o-hub/rust-self-learning-memory/commit/04168b3680d31d4e28ecc84e0eb1889cc0699040))

- add allow attributes to all redb test files ([ce5e703](https://github.com/d-o-hub/rust-self-learning-memory/commit/ce5e7038dbc70326054176506c2cde02974f8cc1))

- add unreadable_literal allow to cache_integration_test ([b930372](https://github.com/d-o-hub/rust-self-learning-memory/commit/b9303729ef0357d4ac45dcc1a6e1dda05c0aa428))

- add panic allow to cli_workflow_test ([991ec13](https://github.com/d-o-hub/rust-self-learning-memory/commit/991ec131ab88aa432ee1385b11c8f40e6e963fb7))

- remove duplicated attribute and add unreadable_literal allow ([9c22ea4](https://github.com/d-o-hub/rust-self-learning-memory/commit/9c22ea47bf9de551b498e2bbf7e377294ba44bca))

- add cast_possible_truncation allow to serialization tests ([f9d0944](https://github.com/d-o-hub/rust-self-learning-memory/commit/f9d0944b6f042b361b71e17e64544b83923ad1c9))

- Add clippy allow attributes to postcard_security_test ([dbb2fdc](https://github.com/d-o-hub/rust-self-learning-memory/commit/dbb2fdc92144e702d77d79eba04fb31a33e5f2cc))

- Remove duplicated expect_used attribute in runtime_wiring test ([b9f763c](https://github.com/d-o-hub/rust-self-learning-memory/commit/b9f763cf6f02646fc527210e2a2b22e05706887d))

- Add crate doc comment for playbook_property_tests ([00de476](https://github.com/d-o-hub/rust-self-learning-memory/commit/00de47641f35a7fb7995306f8fe850f45df49f9e))

- Add clippy allow attributes for test-specific patterns ([4c1d794](https://github.com/d-o-hub/rust-self-learning-memory/commit/4c1d794677049dd47de10f92634136306ae40681))

- Add panic allow attribute for execution_step_property_tests ([cf11e65](https://github.com/d-o-hub/rust-self-learning-memory/commit/cf11e65ed6c29711b2c38207d7272dd1a3122902))

- Add clippy allow attributes for transport compression tests ([689b447](https://github.com/d-o-hub/rust-self-learning-memory/commit/689b44703f18e944731bb39244ed1aea21ef8f04))

- Use Display formatting for PathBuf in println ([6ac94db](https://github.com/d-o-hub/rust-self-learning-memory/commit/6ac94db9ea2bea06adeb96af51ab37ead5d243ab))

- Use String::new() instead of "".to_string() ([9f71c73](https://github.com/d-o-hub/rust-self-learning-memory/commit/9f71c7302d7784bb83b4842404db0e91671b00fd))

- Replace redundant closure with method reference ([dc8779b](https://github.com/d-o-hub/rust-self-learning-memory/commit/dc8779b0e62cd3a7f8f79984843d8dd8f69d0ae1))

- Add clippy allow attributes for migration example ([edd33eb](https://github.com/d-o-hub/rust-self-learning-memory/commit/edd33eb5ba424c08da950cad57e2e67a6a55645e))

- add float_cmp and cast_precision_loss allows to test file ([40f5354](https://github.com/d-o-hub/rust-self-learning-memory/commit/40f535438fd78e6c488f94c739fc4efc0016f538))

- add doc_markdown allow to serialization_property_tests.rs ([96b63a0](https://github.com/d-o-hub/rust-self-learning-memory/commit/96b63a04196109cec4cda45bb498f1ac4034561b))

- add panic allow to premem_integration_test.rs ([8283c1f](https://github.com/d-o-hub/rust-self-learning-memory/commit/8283c1f411775e0dcdcf47c4d9dcdefe3c2ef60b))

- add doc_markdown and unwrap_used allows to integration_test.rs ([d5ee242](https://github.com/d-o-hub/rust-self-learning-memory/commit/d5ee2427adae9bbbfa1c2fecb2a63be2cd476027))

- add crate-level and module-level allow attributes ([7410a30](https://github.com/d-o-hub/rust-self-learning-memory/commit/7410a30c8492d0473780645c1818d8a1808a5058))

- add more allow attributes to benchmarks, test-utils, examples ([641abcc](https://github.com/d-o-hub/rust-self-learning-memory/commit/641abcc200475acc7d27032f10b456da7b496a4b))

- add comprehensive lint suppressions for CI quality gates ([9e578c7](https://github.com/d-o-hub/rust-self-learning-memory/commit/9e578c7c9804132213a13dbe9112696613c51158))

- use Duration::from_secs instead of from_millis for readability ([a7c1cf9](https://github.com/d-o-hub/rust-self-learning-memory/commit/a7c1cf9dde9b2192d69a6d5acb5eb6d90b7494a4))

- resolve lint issues and skip CI-specific test ([79610fc](https://github.com/d-o-hub/rust-self-learning-memory/commit/79610fc147b780d3999b3d954f94d88277bd60a2))

- fix doc test and expand yamllint CI to entire repo (#548) ([e6bbfc4](https://github.com/d-o-hub/rust-self-learning-memory/commit/e6bbfc449723ef0d3bb43c63ad70608f5f861111))

- enforce input bounds clamping on all public tool parameters (CWE-770) (#546) ([a641a29](https://github.com/d-o-hub/rust-self-learning-memory/commit/a641a29751afbb7c7e3ed83b8ecad6bf645928f7))

- address all coderabbitai review comments on PR #546 (post-merge) ([93d4a3b](https://github.com/d-o-hub/rust-self-learning-memory/commit/93d4a3b853c685eed336b206fff2bf840a5b3aca))

- address feedback for PR #564 and fix clippy issues ([bfc66dc](https://github.com/d-o-hub/rust-self-learning-memory/commit/bfc66dca05ef5e491aed23301de85ff3ce08f3ed))

- correct record() call signature in test ([b2b868e](https://github.com/d-o-hub/rust-self-learning-memory/commit/b2b868eceb29d2552ce66ef521076eaca7ca8579))

- derive AgentFS status from real config and fix connection test ([55fc186](https://github.com/d-o-hub/rust-self-learning-memory/commit/55fc186903b8035b0f463b6a8fd34e54afe11519))

- ignore 3 crud tests affected by upstream libsql SIGSEGV (ADR-027) ([b34063f](https://github.com/d-o-hub/rust-self-learning-memory/commit/b34063f92e437a60c90ea4ae9b9c7098bd3e76fa))

- resolve all pre-existing clippy warnings and split oversized api.rs (#579) ([32cad33](https://github.com/d-o-hub/rust-self-learning-memory/commit/32cad334c0774034873116ae9864f80edbcca773))

- prevent resource exhaustion via input bounds (#583) ([573dcfc](https://github.com/d-o-hub/rust-self-learning-memory/commit/573dcfcd89577179a639e4b08772099cc1d03554))


### Maintenance

- bump openssl from 0.10.76 to 0.10.78 ([f52c6fb](https://github.com/d-o-hub/rust-self-learning-memory/commit/f52c6fb88ba09a5380d675887894625b4de85ebb))

- bump openssl from 0.10.76 to 0.10.78 ([cb1f831](https://github.com/d-o-hub/rust-self-learning-memory/commit/cb1f831df0f62d971843725832b9d968cc77f99c))

- bump openssl from 0.10.76 to 0.10.78 ([313790a](https://github.com/d-o-hub/rust-self-learning-memory/commit/313790a00dcc9ae5a41f75d2722a3e6863ae9ca4))

- bump openssl from 0.10.76 to 0.10.78 ([6d49722](https://github.com/d-o-hub/rust-self-learning-memory/commit/6d4972238519ecdb24ea2a35a6bc9983dc344c6e))

- bump lru from 0.16.3 to 0.17.0 in the rust-major group ([2ed4476](https://github.com/d-o-hub/rust-self-learning-memory/commit/2ed4476e8def76699d11f54de778fe588c4d71e8))

- bump the rust-patch-minor group across 1 directory with 9 updates ([76baf0d](https://github.com/d-o-hub/rust-self-learning-memory/commit/76baf0d6f507ebf97854b060f9e67525f71c4473))

- bump lru from 0.17.0 to 0.18.0 (#495) ([d20400b](https://github.com/d-o-hub/rust-self-learning-memory/commit/d20400b98b71e738d3949c13d87baa591cbdcaf5))

- bump openssl from 0.10.76 to 0.10.78 ([322fbdd](https://github.com/d-o-hub/rust-self-learning-memory/commit/322fbddbffbf45c197c25dca7d44d66949e39671))

- bump jsonwebtoken from 9.3.1 to 10.3.0 ([8a443a8](https://github.com/d-o-hub/rust-self-learning-memory/commit/8a443a8eb594856cd7c21d7b8ce684b0c4571e66))

- raise ignored test ceiling for coverage sprint ([de1a24e](https://github.com/d-o-hub/rust-self-learning-memory/commit/de1a24e540fb233648c0813d7dc31f604cd3e9ed))

- add comprehensive lint suppressions across workspace ([b7af99e](https://github.com/d-o-hub/rust-self-learning-memory/commit/b7af99e7c808033cdc5fe3ba950f44ea91e538fb))

- add workspace lint configuration and OAuth improvements ([1f9f871](https://github.com/d-o-hub/rust-self-learning-memory/commit/1f9f87152feefa04403d446957d0144f75d4df67))

- bump tokenizers in the rust-major group (#513) ([7f3aeb2](https://github.com/d-o-hub/rust-self-learning-memory/commit/7f3aeb20067658c0a437ad389a2a8e7697f1c1d6))

- bump the rust-patch-minor group across 1 directory with 2 updates (#514) ([583f434](https://github.com/d-o-hub/rust-self-learning-memory/commit/583f4344255f56b2a0e2c4c5333f72df660313a7))

- bump openssl from 0.10.78 to 0.10.79 (#525) ([bebb650](https://github.com/d-o-hub/rust-self-learning-memory/commit/bebb65038822074ffed875168b948c4b37adef1b))

- bump the rust-major group with 2 updates (#537) ([2864f9c](https://github.com/d-o-hub/rust-self-learning-memory/commit/2864f9cac700afee9808180b0c37d10a2b57d77a))

- bump the rust-patch-minor group across 1 directory with 5 updates (#538) ([9322da9](https://github.com/d-o-hub/rust-self-learning-memory/commit/9322da981a06c5d05e69fdd461151af55281c5b6))

- bump openssl from 0.10.78 to 0.10.79 (#541) ([a2b288d](https://github.com/d-o-hub/rust-self-learning-memory/commit/a2b288d990a26aebf166a4eb7574ddb8a0671081))

- YAML frontmatter validation and Dependabot fix plan (#545) ([e687ded](https://github.com/d-o-hub/rust-self-learning-memory/commit/e687ded79c7c97545d454b5a02b7c061ffc31fb1))

- resolve merge conflicts with main; use create-pull-request for automated commits ([7db5cc4](https://github.com/d-o-hub/rust-self-learning-memory/commit/7db5cc46e323237180d592ff0077a653194aa6e5))

- bump sysinfo in the rust-patch-minor group (#575) ([bc6d06a](https://github.com/d-o-hub/rust-self-learning-memory/commit/bc6d06a5560007db37adde1d31f3be13deec9bed))

- bump openssl from 0.10.79 to 0.10.80 (#582) ([f0ab6a4](https://github.com/d-o-hub/rust-self-learning-memory/commit/f0ab6a440a9ca9bc265419f39aeada7aa9f67bb4))

- prepare v0.1.32 - CI infrastructure fixes ([e4bcda6](https://github.com/d-o-hub/rust-self-learning-memory/commit/e4bcda6061ee3bc50f865c2f828c625e6d246c63))


### Performance

- address PR feedback and improve coverage with more tests ([e946941](https://github.com/d-o-hub/rust-self-learning-memory/commit/e946941b7dff5b3e6fd3f5e0c2cd6f77f75e320d))

- finalize async I/O in storage and fix Rust 2024 compliance ([f406229](https://github.com/d-o-hub/rust-self-learning-memory/commit/f4062293108965d6509c32e70f796ed085ac9bc1))

- refactor storage initialization for robustness and fix tests ([c6ce238](https://github.com/d-o-hub/rust-self-learning-memory/commit/c6ce23888aa6c61e85d794cfad0f4f71922ba95c))

- use async create_dir_all in async contexts ([06d2879](https://github.com/d-o-hub/rust-self-learning-memory/commit/06d2879c1ac440d16359cd3daeeb334fb3423683))

- use async create_dir_all in async contexts ([d932b19](https://github.com/d-o-hub/rust-self-learning-memory/commit/d932b1972d6742c39a758c20eb73af78eb4738ec))

- use async tokio fs metadata in verify_storage example ([36a0603](https://github.com/d-o-hub/rust-self-learning-memory/commit/36a0603a4b1bc72dfc8df680aa28682c7faab3c1))

- optimize legacy retrieval path with Schwartzian Transform ([4b1d564](https://github.com/d-o-hub/rust-self-learning-memory/commit/4b1d564407dd2d0eb54b0a48c1aa322bbd9583f7))

- finalize retrieval optimization with unit tests and allocation reductions ([85229c5](https://github.com/d-o-hub/rust-self-learning-memory/commit/85229c5d1a5c2d7ccc0594076e2d24b1e70bf8db))

- optimize legacy retrieval path and add unit tests ([8558298](https://github.com/d-o-hub/rust-self-learning-memory/commit/85582980b162b68265b45a55ac6d0a411bf8a090))

- batch capacity eviction and improved coverage (#516) ([49f7885](https://github.com/d-o-hub/rust-self-learning-memory/commit/49f78858cc2ec72ccf87a50bf8976176f9cd43e8))

- optimize similarity search with top-k selection (#539) ([3d30262](https://github.com/d-o-hub/rust-self-learning-memory/commit/3d302623fab01b511791a117bfc482dbcdec562c))

- reduce dead_code in production source (38→24, 14 removals) ([5724b80](https://github.com/d-o-hub/rust-self-learning-memory/commit/5724b80b7d6227ef9145f732cfc271896bdb82d1))

- eliminate all dead_code warnings and address coderabbitai review (PR #552) ([419ce27](https://github.com/d-o-hub/rust-self-learning-memory/commit/419ce27bd131162226a7dcd10b19b0e894cd2b06))


### Testing

- add comprehensive tests for CompressedPayload ([bd2b781](https://github.com/d-o-hub/rust-self-learning-memory/commit/bd2b781f4af39dbaaeacfa98bdf1d7b987b4f5da))

- add comprehensive tests for CompressedPayload ([35c108b](https://github.com/d-o-hub/rust-self-learning-memory/commit/35c108b19596a81ff94a0a567f3688244b35a7cd))

- add comprehensive tests for CompressedPayload ([374bd20](https://github.com/d-o-hub/rust-self-learning-memory/commit/374bd20c7a5e7ffc9fa2d92bd05a38cf90f921b4))

- add comprehensive tests for 0% coverage modules ([3defb09](https://github.com/d-o-hub/rust-self-learning-memory/commit/3defb0904bb651925428a844d7f17c76939faedb))

- add coverage tests for codecov patch compliance ([a618e3c](https://github.com/d-o-hub/rust-self-learning-memory/commit/a618e3c140ec08c0946fb0a13482c323e1aeb37d))


### WG-126

- MemCollab Implementation (#572) ([c62d709](https://github.com/d-o-hub/rust-self-learning-memory/commit/c62d709ccc2c367b5da2afb8d791f056b0a3d354))


### WG-127

- Semantic Gist Extraction + CogniRank Reranking (#568) ([0b49c85](https://github.com/d-o-hub/rust-self-learning-memory/commit/0b49c8521237231298f02e2c68e626096aa83e45))


## [0.1.31] - 2026-04-22



### Added

- CPU/token efficiency sprint implementation (#458) ([b8564af](https://github.com/d-o-hub/rust-self-learning-memory/commit/b8564af4a64303b048ed4c8ef65c9c14f14dae62))

- add CSM cascading retrieval integration (WG-128/129/130/131) ([3134755](https://github.com/d-o-hub/rust-self-learning-memory/commit/3134755966c3ebc31e23f209181795847f96f3a1))

- add stale_threshold to RateLimitConfig ([b0a36cd](https://github.com/d-o-hub/rust-self-learning-memory/commit/b0a36cdf11afc190c1bd9e9cf871a3c717de4c78))

- add embeddings_multi module for batch retrieval ([2b66284](https://github.com/d-o-hub/rust-self-learning-memory/commit/2b66284845995f3a6ae19303410f6a8e0053727c))

- add hierarchical/gist reranking (WG-118) ([7d4eabe](https://github.com/d-o-hub/rust-self-learning-memory/commit/7d4eabe6cb8821661995069801655cc994bc8b8c))

- export gist types and add WG-118 module ([35d325b](https://github.com/d-o-hub/rust-self-learning-memory/commit/35d325bd3d1fd538d116af6a3aa11ca8335f1cde))


### CI/CD

- bump actions/github-script in the actions-all group (#445) ([c256e55](https://github.com/d-o-hub/rust-self-learning-memory/commit/c256e556a7cdc1f827845be86ac1d871a49008b8))

- bump the actions-all group with 2 updates ([0cfce22](https://github.com/d-o-hub/rust-self-learning-memory/commit/0cfce22873eee706d3b8527e6ad3594b655864e8))


### Changed

- v0.1.31 skills consolidation (ADR-053 Phase 1) (#452) ([9c31639](https://github.com/d-o-hub/rust-self-learning-memory/commit/9c31639fee86d0bf7a2042dcbdab4b949bfc6d5d))

- skills cleanup and clippy suppression audit (WG-113, WG-130) (#456) ([9331aa2](https://github.com/d-o-hub/rust-self-learning-memory/commit/9331aa22fbb51e812066e2c7712e2c36812bcae7))

- consolidate skills from 36→31 (WG-137) (#460) ([d3754a0](https://github.com/d-o-hub/rust-self-learning-memory/commit/d3754a08fbdbddae22581b411eb4833f181fd1bf))

- split gist.rs into submodules, fix benchmark imports ([ed564cb](https://github.com/d-o-hub/rust-self-learning-memory/commit/ed564cb19b3c23acc97d2263ffceb4d80cd98bba))


### Documentation

- refresh CURRENT.md with accurate metrics (#455) ([fad2e8f](https://github.com/d-o-hub/rust-self-learning-memory/commit/fad2e8f28c367dbd6c94619db414a018edd556b7))

- Fix CURRENT.md contradictions, add performance skill ([bd86959](https://github.com/d-o-hub/rust-self-learning-memory/commit/bd86959841284877884c32c81b230bda873f5bb3))

- update plans and docs for CSM integration completion ([3f36865](https://github.com/d-o-hub/rust-self-learning-memory/commit/3f36865d79dffd7cf8368856f459ad8b0e8b8ae9))

- add CSM cascading retrieval documentation ([01ad8ef](https://github.com/d-o-hub/rust-self-learning-memory/commit/01ad8efed4d507b7e354fb7e3ac2482614a6f1ff))

- mark WG-118 (hierarchical/gist reranking) complete ([10be4b8](https://github.com/d-o-hub/rust-self-learning-memory/commit/10be4b8ff5ddc8d959e1c4ece587bbfc7e8e4b67))


### Fixed

- fix SQL injection in metadata query (#454) ([195d5a0](https://github.com/d-o-hub/rust-self-learning-memory/commit/195d5a087fc7588552a9d3087b035491d84f1db7))

- AgentFS SDK stub indicates unavailability ([90e6340](https://github.com/d-o-hub/rust-self-learning-memory/commit/90e63401425472de6db62203b501ce62a63ea1f3))

- update external signals tools for SDK unavailability ([6e3427c](https://github.com/d-o-hub/rust-self-learning-memory/commit/6e3427cc0064618afcd41d7092b10ce4878d2872))


### Maintenance

- bump version to 0.1.31 ([b8b0687](https://github.com/d-o-hub/rust-self-learning-memory/commit/b8b0687ec86aa9d68ddb610d2f403b4db76d65e8))


### Performance

- replace std::sync::RwLock with parking_lot::RwLock in QueryCache (#450) ([1b0d713](https://github.com/d-o-hub/rust-self-learning-memory/commit/1b0d7134e57804d2335b9a092de34ac6499422f9))


### security

- ignore RUSTSEC-2026-0104 (rustls-webpki CRL panic) (#462) ([4a6efce](https://github.com/d-o-hub/rust-self-learning-memory/commit/4a6efcec5bc5795dae0f4056571dd2c1ee43fd82))


## [0.1.30] - 2026-04-16



### Added

- add learn and memory-context skills ([8178a1a](https://github.com/d-o-hub/rust-self-learning-memory/commit/8178a1a858524c3eb16e766fa7ad1b209fddd043))

- add MemoryEvent broadcast channel and top-k utilities ([8a99353](https://github.com/d-o-hub/rust-self-learning-memory/commit/8a99353ba390cca48b6970c99597d183ecbce4ab))

- add learn and memory-context skills ([70916e8](https://github.com/d-o-hub/rust-self-learning-memory/commit/70916e885e2bbe80b5c9ea2df944d426bf5f0933))

- add MemoryEvent broadcast channel and top-k utilities ([3ddf483](https://github.com/d-o-hub/rust-self-learning-memory/commit/3ddf483ac03064c2aa808acad9a7d0b796c038f0))


### CI/CD

- bump the actions-all group across 1 directory with 8 updates ([f48536b](https://github.com/d-o-hub/rust-self-learning-memory/commit/f48536bbd627e435509eae88f5045cc09d7999c7))


### Documentation

- update for v0.1.30 sprint planning ([254b43f](https://github.com/d-o-hub/rust-self-learning-memory/commit/254b43f74b45098c7dceb6deddfe8e55c7ae57a1))

- mark v0.1.30 sprint complete ([4cf566e](https://github.com/d-o-hub/rust-self-learning-memory/commit/4cf566ecde7ae47e1e135ac654bfe4726c921a6b))

- update for v0.1.30 sprint planning ([70d92b3](https://github.com/d-o-hub/rust-self-learning-memory/commit/70d92b350c5441c25e6d8f24c7b9e3d5780def8c))

- mark v0.1.30 sprint complete ([50b8568](https://github.com/d-o-hub/rust-self-learning-memory/commit/50b85686d9570890511415e0ddf8e7bdc7bf5795))


### Fixed

- allow manual crates.io publish via workflow_dispatch ([d76d4c2](https://github.com/d-o-hub/rust-self-learning-memory/commit/d76d4c27ddbb4718d8241d455ddbadf6a6db0c38))

- allow workflow_dispatch for Pages deployment ([b8dd04a](https://github.com/d-o-hub/rust-self-learning-memory/commit/b8dd04a63dd523d677993167e08db1d37e2c62f8))

- publish crates in dependency order with explicit dry-run=false ([f34ef31](https://github.com/d-o-hub/rust-self-learning-memory/commit/f34ef3158001dc0c43be43f138db1eaf9bb89b77))

- add missing newline at end of publish-crates.yml ([b11177b](https://github.com/d-o-hub/rust-self-learning-memory/commit/b11177bd25d447c90530cfa498c1e5c74ad417a8))

- use fromJSON() for boolean input comparison (#431) ([53bf1a0](https://github.com/d-o-hub/rust-self-learning-memory/commit/53bf1a08d9a98006b937685283ed0dcb0d80d9eb))

- use CARGO_REGISTRY_TOKEN for crates.io publishing (#432) ([3ccaa33](https://github.com/d-o-hub/rust-self-learning-memory/commit/3ccaa33b6c10fc6ecc860463fe8e709b148ab724))

- publish redb before turso due to dependency chain (#435) ([ffd32a2](https://github.com/d-o-hub/rust-self-learning-memory/commit/ffd32a2cf6378c3fd899b41250c72e2ba0b4208c))

- make publish workflow idempotent (#437) ([1ce8bc7](https://github.com/d-o-hub/rust-self-learning-memory/commit/1ce8bc787b3b16bdc6d256257605287f81748ffe))

- handle jq null error in version check (#438) ([26fd1f3](https://github.com/d-o-hub/rust-self-learning-memory/commit/26fd1f340fed8d3007a51f8b8d71f1a3dbc2ef0b))

- add debug output to version check (#439) ([479f8d6](https://github.com/d-o-hub/rust-self-learning-memory/commit/479f8d6b37b736656a8ad1c9253f3c46e3c17295))

- add User-Agent header for crates.io API (#440) ([969d5d9](https://github.com/d-o-hub/rust-self-learning-memory/commit/969d5d93a4da8225c867eba56fcca50f43b31486))

- update fastrand from yanked v2.4.0 to v2.4.1 ([f8afd7f](https://github.com/d-o-hub/rust-self-learning-memory/commit/f8afd7fbd4a939dfac6e58963eccbf7c2917fb97))

- update fastrand from yanked v2.4.0 to v2.4.1 ([bc678bd](https://github.com/d-o-hub/rust-self-learning-memory/commit/bc678bd729eb3ef3d85af2fa57ae3f2ef40ff55d))


### Maintenance

- bump redb from 3.1.3 to 4.0.0 in the rust-major group ([891a401](https://github.com/d-o-hub/rust-self-learning-memory/commit/891a401c501c3bc03bdd3c1af5ce7173a5283007))

- bump the rust-patch-minor group with 3 updates (#446) ([7a8fd15](https://github.com/d-o-hub/rust-self-learning-memory/commit/7a8fd15155524045763ff3c25b58c997acac9a8f))


### release

- v0.1.30 sprint completion (#444) ([94d0bc9](https://github.com/d-o-hub/rust-self-learning-memory/commit/94d0bc9067838eca2d670f93e55177a303b3707e))

- v0.1.30 preparation - CI fix and sprint planning (#451) ([8a10825](https://github.com/d-o-hub/rust-self-learning-memory/commit/8a1082558f3d266637f4347c75ad691d618a2f84))


## [0.1.29] - 2026-04-05



### Added

- add MCP Server Card and GitHub Pages infrastructure ([cf0bb71](https://github.com/d-o-hub/rust-self-learning-memory/commit/cf0bb717de8a34f2b6471313ca68db636e5f1f8b))

- implement Bayesian ranking, Episode GC/TTL, spawn_blocking audit ([01d3de5](https://github.com/d-o-hub/rust-self-learning-memory/commit/01d3de52f05e100acb9095c287b0b93e664658d6))

- add DyMoE routing-drift protection (WG-089) ([15d7b1d](https://github.com/d-o-hub/rust-self-learning-memory/commit/15d7b1dc4096b6162e3f0ed7350dcf1fb8407268))

- add DualRewardScore for DyMoE pattern management (WG-090) ([f13a3e0](https://github.com/d-o-hub/rust-self-learning-memory/commit/f13a3e03356cee14c593863b19bbbe5c7881177e))

- implement native vector_top_k search for Turso ([9610db6](https://github.com/d-o-hub/rust-self-learning-memory/commit/9610db60a9f4fdc6ea8815067acc3bdbb708f651))

- add embedding format migration for native vector search ([a964900](https://github.com/d-o-hub/rust-self-learning-memory/commit/a9649001043eaed90f304bb62cbb9ce1147ae607))


### CI/CD

- add tag-version verification and release drift workflow ([97b9c85](https://github.com/d-o-hub/rust-self-learning-memory/commit/97b9c85b03c5b3f782e17a5f1d7078e86eaf12a3))


### Changed

- update binary names to do-memory-* naming ([1c27ee3](https://github.com/d-o-hub/rust-self-learning-memory/commit/1c27ee3cb8ccf990b77678b5d10bcab7efc92cc0))

- remove WASM sandbox and wasmtime/rquickjs deps ([4661534](https://github.com/d-o-hub/rust-self-learning-memory/commit/466153488aa8b8b088b9b48b8a914ce5d13d4571))

- split large files into module directories ([474ecbe](https://github.com/d-o-hub/rust-self-learning-memory/commit/474ecbe2c192c648a8be26fcd9b4d70119adf563))

- split search.rs into module directory ([17f814d](https://github.com/d-o-hub/rust-self-learning-memory/commit/17f814d836c7aed2a1a76d00687dcef9c5e00fd6))

- extract EmbeddingStorageBackend impl to separate file ([330cbae](https://github.com/d-o-hub/rust-self-learning-memory/commit/330cbaee0c635ab6bf2b6adcff32442f5d1ea0cd))

- split handlers.rs into module directory ([438d5a5](https://github.com/d-o-hub/rust-self-learning-memory/commit/438d5a537214783d826dc14725bef868bbcc8e96))


### Documentation

- update status files to v0.1.26 and add llms.txt ([7fd1f46](https://github.com/d-o-hub/rust-self-learning-memory/commit/7fd1f4663762bb84179c9384b895ecc3a8511eb7))

- update roadmap with completed v0.1.27 sprint tasks ([a08fbaa](https://github.com/d-o-hub/rust-self-learning-memory/commit/a08fbaa2ed15365191a3a8abfffcd0171548461d))

- update roadmap with v0.1.27 sprint completion and semver fix ([7b144f7](https://github.com/d-o-hub/rust-self-learning-memory/commit/7b144f7bd0d87a10552f365ae43c67c3ce0b9ba5))

- consolidate and archive completed sprints, reduce noise ([658619c](https://github.com/d-o-hub/rust-self-learning-memory/commit/658619ca2b37daa69f55d8722713efcee6adc56e))

- update progress + DyMoE impact analysis for issue #419 ([80ab252](https://github.com/d-o-hub/rust-self-learning-memory/commit/80ab252f8434361f9d42099ebcc5099f927f94a1))

- update v0.1.28 sprint progress ([aab15dd](https://github.com/d-o-hub/rust-self-learning-memory/commit/aab15dd8d0843fefe9fb745339ebef7ab113c3a6))

- mark v0.1.28 sprint complete ([8e10e88](https://github.com/d-o-hub/rust-self-learning-memory/commit/8e10e88eb445a117f657abcff3c1983e5044b991))

- add ADR-052 comprehensive analysis and v0.1.29 sprint roadmap ([535cd8c](https://github.com/d-o-hub/rust-self-learning-memory/commit/535cd8c7b0e26f42f2724d79849115638ea217c4))

- update GOAP_STATE.md for v0.1.29 sprint ([783a10d](https://github.com/d-o-hub/rust-self-learning-memory/commit/783a10de969b97480ffe2c250e31f95a365edc46))

- update ADR-052 with implementation notes ([8d380ef](https://github.com/d-o-hub/rust-self-learning-memory/commit/8d380ef09c956a39e1ecf173768547655efc67a0))

- mark v0.1.29 sprint complete ([7adb658](https://github.com/d-o-hub/rust-self-learning-memory/commit/7adb65895d09be03a3279116af08d4f2cc33f532))

- update Dependabot analysis with actions taken ([484612c](https://github.com/d-o-hub/rust-self-learning-memory/commit/484612cfd84f0ac6092df1e3a03b43d3e8292222))


### Fixed

- resolve CI failures in pages workflow and book config ([7d98fe1](https://github.com/d-o-hub/rust-self-learning-memory/commit/7d98fe1bf4fc28f1f95cca35527d151b4c698a6f))

- add trailing newline to pages.yml ([770cb68](https://github.com/d-o-hub/rust-self-learning-memory/commit/770cb6821daffb7571368fefcd049ada7cb31e93))

- increase semver check timeout and add baseline caching ([f501e4e](https://github.com/d-o-hub/rust-self-learning-memory/commit/f501e4ee4d922a3dec2e56efd58bcbbe016d715e))

- resolve compile errors and update test for turso feature ([f1607f9](https://github.com/d-o-hub/rust-self-learning-memory/commit/f1607f965cf2bad5a8da5ea39550963d0455ad99))

- add document-start marker and quote 'on' key for yamllint compliance ([c1d1038](https://github.com/d-o-hub/rust-self-learning-memory/commit/c1d1038ae34280adbea58131a813821a010c8921))

- remove cleartext session/episode IDs from dry-run logging ([4e97d8e](https://github.com/d-o-hub/rust-self-learning-memory/commit/4e97d8e1f24bffacd330e18a9b49f9b01521d427))

- rustdoc issues + quality gates integration (#423) ([bddfa13](https://github.com/d-o-hub/rust-self-learning-memory/commit/bddfa1348a2f4fbf97c16a3d9abf25a5907f0a83))

- update embedding tests for native vector search compatibility ([6a1824c](https://github.com/d-o-hub/rust-self-learning-memory/commit/6a1824c0765f2ccff4cfee906ca966ace55f7897))

- update libsql to 0.9.30, clean up audit.toml ([4eab1cd](https://github.com/d-o-hub/rust-self-learning-memory/commit/4eab1cd9d7700931dd2352ba1989d712ba58251c))


### Maintenance

- bump to 0.1.29 ([1431636](https://github.com/d-o-hub/rust-self-learning-memory/commit/1431636862df61ccbd9b6f6ed979c9afbb60272d))


## [0.1.26] - 2026-04-01



### Fixed

- add version specs for inter-crate dependencies ([dceea0e](https://github.com/d-o-hub/rust-self-learning-memory/commit/dceea0e74aa20d46a5eb4514a978b85cbb2cd251))


### Maintenance

- bump version to 0.1.26 for crates.io publishing ([e299588](https://github.com/d-o-hub/rust-self-learning-memory/commit/e2995887fbdcf86e0796f3db912ad4ebc54e615d))


## [0.1.25] - 2026-04-01



### Changed

- rename crates from memory-* to do-memory-* for crates.io ([83912ba](https://github.com/d-o-hub/rust-self-learning-memory/commit/83912ba68093fd66cde10c10ab9fed6b0da98dba))


### Fixed

- remove unused CliHarness import in security_tests.rs ([ee9b8a8](https://github.com/d-o-hub/rust-self-learning-memory/commit/ee9b8a8a090a3cf8adbe7c93948479c77f5d57f0))

- update package names in CI workflows to do-memory-* prefix ([8bdd7a9](https://github.com/d-o-hub/rust-self-learning-memory/commit/8bdd7a98c5facd7de02367a964d9901dcd6a4f49))

- update binary name references from memory-cli to do-memory-cli ([9cf1fb6](https://github.com/d-o-hub/rust-self-learning-memory/commit/9cf1fb6c2a68ea5b9fe04b0b38befd1d0869a96c))

- resolve CodeQL and codecov/patch failures ([7185e74](https://github.com/d-o-hub/rust-self-learning-memory/commit/7185e74bef444b738e3bcd4f33b730d1b3d8248e))

- add missing description for do-memory-cli package ([fe84e7f](https://github.com/d-o-hub/rust-self-learning-memory/commit/fe84e7fcb7a1336b1fb435b1e9e20ea20b10adad))


### Maintenance

- add codecov.yml configuration to main branch ([cb6c5c4](https://github.com/d-o-hub/rust-self-learning-memory/commit/cb6c5c463afcc7e7fb59ae34dbb03d3ec1c56be1))

- update npm package name to do-memory-mcp-server ([d279f0a](https://github.com/d-o-hub/rust-self-learning-memory/commit/d279f0a074a0453d1008b8bff39c9ad982790be2))

- bump version to 0.1.25 for package publishing ([2e4edfc](https://github.com/d-o-hub/rust-self-learning-memory/commit/2e4edfc7666be549de17725da2f96563eb053ad5))


### Testing

- update snapshots for do-memory-cli binary name ([72a30bb](https://github.com/d-o-hub/rust-self-learning-memory/commit/72a30bb7c3530e87dabb1eba87b7b2426f18bb16))


## [0.1.24] - 2026-03-31



### CI/CD

- bump the actions-all group across 1 directory with 3 updates (#402) ([5ad3299](https://github.com/d-o-hub/rust-self-learning-memory/commit/5ad3299c08160808a76f5c00b9abcb933a4521ee))


### Documentation

- add v0.1.24 release plan and v0.1.25 sprint planning ([320de3b](https://github.com/d-o-hub/rust-self-learning-memory/commit/320de3b1c95273c0fe648d1ce1af6cf31d394f4b))

- changelog backfill and v0.1.24 release updates ([0ad11f0](https://github.com/d-o-hub/rust-self-learning-memory/commit/0ad11f04cba114ff8116cd73832ca3ad03b45374))


### Fixed

- v0.1.24 test stability and dependency updates (#404) ([f328c15](https://github.com/d-o-hub/rust-self-learning-memory/commit/f328c15ea8cf91e996c162eeeba085598bb18082))

- formatting after storage_ops split ([bc6e5a4](https://github.com/d-o-hub/rust-self-learning-memory/commit/bc6e5a4bf16781a625bf7d5ff1d3e3f2ed63c8e6))

- YAML syntax errors in ai-slop workflows ([238de8a](https://github.com/d-o-hub/rust-self-learning-memory/commit/238de8a92202bef76f271f9018f09b76f70904fe))

- correct eval heuristic drift and learn workflow git-add path ([e7f6b31](https://github.com/d-o-hub/rust-self-learning-memory/commit/e7f6b3184291d76c1b844ad05a368dfacf1601ab))

- YAML colon spacing in ai-slop-learn.yml ([ac6567b](https://github.com/d-o-hub/rust-self-learning-memory/commit/ac6567bba0572c3106b1863dbe5a626941307318))


### Maintenance

- bump toml in the rust-patch-minor group (#397) ([ee8bab5](https://github.com/d-o-hub/rust-self-learning-memory/commit/ee8bab548698d5a637bf5bb9577c133a1cadab42))

- bump the rust-patch-minor group with 9 updates (#403) ([a302863](https://github.com/d-o-hub/rust-self-learning-memory/commit/a3028639280eff1d398b86bc291aca1574d22412))

- bump version to 0.1.24 ([16f85d3](https://github.com/d-o-hub/rust-self-learning-memory/commit/16f85d3e4e471ebc73d496209fa59f499b726baa))


## [0.1.23] - 2026-03-25



### Added

- persist durable attribution/checkpoint state and align batch MCP contract ([2bffd28](https://github.com/d-o-hub/rust-self-learning-memory/commit/2bffd28d5df49bd7297935b91e7e9370bfad28eb))

- complete WG-054 to WG-058 truth-source and CI updates ([6f697a0](https://github.com/d-o-hub/rust-self-learning-memory/commit/6f697a0f9daea2bd7c604fe53e50420eb48e7a10))

- add schema versioning with automatic cache invalidation ([1ce7e96](https://github.com/d-o-hub/rust-self-learning-memory/commit/1ce7e96bb2a02cb5247b302b19f289348300d21c))


### Changed

- fix rustfmt formatting in snapshot test ([314348f](https://github.com/d-o-hub/rust-self-learning-memory/commit/314348f27784c84be707279b593dbf6fd4e70d2c))

- normalize recommendation import ordering ([955b33b](https://github.com/d-o-hub/rust-self-learning-memory/commit/955b33b6d00775f7ba6d84946bcf131826bb941f))

- apply cargo fmt to schema versioning code ([d60081f](https://github.com/d-o-hub/rust-self-learning-memory/commit/d60081f0bb924089c5b20e7b05fb54155cfb471f))


### Documentation

- add version-tag verification to prevent cargo-dist failures ([beb1381](https://github.com/d-o-hub/rust-self-learning-memory/commit/beb13810cec177722207ae9b402d24a8927e4f23))

- centralize advisory ignore documentation ([7605fc6](https://github.com/d-o-hub/rust-self-learning-memory/commit/7605fc63c24660ae43e0c8cab921d831c2ead605))

- mark WG-043 through WG-050 as complete in GOALS.md ([f846ef5](https://github.com/d-o-hub/rust-self-learning-memory/commit/f846ef5132cfb31a35b1873ee3c9860589a8184e))

- update GOAP_STATE.md branch and remediation status ([6683b43](https://github.com/d-o-hub/rust-self-learning-memory/commit/6683b4353e8a68a5dab96fd4e10ad62581debecb))

- update skills, agent_docs, and plans with verification results ([c4dbbd2](https://github.com/d-o-hub/rust-self-learning-memory/commit/c4dbbd25d06b68bff0bdffa593823f3bd40fa9fa))

- update documentation and validation status ([9d80616](https://github.com/d-o-hub/rust-self-learning-memory/commit/9d8061698cdfcfdce910df60d7caa2faf0d46c1d))


### Fixed

- Bump version to 0.1.22 and fix changelog workflow ([596194f](https://github.com/d-o-hub/rust-self-learning-memory/commit/596194f8a8b069f9ccdfd059608b9e40ce64aeb9))

- make version snapshot test version-agnostic ([5bd661f](https://github.com/d-o-hub/rust-self-learning-memory/commit/5bd661fe960100e6d5187f46f962fb3620c9a050))

- update libsql to 0.9.30 to fix RUSTSEC-2026-0049 ([516632c](https://github.com/d-o-hub/rust-self-learning-memory/commit/516632c813bf280667e4d24d8d240535a8eed1b1))

- ignore RUSTSEC-2026-0049 for rustls-webpki 0.102.x ([73dc09f](https://github.com/d-o-hub/rust-self-learning-memory/commit/73dc09f941eca82660dee5f3d7b7eaf9ad81291a))

- ignore RUSTSEC-2026-0049 in cargo audit step ([7d3fba8](https://github.com/d-o-hub/rust-self-learning-memory/commit/7d3fba8703e42ecc4e1fdb30da871d06bb4ead61))

- also ignore RUSTSEC-2026-0049 in audit JSON report step ([62acb22](https://github.com/d-o-hub/rust-self-learning-memory/commit/62acb22db35904871803250eb39ad745f06a36ed))

- satisfy clippy in recommendation stats iteration ([de3f83a](https://github.com/d-o-hub/rust-self-learning-memory/commit/de3f83ab28516576917f00ff81f43e04633e9322))

- extend macOS test timeout and relax codecov patch gate ([3927c2c](https://github.com/d-o-hub/rust-self-learning-memory/commit/3927c2c456e5bd5ce1e82ec2c27406026ec89e84))

- stabilize macOS nextest timeouts and patch coverage gate ([0742608](https://github.com/d-o-hub/rust-self-learning-memory/commit/0742608618ca590bd8f59bc621cf7b59d9512c76))

- align benchmark job timeout with benchmark runtime budget ([3e461a6](https://github.com/d-o-hub/rust-self-learning-memory/commit/3e461a661a34469c979f65cad2ab92eb75f1938e))


### Maintenance

- bump the rust-major group with 4 updates ([ec4d706](https://github.com/d-o-hub/rust-self-learning-memory/commit/ec4d706bf523a662d73747be9a288bd66b38080b))

- bump version to 0.1.23 ([5b9ea75](https://github.com/d-o-hub/rust-self-learning-memory/commit/5b9ea75e205b4d9abd98d6fe8f7a8940d973b8d4))


## [0.1.22] - 2026-03-20



### Added

- add attribution, checkpoint, and feedback features ([f0348ce](https://github.com/d-o-hub/rust-self-learning-memory/commit/f0348ce40e06a8b58d34c3664072d66cd54c9197))

- add playbook support for memory retrieval patterns ([a651acb](https://github.com/d-o-hub/rust-self-learning-memory/commit/a651acb9299363acbbb00305eeeff3ce2fffa358))

- restructure AGENTS.md to 140 LOC with agents-update skill ([6edb254](https://github.com/d-o-hub/rust-self-learning-memory/commit/6edb254fa8c36b3867b54bc95966cba8391be1d0))

- implement v0.1.22 sprint tasks and quality polish ([f2914c3](https://github.com/d-o-hub/rust-self-learning-memory/commit/f2914c34088d4bd7c07798f7e9cf4e97a6634285))

- Complete v0.1.22 quality and feature polish sprint ([677aacc](https://github.com/d-o-hub/rust-self-learning-memory/commit/677aacc825d7da21e35e27e52317893f6996e9ff))


### CI/CD

- bump actions/upload-artifact in the actions-all group ([b4af290](https://github.com/d-o-hub/rust-self-learning-memory/commit/b4af29016c78168ef9588a45e7588f7f44b75b95))

- add codecov.yml with patch threshold configuration ([832da83](https://github.com/d-o-hub/rust-self-learning-memory/commit/832da83555b31b860bb34b487d1d0e8e6b73b676))

- add codecov.yml with patch threshold configuration ([6e2e9a6](https://github.com/d-o-hub/rust-self-learning-memory/commit/6e2e9a607e4e702c5b93085c65eff7f555ab4116))


### Changed

- reduce operations.rs docstrings to meet 500 LOC limit ([d0deb8e](https://github.com/d-o-hub/rust-self-learning-memory/commit/d0deb8e44501196e83f00708459cc613d2bfa5b2))


### Documentation

- update plans/ for v0.1.21 release ([dd05ade](https://github.com/d-o-hub/rust-self-learning-memory/commit/dd05ade74aca509869d4f8773e61de8e554c4e66))

- implement ADR-046 Claude Code configuration improvements ([bc90d99](https://github.com/d-o-hub/rust-self-learning-memory/commit/bc90d99ac0d077fa72234214d060a796ca15d14a))

- update npm package scope from @d-o to @d-o-hub ([5503ed1](https://github.com/d-o-hub/rust-self-learning-memory/commit/5503ed13f989d72c69db7af3c43e6ad765f98348))

- mark ADR-046 as Accepted ([72e6889](https://github.com/d-o-hub/rust-self-learning-memory/commit/72e688962137dc71e2e69c172c616fc294bc0558))

- update GOAP_STATE.md with ADR-046 completion ([739671a](https://github.com/d-o-hub/rust-self-learning-memory/commit/739671a2fb6597acb1a3716694b539f83bf501ef))

- update feature-implementer with memory constraint ([a15fc49](https://github.com/d-o-hub/rust-self-learning-memory/commit/a15fc49bfa537e29571aa7affdc681e592417df2))

- update plans/ for v0.1.22 release ([1d98bad](https://github.com/d-o-hub/rust-self-learning-memory/commit/1d98bad0b8fd5a763ca849158b02512a091dfeb1))

- update GOAP_STATE.md with v0.1.22 release completion ([2f43c75](https://github.com/d-o-hub/rust-self-learning-memory/commit/2f43c7533b42f552745bbe154f3b39d4b61209c3))

- update AGENTS.md and agent_docs for v0.1.22 features ([60ff3fd](https://github.com/d-o-hub/rust-self-learning-memory/commit/60ff3fd5d8f93fba501c513ad255735232499c24))

- simplify AGENTS.md and update plans for v0.1.22 ([6613a18](https://github.com/d-o-hub/rust-self-learning-memory/commit/6613a185dd3753f467ee7fe8d7e7db9493590dc8))

- add v0.1.22 sprint plan with quality and feature polish goals ([229a285](https://github.com/d-o-hub/rust-self-learning-memory/commit/229a285003cd7da6b4baf29f8d97de64f38d5175))

- add PR #369 and Epic #373 links to planning docs ([212380b](https://github.com/d-o-hub/rust-self-learning-memory/commit/212380b10c11d35255888869f930dad92810aeb9))

- add benchmark workflow sync and subshell learnings ([0a7e992](https://github.com/d-o-hub/rust-self-learning-memory/commit/0a7e9926c3e3ddf06b375887032f53982c6e8666))

- update issue verification status for v0.1.22 sprint ([1dfcb28](https://github.com/d-o-hub/rust-self-learning-memory/commit/1dfcb28601be199d5cc0a201eff0f935f7792271))

- consolidate code_conventions.md to reduce duplication ([15bc3ab](https://github.com/d-o-hub/rust-self-learning-memory/commit/15bc3ab3972b48b2822bf49fce9492641b14fd6f))

- consolidate agent_docs to remove duplication and bloat ([3e08f5b](https://github.com/d-o-hub/rust-self-learning-memory/commit/3e08f5b64adff035171b4e078d3d5af6a3617db5))


### Fixed

- remove unused import in recommendation_feedback tool ([fbf2e98](https://github.com/d-o-hub/rust-self-learning-memory/commit/fbf2e982f0a69c28dc188718d7e1431da247e849))

- ignore slow async extraction test causing CI timeout ([74504f8](https://github.com/d-o-hub/rust-self-learning-memory/commit/74504f80fd31b0a3b2b57b5f857dfb034a427dec))

- fix benchmark workflow for deleted benchmarks and docs-only PRs ([4bc4b1a](https://github.com/d-o-hub/rust-self-learning-memory/commit/4bc4b1aa770fd199effad841774f322e1e0cb877))

- resolve all P0 quality issues for v0.1.22 (WG-040/041/042) ([fe3e786](https://github.com/d-o-hub/rust-self-learning-memory/commit/fe3e786fbf47c8c7cd29b972ca3a4ded42ca7ac5))

- resolve CLI test timeouts with serial execution (#392) ([d21ca42](https://github.com/d-o-hub/rust-self-learning-memory/commit/d21ca42a5c8cb8cdaba71efad4d33e3f3ae8736c))

- resolve linting and test failures for v0.1.22 sprint ([9b9e7e7](https://github.com/d-o-hub/rust-self-learning-memory/commit/9b9e7e73317ea6680f844b7ea49e8ffe40913e4a))

- resolve clippy warnings in test files ([ae5202c](https://github.com/d-o-hub/rust-self-learning-memory/commit/ae5202c368311ad210c580ba361be104990e4f9a))

- resolve all clippy warnings and compilation errors ([abbf3ec](https://github.com/d-o-hub/rust-self-learning-memory/commit/abbf3ec45bfda2cdfc90b663497c45f08866453a))

- fix broken property tests and update plans with verified metrics ([fea988a](https://github.com/d-o-hub/rust-self-learning-memory/commit/fea988a4eae01a09ae3749d9fbfb051675b12eef))

- Update aws-lc-sys to 0.39.0 to resolve 5 CVEs ([05c0481](https://github.com/d-o-hub/rust-self-learning-memory/commit/05c0481f1b7c4e0ee856591cf7c799b118a9dc92))


### Maintenance

- bump lz4_flex from 0.12.0 to 0.13.0 in the rust-major group ([f594aae](https://github.com/d-o-hub/rust-self-learning-memory/commit/f594aaece5e0fff79300a8d385bbc561b1ecf61c))

- bump the rust-patch-minor group with 7 updates ([b26f073](https://github.com/d-o-hub/rust-self-learning-memory/commit/b26f073e35b73fec8554f8cabe5e166237fbe878))


### Testing

- update episode help snapshot for checkpoint commands ([218b336](https://github.com/d-o-hub/rust-self-learning-memory/commit/218b33604d0e96d5f433389ec3afbf0cf4e1e039))

- add snapshot tests for ADR-044 playbook types (WG-045) ([702e9f9](https://github.com/d-o-hub/rust-self-learning-memory/commit/702e9f916f8012bb0d541c06efbacc4d5c8c8961))

- add property tests for ADR-044 features (WG-046) ([ac8eb97](https://github.com/d-o-hub/rust-self-learning-memory/commit/ac8eb9799fca053a53e3b6e307d9c15051dcefcb))


## [0.1.21] - 2026-03-15



### Added

- add publishing infrastructure for ADR-045 (v0.1.21) ([5b56ffc](https://github.com/d-o-hub/rust-self-learning-memory/commit/5b56ffc5253f08844405afbce2b8e5cbc1ea7913))


### Changed

- remove dead code and fix stale documentation ([66ad4d1](https://github.com/d-o-hub/rust-self-learning-memory/commit/66ad4d1ad886442cfb0adf5842a15cc8a4bfaf4c))

- remove duplicate embedding.rs module ([c41ff99](https://github.com/d-o-hub/rust-self-learning-memory/commit/c41ff99ebac58735e21a5e49cf5e3aec90a5a85f))


### Documentation

- update GOAP_STATE with Phase 1 completion and corrected gap analysis ([36340ff](https://github.com/d-o-hub/rust-self-learning-memory/commit/36340ff00b948b9a0f1aa4a0d17489c21a9d2f79))

- complete gap analysis with corrected findings ([7a3def1](https://github.com/d-o-hub/rust-self-learning-memory/commit/7a3def1215ec09d5fecc0e3a5c139ed97902f82d))

- complete ADR-040 Phase 2 & 3 implementation ([590ddc7](https://github.com/d-o-hub/rust-self-learning-memory/commit/590ddc701f33fb7c26469e8543982641e9d5aae4))

- update GOAP_STATE and ADR-042 with v0.1.19 release status ([ad232a9](https://github.com/d-o-hub/rust-self-learning-memory/commit/ad232a99dd66658052d940c519fecffad65334f2))

- add git workflow checklist to prevent uncommitted changes ([5e74be0](https://github.com/d-o-hub/rust-self-learning-memory/commit/5e74be012b7552d2ae807013722e9293b9c0b9f2))

- refactor AGENTS.md to stay under 140 LOC ([ce6992b](https://github.com/d-o-hub/rust-self-learning-memory/commit/ce6992b91442b3055e6b0508a16741b26c486b5b))

- add gap analysis for v0.1.21 sprint ([226f021](https://github.com/d-o-hub/rust-self-learning-memory/commit/226f0218653401912092b2c673b3f2d86a77dc2d))


### Fixed

- stabilize nightly tests and changelog workflows ([9522f83](https://github.com/d-o-hub/rust-self-learning-memory/commit/9522f83757aac1e6c93e93a1f18fb981dedb7b0b))

- ADR-041 build and CLI dispatch errors (#363) ([29dd36b](https://github.com/d-o-hub/rust-self-learning-memory/commit/29dd36b18d7a19fc367be3903e81c445f56996c7))

- resolve flaky test_server_creation and update snapshots ([2027077](https://github.com/d-o-hub/rust-self-learning-memory/commit/2027077b05e5cc890eb46b7b5b24bb69be59c652))

- correct cargo-deny and cyclonedx CLI arguments ([4bec465](https://github.com/d-o-hub/rust-self-learning-memory/commit/4bec46526f50a7fe332d6b2aea3c16aa735cf458))

- resolve YAML lint issues in workflow files ([26fe916](https://github.com/d-o-hub/rust-self-learning-memory/commit/26fe9163006232bd734b37051e9cad47f489b7fd))

- update version snapshot to 0.1.21 ([f0860fa](https://github.com/d-o-hub/rust-self-learning-memory/commit/f0860fa1b31637d0226e1f5a82785fd6bc887a56))


### Maintenance

- complete rust-cache upgrade and remove dead_code attrs ([4eb42dc](https://github.com/d-o-hub/rust-self-learning-memory/commit/4eb42dcb8c2c7b97768c0548576e5913889a8997))

- release v0.1.19 ([53db206](https://github.com/d-o-hub/rust-self-learning-memory/commit/53db206c438f8572cdef585c5246f7f23b81e27b))

- clean up temporary snap.new file ([137da14](https://github.com/d-o-hub/rust-self-learning-memory/commit/137da143b1b3ca7202a5e9d6e3e340551d30308b))

- update Cargo.lock for v0.1.21 ([f3326d8](https://github.com/d-o-hub/rust-self-learning-memory/commit/f3326d8c7368d4831dfab61c0dff2b4023324a45))


### release

- v0.1.20 sprint fixes (#365) ([6bb6acc](https://github.com/d-o-hub/rust-self-learning-memory/commit/6bb6acc0504c36482ba46165565ffaaa7121c6d7))


## [0.1.18] - 2026-03-12



### Added

- implement WG-008 tasks and consolidate plans folder ([4b008b2](https://github.com/d-o-hub/rust-self-learning-memory/commit/4b008b26a685b66edd6985ed0e867bae46dcfe35))

- add domain and task type filters to episode commands ([5f4cd49](https://github.com/d-o-hub/rust-self-learning-memory/commit/5f4cd49912c4532270a746e709bca58b1a809f09))

- add adaptive LRU cache entry module ([3c516a5](https://github.com/d-o-hub/rust-self-learning-memory/commit/3c516a5e8da19d28c09b6bfa6e3428bf8e3347dd))

- add AdaptiveCache with Cache trait adapter ([32f2dad](https://github.com/d-o-hub/rust-self-learning-memory/commit/32f2dad3decf59dc23f787388035613854db4012))


### Documentation

- update GOAP state with WG-009 completion (#358) ([10a1093](https://github.com/d-o-hub/rust-self-learning-memory/commit/10a10935457fcfa14b1a543ac8504349b09a81a9))

- mark WG-010 complete - production code already follows best practices ([7e48d19](https://github.com/d-o-hub/rust-self-learning-memory/commit/7e48d19a8668fb9eec70d1a82da7ee8ce41c89e4))

- mark WG-011 complete - target not achievable for architectural reasons ([8222a9b](https://github.com/d-o-hub/rust-self-learning-memory/commit/8222a9b0e97447a558b79777445427956ec09194))

- update GOAP_STATE.md with v0.1.18 sprint completion ([508874e](https://github.com/d-o-hub/rust-self-learning-memory/commit/508874e2775112358c5e21379d3c31fb65769c51))

- fix 29 broken markdown links in active documentation ([02fdfdb](https://github.com/d-o-hub/rust-self-learning-memory/commit/02fdfdb28984dbe6a362cc02ab86d28fd309437f))

- update GOAP tracking with v0.1.18 completion status ([8b60dab](https://github.com/d-o-hub/rust-self-learning-memory/commit/8b60dabaa56eead974bafe4427d9d6e7a56e722a))

- document transport compression architecture in TursoConfig ([b63bdeb](https://github.com/d-o-hub/rust-self-learning-memory/commit/b63bdeb9a621853b4e10f4db3f1c4a2f366aba1f))

- update GOAP_STATE.md with PR #360 CI status ([f8a62e7](https://github.com/d-o-hub/rust-self-learning-memory/commit/f8a62e714a420627de2d5eb44df7b1cf0b824386))


### Fixed

- remove unreachable batch tool parameter schemas (#357) ([d693924](https://github.com/d-o-hub/rust-self-learning-memory/commit/d693924db04ff0783e823d3b8a0ce32ad1f71013))


### Maintenance

- remove unused libsql dependency from test-utils ([47ac45e](https://github.com/d-o-hub/rust-self-learning-memory/commit/47ac45e3fa3162dcd63ea96187464e06d992d17e))

- remove unused libsql dependency from test-utils ([664d025](https://github.com/d-o-hub/rust-self-learning-memory/commit/664d025ea54391d2e78be22374bf0d4a3b0778e7))

- remove unused .claude agent configuration files ([e82a317](https://github.com/d-o-hub/rust-self-learning-memory/commit/e82a31773dc903decba3d88b322a71109b2895c0))

- release v0.1.18 ([d9e1ffa](https://github.com/d-o-hub/rust-self-learning-memory/commit/d9e1ffabf992ce0a298835a7effae49905498750))


### Testing

- enable episode search/filter test with new CLI flags ([60aa3a7](https://github.com/d-o-hub/rust-self-learning-memory/commit/60aa3a7701b7fb111452066406fc9d1d8ca1c784))

- update version snapshot for v0.1.18 ([0af182c](https://github.com/d-o-hub/rust-self-learning-memory/commit/0af182c232bd2e3d791bee9ad5bbf3a147ddfe7e))


## [0.1.17] - 2026-03-10



### Added

- re-enable batch module in tools ([3b74c00](https://github.com/d-o-hub/rust-self-learning-memory/commit/3b74c003b41f8b97ed32549ae1e1fa6e1c4e95c9))

- add git-cliff configuration and documentation ([f0a8a9f](https://github.com/d-o-hub/rust-self-learning-memory/commit/f0a8a9fa2b027e0a11223e3713716e37bac491bf))

- add MCP embedding tools and property tests ([1f8643c](https://github.com/d-o-hub/rust-self-learning-memory/commit/1f8643c2fd1280cfb2d95e314ac8dcd338f80f14))

- add ADR-024 integration tests for lazy tool loading ([385b256](https://github.com/d-o-hub/rust-self-learning-memory/commit/385b256ed1273b284d19d156bb0d9603aab9bf41))

- implement embedding tools for MCP server ([c6ac442](https://github.com/d-o-hub/rust-self-learning-memory/commit/c6ac44225d3cba8e97d9cf418b11729e3d2b3ed3))

- add error message snapshot tests ([0d34ede](https://github.com/d-o-hub/rust-self-learning-memory/commit/0d34edecbe840ae356fd8d1469f9aa81618500e2))

- add embedding tool snapshot tests ([41b43ed](https://github.com/d-o-hub/rust-self-learning-memory/commit/41b43ed7f685cd5a4a4ba9d54aee4c612ae19036))

- expand snapshot test coverage ([594a210](https://github.com/d-o-hub/rust-self-learning-memory/commit/594a210c6b9db853c899dc370c70f693e389fa29))

- enhance cache configuration loader ([59a2882](https://github.com/d-o-hub/rust-self-learning-memory/commit/59a28823a1cdaee67d080cd25c72066021bb5b21))

- add semantic embedding service ([c3ac4d4](https://github.com/d-o-hub/rust-self-learning-memory/commit/c3ac4d45afa1eb211752e2ad299b69ca9918c0ee))

- add token-efficient agents for development workflow ([9c878d6](https://github.com/d-o-hub/rust-self-learning-memory/commit/9c878d6f48fde9ab508086e32138d318863f9417))

- integrate existing skills into agents for enhanced capabilities ([360cdea](https://github.com/d-o-hub/rust-self-learning-memory/commit/360cdea901e7c7cc48cff8961ddbd72bc93c6a4f))

- implement O1/O3/O5 opportunities - tool parity, docs fixes, runtime wiring tests ([6d8b61d](https://github.com/d-o-hub/rust-self-learning-memory/commit/6d8b61dbc37b1dabea29cf7599b470a0b5744c67))


### CI/CD

- bump reviewdog/action-actionlint from 1.70.0 to 1.71.0 ([4215dc6](https://github.com/d-o-hub/rust-self-learning-memory/commit/4215dc6ff8fb262c3efe059ba5b15c327b8234d1))

- isolate CI target dirs and update GOAP progress (#322) ([e8545e1](https://github.com/d-o-hub/rust-self-learning-memory/commit/e8545e17dbccbdfc4248e102c6b0ee0dcfff278d))

- bump actions/upload-artifact from 4 to 7 ([e4e8ffb](https://github.com/d-o-hub/rust-self-learning-memory/commit/e4e8ffb4ac0889d57c23ad663c3e8fc1c9733b28))

- bump actions/download-artifact from 4 to 8 ([4cf6c45](https://github.com/d-o-hub/rust-self-learning-memory/commit/4cf6c45d53fae04566f14d188f9e38499ceabbb6))

- ADR-029 Phase 3/4 quick wins ([672e57b](https://github.com/d-o-hub/rust-self-learning-memory/commit/672e57b321367ce46a8d3e08f524858a62d8c663))

- bump the actions-all group with 2 updates ([0e4e44a](https://github.com/d-o-hub/rust-self-learning-memory/commit/0e4e44ae741b581e670244b107c9d1ae87a058a8))

- implement fast-checks-first pattern for all workflows ([88e1f03](https://github.com/d-o-hub/rust-self-learning-memory/commit/88e1f037a81da3162b7d868103ed4f5806351896))

- implement fast-checks-first pattern for all workflows (#348) ([be9b955](https://github.com/d-o-hub/rust-self-learning-memory/commit/be9b9552cb7a9d931b0f2e37b5088454f77cbdd0))


### Changed

- split large modules into modular structure ([b38dd98](https://github.com/d-o-hub/rust-self-learning-memory/commit/b38dd988513a992b0a6e34b36fec5e9d027f02e2))

- flatten test modules for indexing and graph algorithms ([db7ed5c](https://github.com/d-o-hub/rust-self-learning-memory/commit/db7ed5c78d4e52cf7bb400d8adc5fae73fc0635a))

- remove expect calls in embedding benchmark ([fee4f31](https://github.com/d-o-hub/rust-self-learning-memory/commit/fee4f31bcab747fe8205196786052055da69a04b))

- split embedding tools into separate modules ([4c576f1](https://github.com/d-o-hub/rust-self-learning-memory/commit/4c576f1c9e9af5f7b6b5b1f308ca6509054076e0))

- implement G2/G9 tasks - remove dead code and split oversized files ([60764af](https://github.com/d-o-hub/rust-self-learning-memory/commit/60764af636a3d0229b493179b699c9ea5e7c1cb1))


### Documentation

- sync Week 1 GOAP execution status ([4252c07](https://github.com/d-o-hub/rust-self-learning-memory/commit/4252c072b489c51ce77be5cc8abad8feb49415e4))

- close INFRA+E3 week1 checkpoint with gate evidence ([ade8c7e](https://github.com/d-o-hub/rust-self-learning-memory/commit/ade8c7e0f6d0b9353a9cf9c96a60cd2b43d38d96))

- sync missing Week 1 plan tasks and status evidence (#321) ([fd2735c](https://github.com/d-o-hub/rust-self-learning-memory/commit/fd2735c55f8e3be6ad5e590bc91761b62774aea1))

- update missing tasks execution plan ([7bdd616](https://github.com/d-o-hub/rust-self-learning-memory/commit/7bdd616e9d4ad69b11d18c29782b73ce292b01e6))

- record post-push PR monitoring status ([d39a429](https://github.com/d-o-hub/rust-self-learning-memory/commit/d39a42911652d6b034ba1c70f2207e877f5033cb))

- capture PR check-attachment learnings across plans and guidance ([55c0db3](https://github.com/d-o-hub/rust-self-learning-memory/commit/55c0db311b2c5aa03f028e548d67711a42af0ff2))

- track WG-005 CI hardening progress ([ae0b806](https://github.com/d-o-hub/rust-self-learning-memory/commit/ae0b806b7a23a18a7701cf2e8c2f6b65745cd814))

- add token efficiency and pre-flight validation sections ([6c80f73](https://github.com/d-o-hub/rust-self-learning-memory/commit/6c80f73b27c08a5a362b31cfc33c446b66f9a132))

- update GOAP state and actions with CI fixes ([6304d69](https://github.com/d-o-hub/rust-self-learning-memory/commit/6304d691e5522df0aa2f594a1d8bed77472f4599))

- update GOAP plans with 2026-03-06 analysis ([bee6910](https://github.com/d-o-hub/rust-self-learning-memory/commit/bee6910fab4402a1a34cc4c344d9bd9c93cf0416))

- add GOAP_AGENT_IMPROVEMENT_PLAN.md ([4bf5d0b](https://github.com/d-o-hub/rust-self-learning-memory/commit/4bf5d0b53eb04b7bf1d0cdd5237a1d48ffec85a3))

- add GOAP_AGENT_QUALITY_GATES.md ([8134cf0](https://github.com/d-o-hub/rust-self-learning-memory/commit/8134cf0af6a0a38cbe97f31b86d9baaa39de1714))

- add GOAP_AGENT_EXECUTION_TEMPLATE.md ([ef1ba7f](https://github.com/d-o-hub/rust-self-learning-memory/commit/ef1ba7f103da2d8e6976abcb39c9d31fec20aa06))

- add GOAP_AGENT_ROADMAP.md ([a472e68](https://github.com/d-o-hub/rust-self-learning-memory/commit/a472e68582a5501c4ade8faf0857d8ea4165c631))

- add GOAP_AGENT_CODEBASE_VERIFICATION.md ([51eef38](https://github.com/d-o-hub/rust-self-learning-memory/commit/51eef38a14aaf32e53869ca78dcdd376b46466b7))

- add agent-coordination skill docs and update GOAP state ([3a45fae](https://github.com/d-o-hub/rust-self-learning-memory/commit/3a45fae2849247016ff5bf94bb7d1119176f6f43))

- fix broken markdown links across codebase ([fc9e6de](https://github.com/d-o-hub/rust-self-learning-memory/commit/fc9e6de492f9a5c7661af1cfb948d416314f8ce0))

- update state with Dependabot PRs merged ([301a3be](https://github.com/d-o-hub/rust-self-learning-memory/commit/301a3be79ec75d8c488afb22f93bfc97f3fe4b80))

- add job dependency pattern to prevent skipped jobs ([f107a5f](https://github.com/d-o-hub/rust-self-learning-memory/commit/f107a5fc26921096cd7890d51e632f8c388296f7))

- update validation status for PR #350 ([1530717](https://github.com/d-o-hub/rust-self-learning-memory/commit/15307173f345ba636896dd77153c619315e51228))

- add skill references to agent definitions ([5317ae2](https://github.com/d-o-hub/rust-self-learning-memory/commit/5317ae2155b69d0dc057472c104ad3a710fcc83e))

- add dead_code and ignored tests analysis ([273ccbf](https://github.com/d-o-hub/rust-self-learning-memory/commit/273ccbf6bcff61ad2a17d07d2cc4b9ce35bff200))

- update validation status with analysis results ([0ebd312](https://github.com/d-o-hub/rust-self-learning-memory/commit/0ebd312e5f213154d96e752eff7422f6230d2d16))

- update GOAP state with G2/G9 analysis progress ([f5809c4](https://github.com/d-o-hub/rust-self-learning-memory/commit/f5809c4bfe0396fa8debdfa9006a0b5495854a99))

- update validation status for CI running ([0e4fd8e](https://github.com/d-o-hub/rust-self-learning-memory/commit/0e4fd8efac1f66aad4d3aaa1d9aa7277aa4d2586))

- update validation status with CI results ([c097539](https://github.com/d-o-hub/rust-self-learning-memory/commit/c0975391914c4b0c9138194c9d73432842296a02))

- update validation status with libsql memory corruption fix ([c6a6794](https://github.com/d-o-hub/rust-self-learning-memory/commit/c6a6794d7aa56af5f304942d90010644492ff7d0))

- update validation status with second libsql fix ([1775b49](https://github.com/d-o-hub/rust-self-learning-memory/commit/1775b49ae3329b311b73f477fa89c93f6dd39448))

- update validation status with complete libsql fix ([fcab225](https://github.com/d-o-hub/rust-self-learning-memory/commit/fcab22564dbe2200daff245e4e895e479148dd68))

- update validation status with complete libsql fix (13 files) ([4145a9d](https://github.com/d-o-hub/rust-self-learning-memory/commit/4145a9d24e486540468e4d9168a61fc0b8c71dda))

- update validation status - ALL CI CHECKS PASSED ✅ ([9d88fa1](https://github.com/d-o-hub/rust-self-learning-memory/commit/9d88fa1389dbe1c42d00a84a8dd4389a9ee6cc88))

- update plans with G2/G9 implementation progress ([1148692](https://github.com/d-o-hub/rust-self-learning-memory/commit/1148692dff3201a27ddaae49bff08b43ecd91c07))

- finalize G2/G9 implementation status ([fe9bfc9](https://github.com/d-o-hub/rust-self-learning-memory/commit/fe9bfc9f63d4b25f02360698e02e541f0996b128))

- update disk space optimization and cleanup scripts ([2832565](https://github.com/d-o-hub/rust-self-learning-memory/commit/28325652596fabc299c4972a48c6c3247479f5a8))


### Fixed

- restore CLI persistence and stabilize multi-crate test gates ([01e4b01](https://github.com/d-o-hub/rust-self-learning-memory/commit/01e4b01d03159572757906bbb0cede84896046bb))

- add missing cache test module file ([99b994d](https://github.com/d-o-hub/rust-self-learning-memory/commit/99b994d704eef23d73e70b771ee34d58da53f488))

- resolve RUSTSEC-2026-0021 wasmtime vulnerability + nightly disk space ([62004ca](https://github.com/d-o-hub/rust-self-learning-memory/commit/62004caca05e253503e91ffe0d454671f3ba7cbe))

- triage ignored tests + correct prod unwrap metrics ([2c5ea87](https://github.com/d-o-hub/rust-self-learning-memory/commit/2c5ea8738aec497408286fb21aeed0b08179cb1d))

- update wasmtime 42.0.1 compatibility ([613dc82](https://github.com/d-o-hub/rust-self-learning-memory/commit/613dc82184e6a6e3b6a721f844fcf1b0b1074d75))

- resolve nightly tests + add git-cliff workflow ([7347c9b](https://github.com/d-o-hub/rust-self-learning-memory/commit/7347c9b9d15fd080433c52d83e6676c3fd5c7869))

- address PR #334 format/yaml failures and sync GOAP status ([cee188f](https://github.com/d-o-hub/rust-self-learning-memory/commit/cee188fa4e9382a15ccfd5dd3c548bf1c90257c3))

- align PR coverage and add stable check anchor ([d5d0043](https://github.com/d-o-hub/rust-self-learning-memory/commit/d5d004363661baa3068fca79e0ac467abf13eaf1))

- run PR check anchor on pull_request ([19b7355](https://github.com/d-o-hub/rust-self-learning-memory/commit/19b735567935900c401e8b071501f34c6f17fd68))

- resolve yaml-lint trailing blank line ([38820c9](https://github.com/d-o-hub/rust-self-learning-memory/commit/38820c94b4f1ae8ea3165e69eddcfb60f82aa25d))

- make codecov patch robust for docs-only diffs ([cbc0e55](https://github.com/d-o-hub/rust-self-learning-memory/commit/cbc0e553f3059eb067a1d3ab66a183def01c4614))

- align property tests with current APIs and deps ([b6eeaa8](https://github.com/d-o-hub/rust-self-learning-memory/commit/b6eeaa8dc9b96e1212edb97f091ae29348220a4a))

- update wasmtime 42.0.1 compatibility ([b5618fd](https://github.com/d-o-hub/rust-self-learning-memory/commit/b5618fd394c59a652bbb939e22507a8dc2a20c4d))

- add missing snapshot baselines for memory-core tests ([d00f58f](https://github.com/d-o-hub/rust-self-learning-memory/commit/d00f58fc3956c177e6993c64b39b83e3c703f601))

- exclude known timing-dependent tests from nightly ([5a69fe1](https://github.com/d-o-hub/rust-self-learning-memory/commit/5a69fe1429ffd7065b8f9c24651372e7397cabdc))

- use static permissions for actionlint compatibility ([f8b1c5e](https://github.com/d-o-hub/rust-self-learning-memory/commit/f8b1c5ebdcd23eecb16595dad9d313471fdee3e8))

- resolve rust-major breaking changes for redb 3.x and rand 0.10 ([3ea4fcb](https://github.com/d-o-hub/rust-self-learning-memory/commit/3ea4fcb33efcbbb4aee6e64116cbde2884f22b8e))

- run benchmarks on push to main when quick-check is skipped ([529c17e](https://github.com/d-o-hub/rust-self-learning-memory/commit/529c17e584be90e89b81484f48ea85495e9fc1aa))

- remove trailing spaces from benchmarks.yml ([3c02058](https://github.com/d-o-hub/rust-self-learning-memory/commit/3c02058d91fbbc0bed16882d3e8cd208c682a875))

- update rand API for 0.10 compatibility ([cc037a0](https://github.com/d-o-hub/rust-self-learning-memory/commit/cc037a0210ed44d233bf3233b244b33d50de9946))

- fix formatting ([2e95cdb](https://github.com/d-o-hub/rust-self-learning-memory/commit/2e95cdb06ad4246f5094bfbff3ca4c89dc0a1176))

- complete rand 0.10 API migration in all crates ([1077da8](https://github.com/d-o-hub/rust-self-learning-memory/commit/1077da8d4dca80e973d8e53dd0f554e6bfffc6cd))

- exclude failing tests from nightly CI ([db601a8](https://github.com/d-o-hub/rust-self-learning-memory/commit/db601a84949e45638a3836c009998d0dd5779630))

- resolve clippy errors in property tests ([c2afb86](https://github.com/d-o-hub/rust-self-learning-memory/commit/c2afb860e5cdab1436fafb63635b635b7a5e295a))

- resolve field_reassign_with_default warnings in property tests ([714f667](https://github.com/d-o-hub/rust-self-learning-memory/commit/714f6674ec0ef43f2eca505792967f7af6e4dd84))

- improve keepalive connection handling ([2528625](https://github.com/d-o-hub/rust-self-learning-memory/commit/25286251962bbcf5bf86ef89082812fc13e20d3f))

- fix caching pool test compilation ([8a55fc4](https://github.com/d-o-hub/rust-self-learning-memory/commit/8a55fc45e8e0d6f8ff75e039d47f5f125a2583b0))

- escape Option<Tool> in documentation comments ([4605ddb](https://github.com/d-o-hub/rust-self-learning-memory/commit/4605ddbab0095115562be5f7d2ec2c51a0159525))

- re-ignore test_pool_creation with better comment ([db8d4b4](https://github.com/d-o-hub/rust-self-learning-memory/commit/db8d4b4c691f718f9c6f1bd45f019c3bf0707b19))

- ignore cache_integration_test due to libsql memory corruption ([b9d8537](https://github.com/d-o-hub/rust-self-learning-memory/commit/b9d8537222d8125183fbc405bf133aaf080956f5))

- ignore capacity_enforcement_test due to libsql memory corruption ([bbee3fc](https://github.com/d-o-hub/rust-self-learning-memory/commit/bbee3fc64fe3e1ad99d0b1869cd3ccacb95263aa))

- ignore all TursoStorage integration tests due to libsql memory corruption ([87c34d6](https://github.com/d-o-hub/rust-self-learning-memory/commit/87c34d66929ede2a029d82d4bd89d6662f08b8e0))

- ignore ALL TursoStorage integration tests due to libsql memory corruption ([2315db0](https://github.com/d-o-hub/rust-self-learning-memory/commit/2315db0006953eba4cfc662dc044a0ea219989a6))

- update quinn-proto to 0.11.14 (RUSTSEC-2026-0037) ([7c1cc4e](https://github.com/d-o-hub/rust-self-learning-memory/commit/7c1cc4efee7752225751fa0649a62725f4eb8151))

- update snapshot and README for v0.1.17 ([c049ed3](https://github.com/d-o-hub/rust-self-learning-memory/commit/c049ed39989be38c158140567bb9d27b4b780fdf))


### Maintenance

- bump toml from 1.0.2+spec-1.1.0 to 1.0.3+spec-1.1.0 (#318) ([a89c70d](https://github.com/d-o-hub/rust-self-learning-memory/commit/a89c70d4c26f1cbe8e2b2abafda28f2c02fce7df))

- bump futures from 0.3.31 to 0.3.32 (#317) ([22b8328](https://github.com/d-o-hub/rust-self-learning-memory/commit/22b83286e351735087290d63b8c22a4fcaa997e3))

- bump anyhow from 1.0.101 to 1.0.102 (#316) ([cc18d87](https://github.com/d-o-hub/rust-self-learning-memory/commit/cc18d875923c19e9044f6cf7fc8ba27af9eade01))

- bump serial_test from 3.3.1 to 3.4.0 (#315) ([2d43cb3](https://github.com/d-o-hub/rust-self-learning-memory/commit/2d43cb3c8aba0d87caa2dc144c3a777c20f3fac9))

- bump clap from 4.5.59 to 4.5.60 (#314) ([cae2edc](https://github.com/d-o-hub/rust-self-learning-memory/commit/cae2edc01652bda999a7ef00c349837903ded3c5))

- bump sysinfo from 0.38.0 to 0.38.2 (#313) ([2a51a6a](https://github.com/d-o-hub/rust-self-learning-memory/commit/2a51a6aadff267a6fd24236a3d58fc799895c843))

- bump wasmtime from 41.0.4 to 42.0.1 ([afb416e](https://github.com/d-o-hub/rust-self-learning-memory/commit/afb416e38fbb81a6cba26764758bd42670dedcf5))

- bump tempfile from 3.25.0 to 3.26.0 (#332) ([49c6dfc](https://github.com/d-o-hub/rust-self-learning-memory/commit/49c6dfcbc6cad28bd6f30935cf8f7662e7ee9a44))

- bump chrono from 0.4.43 to 0.4.44 (#328) ([4d5e605](https://github.com/d-o-hub/rust-self-learning-memory/commit/4d5e605fa193e3814e81261ca03488d8ba6401d1))

- bump augurs-changepoint from 0.10.1 to 0.10.2 (#329) ([b4a8fc4](https://github.com/d-o-hub/rust-self-learning-memory/commit/b4a8fc438024e427e8b342454739b0ee77a234cd))

- bump wasmtime-wasi from 41.0.4 to 42.0.1 (#333) ([09062b7](https://github.com/d-o-hub/rust-self-learning-memory/commit/09062b7b3d06dc1652476b8a56a64ebfa0bc617b))

- remove legacy claude command artifacts ([87c6b70](https://github.com/d-o-hub/rust-self-learning-memory/commit/87c6b70b1016fed6f59e924d27c1b2e0ef80bbcc))

- bump the rust-patch-minor group with 5 updates ([4d4a02a](https://github.com/d-o-hub/rust-self-learning-memory/commit/4d4a02afc6128033907eb65cafb5c12fcb6a7403))

- bump the rust-major group with 3 updates ([6aeaed6](https://github.com/d-o-hub/rust-self-learning-memory/commit/6aeaed65e712fb522d3ffd315880a8b025e129b3))

- update dependencies for embedding tools ([a13fcf2](https://github.com/d-o-hub/rust-self-learning-memory/commit/a13fcf2aba34085fd6993a0081b9f648daf7e7c1))

- bump the rust-patch-minor group with 3 updates ([366ad97](https://github.com/d-o-hub/rust-self-learning-memory/commit/366ad972c2b85889bc0b560be733f13c5cba258c))


### Testing

- add insta for snapshot testing ([0fb0d9a](https://github.com/d-o-hub/rust-self-learning-memory/commit/0fb0d9adf70c9848108a036eb5d9c11eb5899589))

- increase wait timeout for slow CI environments ([c4da106](https://github.com/d-o-hub/rust-self-learning-memory/commit/c4da106cdc4f42c2a02d602fd92f52ded11d799a))

- add snapshot tests for backup, monitor, logs, eval commands ([dccb77e](https://github.com/d-o-hub/rust-self-learning-memory/commit/dccb77ef535b72227c69096431433bcb2ade6a17))

- add property-based and snapshot tests ([b0324a4](https://github.com/d-o-hub/rust-self-learning-memory/commit/b0324a476cf96cb8aa5fa22c48a3f4aa0918960e))

- update snapshot baselines for help output ([f496c23](https://github.com/d-o-hub/rust-self-learning-memory/commit/f496c23ce555234c247f905d56acd7c4a282f885))


### release

- v0.1.17 ([776e910](https://github.com/d-o-hub/rust-self-learning-memory/commit/776e910856ee4f215435ab13ad0f135317623c3c))


## [0.1.16] - 2026-02-22



### Added

- add general-purpose agent documentation ([f378328](https://github.com/d-o-hub/rust-self-learning-memory/commit/f3783285df1909ee73d22c024a9b8a29b63a298b))

- add CI workflow optimization tools ([ff2c1dc](https://github.com/d-o-hub/rust-self-learning-memory/commit/ff2c1dcb6889bf04e2a11a723c7a3f53e914edbc))

- Add list_tool_names() for 98% token reduction in tool discovery ([54a243c](https://github.com/d-o-hub/rust-self-learning-memory/commit/54a243cddc012896be1f60bd94377a409c52905e))

- Phase 4 - Concurrency + PR/Issue templates ([e6167b8](https://github.com/d-o-hub/rust-self-learning-memory/commit/e6167b83ba7000726dfa76eebdf74f8400e92ed4))


### Build

- optimize build config and add release tooling ([53e453a](https://github.com/d-o-hub/rust-self-learning-memory/commit/53e453a8150a7a6f9cfc2eae514ef02147909faa))


### CI/CD

- add nextest.toml profiles (ADR-033) ([bfda9cb](https://github.com/d-o-hub/rust-self-learning-memory/commit/bfda9cb342ffcf17c5c12bbc27790f6a8633be76))

- standardize on cargo-nextest (ADR-033) ([1d0580c](https://github.com/d-o-hub/rust-self-learning-memory/commit/1d0580ccde131b2a1ec52565849d580a414c3e20))

- add cargo-release configuration (ADR-034) ([170b446](https://github.com/d-o-hub/rust-self-learning-memory/commit/170b446f0fcd3fb26dab6532ff735feba60241bc))

- add cargo-dist binary distribution (ADR-034) ([33e0b88](https://github.com/d-o-hub/rust-self-learning-memory/commit/33e0b887d4d1e7b63f89e582138bcb98b5ecefc8))

- add mutation testing with cargo-mutants (ADR-033) ([020b0e3](https://github.com/d-o-hub/rust-self-learning-memory/commit/020b0e3484f1971c67a240b24dede8271e300a29))

- add dependency metrics tracking (ADR-036) ([e998d56](https://github.com/d-o-hub/rust-self-learning-memory/commit/e998d56dc87b98abb20480cfdda13312055a01df))

- add mold linker installation (ADR-032) ([ffe9fd2](https://github.com/d-o-hub/rust-self-learning-memory/commit/ffe9fd24dcd78837648be0cc5740bfdb6e3894e1))

- add shellcheck disable comments (ADR-034) ([f204e6a](https://github.com/d-o-hub/rust-self-learning-memory/commit/f204e6a6bb9b0aedc2f6d4c892d9e91d8d781ea8))


### Changed

- remove claude flow mcp usage from general agent ([55d5e4c](https://github.com/d-o-hub/rust-self-learning-memory/commit/55d5e4c6027e5ae925d04724ca7274c1cfc83039))

- align general agent with project conventions ([650cad3](https://github.com/d-o-hub/rust-self-learning-memory/commit/650cad382db0e15fbf00d82c0918e27129ed236f))

- improve dev experience in general agent ([0bfc80e](https://github.com/d-o-hub/rust-self-learning-memory/commit/0bfc80e0998ae32d9b2731d0653a9570949ca27d))

- remove claude-code-flow agents and consolidate general agent ([a7a514c](https://github.com/d-o-hub/rust-self-learning-memory/commit/a7a514cfceca0de388a8c2f095c5593ec78dfcdc))

- apply cargo fmt for Rust 2024 edition ([e807933](https://github.com/d-o-hub/rust-self-learning-memory/commit/e807933c2c81a1abd2142b5d03bdf33dc1906bc2))


### Documentation

- Add comprehensive execution patterns and verification docs ([c60f3bd](https://github.com/d-o-hub/rust-self-learning-memory/commit/c60f3bd898032827adef331d76d7a9c92becc150))

- update GOAP GitHub Actions plan with completed tasks ([e0c761f](https://github.com/d-o-hub/rust-self-learning-memory/commit/e0c761f9082a5ee227014c0ee054fa73d83220de))

- Update status - CI ALL PASSING, Phase 2 complete ([d482488](https://github.com/d-o-hub/rust-self-learning-memory/commit/d48248861aafc21f33f023712844d533316e2792))

- Update status - MERGED TO MAIN #283 ([a7e4029](https://github.com/d-o-hub/rust-self-learning-memory/commit/a7e402902709f31b82c846bef1b074f55b5cbcca))

- Update README.md status - v0.1.14, CI ALL PASSING ([8f244ad](https://github.com/d-o-hub/rust-self-learning-memory/commit/8f244ad8de75d8bec7629ab7e6ff5e7558366fba))

- archive deprecated Phase 1 planning documents ([bae2780](https://github.com/d-o-hub/rust-self-learning-memory/commit/bae2780a88f9c0c780a04b270d8efe08df38f36d))

- add goap rescue note for pr304 ([c7cc4f2](https://github.com/d-o-hub/rust-self-learning-memory/commit/c7cc4f2bacba389bebcaf65f835d8d9250f4a264))

- Add 2026 architecture decisions for disk, testing, release, edition, and deps ([dc2e398](https://github.com/d-o-hub/rust-self-learning-memory/commit/dc2e3985f2d6d31900e01a23196cd1b30caadb1a))

- Mark Phase 2 (mold linker) as complete ([de1666f](https://github.com/d-o-hub/rust-self-learning-memory/commit/de1666f32c734ecf6bfe7705c5b5e145bd36423f))

- add final GOAP execution summary ([cd731a1](https://github.com/d-o-hub/rust-self-learning-memory/commit/cd731a177d2d7098afc5bf2550fac2c1a2a15922))

- Align MCP lazy listing defaults ([2ed7a0a](https://github.com/d-o-hub/rust-self-learning-memory/commit/2ed7a0a15d17e57422143a95e7d00409e7c34488))

- Add token optimization docs and memory-agent config ([a08ab44](https://github.com/d-o-hub/rust-self-learning-memory/commit/a08ab4496ba993e66e18178890766268b3634a30))


### Fixed

- Phase 1 critical fixes for GitHub Actions workflows ([e63ebf3](https://github.com/d-o-hub/rust-self-learning-memory/commit/e63ebf3331bdf7597c0958b68b736b495fb56114))

- Phase 1-2 GitHub Actions modernization ([5e16b77](https://github.com/d-o-hub/rust-self-learning-memory/commit/5e16b7757a0ba9d9554eb65b34a10cb7da56efd5))

- fix remaining workflow issues ([c4493d5](https://github.com/d-o-hub/rust-self-learning-memory/commit/c4493d5894774a2003189db477772780d9a39402))

- Nightly Full Tests CI FIXED ([7a93f5c](https://github.com/d-o-hub/rust-self-learning-memory/commit/7a93f5c73fb5dc5b9dcdf5b8fcaab4aa189d1b3b))

- Remove benchmarks and tests with missing dependencies ([19377c4](https://github.com/d-o-hub/rust-self-learning-memory/commit/19377c4a4bfd2259f62e0f4eb54db98595a1885b))

- Phase B1.3 - High-priority unwrap() fixes (#299) ([bdc6240](https://github.com/d-o-hub/rust-self-learning-memory/commit/bdc624030340ab4efeb582b24fc6e0c2637f1b8e))

- add oauth feature to Cargo.toml and update lockfile ([989e431](https://github.com/d-o-hub/rust-self-learning-memory/commit/989e431a17c6cf7a0ee885e37e16e88337597599))

- restore oauth loader and storage config test defaults ([dbeefb2](https://github.com/d-o-hub/rust-self-learning-memory/commit/dbeefb26d03520f74857f1b11c5b93684cd6eb04))

- remediate PR306 cfg checks and lockfile audit panic ([faef0fd](https://github.com/d-o-hub/rust-self-learning-memory/commit/faef0fd7c0bd898147e6f2856ed9be7a46d4368a))

- update dev profile for smaller artifacts and add clean artifacts script ([efd46ab](https://github.com/d-o-hub/rust-self-learning-memory/commit/efd46ab00d3812e99690cb36320ec2bece882288))

- Fix Rust 2024 edition pattern matching errors ([d3de69b](https://github.com/d-o-hub/rust-self-learning-memory/commit/d3de69b15a7bd0382e56fc6ee97ea2dece38387f))

- Complete Rust 2024 edition migration fixes ([757dae5](https://github.com/d-o-hub/rust-self-learning-memory/commit/757dae54d7a18453ab6fe5c7702530086881e13f))

- remove mold linker for CI compatibility (ADR-032) ([8129f9b](https://github.com/d-o-hub/rust-self-learning-memory/commit/8129f9b0a50af0620b30d936ec9bec7a40a073c9))

- resolve clippy errors in property_tests.rs (ADR-033) ([0912974](https://github.com/d-o-hub/rust-self-learning-memory/commit/09129743306d79fe91cae541602deadff622ae00))

- allow dirty dist-workspace and fix benchmarks shellcheck ([21f1434](https://github.com/d-o-hub/rust-self-learning-memory/commit/21f143460a5274477bb9ae55ae39d6aef641f75d))

- add allow attribute for manual_async_fn in tests (ADR-033) ([95d8029](https://github.com/d-o-hub/rust-self-learning-memory/commit/95d80298ad049cc57c7347fbf4a669021224d7b4))

- escape 'gen' keyword for Rust 2024 edition (ADR-035) ([04e3d8b](https://github.com/d-o-hub/rust-self-learning-memory/commit/04e3d8b3e21e69b5c4a77a703c4df61ef0475e0e))

- disable line-length rule in yamllint ([509f801](https://github.com/d-o-hub/rust-self-learning-memory/commit/509f8017198eb2a78342e1cbba86329bc939b35b))

- disable shellcheck in actionlint ([8c36f35](https://github.com/d-o-hub/rust-self-learning-memory/commit/8c36f350728c4a7e85af5e69836e150d104dc361))

- restore regex dependency for pattern_extraction ([5ea2a05](https://github.com/d-o-hub/rust-self-learning-memory/commit/5ea2a05ac1362b27376a36ac768ea14852a7ca33))

- resolve MCP build failures (ADR-029) ([dce0f2a](https://github.com/d-o-hub/rust-self-learning-memory/commit/dce0f2a948aea5735eae98422a626c08a77851ee))

- replace install-action with cargo install ([f0aa69b](https://github.com/d-o-hub/rust-self-learning-memory/commit/f0aa69bc7e9dda20594bd5638c5257ac5762b708))

- remove remaining install-action usage ([cf23a04](https://github.com/d-o-hub/rust-self-learning-memory/commit/cf23a04fad2d14958ba93c729dab70c7136abf72))

- extend MCP build timeout to 600s ([684bb57](https://github.com/d-o-hub/rust-self-learning-memory/commit/684bb57628d5d1d31a225bbba05e4ce223c3e248))

- minor build and test updates ([c9a1016](https://github.com/d-o-hub/rust-self-learning-memory/commit/c9a1016dbab0f1522d5d6df98245ecfa0e8a32b8))


### Maintenance

- Remove deleted benchmark and test references ([c258dc6](https://github.com/d-o-hub/rust-self-learning-memory/commit/c258dc61974119800d2c1210ffa443475e231a17))

- bump tempfile from 3.24.0 to 3.25.0 ([c8b8ffd](https://github.com/d-o-hub/rust-self-learning-memory/commit/c8b8ffdc8acd7b4eda1b65165fe9000e3aec838d))

- bump toml from 0.9.11+spec-1.1.0 to 1.0.1+spec-1.1.0 ([ebc4e54](https://github.com/d-o-hub/rust-self-learning-memory/commit/ebc4e54b4a9914dc8387502af3848b89396fa76b))

- bump indicatif from 0.18.3 to 0.18.4 ([8b7f529](https://github.com/d-o-hub/rust-self-learning-memory/commit/8b7f52911463e0e18ad454e8896d4159f51be17c))

- bump predicates from 3.1.3 to 3.1.4 ([41785ef](https://github.com/d-o-hub/rust-self-learning-memory/commit/41785ef52ac5bfda2f1595d523cc6b0fc8a5b14b))

- bump deep_causality from 0.13.3 to 0.13.4 ([59d6ad8](https://github.com/d-o-hub/rust-self-learning-memory/commit/59d6ad8a6445c8990316bc981f45a9b1bdec5fc1))

- bump uuid from 1.20.0 to 1.21.0 ([86b5ef3](https://github.com/d-o-hub/rust-self-learning-memory/commit/86b5ef35ab3cda081c5beee0be7944fc9360fc99))

- bump libc from 0.2.180 to 0.2.182 ([2d87f5c](https://github.com/d-o-hub/rust-self-learning-memory/commit/2d87f5c802d565ee027d8f0ea1541dce8269bc87))

- bump clap_complete from 4.5.65 to 4.5.66 ([794080a](https://github.com/d-o-hub/rust-self-learning-memory/commit/794080a949c4570c075316769003e2dba2d425b4))

- bump wat from 1.244.0 to 1.245.1 ([c3d252a](https://github.com/d-o-hub/rust-self-learning-memory/commit/c3d252ad13776ad1a0e3fd3ba049750d4d372517))

- bump clap from 4.5.57 to 4.5.58 ([ef831d6](https://github.com/d-o-hub/rust-self-learning-memory/commit/ef831d6b3681988333a723a3bf62e5b192405c89))

- Phase 1 modernization - ADR-032 build profiles + Rust 2024 edition ([47a1991](https://github.com/d-o-hub/rust-self-learning-memory/commit/47a1991818115a6a9a291546d98d463ea9689331))

- remove unused dependencies (ADR-036) ([88a59a6](https://github.com/d-o-hub/rust-self-learning-memory/commit/88a59a63877161501f91dd26318a7184e3aa17c8))

- Add commit skill with enforced quality gates ([ca38d4a](https://github.com/d-o-hub/rust-self-learning-memory/commit/ca38d4a30a15bd190680dabc9d53299dc721638f))


### Testing

- fix cli flag order and ignore flaky regression test ([d5a211c](https://github.com/d-o-hub/rust-self-learning-memory/commit/d5a211c73518f6c6acbc81f0629e9381ade60620))

- add proptest property-based testing (ADR-033) ([0e2badb](https://github.com/d-o-hub/rust-self-learning-memory/commit/0e2badb44aafcf18c9fd6ed086d426e187e4ee2b))

- add insta snapshot testing (ADR-033) ([d61e11c](https://github.com/d-o-hub/rust-self-learning-memory/commit/d61e11c57dca69e05a81ee5d4a917e55a0c4bc19))


### release

- v0.1.15 - MCP Token Optimization + CI Modernization (#285) ([f29808b](https://github.com/d-o-hub/rust-self-learning-memory/commit/f29808bfa66991ee7ccdcd068a32a40d4e903f21))

- v0.1.16 - ADR Implementation Complete (032-036) ([19890b5](https://github.com/d-o-hub/rust-self-learning-memory/commit/19890b539baa081e41497822d7d146d8d34ae07b))


## [0.1.14] - 2026-02-14



### Added

- implement contrastive learning with triplet loss for task adapters ([bcf6784](https://github.com/d-o-hub/rust-self-learning-memory/commit/bcf6784ae5e4cd09f615dfa07b4f9fbb221c714d))

- implement Completion utility (MCP 2025-11-25) ([97d365d](https://github.com/d-o-hub/rust-self-learning-memory/commit/97d365d78cbc2ec28599a40ed1fd75bec3ec692e))

- implement OAuth 2.1 authorization for production deployments ([94494ad](https://github.com/d-o-hub/rust-self-learning-memory/commit/94494adc91c417df25798eabb7129f02e61988cc))

- fix completion utility external tagging and add OAuth 2.1 tests ([19bb1e0](https://github.com/d-o-hub/rust-self-learning-memory/commit/19bb1e0d7e8cf6e771c746ebe97f9d2e460f4cbc))

- implement Elicitation support (MCP 2025-11-25) ([c4afc6f](https://github.com/d-o-hub/rust-self-learning-memory/commit/c4afc6fa00ac925ad9023a60d81f5a6ddd8c9750))

- implement Tasks utility and embedding configuration ([87541e3](https://github.com/d-o-hub/rust-self-learning-memory/commit/87541e34f534d88e4e847121b3a4941981b4e3ca))

- Add MCP protocol version negotiation support ([0ffbf8d](https://github.com/d-o-hub/rust-self-learning-memory/commit/0ffbf8d8c44159705a447c76dfa683d2ddd197d4))

- Add package.json with inspector scripts ([aaae8fb](https://github.com/d-o-hub/rust-self-learning-memory/commit/aaae8fb00e84c56a879fe42b4688b4651d8dedc9))

- Implement semantic pattern search and recommendations ([8291d18](https://github.com/d-o-hub/rust-self-learning-memory/commit/8291d1873fe24acc678c4c723ab7c9ea33aa97e2))

- Add semantic pattern search and recommendation engine ([4471bed](https://github.com/d-o-hub/rust-self-learning-memory/commit/4471bed4745a4690adc3fafbb7cdd4c56db80492))

- Add delete_episode method to StorageBackend trait ([78a9a83](https://github.com/d-o-hub/rust-self-learning-memory/commit/78a9a83a94681d2f3604b756a62d11a5c8d4a0d0))

- Add episode deletion support and semantic search improvements ([985a47a](https://github.com/d-o-hub/rust-self-learning-memory/commit/985a47ac361f1d9015fdba5cb9cbb71843a5f99f))

- Add episode filtering and batch operations ([65d4798](https://github.com/d-o-hub/rust-self-learning-memory/commit/65d47983c03576179aa9914584f9ca0441b8594e))

- implement test optimization strategy with skills, hooks, and agents ([c9a1d71](https://github.com/d-o-hub/rust-self-learning-memory/commit/c9a1d71ed209f47db5213177e283785f498f0049))

- comprehensive v0.1.13 release with file compliance and quality improvements ([790e5f1](https://github.com/d-o-hub/rust-self-learning-memory/commit/790e5f12c80cc73ddabdd25dfd0b5b05a44ccce9))

- Add changepoint detection and DBSCAN clustering modules ([ad1124a](https://github.com/d-o-hub/rust-self-learning-memory/commit/ad1124a8f48cc560625685581d6810692f1bac3a))

- Implement episodic memory system with embeddings ([0ae795a](https://github.com/d-o-hub/rust-self-learning-memory/commit/0ae795a5f018d26d526a8ecc27bbd349ddb9ac3d))

- Add bulk episode retrieval command and MCP tool ([f69c57f](https://github.com/d-o-hub/rust-self-learning-memory/commit/f69c57f23c40a589ff781095c19be5396dbe3dc9))

- add sort option for episode search ([0b7d58f](https://github.com/d-o-hub/rust-self-learning-memory/commit/0b7d58f19aa08e87e454291924750144ba2d3e91))

- implement cache integration layer with adaptive caching ([9204a6e](https://github.com/d-o-hub/rust-self-learning-memory/commit/9204a6e18e52bff2f4e5ac265545875842c96535))

- add compression module for 40% network bandwidth reduction ([a8eee7b](https://github.com/d-o-hub/rust-self-learning-memory/commit/a8eee7bb6fe6344991b7ad29912383f72c44608a))

- implement cache integration layer with adaptive caching ([49d2d51](https://github.com/d-o-hub/rust-self-learning-memory/commit/49d2d515e495379909f28b885007e2792003e305))

- Add compression benchmark for data optimization testing ([4688abd](https://github.com/d-o-hub/rust-self-learning-memory/commit/4688abde5a1879d001ffa8c8b132af49bebe007e))

- reduce clone operations with Arc-based episode retrieval ([f20b346](https://github.com/d-o-hub/rust-self-learning-memory/commit/f20b346812f95802eb6e83261c83935b0b9715ef))

- complete Phase 3 core features and file compliance ([571e8c0](https://github.com/d-o-hub/rust-self-learning-memory/commit/571e8c0e42d286ba59cd54e133269ef96379630e))

- add relationship module to Turso storage ([3006d8d](https://github.com/d-o-hub/rust-self-learning-memory/commit/3006d8de540045525b245540ba472866fe19d5e6))

- Complete Phase 3 with episode relationships, security hardening, and performance optimizations ([28f2205](https://github.com/d-o-hub/rust-self-learning-memory/commit/28f22054c0c5d355bacd4496485655bfbfbca219))

- Complete Phase 4 Sprint 1 - Performance Improvements and Test Fixes (#263) ([c7d5b9b](https://github.com/d-o-hub/rust-self-learning-memory/commit/c7d5b9b111d3334e17b9dacfeefc80c7bf1d4ff6))

- Integrate PR #265 relationship tools and CLI commands onto PR #272 (#274) ([0b0a62b](https://github.com/d-o-hub/rust-self-learning-memory/commit/0b0a62bea45f097dbb4c477b5f2465972a9e709a))

- add field selection support to query_memory handler ([e3b376f](https://github.com/d-o-hub/rust-self-learning-memory/commit/e3b376f7a980146cfdeb49dfd90c9bef20bba1b0))

- Add metrics and tracing infrastructure ([064e18f](https://github.com/d-o-hub/rust-self-learning-memory/commit/064e18f8e161d3402a1ebae600add8493286ec4a))

- Add storage metrics and error handling ([75ed04f](https://github.com/d-o-hub/rust-self-learning-memory/commit/75ed04feb32f0def08817d87ac12cf4d44ec2377))

- add NetMind provider configuration ([43a4357](https://github.com/d-o-hub/rust-self-learning-memory/commit/43a435710474992d90ada5dedff0d6750dc59135))

- add YAML validator skill documentation ([a5fb555](https://github.com/d-o-hub/rust-self-learning-memory/commit/a5fb5552212060a370b58c2d3c2dbf26bec6a125))

- integrate rate limiting and audit logging for episode relationships ([9f6b366](https://github.com/d-o-hub/rust-self-learning-memory/commit/9f6b366a910de3f8dc1bdad645e4b9a2d7c5fbe7))

- comprehensive implementation status - infrastructure complete ([1f62c73](https://github.com/d-o-hub/rust-self-learning-memory/commit/1f62c733ae120090cfd8b45dca146dbe4fa2977e))


### CI/CD

- add pull_request trigger to run CI on PRs before merge ([5fc8cf6](https://github.com/d-o-hub/rust-self-learning-memory/commit/5fc8cf68abf5a265ad504a0f7bd836f25a61e373))

- update ci-guard to allow running CI on pull_request events ([263e636](https://github.com/d-o-hub/rust-self-learning-memory/commit/263e6369bab5a12ad1dd2cc83ae9b5b9e3a167db))

- add nightly workflow for slow integration tests ([65ee8e9](https://github.com/d-o-hub/rust-self-learning-memory/commit/65ee8e9f4a83a8abb4f85da2aa93060cd5e5c5bc))

- bump codecov/codecov-action from 4 to 5 ([3967f47](https://github.com/d-o-hub/rust-self-learning-memory/commit/3967f47655ff3abfdc5a3ee5397f108f818ca4e6))

- optimize quality gates timeout and add independent coverage workflow ([b62e88f](https://github.com/d-o-hub/rust-self-learning-memory/commit/b62e88f4fade90fd1d1bd78a6c6bdf3e00d342bb))

- bump actions/cache from 4 to 5 ([2b4d07b](https://github.com/d-o-hub/rust-self-learning-memory/commit/2b4d07bf0dcc2dbd252da333e2476f1ac058ee08))

- bump actions/setup-python from 5 to 6 ([35e648f](https://github.com/d-o-hub/rust-self-learning-memory/commit/35e648feb5ceb8b314a847eb416ff3108dd8b812))

- bump actions/upload-artifact from 4 to 6 (#258) ([ba2dabb](https://github.com/d-o-hub/rust-self-learning-memory/commit/ba2dabb3f0f5df5e9beea6cea8c3b9a35d59661b))

- bump reviewdog/action-actionlint from 1.69.1 to 1.70.0 (#256) ([1951295](https://github.com/d-o-hub/rust-self-learning-memory/commit/195129522ee311c42eb22763b353f6b0a8d38e36))

- bump lewagon/wait-on-check-action from 1.4.1 to 1.5.0 (#244) ([53099b8](https://github.com/d-o-hub/rust-self-learning-memory/commit/53099b80022dca3ff62602200ea74e8a71fa6419))

- add timeout and concurrency controls ([fdbfefd](https://github.com/d-o-hub/rust-self-learning-memory/commit/fdbfefdb436e975b396a4a6d1211121d6bc9f020))


### Changed

- split retriever.rs into modular structure ([5823f89](https://github.com/d-o-hub/rust-self-learning-memory/commit/5823f89376cd93678ad969d9c9946c0941f80964))

- apply rustfmt formatting ([18aa6f2](https://github.com/d-o-hub/rust-self-learning-memory/commit/18aa6f29f470508c9cf640127d967d06135da6e4))

- split extractor.rs into modular structure ([8957ca3](https://github.com/d-o-hub/rust-self-learning-memory/commit/8957ca3441e087203fc1c6e460369b38bf6ede0e))

- Split large files into modular structure for 500 LOC compliance ([de6ec38](https://github.com/d-o-hub/rust-self-learning-memory/commit/de6ec3856aedf276b4a73fc480828ccab6fcb3fb))

- Split large files into modular structures for 500 LOC compliance ([e3ddaff](https://github.com/d-o-hub/rust-self-learning-memory/commit/e3ddaff57a840cca8b47773499fa336aaa17d27f))

- split analysis.rs into bocpd/engine/types modules ([ab07aeb](https://github.com/d-o-hub/rust-self-learning-memory/commit/ab07aeb3cc1c0596dbd9e56732447772a38e2ea9))

- split mcp.rs into completion/elicitation/tasks modules ([bf3afcf](https://github.com/d-o-hub/rust-self-learning-memory/commit/bf3afcfc199ecfc101f83495a83c5b0cefb6d1b5))

- split advanced_pattern_analysis.rs into types/tool modules ([1677cd8](https://github.com/d-o-hub/rust-self-learning-memory/commit/1677cd86df738968896eddd34d1a1af7da8b072c))

- Split large command files into modules to meet 500 LOC limit ([d94eae4](https://github.com/d-o-hub/rust-self-learning-memory/commit/d94eae4a17fb703dd27b9c9a113fb5409fd0e642))

- Split test modules and quality_metrics for 500 LOC compliance ([8411846](https://github.com/d-o-hub/rust-self-learning-memory/commit/8411846207458a8e8b6f0365ec64a9bcb7808d69))

- Split embeddings and summary modules for 500 LOC compliance ([9953495](https://github.com/d-o-hub/rust-self-learning-memory/commit/9953495d2ade8ea9f796af88d91dfd4aadafb44b))

- Split 5 files for 500 LOC compliance ([0c4cd89](https://github.com/d-o-hub/rust-self-learning-memory/commit/0c4cd89a21774040823aacfba83e6a9a246d8cc3))

- Fix import ordering for cargo fmt ([ac833d0](https://github.com/d-o-hub/rust-self-learning-memory/commit/ac833d03aee49a07aa1e81f0c87ebb1abe2db4d3))

- Fix code formatting with cargo fmt ([9bb32cb](https://github.com/d-o-hub/rust-self-learning-memory/commit/9bb32cb3547ae7ea2bc299c9288c0fcfecc24902))

- Split large files into modules for 500 LOC compliance ([334cd91](https://github.com/d-o-hub/rust-self-learning-memory/commit/334cd91bc69bdd14aa8d2e410d964a41b236154d))

- split sync.rs and batch.rs into modules for 500 LOC compliance ([26fc146](https://github.com/d-o-hub/rust-self-learning-memory/commit/26fc146115ea567a397a11d624956e0b901775e3))

- remove old sync.rs and batch.rs files (modules now in use) ([ce5897e](https://github.com/d-o-hub/rust-self-learning-memory/commit/ce5897e8ddead3b9feb343f6f82158f4594c1289))

- resolve 6 GitHub issues (file size, clones, error handling, deps) ([a713f4d](https://github.com/d-o-hub/rust-self-learning-memory/commit/a713f4dd2f9aa5ee8f2907bc06cf7cf6fa77c6a7))

- optimize WASM sandbox availability check and restore mcp.json configuration ([5ce1a87](https://github.com/d-o-hub/rust-self-learning-memory/commit/5ce1a874b9cf672e9557a758d2c3162f9173e728))

- run cargo fmt on episode list module ([21adfb1](https://github.com/d-o-hub/rust-self-learning-memory/commit/21adfb16b203d400cb6008dac6c7563c9959238b))

- split modules for 500 LOC compliance ([27fff61](https://github.com/d-o-hub/rust-self-learning-memory/commit/27fff614c2196ea18ddb5d7cd0a2619e9bba8281))

- split index module into sub-500 LOC files ([d7c7701](https://github.com/d-o-hub/rust-self-learning-memory/commit/d7c7701571f7b807583b115f771d84ad6c81d994))

- Optimize memory usage with Arc<Episode> storage ([3bcf0c7](https://github.com/d-o-hub/rust-self-learning-memory/commit/3bcf0c756c2630b3532341756bc9d09c3b83ecc7))

- Split large files into subdirectories ([96d4a3e](https://github.com/d-o-hub/rust-self-learning-memory/commit/96d4a3eb103ffb670ec12b1ee40fe63a986260f6))

- improve test visibility and fix warnings ([d439907](https://github.com/d-o-hub/rust-self-learning-memory/commit/d439907aaddc2859adac2fd9a54ef9c7c972a802))

- clean up code and fix compiler warnings ([8902b57](https://github.com/d-o-hub/rust-self-learning-memory/commit/8902b57c83fdd802e4a3db435d5215eaa42d5718))

- Reorganize memory module structure ([3c59f0b](https://github.com/d-o-hub/rust-self-learning-memory/commit/3c59f0b19bdb8b7f2f62048f0a7d23f69f2026c2))

- split config/tests.rs into test modules ([4ecf04e](https://github.com/d-o-hub/rust-self-learning-memory/commit/4ecf04e1a6c5a1ba185c801acefc6ec44d0266ce))

- split config/mod.rs into type modules ([c035c3a](https://github.com/d-o-hub/rust-self-learning-memory/commit/c035c3ad869aeb8445fc2b6fcec1afbb1d1b67c0))

- Split large files into smaller modules and reorganize documentation ([e87c119](https://github.com/d-o-hub/rust-self-learning-memory/commit/e87c1193b8e0d03c9087d3a4e87ce90833fbc582))

- Fix formatting issues detected by CI ([f372a53](https://github.com/d-o-hub/rust-self-learning-memory/commit/f372a53e667704343e8d07d1ca0225d8a240786e))

- run cargo fmt ([8525632](https://github.com/d-o-hub/rust-self-learning-memory/commit/85256329ecbd2c644ace59aa49e31884f5ea6514))

- Improve clustering with adaptive epsilon and FNV-1a hash ([78876c5](https://github.com/d-o-hub/rust-self-learning-memory/commit/78876c5cf71e7f206d232bdbb8998628dee0847f))

- refactor tools into modular structure for better maintainability ([6124430](https://github.com/d-o-hub/rust-self-learning-memory/commit/6124430cc1a86e21e29d39925a78aa4eee2f723f))

- fix keepalive pool Arc ownership and complete 500 LOC refactoring ([c5d4284](https://github.com/d-o-hub/rust-self-learning-memory/commit/c5d428449d12a35a3e654fe8db65e3ecab3899d3))

- fix pattern to track turso storage module ([b06d449](https://github.com/d-o-hub/rust-self-learning-memory/commit/b06d44931f090f050191e5ddbedabef832dfbeae))

- apply cargo fmt to fix formatting issues in benchmarks ([c2a04d2](https://github.com/d-o-hub/rust-self-learning-memory/commit/c2a04d2cf4ddb2f857b14a792ed22c9193686636))

- consolidate skills to .agents/skills directory ([62ace91](https://github.com/d-o-hub/rust-self-learning-memory/commit/62ace91e0f40ff38644a492f5ea0e06149cd6bcc))

- move monitor_actions.sh to scripts/ directory ([0088d5f](https://github.com/d-o-hub/rust-self-learning-memory/commit/0088d5f679119e178874939fc7d81296eaf7fc57))


### Documentation

- update to 100% compliance with Elicitation and Tasks ([03bf787](https://github.com/d-o-hub/rust-self-learning-memory/commit/03bf7874d9f0fe885e1e18d4632804fd4d77ba32))

- add contrastive learning feature to unreleased section ([c05406e](https://github.com/d-o-hub/rust-self-learning-memory/commit/c05406e0c36275bda9ac9049b14508c36aa24e6a))

- mark spatiotemporal retriever refactoring as completed ([07d8fba](https://github.com/d-o-hub/rust-self-learning-memory/commit/07d8fba7888e26e53a9d81c6c13f312081b4a87a))

- add spatiotemporal retriever refactoring entry ([3cca0ff](https://github.com/d-o-hub/rust-self-learning-memory/commit/3cca0ffe9a85b5bc3918bafab29d1141bd05f4ef))

- update status and changelog for extractor refactoring ([2d523dc](https://github.com/d-o-hub/rust-self-learning-memory/commit/2d523dca3ff86d38c61a6c09b08a37c652a5c665))

- add GitHub release workflow documentation ([a0b5483](https://github.com/d-o-hub/rust-self-learning-memory/commit/a0b54834d32075611501f4270a13330057c406d5))

- enhance pre-push checklist with quality gates ([a50d750](https://github.com/d-o-hub/rust-self-learning-memory/commit/a50d7503a049eef1ec64e8689ddeb83227cda3bc))

- prepare v0.1.12 release ([08e60c9](https://github.com/d-o-hub/rust-self-learning-memory/commit/08e60c95403484ea4206a540af152485ab557ff4))

- update to 100% compliance with Elicitation and Tasks ([f14292e](https://github.com/d-o-hub/rust-self-learning-memory/commit/f14292ef2e481d3d9259914513f0f0c071256b6d))

- Add entry for file splitting refactor ([b416e5e](https://github.com/d-o-hub/rust-self-learning-memory/commit/b416e5ea2af7461f9eefccef0df6b952e811d679))

- update test expectations and refresh project documentation ([3d2233a](https://github.com/d-o-hub/rust-self-learning-memory/commit/3d2233acb46a266e8409a83b36dfa3689cfb7bf8))

- update tracker with latest fix for unused function ([292adb4](https://github.com/d-o-hub/rust-self-learning-memory/commit/292adb427087e23114b898b2b231cd3648582880))

- add atomic git commit policy and issue fix guidance to GOAP agent ([52b2920](https://github.com/d-o-hub/rust-self-learning-memory/commit/52b292048e8e4b8c7f8af92c932ff04962d8d3ca))

- add backticks to EpisodeFilter references in comments ([dc9c6ef](https://github.com/d-o-hub/rust-self-learning-memory/commit/dc9c6efad8e4a76bb4618152fd9c647888c6175e))

- clarify test result requirements ([0bcb66a](https://github.com/d-o-hub/rust-self-learning-memory/commit/0bcb66ad221a09cd2b4732f7f7b95d63f5a51a46))

- Add Turso database optimization plan for 6-8x performance improvement ([fb08f4a](https://github.com/d-o-hub/rust-self-learning-memory/commit/fb08f4adfcd9fd7a46a83a2db6046233da2e7210))

- update badges and add security audit badge ([43b0a8d](https://github.com/d-o-hub/rust-self-learning-memory/commit/43b0a8d4a211ca5780fe40a74c848d09ee3da652))

- Update project status, badges, and Phase 2 Turso optimization at 75% ([be22e79](https://github.com/d-o-hub/rust-self-learning-memory/commit/be22e79f27b7f91a432f6cf2e011e0078d0451da))

- update Dependabot triage report with resolution status ([cf053c5](https://github.com/d-o-hub/rust-self-learning-memory/commit/cf053c5ea20ce1c340f97b339a4f994d4cdd5793))

- update GOAP execution plan with Phase 1 completion status ([cccf637](https://github.com/d-o-hub/rust-self-learning-memory/commit/cccf637f9725b9d3770e3b7f31eb19c2b33ca70b))

- index ADR-024 MCP Lazy Tool Loading ([32f50af](https://github.com/d-o-hub/rust-self-learning-memory/commit/32f50af572181a50b23252ff90071b52b939aac7))

- Add README for agent documentation ([ac57993](https://github.com/d-o-hub/rust-self-learning-memory/commit/ac579932d06f0d6675d40c0c8a16bdd7668c6a08))

- Add ADR-025 and GOAP execution plan ([519ed4d](https://github.com/d-o-hub/rust-self-learning-memory/commit/519ed4d65229dc5577be09b1373615b26dddfa64))

- add Skill + CLI pattern documentation to prevent inefficient task orchestration ([a3a8649](https://github.com/d-o-hub/rust-self-learning-memory/commit/a3a86499db958453fd702ed64ce2261b25da7524))

- update ADR-023 status and create Dependabot triage report ([a819e4c](https://github.com/d-o-hub/rust-self-learning-memory/commit/a819e4ca1319b8c5bce62a4dc584258387f62a77))

- add CI/CD issues analysis document ([e13b585](https://github.com/d-o-hub/rust-self-learning-memory/commit/e13b585531848b483571c585afb7e98f01e0bc5c))

- add implementation status report for Feb 13 2026 ([0378ad1](https://github.com/d-o-hub/rust-self-learning-memory/commit/0378ad1eb674f1d706bb854bc92f2a0b4cb3a273))


### Fixed

- replace useless vec! with arrays in retriever tests ([090b938](https://github.com/d-o-hub/rust-self-learning-memory/commit/090b93882bc80376c88d2190e3f49032db02be97))

- remove redundant wildcard pattern in memory-mcp server ([be405ca](https://github.com/d-o-hub/rust-self-learning-memory/commit/be405caa1f0234089f331f47be7ca85b9dfe10aa))

- resolve clippy warnings and update integration tests ([e92d96b](https://github.com/d-o-hub/rust-self-learning-memory/commit/e92d96bcbb1235ebfada71fe2278b9fccffe24e4))

- rename retriever::ScoredEpisode to HierarchicalScore ([776a191](https://github.com/d-o-hub/rust-self-learning-memory/commit/776a191a11edce677c8d1d0cf01844504b325c5d))

- resolve documentation and must_use warnings in retriever module ([b77ea85](https://github.com/d-o-hub/rust-self-learning-memory/commit/b77ea858ee92fba1045dd57af1ceb8257202cac1))

- resolve test compilation warnings ([0732a3d](https://github.com/d-o-hub/rust-self-learning-memory/commit/0732a3d3fe68af7babc4f69c56023f5ed347ca8c))

- resolve warnings in OAuth 2.1 implementation ([e1745d8](https://github.com/d-o-hub/rust-self-learning-memory/commit/e1745d85182447221d781325331f9db7f9eddc77))

- replace useless vec! with arrays in retriever tests ([9a6eb51](https://github.com/d-o-hub/rust-self-learning-memory/commit/9a6eb5145a85ecebe42ea272029f91596f41c55a))

- remove redundant wildcard pattern in memory-mcp server ([55e11a8](https://github.com/d-o-hub/rust-self-learning-memory/commit/55e11a84bce1157c57d5161dccfed43ad71763e5))

- set draft=false for GitHub releases ([7ea40d4](https://github.com/d-o-hub/rust-self-learning-memory/commit/7ea40d4399721f4feb1a09445d9dc9b9302d3dae))

- Un-ignore memory-core/src/retrieval/cache module directory ([8c0e95a](https://github.com/d-o-hub/rust-self-learning-memory/commit/8c0e95adc18aa2e0624683b0d7594a7526369dd5))

- Rename inner test module to avoid clippy module_inception ([febd1e2](https://github.com/d-o-hub/rust-self-learning-memory/commit/febd1e2c4d1f26216053827959f724bf2f0e6f7a))

- resolve clippy warnings ([13ad6e8](https://github.com/d-o-hub/rust-self-learning-memory/commit/13ad6e8ff54d8907158a55d74ecbe90526746b29))

- Resolve GitHub Actions CI failures ([8e6380c](https://github.com/d-o-hub/rust-self-learning-memory/commit/8e6380c6f3588c7fcecabb92bc613a3af9ab0c86))

- Add circuit_breaker module to git tracking ([f5803f9](https://github.com/d-o-hub/rust-self-learning-memory/commit/f5803f92762d32072d3c46cdf9c577eb68a26282))

- Add DiversityMaximizer import to tests ([f6587c4](https://github.com/d-o-hub/rust-self-learning-memory/commit/f6587c4dbca1c8d8f886f2f8888606beb3140a26))

- Resolve clippy warnings and test failures ([bfb4bed](https://github.com/d-o-hub/rust-self-learning-memory/commit/bfb4bed9a92b30203181ce1385176f70badfce4e))

- Add missing cache module files to git tracking ([ce0ad3b](https://github.com/d-o-hub/rust-self-learning-memory/commit/ce0ad3bb001801348d0a6a398270d6c5a8d0d12d))

- Fix rquickjs compilation errors in executor.rs ([04cb06a](https://github.com/d-o-hub/rust-self-learning-memory/commit/04cb06a48bd32002b3be57fc59d61dda4aee7066))

- Fix cache pollution on episode get for non-existent items ([dea6181](https://github.com/d-o-hub/rust-self-learning-memory/commit/dea6181c2d1ba711c6953710a7832e80ff1f2027))

- resolve pre-existing CI failures from PR #221 ([a2c6122](https://github.com/d-o-hub/rust-self-learning-memory/commit/a2c61224449a9734230bbce791b6065391b098c4))

- Change task actions to objects in handle_initialize response ([0a01ddd](https://github.com/d-o-hub/rust-self-learning-memory/commit/0a01ddddf517a05315eb4704d6e8527e0d72dba8))

- Rename server_info to serverInfo in initialize response ([6970cf4](https://github.com/d-o-hub/rust-self-learning-memory/commit/6970cf460c318d1fe0f0e9bef5233b3e006f674b))

- Resolve flaky cache tests and build compatibility ([1911e89](https://github.com/d-o-hub/rust-self-learning-memory/commit/1911e8900fe4349840a23544c2c197b9e7659013))

- resolve race condition in quality_gates unit tests ([ed38939](https://github.com/d-o-hub/rust-self-learning-memory/commit/ed38939b3be76cdf042fc3b2a6ea37b4edfac7c7))

- Add optional title field to McpTool struct ([b4173b2](https://github.com/d-o-hub/rust-self-learning-memory/commit/b4173b296b615cf49638c0e3bb0ab43bff300a31))

- Remove coverage cfg attributes to fix lint errors ([259a4cd](https://github.com/d-o-hub/rust-self-learning-memory/commit/259a4cd2125627b12c02624b46e2c40ad8974bd3))

- Format test file for rustfmt compliance ([7572c2a](https://github.com/d-o-hub/rust-self-learning-memory/commit/7572c2a60225e528f5a19d7bfe210dd9802160f5))

- Update MCP tool count for pattern search feature ([94cf744](https://github.com/d-o-hub/rust-self-learning-memory/commit/94cf7446e0646fc42aff0b7bacb6f43fd739e6fd))

- update test expectations for non-configured state ([428d5ea](https://github.com/d-o-hub/rust-self-learning-memory/commit/428d5ea4d5fe502b43b4026105ba54d43bdf7cbc))

- resolve clippy warnings and test configuration ([1df06bf](https://github.com/d-o-hub/rust-self-learning-memory/commit/1df06bf4222d73c7db4d4c4067ea96ed2083d8f7))

- remove unused create_test_server helper in batch_operations_test ([09a3093](https://github.com/d-o-hub/rust-self-learning-memory/commit/09a30930ba51aebdbd3f3539e37ea1596cd6bc5b))

- add backticks to pattern_search_demo example command ([0763eb3](https://github.com/d-o-hub/rust-self-learning-memory/commit/0763eb3ba7d4b4788cd00583001e857dd1de0f66))

- resolve all warnings for RUSTFLAGS=-D warnings compliance ([8dac697](https://github.com/d-o-hub/rust-self-learning-memory/commit/8dac697f2fca6a8d7446a0d5525d0027877e073e))

- resolve compilation errors from struct refactoring ([76f2e19](https://github.com/d-o-hub/rust-self-learning-memory/commit/76f2e1934675926f90d4b09b03daf16e23913d75))

- Correct clamp parameter order and flush method handling ([1829858](https://github.com/d-o-hub/rust-self-learning-memory/commit/1829858c25fdd80c2aa8ae4a6638d9b4ce57f23f))

- Add missing monitoring/storage module files ([b11da4d](https://github.com/d-o-hub/rust-self-learning-memory/commit/b11da4de074e9203a957f871e95b736b13f788ae))

- Add MPL-2.0 exceptions for wasmtime dependencies ([230254d](https://github.com/d-o-hub/rust-self-learning-memory/commit/230254dbce73206748d324f982cbc092a2baecd3))

- Resolve test compilation issues and fix compiler warnings ([a27786a](https://github.com/d-o-hub/rust-self-learning-memory/commit/a27786a96319c20e36db2c82ac062999cc0861d9))

- Resolve clippy warnings and doc test errors ([ec62dee](https://github.com/d-o-hub/rust-self-learning-memory/commit/ec62dee5e580772e197978b1da8133da7b5c185e))

- Resolve additional clippy warnings in test files ([48f1c94](https://github.com/d-o-hub/rust-self-learning-memory/commit/48f1c940914929151cc7c5cdfcc915ead5251e40))

- Remove redundant stats variable binding in test ([bfdeca2](https://github.com/d-o-hub/rust-self-learning-memory/commit/bfdeca2b1066af0ff3a8d11110f87e0684d0fe36))

- Add backticks to Arc<Episode> in doc comment ([e788b10](https://github.com/d-o-hub/rust-self-learning-memory/commit/e788b104e8683afb2628042a650da8af4ea7206d))

- Remove unused serde imports in openai client ([cd772d0](https://github.com/d-o-hub/rust-self-learning-memory/commit/cd772d0f83311c1760ded8ab01d1008b4b07028f))

- Resolve CLI build errors and MCP test assertions ([1763c06](https://github.com/d-o-hub/rust-self-learning-memory/commit/1763c0673193469a775e2243065b1c8b094d0f8f))

- format regex.rs ([fed3dad](https://github.com/d-o-hub/rust-self-learning-memory/commit/fed3dadee143b90f1f595630028ec8aa28ac706e))

- improve CI timeout handling and test reliability ([d9a98dc](https://github.com/d-o-hub/rust-self-learning-memory/commit/d9a98dc5f78abe9e936b039e4abaa0a6ac442902))

- resolve 10 failing tests in memory-core ([bf77446](https://github.com/d-o-hub/rust-self-learning-memory/commit/bf77446386aa649e8b6013db63370c66ca7bcffc))

- fix cross-platform timeout handling for macOS compatibility ([f8849a3](https://github.com/d-o-hub/rust-self-learning-memory/commit/f8849a3256ed8e376cbd317d1a9f630df883a782))

- resolve all warnings in tests and production code ([3cd2e62](https://github.com/d-o-hub/rust-self-learning-memory/commit/3cd2e62a9011eda8fe1198e3828f42df5bd2a21b))

- move misplaced markdown files to plans/ and update file-structure workflow ([d0d1a8d](https://github.com/d-o-hub/rust-self-learning-memory/commit/d0d1a8db26eb4a5687809a57b83bd8227e184152))

- add newline at end of ci.yml ([9fdfbb6](https://github.com/d-o-hub/rust-self-learning-memory/commit/9fdfbb6872619091f4028d84e697aeb83906dbff))

- ignore flaky WASM timeout enforcement test ([1c3d42a](https://github.com/d-o-hub/rust-self-learning-memory/commit/1c3d42a6e83f84bd710c5b18b09fef849e5c2d94))

- increase test timeouts to prevent premature termination ([f7f0887](https://github.com/d-o-hub/rust-self-learning-memory/commit/f7f088710aa76add1873379215dc632b6513b6cc))

- ignore slow tests that cause CI timeouts ([21019e2](https://github.com/d-o-hub/rust-self-learning-memory/commit/21019e26554a225edcb900a492e4f98db4b5b8be))

- fix SearchConfig default and ignore more slow tests ([540e4e2](https://github.com/d-o-hub/rust-self-learning-memory/commit/540e4e2fa799ecabc512dd920536010a9cd7e484))

- ignore flaky changepoint detection tests ([4f8b688](https://github.com/d-o-hub/rust-self-learning-memory/commit/4f8b68852f3640d6a4d865d98864838780ec68b4))

- resolve clippy warnings and compilation errors ([d114417](https://github.com/d-o-hub/rust-self-learning-memory/commit/d11441789e2f1953f5b10ebeee37b96537497b54))

- resolve 12 clippy warnings in test files ([83608e9](https://github.com/d-o-hub/rust-self-learning-memory/commit/83608e937d070ac9209ad406aa4dc50a07f4e7c6))

- resolve undefined variables in cache tests ([4c2c9ac](https://github.com/d-o-hub/rust-self-learning-memory/commit/4c2c9acf0fbd0f5057b5da00184e64067809603c))

- resolve 4 clippy warnings in cache_benchmarks ([9d36580](https://github.com/d-o-hub/rust-self-learning-memory/commit/9d365805a9aae82398cc045e3625d165bf4e04fc))

- optimize test execution to prevent timeouts ([a76c94d](https://github.com/d-o-hub/rust-self-learning-memory/commit/a76c94d78ad290ba2777c8a23b69535655de94a9))

- add serial_test to env tests for test isolation ([ccb4940](https://github.com/d-o-hub/rust-self-learning-memory/commit/ccb4940d9354e7e313e5b8ac315240ad2b886539))

- correct nextest filter to exclude compliance tests ([0b70eec](https://github.com/d-o-hub/rust-self-learning-memory/commit/0b70eec30f06b53a802e2299171218d86bdda609))

- make test_execution_stats less flaky ([8ca2c08](https://github.com/d-o-hub/rust-self-learning-memory/commit/8ca2c084b1e6beb7a523d4166999754f47dfe1f7))

- resolve compilation errors in cache and query modules ([4a8d2e1](https://github.com/d-o-hub/rust-self-learning-memory/commit/4a8d2e188fff96e976142435ebdf6afe9af1bfec))

- resolve compilation errors in compression_benchmark.rs ([3b4ee50](https://github.com/d-o-hub/rust-self-learning-memory/commit/3b4ee506cdb2068fe7809dcd498cc720fee9b35a))

- resolve clippy issues in keepalive_pool_benchmark.rs ([825c50b](https://github.com/d-o-hub/rust-self-learning-memory/commit/825c50b69f7554ddcf744ecf6d15208e791eb1f6))

- remove unused base64::Engine imports ([59b4714](https://github.com/d-o-hub/rust-self-learning-memory/commit/59b471470845932249d8282104672736d740e6c6))

- use strip_prefix instead of manual string slicing ([7c167db](https://github.com/d-o-hub/rust-self-learning-memory/commit/7c167db61081f787224c2a6d683854383b24dace))

- restore mut group and fix excessive nesting ([96c4017](https://github.com/d-o-hub/rust-self-learning-memory/commit/96c4017bae2362272325abbb4ff4f56cd06507fd))

- use struct init instead of field reassignment to avoid clippy lint ([38bc794](https://github.com/d-o-hub/rust-self-learning-memory/commit/38bc794770797402fc1a0036f218fb4e62e957a9))

- resolve compilation and clippy errors for unified feature builds ([a6a5764](https://github.com/d-o-hub/rust-self-learning-memory/commit/a6a576450496b6611e9e1c2edbb62a094807d5f9))

- track memory-storage-turso/src/cache module ([0952f63](https://github.com/d-o-hub/rust-self-learning-memory/commit/0952f631c481da99d8138df50067e6f9efde32a8))

- resolve needless_update warnings in TursoConfig initialization ([5cb029b](https://github.com/d-o-hub/rust-self-learning-memory/commit/5cb029b2069ee20ba1a23f22a7cb5a21ab7a781c))

- apply rustfmt to all files ([82f2638](https://github.com/d-o-hub/rust-self-learning-memory/commit/82f263836eeac4f2595da37e32575ab72044dce6))

- wrap episodes in Arc for cache.put() calls ([7c37dd6](https://github.com/d-o-hub/rust-self-learning-memory/commit/7c37dd648352ca266f9a24a75d6b6752bc6fd785))

- update query_cache_benchmark to use Arc<Episode> ([2e4a44c](https://github.com/d-o-hub/rust-self-learning-memory/commit/2e4a44c53b00e15cb6229d3a8dbb72f3a19bd37e))

- add feature gates for keepalive tests and fix unwrap ([4673820](https://github.com/d-o-hub/rust-self-learning-memory/commit/467382074be1277d9e99fdcb094ea55315cd1b99))

- remove API keys and update gitleaks ignore ([ea6ab9a](https://github.com/d-o-hub/rust-self-learning-memory/commit/ea6ab9a43ce0dfc0833e82194b65454285d98ffa))

- add additional README commit fingerprints ([684590d](https://github.com/d-o-hub/rust-self-learning-memory/commit/684590d7551dddcc1d807a45363da6501f2ce976))

- add JWT token from settings.local.json ([0bcb386](https://github.com/d-o-hub/rust-self-learning-memory/commit/0bcb38645363e12f68fab440ecbb12dd6dec4be1))

- remove sensitive files from git tracking ([a159f3b](https://github.com/d-o-hub/rust-self-learning-memory/commit/a159f3b381912a50a2e305085a0278561696567f))

- resolve all clippy warnings in test code ([a672efb](https://github.com/d-o-hub/rust-self-learning-memory/commit/a672efbcc4100397fc39f767d8af3aad3a7664b7))

- inline format argument in relationship_manager test ([b672d97](https://github.com/d-o-hub/rust-self-learning-memory/commit/b672d97afd8f1069aa959a9fa0d427cef86f4b8e))

- update time to 0.3.47 and bytes to 1.11.1 ([c1d2e6a](https://github.com/d-o-hub/rust-self-learning-memory/commit/c1d2e6aa1ea32ccddfca3359ccdddd0d5db6fd99))

- skip all flaky sandbox tests in CI environment ([a95cae5](https://github.com/d-o-hub/rust-self-learning-memory/commit/a95cae57bd37b26102d3ff76641e4e4de22ef9cf))

- remove invalid --timeout option from cargo llvm-cov ([eb70948](https://github.com/d-o-hub/rust-self-learning-memory/commit/eb70948d22b29dec194f2542f24033ade6f33cba))

- add disk space maximization to coverage workflow ([bde86ab](https://github.com/d-o-hub/rust-self-learning-memory/commit/bde86abdbe8afa53f6a93a6a1e2565db2bd97d2b))

- resolve criterion deprecation warnings and add compression statistics ([3e8fe9b](https://github.com/d-o-hub/rust-self-learning-memory/commit/3e8fe9b39ba6120ea35278e54c059e03d8b3e0d9))

- remove invalid workflow-level timeout and add macOS disk optimization ([226ef36](https://github.com/d-o-hub/rust-self-learning-memory/commit/226ef36af2b50906d617fa1acc3db1c79cf00d09))

- apply cargo fmt to remaining benchmark files ([da165fe](https://github.com/d-o-hub/rust-self-learning-memory/commit/da165fef007700add3b22e0568354deb679f0c79))

- update cargo config and skill documentation for CI improvements ([f3818a8](https://github.com/d-o-hub/rust-self-learning-memory/commit/f3818a8af9266929dae493fdb887de905c70b3b3))

- add timeout and concurrency controls to benchmarks workflow ([a820012](https://github.com/d-o-hub/rust-self-learning-memory/commit/a82001273917ca68cb89287ba4c4ea1bb5de16dd))

- add free disk space action and exclude benches from coverage ([3666ae8](https://github.com/d-o-hub/rust-self-learning-memory/commit/3666ae8446cfd558a69a5e98a7b2cbc8c35a27bb))

- add allow attributes to benchmark files ([ece4295](https://github.com/d-o-hub/rust-self-learning-memory/commit/ece42955e6b2e9702eb48e0e054d5f6fb1a8c2b4))

- apply cargo fmt to benchmark and test files ([2962f2b](https://github.com/d-o-hub/rust-self-learning-memory/commit/2962f2b3996739e42b2d2f55c7b97a291e36e914))

- remove trailing spaces from benchmarks.yml ([d65ac59](https://github.com/d-o-hub/rust-self-learning-memory/commit/d65ac591820218b2732c5c49703b7299cb25083f))

- remove incorrect metrics module declarations ([e26d75c](https://github.com/d-o-hub/rust-self-learning-memory/commit/e26d75c5e143426513d59c77cb4067140d74cf28))

- add gitleaks ignore for documentation false positive ([3684891](https://github.com/d-o-hub/rust-self-learning-memory/commit/3684891d69c08a2e25730d21a115bb97fff7a412))

- remove incorrectly added tracing module declaration ([35a2747](https://github.com/d-o-hub/rust-self-learning-memory/commit/35a274775bfb2275353f00d501cf41803493a545))

- remove all references to removed tracing module ([a4698a4](https://github.com/d-o-hub/rust-self-learning-memory/commit/a4698a410b59602b3f10a64bf78deb54673acf85))

- replace CorrelationId with Uuid after tracing module removal ([8ba29aa](https://github.com/d-o-hub/rust-self-learning-memory/commit/8ba29aac53eb2b83f0422468e426642555395367))

- enable wasmtime-backend by default and fix conditional compilation ([3c8baf5](https://github.com/d-o-hub/rust-self-learning-memory/commit/3c8baf51a4eb58e59c8d5326ef3a2a6f838f63a4))

- resolve clippy errors and compilation issues ([55b7651](https://github.com/d-o-hub/rust-self-learning-memory/commit/55b7651aacccea32150acc319fafa409de6e2b8b))

- further relax streaming benchmark threshold for larger windows ([bc64907](https://github.com/d-o-hub/rust-self-learning-memory/commit/bc649073e36c77366496589b234da4eb73908a01))

- remove deprecated inputs from wait-on-check-action ([9239a4d](https://github.com/d-o-hub/rust-self-learning-memory/commit/9239a4da36386cfdd0f68c5c7db9b2e5f7c42ffe))


### Maintenance

- quality fixes and MCP protocol upgrade ([0a19d7e](https://github.com/d-o-hub/rust-self-learning-memory/commit/0a19d7ead64c8def9ccf0edb714ac5608c37792e))

- bump deep_causality from 0.13.0 to 0.13.1 (#210) ([11f498c](https://github.com/d-o-hub/rust-self-learning-memory/commit/11f498c055ba4acb2b86a10fd089dc1495eeba15))

- bump clap_complete from 4.5.62 to 4.5.64 (#211) ([28ede30](https://github.com/d-o-hub/rust-self-learning-memory/commit/28ede305e768a3d8a99689dd68e6e2c52ff947ef))

- bump clap from 4.5.53 to 4.5.54 (#207) ([cf02a5b](https://github.com/d-o-hub/rust-self-learning-memory/commit/cf02a5b83b78479ebd041c9a568acb42722d4d18))

- bump reqwest from 0.12.28 to 0.13.1 (#206) ([ed5914c](https://github.com/d-o-hub/rust-self-learning-memory/commit/ed5914ceb28101eae172b9c3ded0fb26c939582f))

- bump lru from 0.12.5 to 0.16.2 (#205) ([5081411](https://github.com/d-o-hub/rust-self-learning-memory/commit/5081411b7fe269420cf0050ba0c6bc0cd378a794))

- bump tokio from 1.48.0 to 1.49.0 (#208) ([e65347f](https://github.com/d-o-hub/rust-self-learning-memory/commit/e65347f18f586167df852d0212c1130dbe20bd60))

- bump tokio-test from 0.4.4 to 0.4.5 (#212) ([1da75c4](https://github.com/d-o-hub/rust-self-learning-memory/commit/1da75c42265fa21eed946b9ed7a1a618c982ba52))

- update paths and configuration for v0.1.12 ([fa68dff](https://github.com/d-o-hub/rust-self-learning-memory/commit/fa68dffa6e985ca130894fcc173159a275b61717))

- Refresh cache module for CI cache invalidation ([3722c2b](https://github.com/d-o-hub/rust-self-learning-memory/commit/3722c2be33342dd203076f5eaac3949e1cbcbe80))

- bump deep_causality from 0.13.1 to 0.13.2 ([594405c](https://github.com/d-o-hub/rust-self-learning-memory/commit/594405c67f917e85442b9d21017514811f6ae0f9))

- bump lru from 0.16.2 to 0.16.3 ([c0254f0](https://github.com/d-o-hub/rust-self-learning-memory/commit/c0254f054502c78916dc16351edaaf4223ab142b))

- bump assert_cmd from 2.1.1 to 2.1.2 ([16d3d5d](https://github.com/d-o-hub/rust-self-learning-memory/commit/16d3d5d669efef06d015cfbdfea9710abf13cb38))

- bump version to v0.1.13 ([c625ec4](https://github.com/d-o-hub/rust-self-learning-memory/commit/c625ec49fc18c975594ff7bcb0e8654a8e9e9ac8))

- remove deployment documentation as workflow has been stabilized ([7c22edf](https://github.com/d-o-hub/rust-self-learning-memory/commit/7c22edf18825d3f1308ab3a5355777d6ef9924c7))

- bump clap from 4.5.54 to 4.5.56 ([199d15f](https://github.com/d-o-hub/rust-self-learning-memory/commit/199d15fbcca151fd71b16bd1e823d7db1e6be9a7))

- bump wasmtime from 40.0.3 to 41.0.1 ([6148851](https://github.com/d-o-hub/rust-self-learning-memory/commit/614885174508670a4a25bc9470c1e06b00393cb9))

- bump sysinfo from 0.37.2 to 0.38.0 ([4f988e0](https://github.com/d-o-hub/rust-self-learning-memory/commit/4f988e0264d3ac9bd3ee8573d4d383c8bfe149ce))

- bump wasmtime-wasi from 40.0.3 to 41.0.1 (#260) ([c795174](https://github.com/d-o-hub/rust-self-learning-memory/commit/c795174708dcd6d720ab9508f28ccc633e660a59))

- update sysinfo, reqwest, and criterion dependencies ([d22c99c](https://github.com/d-o-hub/rust-self-learning-memory/commit/d22c99c07887f9dac68bbafa7f7e477f3b9c6d15))

- remove non-functional playwright-cli skill ([c2ad1ce](https://github.com/d-o-hub/rust-self-learning-memory/commit/c2ad1ce29add4bdc268ea2b5fdbd409345c371a5))

- remove outdated documentation files ([27a4d3b](https://github.com/d-o-hub/rust-self-learning-memory/commit/27a4d3bd316658f6d1b2cfe13de1d3d76086726c))

- cleanup legacy .claude/agents and update skills/docs ([09222d7](https://github.com/d-o-hub/rust-self-learning-memory/commit/09222d71a5fbee012ce21bbcb20a0dd530acad2e))


### Performance

- eliminate double-clones and add Arc caching ([87fb850](https://github.com/d-o-hub/rust-self-learning-memory/commit/87fb850f2f5a4340a9a52f9829f8dd831f49a8a1))

- mark slow integration tests with #[ignore] ([6bfd8fd](https://github.com/d-o-hub/rust-self-learning-memory/commit/6bfd8fd17edf454adcaf2b35441dae92ba507285))

- mark additional slow integration tests with #[ignore] ([604ac52](https://github.com/d-o-hub/rust-self-learning-memory/commit/604ac523c6aff10cc531e04c92cd9bf1fdb4d86a))

- mark more slow integration tests with #[ignore] ([70749d8](https://github.com/d-o-hub/rust-self-learning-memory/commit/70749d814362d94bc53e15c3ef890a1972485f39))


### Testing

- Skip performance tests during coverage runs ([2912a79](https://github.com/d-o-hub/rust-self-learning-memory/commit/2912a79d3a8672763c1515619812ffa5adaf96b9))


### cleanup

- remove deprecated plans and create consolidated INDEX ([a2fdda6](https://github.com/d-o-hub/rust-self-learning-memory/commit/a2fdda6af9adbd7a7cca0472bf1c0881d95cf069))


### merge

- sync develop with main for release fix ([3ed1eaf](https://github.com/d-o-hub/rust-self-learning-memory/commit/3ed1eaf1411a46dca3fc2cd1231f41ab336ee779))


### revert

- Undo incomplete episode deletion trait ([865f1cf](https://github.com/d-o-hub/rust-self-learning-memory/commit/865f1cf5647d26e757254d5a870c721099bace3c))


## [0.1.12] - 2026-01-05



### Added

- implement Elicitation support (MCP 2025-11-25) ([fa06113](https://github.com/d-o-hub/rust-self-learning-memory/commit/fa06113f767e3e264bd0756b6a72667f39399ef2))

- implement Tasks utility and embedding configuration ([076992b](https://github.com/d-o-hub/rust-self-learning-memory/commit/076992b9560183512d1d001c4d4e1e354e4ec035))


### CI/CD

- add pull_request trigger to run CI on PRs before merge ([d8ba202](https://github.com/d-o-hub/rust-self-learning-memory/commit/d8ba20214f8971e8d5347fbc8f27f5c7f46ca99f))

- update ci-guard to allow running CI on pull_request events ([4a5d197](https://github.com/d-o-hub/rust-self-learning-memory/commit/4a5d197c587427afa8c2f95f32da09155c26da0a))


### Documentation

- prepare v0.1.12 release ([98fa2c6](https://github.com/d-o-hub/rust-self-learning-memory/commit/98fa2c6554b7e0e8869ae51b5f886553892ebc31))


### Maintenance

- bump deep_causality from 0.13.0 to 0.13.1 (#210) ([f0e0412](https://github.com/d-o-hub/rust-self-learning-memory/commit/f0e04125dfaacf8c200ac1077d2fb55f4ed2c3ef))

- bump clap_complete from 4.5.62 to 4.5.64 (#211) ([79f10d6](https://github.com/d-o-hub/rust-self-learning-memory/commit/79f10d6189fd0b0c59d92bebaf36727cc3b0539e))

- bump clap from 4.5.53 to 4.5.54 (#207) ([3bdadf8](https://github.com/d-o-hub/rust-self-learning-memory/commit/3bdadf804c313f414f3cd2924983e30e330aa91e))

- bump reqwest from 0.12.28 to 0.13.1 (#206) ([b478686](https://github.com/d-o-hub/rust-self-learning-memory/commit/b478686ef38bd2f767ee2c9d18440b1ce2a51f51))

- bump lru from 0.12.5 to 0.16.2 (#205) ([0144db5](https://github.com/d-o-hub/rust-self-learning-memory/commit/0144db563f89e1187670744658a5f17e3d953bf3))

- bump tokio from 1.48.0 to 1.49.0 (#208) ([75f9de5](https://github.com/d-o-hub/rust-self-learning-memory/commit/75f9de5df471a5b530a1c8a8447ae801a058088b))

- bump libc from 0.2.178 to 0.2.179 ([67a88d9](https://github.com/d-o-hub/rust-self-learning-memory/commit/67a88d9b116ba0f20edf92da32e738d22e2cfbff))

- bump tokio-test from 0.4.4 to 0.4.5 (#212) ([4523221](https://github.com/d-o-hub/rust-self-learning-memory/commit/4523221e2acdd70a4840685324cf775566919abc))


## [0.1.11] - 2026-01-05



### Added

- implement contrastive learning with triplet loss for task adapters ([2b8cb92](https://github.com/d-o-hub/rust-self-learning-memory/commit/2b8cb928cbbdf1ef43df05e561161990f7ef8a22))

- implement Completion utility (MCP 2025-11-25) ([ee883d4](https://github.com/d-o-hub/rust-self-learning-memory/commit/ee883d46355cbb909258fcb79c1c7ab8e240384d))

- implement OAuth 2.1 authorization for production deployments ([57d6543](https://github.com/d-o-hub/rust-self-learning-memory/commit/57d6543a065dfc1f225cb2c0b70412245374436b))

- fix completion utility external tagging and add OAuth 2.1 tests ([1c19dde](https://github.com/d-o-hub/rust-self-learning-memory/commit/1c19dde4000b4dc810504f945b61b140779d04fe))


### Changed

- split retriever.rs into modular structure ([b619770](https://github.com/d-o-hub/rust-self-learning-memory/commit/b6197702eb4f44039293c82738872bc0b2723dd5))

- apply rustfmt formatting ([6bc45ad](https://github.com/d-o-hub/rust-self-learning-memory/commit/6bc45ad6d65f91a4c53100090ae74af9740a6c7f))

- split extractor.rs into modular structure ([95e45d2](https://github.com/d-o-hub/rust-self-learning-memory/commit/95e45d2af714a41e9db7d7c7a756c8719ca74905))


### Documentation

- add contrastive learning feature to unreleased section ([e7692a4](https://github.com/d-o-hub/rust-self-learning-memory/commit/e7692a46ee54408c4fdb411aae82d3fe45d61a74))

- mark spatiotemporal retriever refactoring as completed ([c42b732](https://github.com/d-o-hub/rust-self-learning-memory/commit/c42b732fc7ba10f0190e21c9839508ef7619104e))

- add spatiotemporal retriever refactoring entry ([c505558](https://github.com/d-o-hub/rust-self-learning-memory/commit/c505558f41b4338f12833512696e151d75e69001))

- update status and changelog for extractor refactoring ([b1c80a4](https://github.com/d-o-hub/rust-self-learning-memory/commit/b1c80a468bf21ef862a28d739879358625051874))

- add GitHub release workflow documentation ([afb797d](https://github.com/d-o-hub/rust-self-learning-memory/commit/afb797d59a879fa64e443c5b79aa3ab171e5f890))

- enhance pre-push checklist with quality gates ([c6da7ed](https://github.com/d-o-hub/rust-self-learning-memory/commit/c6da7eddb5c54a44513d1ab3acbeb062c00f7e6a))


### Fixed

- resolve clippy warnings and update integration tests ([19485c3](https://github.com/d-o-hub/rust-self-learning-memory/commit/19485c3dce5a0c9cd0cd99e0665ba9dd6cebd285))

- rename retriever::ScoredEpisode to HierarchicalScore ([5c34dfb](https://github.com/d-o-hub/rust-self-learning-memory/commit/5c34dfb72751c4ecf657f59d6ce5d1e58eb8548b))

- resolve documentation and must_use warnings in retriever module ([e3a9a74](https://github.com/d-o-hub/rust-self-learning-memory/commit/e3a9a7407a031b04254abce4b3285e00b29282ec))

- resolve test compilation warnings ([672d3bd](https://github.com/d-o-hub/rust-self-learning-memory/commit/672d3bd6a2ba37939d181d204e82f3bcfdeeb8dd))

- resolve warnings in OAuth 2.1 implementation ([91581fb](https://github.com/d-o-hub/rust-self-learning-memory/commit/91581fbd3a7d1af4c92cdc5cc6b2c26ea1ec1520))


### Maintenance

- quality fixes and MCP protocol upgrade ([5b96562](https://github.com/d-o-hub/rust-self-learning-memory/commit/5b9656292fda84d1e51ca56bf5d7da7486359090))


## [0.1.11-2026-01-04] - 2026-01-04



### Added

- add codebase-analyzer and codebase-locator agents ([4cd7897](https://github.com/d-o-hub/rust-self-learning-memory/commit/4cd78974b365396a4fe41cdfdaf21c411b809160))


### Changed

- split mod.rs into init/monitoring/queries/tests modules ([03cc45d](https://github.com/d-o-hub/rust-self-learning-memory/commit/03cc45d913b85ea02940d05900cafce0fca677ae))

- split storage.rs into episodes/patterns/heuristics/embeddings modules ([28be5c1](https://github.com/d-o-hub/rust-self-learning-memory/commit/28be5c12cfb01bcc88392bde4a15aa37fb7a4e62))

- split server.rs into tools/monitoring/tests modules ([b71824e](https://github.com/d-o-hub/rust-self-learning-memory/commit/b71824eab6cd51e270cdb68184cc08ec5924468e))

- split large modules into submodules ([9dedbd0](https://github.com/d-o-hub/rust-self-learning-memory/commit/9dedbd07ad4b8180168d7c0b12c015e80e2d0b69))

- split lib.rs tests into separate module ([5c81422](https://github.com/d-o-hub/rust-self-learning-memory/commit/5c8142223acfde1f1c777cf4405e8f4216157cfd))

- split index.rs into types.rs module ([244377f](https://github.com/d-o-hub/rust-self-learning-memory/commit/244377f313adf41215f6c0a8a60ca45551028ecc))

- split tests into separate tests.rs module ([8868ef9](https://github.com/d-o-hub/rust-self-learning-memory/commit/8868ef9a96bcf9ab7ac721b5794c51975fd95d4c))

- run cargo fmt on modified files ([fe13737](https://github.com/d-o-hub/rust-self-learning-memory/commit/fe137379399507ecb5ece0d089fa503715f9fdf8))


### Documentation

- update file splitting progress ([c0fcac4](https://github.com/d-o-hub/rust-self-learning-memory/commit/c0fcac4ffdd836c742417bbf36ab74aad400a229))

- update implementation status for file splitting progress ([0a58031](https://github.com/d-o-hub/rust-self-learning-memory/commit/0a580311feefc10107a6383ebd715efde0ee294c))


### Fixed

- update MCP tool count assertion from 10 to 9 ([7342b0b](https://github.com/d-o-hub/rust-self-learning-memory/commit/7342b0bc0ba899f2975d5949217529aeeb505a10))

- resolve all clippy warnings in memory-core ([a4236de](https://github.com/d-o-hub/rust-self-learning-memory/commit/a4236de176a2ed4a8cc9190bd6b49e3f99826cd7))

- correct test enum variants ([e7b269e](https://github.com/d-o-hub/rust-self-learning-memory/commit/e7b269e9568162b8813c3eaaf0d6a044eb8652e0))

- correct test assertion for query results ([f77fdcb](https://github.com/d-o-hub/rust-self-learning-memory/commit/f77fdcbf8bda88cd3bcce3fd0bd1ebfbf2c98f39))

- resolve warnings in spatiotemporal/types.rs ([d2b9f48](https://github.com/d-o-hub/rust-self-learning-memory/commit/d2b9f48887598b9f03ecf74a8f806cd570becbb7))

- resolve all warnings in spatiotemporal module ([2b59742](https://github.com/d-o-hub/rust-self-learning-memory/commit/2b597428a224b16c4152962c628436afc99bc135))

- add #[allow(dead_code)] to is_valid_wasm_file function ([cb725e0](https://github.com/d-o-hub/rust-self-learning-memory/commit/cb725e04d5757c343c26bd7d3fdd9a6eb69a2288))

- suppress unused import warning in spatiotemporal tests ([1a0b9ac](https://github.com/d-o-hub/rust-self-learning-memory/commit/1a0b9ac7fdf7a99333cd905d4354943be5353d8c))

- resolve CI failures in develop branch ([1febc9c](https://github.com/d-o-hub/rust-self-learning-memory/commit/1febc9ccc1d29d8b02e30568eb01c325268bffb1))

- suppress unused import warning in spatiotemporal tests ([0373337](https://github.com/d-o-hub/rust-self-learning-memory/commit/0373337af11d2a1c3b3f5050b5be9d42f2beefab))

- resolve CI failures in develop branch ([1dbd15c](https://github.com/d-o-hub/rust-self-learning-memory/commit/1dbd15c1071e1965c5cb1a7c892976b87217bf96))


### Maintenance

- remove stale .opencode and plans/GOAP directories ([9250c12](https://github.com/d-o-hub/rust-self-learning-memory/commit/9250c128c7f35d3403f3eb7d06a5a203eb592344))

- update checkout action and add opencode config ([b70c0f6](https://github.com/d-o-hub/rust-self-learning-memory/commit/b70c0f6274b0aac74fe1cc6a8c304159d72d6895))

- update checkout action and add opencode config ([5c39c7d](https://github.com/d-o-hub/rust-self-learning-memory/commit/5c39c7db65f3a57fb5b719e11c7dd6efd1f71314))


### merge

- pull latest from main remote ([50e48cd](https://github.com/d-o-hub/rust-self-learning-memory/commit/50e48cd21f726fec10cc6198614f6c0b7b150113))


## [0.1.10] - 2026-01-03



### Changed

- Fix conditional imports and improve code style ([06542bb](https://github.com/d-o-hub/rust-self-learning-memory/commit/06542bbf08019c3c67a2852393a7feb6a690876c))

- Fix conditional import handling for javy-backend feature ([64680bd](https://github.com/d-o-hub/rust-self-learning-memory/commit/64680bdd0535267db80bb23398eec235b0f140d5))


### Fixed

- update MCP tool count assertions ([3071444](https://github.com/d-o-hub/rust-self-learning-memory/commit/30714449a4ccff458049494b8ce83ca85b776246))

- update MCP tool count assertions ([e71881b](https://github.com/d-o-hub/rust-self-learning-memory/commit/e71881b6223c867f9d15d3a183e98aa9112477df))

- properly serialize episode data for output display ([3c105ea](https://github.com/d-o-hub/rust-self-learning-memory/commit/3c105ea5a65d3a091f403b51165af43e0719535d))

- handle WASM availability in MCP tool tests ([d939060](https://github.com/d-o-hub/rust-self-learning-memory/commit/d939060d2c0c443df66a274f5e5255f3e295e4f1))

- resolve database locking and capacity test issues ([c255183](https://github.com/d-o-hub/rust-self-learning-memory/commit/c255183b6d49a4e1067ea041ec226f85792dc365))

- disable WASM for consistent test tool counts ([cc0072a](https://github.com/d-o-hub/rust-self-learning-memory/commit/cc0072aedc60446a9014e05b74002ce7fa830920))

- remove turso_multi_dimension feature from benches and examples ([e00e8cc](https://github.com/d-o-hub/rust-self-learning-memory/commit/e00e8cc496c32aa2c933a19ac32a4a4bd74f93ea))

- skip cache_stats test on macOS ([3be014b](https://github.com/d-o-hub/rust-self-learning-memory/commit/3be014b995ce31ba647e05705147d1b598dea5de))

- add disk cleanup steps to prevent macOS out-of-space failures ([b2f834c](https://github.com/d-o-hub/rust-self-learning-memory/commit/b2f834c5fb6af37c71b86058f0138553ebcd7d84))


### Maintenance

- bump version to v0.1.10 ([5b7808d](https://github.com/d-o-hub/rust-self-learning-memory/commit/5b7808d3ab6ea15ace731db5c2e7c6e120b4b493))


## [0.1.10-2026-01-02] - 2026-01-02



### Added

- Local database setup and MCP server fixes ([e94271e](https://github.com/d-o-hub/rust-self-learning-memory/commit/e94271edd3aa86d5d759d70e479291c1f26ab72b))

- Add LSP-style framing and unified sandbox support ([ac5e5e0](https://github.com/d-o-hub/rust-self-learning-memory/commit/ac5e5e04995ac38d9d67b6a5d22a7aeae1855bb2))

- Update to latest dependencies and enable 50% WASM usage ([f7ac6d1](https://github.com/d-o-hub/rust-self-learning-memory/commit/f7ac6d16c31493c968be71640a6cc73e0776e8a1))

- add wasmtime sandbox POC for GC-free execution ([0f5753a](https://github.com/d-o-hub/rust-self-learning-memory/commit/0f5753a42345b6454640a5a0a90b7ea41b6c3922))

- add WASI support and fuel-based timeout enforcement ([6ae3a2e](https://github.com/d-o-hub/rust-self-learning-memory/commit/6ae3a2e3b6fb27c62cbd3ee57c5b7787000bb08e))

- wire wasmtime into UnifiedSandbox ([d07478d](https://github.com/d-o-hub/rust-self-learning-memory/commit/d07478d8b284179d3863baedac241a6df91f3391))

- Add comprehensive architecture and code quality analysis for rust-self-learning-memory project ([acde00e](https://github.com/d-o-hub/rust-self-learning-memory/commit/acde00ec5445443bff1ac5d99c91846b2e0514e0))

- enhance episode retrieval and caching mechanisms ([4ffb0a4](https://github.com/d-o-hub/rust-self-learning-memory/commit/4ffb0a4c6964ba2d96746164357afeb07c111893))

- add MCP matrix job to CI workflow and improve job strategy ([5a778be](https://github.com/d-o-hub/rust-self-learning-memory/commit/5a778be59dc9594ad3800b6ea0d9f2195850968c))

- Complete Javy JavaScript-to-WASM Integration - 100% Tests Passing ([352b9b1](https://github.com/d-o-hub/rust-self-learning-memory/commit/352b9b14b56580d2c781926cdaa949ffac5df5cb))

- Implement comprehensive GitHub Actions optimization plan ([e315f13](https://github.com/d-o-hub/rust-self-learning-memory/commit/e315f13efb169f35534ad8bba6c9608c9b61a371))

- integrate develop branch updates into main ([a18e572](https://github.com/d-o-hub/rust-self-learning-memory/commit/a18e5726cd08ff63a3442adb7a89bfb7a8e4e761))

- Complete embeddings refactor and configuration optimization (#162) ([35ad97f](https://github.com/d-o-hub/rust-self-learning-memory/commit/35ad97fc199c67958195903a40b4d8a155c33839))

- Add multi-provider support with circuit breaker and config caching ([8438136](https://github.com/d-o-hub/rust-self-learning-memory/commit/8438136dfeb3e35f85547a0ae0bd519e9621e7b6))

- Add doctest validation to prevent CI failures ([94cddda](https://github.com/d-o-hub/rust-self-learning-memory/commit/94cdddaba653edb1d13d1429b557747619217681))

- Complete phase 3 implementation with multi-embedding support ([799d395](https://github.com/d-o-hub/rust-self-learning-memory/commit/799d39576a828af9b92747a1fdd5491d3deb0c3a))

- Add multi-dimension and FTS5 hybrid search ([a926bcc](https://github.com/d-o-hub/rust-self-learning-memory/commit/a926bccbf175a2351f6be4694e348b2e50fada81))

- Add dimension column to embeddings tables for multi-dimension tracking ([9249c00](https://github.com/d-o-hub/rust-self-learning-memory/commit/9249c00ec8b880fdc588f82eb9b0bea9372ef3a7))

- add delete_episode method to TursoStorage ([9f6cf44](https://github.com/d-o-hub/rust-self-learning-memory/commit/9f6cf448ef67b263478ce5775fa11215f355bccf))

- add store_episode_summary and get_episode_summary methods ([3914a3f](https://github.com/d-o-hub/rust-self-learning-memory/commit/3914a3fb0d61980cdf56aae7f782e6f5f9f98fef))


### CI/CD

- bump softprops/action-gh-release from 2.4.2 to 2.5.0 ([0085a73](https://github.com/d-o-hub/rust-self-learning-memory/commit/0085a733d04d14cc3e4af09395e22dbfb311d3f8))

- bump codecov/codecov-action from 5.5.1 to 5.5.2 ([09bede4](https://github.com/d-o-hub/rust-self-learning-memory/commit/09bede4863e63e8cb877203021d4d5e200dbacd7))

- prevent disk exhaustion and loosen flaky tests; fix wasm doctest and tests ([982444a](https://github.com/d-o-hub/rust-self-learning-memory/commit/982444a3e83eb152838bee3d0203ea2cbfb04f71))

- run Security workflow for all branches and add manual dispatch ([227e02d](https://github.com/d-o-hub/rust-self-learning-memory/commit/227e02dd2a01ef0337daae9ce5954a9608f4c3a3))

- update deprecated GitHub Actions versions ([134935b](https://github.com/d-o-hub/rust-self-learning-memory/commit/134935b6b7dedc908da27beda90891533b040b99))

- switch actions/checkout to v4 in both jobs ([4958482](https://github.com/d-o-hub/rust-self-learning-memory/commit/495848286f6e1a537f37c7d22f7fb74944bd4d67))

- update all workflows to 2025 best practices ([cd38473](https://github.com/d-o-hub/rust-self-learning-memory/commit/cd3847374491f9790e77476b5066580fab1b2bc7))

- bump actions/cache from 4.3.0 to 5.0.1 ([3b6619e](https://github.com/d-o-hub/rust-self-learning-memory/commit/3b6619e408f87a4de4bef383bf8aab96b5582cc8))

- bump actions/upload-artifact from 4 to 6 (#163) ([40d1606](https://github.com/d-o-hub/rust-self-learning-memory/commit/40d16061b47944f331a83458e01ce77e86ca561f))

- bump actions/download-artifact from 4 to 7 (#165) ([ad6e9ba](https://github.com/d-o-hub/rust-self-learning-memory/commit/ad6e9ba197430d411bc1e7e6b43e7028e340d27f))

- bump actions/setup-python from 5 to 6 (#164) ([6e50ffc](https://github.com/d-o-hub/rust-self-learning-memory/commit/6e50ffcbab84f670c8ed106627ab816d3a65bb7a))


### Changed

- fix formatting to pass CI checks ([f4eceef](https://github.com/d-o-hub/rust-self-learning-memory/commit/f4eceef964bff1942da186c984837983f3aed39b))

- Fix rustfmt formatting issues in test files ([298d132](https://github.com/d-o-hub/rust-self-learning-memory/commit/298d132ce21e13f0c1d73e351078e28ca6d693a8))

- improve error handling in integration tests ([2fa453a](https://github.com/d-o-hub/rust-self-learning-memory/commit/2fa453ac6dea9ed87f792e29064161d5ca427047))

- Modernize base64 API usage ([11dcb75](https://github.com/d-o-hub/rust-self-learning-memory/commit/11dcb75775eb389fbd2db81d62db115692e2363a))

- Add namespace support to embedding storage interface ([f1fcd3f](https://github.com/d-o-hub/rust-self-learning-memory/commit/f1fcd3fe707c50746a9b1a3a193a6b44e7c88f0b))

- fix formatting and clippy warnings ([a12b5cf](https://github.com/d-o-hub/rust-self-learning-memory/commit/a12b5cf88b7a5310e8ad427606fbec3cc36dce3c))

- fix formatting and clippy warnings for sysinfo update ([6c2499a](https://github.com/d-o-hub/rust-self-learning-memory/commit/6c2499a69af7198164765bb37343d55e81268bc5))

- fix clippy useless_format warning in tests ([e11087b](https://github.com/d-o-hub/rust-self-learning-memory/commit/e11087b0d7094fb057d51ce53980bda18227a00e))

- remove unused imports in phase1_validation test ([b12d8c7](https://github.com/d-o-hub/rust-self-learning-memory/commit/b12d8c7310baa9dec12c42c890df56527b6b7d4c))

- fix clippy useless_vec warning in semantic_retrieval_test ([06223d4](https://github.com/d-o-hub/rust-self-learning-memory/commit/06223d4c1c48e66cf2944d006b5e6e72c41cea9d))

- update embedding API calls to 2-parameter signature ([2ac2674](https://github.com/d-o-hub/rust-self-learning-memory/commit/2ac2674e61a9a777bc0c0f60f3cffcce6d46e970))

- fix clippy assertions_on_constants warnings in constants tests ([b4c0cdb](https://github.com/d-o-hub/rust-self-learning-memory/commit/b4c0cdb07272697c4ff43bb588f15b730939b6dd))

- add #[allow] for assertions_on_constants in constants tests ([896171a](https://github.com/d-o-hub/rust-self-learning-memory/commit/896171a8528a677078dab2f081629e32f3132202))

- add #[allow] for excessive_nesting in security_tests ([9865786](https://github.com/d-o-hub/rust-self-learning-memory/commit/98657868c172c3c97f6b0599d8231126f9bc9cfe))

- add #[allow] for excessive_nesting in quality_gates.rs ([6cd5843](https://github.com/d-o-hub/rust-self-learning-memory/commit/6cd58434c25d3f48792e554cf44a1953c338dbf5))

- fix clippy warnings (unreadable_literal, excessive_nesting) ([3f10f52](https://github.com/d-o-hub/rust-self-learning-memory/commit/3f10f5235cb0d01270636bcb3ed0ac8e651bbc15))

- fix clippy warnings and clean up test assertions ([f1cb13c](https://github.com/d-o-hub/rust-self-learning-memory/commit/f1cb13c001c89926ff029d490bc17ed6df95ca80))

- split episode and pattern commands into submodules ([16b4016](https://github.com/d-o-hub/rust-self-learning-memory/commit/16b4016a1261190beb1826758266dfb9e5fa5504))

- split episode and pattern commands into submodules ([1dbc876](https://github.com/d-o-hub/rust-self-learning-memory/commit/1dbc876b3c61eb7598cc9769e38e277be8c37cbb))

- run cargo fmt on all files ([1341826](https://github.com/d-o-hub/rust-self-learning-memory/commit/1341826844f4a0d745210d2a1be4e36e7b77718e))

- run cargo fmt ([91de45c](https://github.com/d-o-hub/rust-self-learning-memory/commit/91de45c4e7260b7ad865a65fc63933196c4408aa))


### Documentation

- Add MCP server validation and finalization documentation ([8019854](https://github.com/d-o-hub/rust-self-learning-memory/commit/8019854a04196de8eb4560d422fb7800bfdc97d1))

- Add final production readiness report ([48fb644](https://github.com/d-o-hub/rust-self-learning-memory/commit/48fb6441d6197a50977ad3ecc4c9da10c54a459e))

- Update release plan with security fixes ([8d6c524](https://github.com/d-o-hub/rust-self-learning-memory/commit/8d6c524d442095397cb8947a7f1e8ab73ff076b3))

- add comprehensive integration documentation ([1bc4db1](https://github.com/d-o-hub/rust-self-learning-memory/commit/1bc4db1d393fdd59ebfd5488cad7e717ff733d0a))

- add Phase 2C Javy integration GOAP plan ([87e5c41](https://github.com/d-o-hub/rust-self-learning-memory/commit/87e5c411c6fe6374ef8df6b39b89c89e133b57a8))

- update documentation and add analysis files ([e1f9a66](https://github.com/d-o-hub/rust-self-learning-memory/commit/e1f9a665c5aa2edc67446c10f3e7f722991a3b79))

- Restructure documentation and update clippy configuration ([98ed40c](https://github.com/d-o-hub/rust-self-learning-memory/commit/98ed40c1cb6789ab0884c319445471a332db1e83))

- Add field renaming guidance to commit messages ([261daa1](https://github.com/d-o-hub/rust-self-learning-memory/commit/261daa1c4a9f54db09af0beb287135c4e27c36dc))

- reorganize documentation and archive v0.1.7-v0.1.9 roadmaps ([63d4437](https://github.com/d-o-hub/rust-self-learning-memory/commit/63d44373aa8f9690e649bd7fff036de032715244))

- Add comprehensive GitHub Actions issues analysis ([7c30894](https://github.com/d-o-hub/rust-self-learning-memory/commit/7c3089431d7869094f38755afc9cd20f66926698))

- update archive index with new folders ([3a2a5d1](https://github.com/d-o-hub/rust-self-learning-memory/commit/3a2a5d183a3ec62cda18713695c48064645a7157))


### Fixed

- Suppress unused_assignments warning for last_input_was_lsp ([a0e3ce7](https://github.com/d-o-hub/rust-self-learning-memory/commit/a0e3ce7e7613ae2a5f3c1c41cfa47053fb2de4b7))

- Fix inputSchema serialization + project cleanup + GOAP verification ([770289f](https://github.com/d-o-hub/rust-self-learning-memory/commit/770289fb2f9ee4a445a38f719d460f06683ffbfb))

- Ignore QuickJS tests with GC issues ([ec6a2b8](https://github.com/d-o-hub/rust-self-learning-memory/commit/ec6a2b8527471fe7f38b7706572410dfb1e67ae7))

- Remove invalid #[ignore] attribute on test module ([ccf12f2](https://github.com/d-o-hub/rust-self-learning-memory/commit/ccf12f29d5d37da82c63b51d4cc4171dc0b2d431))

- Remove leaked JWT token and update secret scanning ignores ([461d8bc](https://github.com/d-o-hub/rust-self-learning-memory/commit/461d8bc45513b1b6d0065629a6472a7e0e855114))

- Allow BSL-1.0 license and ignore wasmtime advisories ([675914f](https://github.com/d-o-hub/rust-self-learning-memory/commit/675914f205f042aa1b53b6f5b6e95433bca8a7a4))

- Upgrade wasmtime to 24.0.5 to resolve security advisories ([e9c9b5e](https://github.com/d-o-hub/rust-self-learning-memory/commit/e9c9b5e487354e585bb81818de98e4c8e22f01bf))

- Ignore test_correlation_calculation due to rquickjs GC issue ([131d914](https://github.com/d-o-hub/rust-self-learning-memory/commit/131d914d132fed79b03627123b5f08e1917948be))

- Replace rustsec/audit-check with direct cargo audit ([01e0ea4](https://github.com/d-o-hub/rust-self-learning-memory/commit/01e0ea40cbba231417c59abb15ffc594ccb0e3fd))

- disable WASM sandbox in all tests to prevent rquickjs GC crashes ([16246f8](https://github.com/d-o-hub/rust-self-learning-memory/commit/16246f84b4d203e055f2f99eef31501708915e25))

- allow 0ms performance timing for fast operations ([bc11a4d](https://github.com/d-o-hub/rust-self-learning-memory/commit/bc11a4d3c22edb912bf06b02886e1d78fb53bac9))

- remove absurd extreme comparisons for unsigned integers ([5d907cb](https://github.com/d-o-hub/rust-self-learning-memory/commit/5d907cb6b0263c1b311d12f52760b0549ad79236))

- add --test-threads=4 to prevent flaky test failures ([1b6edd5](https://github.com/d-o-hub/rust-self-learning-memory/commit/1b6edd5134c10c4625fc6b2d62e3a501e7e6883f))

- fix --test-threads flag placement in workflow ([69615cd](https://github.com/d-o-hub/rust-self-learning-memory/commit/69615cdca5d17ee6df1041085b95570194234aba))

- disable WASM sandbox to prevent rquickjs GC crashes ([b28df93](https://github.com/d-o-hub/rust-self-learning-memory/commit/b28df932dbc875e9781e51adefbea90c21cc3400))

- update test to use wasmtime_pool_stats field ([d6bd181](https://github.com/d-o-hub/rust-self-learning-memory/commit/d6bd1816a90fb1647d69d016ef8f869a599199f7))

- use struct initialization for WasmtimeConfig ([f9f0587](https://github.com/d-o-hub/rust-self-learning-memory/commit/f9f05874da3029b7c9f7373ac99c9c038e9099bd))

- Fix all quality gate test failures for v0.1.7 ([b2f6e55](https://github.com/d-o-hub/rust-self-learning-memory/commit/b2f6e55dcea5c703138f0fccb59a33d5ab5ab387))

- Update test job to use ubuntu-latest and add steps for checkout and Rust toolchain installation ([f2953c7](https://github.com/d-o-hub/rust-self-learning-memory/commit/f2953c7f3553d8e80664bb6e503bcf229a8b2814))

- Update .gitleaksignore to exclude JWT token from commit 144e3a2 ([a48d6b6](https://github.com/d-o-hub/rust-self-learning-memory/commit/a48d6b64dd9e08fa02c4c8a3d01cccbf0cdf4ce7))

- Resolve all clippy warnings and errors ([40fbec6](https://github.com/d-o-hub/rust-self-learning-memory/commit/40fbec6155d3ad005b55cf48301ca2a003447387))

- modernize workflow using 2025 best practices ([e88026c](https://github.com/d-o-hub/rust-self-learning-memory/commit/e88026cb5788d30544ac3382888e8a6ed74aaae1))

- ignore documentation examples in gitleaks ([ffd5ee4](https://github.com/d-o-hub/rust-self-learning-memory/commit/ffd5ee45823a4048d31650a873c56cf8b1461f1a))

- update to 2025 best practices and fix issues ([ec364e2](https://github.com/d-o-hub/rust-self-learning-memory/commit/ec364e2b9932f99d36e83137bd5ce4b4f88fc2b4))

- ignore broken WASM backend tests that fail with String::from_utf8 ([5867021](https://github.com/d-o-hub/rust-self-learning-memory/commit/5867021e279f655ce324660933650852fe2292f0))

- revert actions/checkout to v4 ([43f4021](https://github.com/d-o-hub/rust-self-learning-memory/commit/43f4021fad7cf21b5c462dea19d249f871ca1de9))

- add path validation to prevent path traversal and sensitive file access ([274a09d](https://github.com/d-o-hub/rust-self-learning-memory/commit/274a09d2c60f0bd215a63d7e369338ff8882f5e0))

- apply rustfmt and update documentation ([a39adba](https://github.com/d-o-hub/rust-self-learning-memory/commit/a39adba6b144a481d69b19901da88c99e9df34b3))

- add path validation to prevent path traversal and sensitive file access ([be1eed2](https://github.com/d-o-hub/rust-self-learning-memory/commit/be1eed2fed4f0939b033bb43d441ece2ae388419))

- add quality config for async extraction tests ([1a22b36](https://github.com/d-o-hub/rust-self-learning-memory/commit/1a22b36e5b00b5f9f45b518e7f385a0d9c75d75c))

- fix formatting and clippy configuration ([ac1124b](https://github.com/d-o-hub/rust-self-learning-memory/commit/ac1124b3f98b42860efe6156958242313cf685ef))

- resolve clippy warnings and formatting issues ([06b3c70](https://github.com/d-o-hub/rust-self-learning-memory/commit/06b3c701fc9473a81c0bd34b7bbbf5ec7a3c5d71))

- add allow(dead_code) to unused function ([10b777f](https://github.com/d-o-hub/rust-self-learning-memory/commit/10b777f6139074b4cae541f99f4e5dfbe43781a1))

- remove unused import ([e603603](https://github.com/d-o-hub/rust-self-learning-memory/commit/e603603676bbd6dc8ac32399a91767062443b024))

- add quality_threshold to performance and step_batching tests ([ed7f6f0](https://github.com/d-o-hub/rust-self-learning-memory/commit/ed7f6f0761232e5dca7c04a26303418ba58f973f))

- lower quality threshold for test episodes across memory-mcp and memory-core ([f2a197d](https://github.com/d-o-hub/rust-self-learning-memory/commit/f2a197de8b1d6f21e23b600e9e8267068869db62))

- update doctests to fix CI compilation errors ([27db9c1](https://github.com/d-o-hub/rust-self-learning-memory/commit/27db9c151a29006128082858596fb337b7c8a9f5))

- use inlined format args and allow MPL-2.0 license ([0746219](https://github.com/d-o-hub/rust-self-learning-memory/commit/074621950e9f6f80074174c04418e705a02ea86d))

- replace unwrap() with expect() in pool integration tests ([648f3cd](https://github.com/d-o-hub/rust-self-learning-memory/commit/648f3cd529d6f93dcef26464e21d1a1077efda0b))

- Improve documentation testing and CI reliability ([7f868a8](https://github.com/d-o-hub/rust-self-learning-memory/commit/7f868a8f9ec35da27d7d13a2d668d3f90769f5ca))

- Prevent lock conflicts in parallel test execution ([f8ccdcc](https://github.com/d-o-hub/rust-self-learning-memory/commit/f8ccdcc074509bf4ba7309cca467cc45d1df657c))

- Remove invalid remove_artifacts parameter from Release workflow ([9897ff3](https://github.com/d-o-hub/rust-self-learning-memory/commit/9897ff3a9e6a4d27399b9edcc901d45be6dbcd9d))

- Skip benchmark jobs on Dependabot PRs to prevent timeouts ([a8f2bd3](https://github.com/d-o-hub/rust-self-learning-memory/commit/a8f2bd35f3f07dbd8d0999f6b0f4c546cad06c5f))

- add excessive_nesting allow attribute ([2a97782](https://github.com/d-o-hub/rust-self-learning-memory/commit/2a97782e65ef4cdb3f9fae5a49be9c385e045b15))

- update embeddings_end_to_end.rs to match current API ([dcb217c](https://github.com/d-o-hub/rust-self-learning-memory/commit/dcb217cb01d73e75ef8207d1f311bc8ba558f67c))

- update benchmarks and MCP tools for current API ([62cb983](https://github.com/d-o-hub/rust-self-learning-memory/commit/62cb9830f9a34f53a34e0219de9afda7f0056151))

- resolve fmt and clippy warnings in benchmarks and examples ([398a2d7](https://github.com/d-o-hub/rust-self-learning-memory/commit/398a2d74a9911b4c928506ae22274451c309c1f4))

- fix f64 comparison in embeddings_integration test ([4f9e1d8](https://github.com/d-o-hub/rust-self-learning-memory/commit/4f9e1d8bd00ec8c9b9665c1cb5349086d0166984))

- resolve clippy warnings in benchmarks and tests ([08a7c46](https://github.com/d-o-hub/rust-self-learning-memory/commit/08a7c46a4254dd65ee3a04919afa930cadab2673))


### Maintenance

- bump uuid from 1.18.1 to 1.19.0 ([d04a836](https://github.com/d-o-hub/rust-self-learning-memory/commit/d04a836b9866108270290fbf874b35287ce383da))

- bump reqwest from 0.12.24 to 0.12.25 ([021221e](https://github.com/d-o-hub/rust-self-learning-memory/commit/021221e281760fb7c45abf32c84daf06e9fbe7b4))

- expand gitignore patterns for database files ([a3ee956](https://github.com/d-o-hub/rust-self-learning-memory/commit/a3ee956b0fefd5014d0d3553a9394b543525a2bd))

- stop tracking .claude/settings.local.json ([8f451dc](https://github.com/d-o-hub/rust-self-learning-memory/commit/8f451dc2492061c705557972ee11bd9680279eb4))

- Bump version to 0.1.7 - Phase 2C Complete ([7074c7f](https://github.com/d-o-hub/rust-self-learning-memory/commit/7074c7f806991b6d3188922e30c85fb19556d0d0))

- Prepare v0.1.6 release ([40a9b88](https://github.com/d-o-hub/rust-self-learning-memory/commit/40a9b88890fd1da325c89e4382f926372b1191bf))

- prepare 0.1.6.1 ([f610e3c](https://github.com/d-o-hub/rust-self-learning-memory/commit/f610e3c850463b435d4f3ed2b223baa215a77df3))

- add sccache to release workflow and document loop-agent work ([17eedec](https://github.com/d-o-hub/rust-self-learning-memory/commit/17eedec278b215a57c4791943a799b4270227bd8))

- bump version to 0.1.7 ([168576b](https://github.com/d-o-hub/rust-self-learning-memory/commit/168576b2d8fe346db6ba8efe28f810f8877785a4))

- bump reqwest from 0.12.25 to 0.12.26 (#171) ([aaf783b](https://github.com/d-o-hub/rust-self-learning-memory/commit/aaf783b5a397243a219ea7a748e6846c6f07dc80))

- bump rquickjs from 0.6.2 to 0.10.0 (#168) ([25b1e06](https://github.com/d-o-hub/rust-self-learning-memory/commit/25b1e0667924c599ed9701045e464bb89848a95a))

- bump deep_causality from 0.11.11 to 0.12.2 ([30d4134](https://github.com/d-o-hub/rust-self-learning-memory/commit/30d4134d68dd6f2c42d3264e177768d64a6c57aa))

- bump augurs from 0.6.3 to 0.10.1 ([dabc872](https://github.com/d-o-hub/rust-self-learning-memory/commit/dabc872b620ab74c7cf10f319638e6cd91bd76fc))

- bump tempfile from 3.23.0 to 3.24.0 ([9c79b0f](https://github.com/d-o-hub/rust-self-learning-memory/commit/9c79b0fca1265aa781fd8e4d9969f0266cd29d02))

- bump tokenizers from 0.20.4 to 0.22.2 ([7172f0e](https://github.com/d-o-hub/rust-self-learning-memory/commit/7172f0e7e9bb1b08db9953462be60eb09043adf1))

- bump dirs from 5.0.1 to 6.0.0 ([39f2605](https://github.com/d-o-hub/rust-self-learning-memory/commit/39f2605b0e43a604e47025859b9875bb13b72d85))

- trigger CI ([6612d20](https://github.com/d-o-hub/rust-self-learning-memory/commit/6612d202ca584876429e05bd7886b956cbb4ef44))

- bump serde_json from 1.0.146 to 1.0.148 ([dfecaf9](https://github.com/d-o-hub/rust-self-learning-memory/commit/dfecaf9ae9ecd7ea1e3347d9e7780d2581a5fcae))

- bump rquickjs from 0.10.0 to 0.11.0 ([8300743](https://github.com/d-o-hub/rust-self-learning-memory/commit/8300743b6f89fdba461d71ad09511f1a39925db9))

- bump ndarray from 0.16.1 to 0.17.1 ([92b0bf7](https://github.com/d-o-hub/rust-self-learning-memory/commit/92b0bf7ad8c590e0f4317dab4a42ce5897d91494))

- bump deep_causality from 0.12.2 to 0.13.0 ([ee01ebe](https://github.com/d-o-hub/rust-self-learning-memory/commit/ee01ebec537339446e328d684628db8bfecdee0a))

- bump sysinfo from 0.30.13 to 0.37.2 ([a2d6001](https://github.com/d-o-hub/rust-self-learning-memory/commit/a2d6001847b0f609bb2858a82c102caeeb36d18c))


### Testing

- Add comprehensive and sequential test scripts ([9c1f3c9](https://github.com/d-o-hub/rust-self-learning-memory/commit/9c1f3c9ecee13534b7b825dc6219278c380e3dc3))

- fix capacity tests to match actual API and add get_episode_by_task_desc ([8af3aa5](https://github.com/d-o-hub/rust-self-learning-memory/commit/8af3aa54d8630e1d9245c80c6fa3831c8aa652da))


### config

- Add MCP server configuration file ([f6ded78](https://github.com/d-o-hub/rust-self-learning-memory/commit/f6ded78cd1a38220615349ab5f617bf444719f35))


### fmt

- Fix formatting in test_utils.rs ([42da13e](https://github.com/d-o-hub/rust-self-learning-memory/commit/42da13e2a6e945b72eb6606a66c61a73bc31ccc2))

- reorder imports in unified_sandbox.rs for rustfmt compliance ([58aadd6](https://github.com/d-o-hub/rust-self-learning-memory/commit/58aadd6579cb7ebb28d822724e6a13887c387f21))


### release

- update workspace version and changelog ([e9c7e1d](https://github.com/d-o-hub/rust-self-learning-memory/commit/e9c7e1d96794384f99d94986d13235e5a52990e5))

- update workspace version and changelog (#190) ([f76a102](https://github.com/d-o-hub/rust-self-learning-memory/commit/f76a1026b7c3b314035de03e56b0691ea4249096))


## [0.1.5] - 2025-12-08



### Added

- Implement v0.1.3 CLI interface with full functionality and testing (#111) ([dde3b74](https://github.com/d-o-hub/rust-self-learning-memory/commit/dde3b74acd274f5efed63286d19be62f66b4634e))

- v0.1.4 - CLI Quality-of-Life Improvements & Enhanced Error Handling (#115) ([0a23b02](https://github.com/d-o-hub/rust-self-learning-memory/commit/0a23b02bbd0067c639d6c404968c9942a99a4c83))


### CI/CD

- bump actions/checkout from 5 to 6 (#117) ([05617f0](https://github.com/d-o-hub/rust-self-learning-memory/commit/05617f02bd2f58ec07c52c3d064ee710435a1240))

- bump lewagon/wait-on-check-action from 1.3.4 to 1.4.1 (#118) ([e75daa5](https://github.com/d-o-hub/rust-self-learning-memory/commit/e75daa57923c72b165e04ca727958a4687778549))

- skip optional quality gates by default + add unit tests (#130) ([3fb8c40](https://github.com/d-o-hub/rust-self-learning-memory/commit/3fb8c401485eccf7a281429e327ec91f7e24c2c5))

- bump actions/download-artifact from 4.6.2 to 6.0.0 (#133) ([ef6dd5f](https://github.com/d-o-hub/rust-self-learning-memory/commit/ef6dd5f1fbbe16c2ca54b1d136cd49767cac28e2))

- bump reviewdog/action-actionlint from 1.69.0 to 1.69.1 (#134) ([577bf81](https://github.com/d-o-hub/rust-self-learning-memory/commit/577bf8178259baa2dd3a989ed010d4fd01551074))

- bump actions/upload-artifact from 4.6.2 to 5.0.0 ([be797c4](https://github.com/d-o-hub/rust-self-learning-memory/commit/be797c4e0c9d65f7a0fc4777db7c09d9a33501fd))


### Documentation

- add memory-cli agent and skill for Claude Code (#114) ([631231e](https://github.com/d-o-hub/rust-self-learning-memory/commit/631231e04ab50f48dcf4dc4b94e2ae533f230edf))

- Add GitHub Actions monitoring and resolution reports ([590db46](https://github.com/d-o-hub/rust-self-learning-memory/commit/590db46883ff412561b8e16737d4ab065f055e7e))


### Fixed

- resolve format and clippy issues in PR #103 ([54831b9](https://github.com/d-o-hub/rust-self-learning-memory/commit/54831b91c2569732eceae1f1a542d3e35a4d76ec))

- upgrade indicatif to resolve unmaintained dependency (RUSTSEC-2025-0119) (#113) ([6af6a1c](https://github.com/d-o-hub/rust-self-learning-memory/commit/6af6a1c2898eec95d7efa795b14d2a0d9d1c5f00))

- remove unsafe semaphore lifetime hack; ensure pool shutdown in integration tests to avoid Windows crashes (#126) ([c3853db](https://github.com/d-o-hub/rust-self-learning-memory/commit/c3853db509d0e62d998344d5d0f5cd643f3c6a75))

- Set version to 0.1.5 for next development ([5955f3e](https://github.com/d-o-hub/rust-self-learning-memory/commit/5955f3edad8e2f7265fef65946e3c63c3f8ffc89))

- Update reqwest to 0.12 to resolve rustls-pemfile vulnerability ([059f8b3](https://github.com/d-o-hub/rust-self-learning-memory/commit/059f8b36d7fb6c2b123d862e0a15126946b4cffd))

- Adjust main branch coverage threshold to 70% ([3d4dd93](https://github.com/d-o-hub/rust-self-learning-memory/commit/3d4dd93d77101191999d99af18373c2da8b7a87c))

- Adjust memory leak threshold to realistic value ([325cbb8](https://github.com/d-o-hub/rust-self-learning-memory/commit/325cbb83ac75e4b04f55a23cb7aea2bb3693e8ff))

- resolve GitHub Actions failures ([997781f](https://github.com/d-o-hub/rust-self-learning-memory/commit/997781f77739eb3c21df8cf5d82d6e5387999f2e))

- align workspace versions and standardize coverage thresholds ([cfba39d](https://github.com/d-o-hub/rust-self-learning-memory/commit/cfba39de788bfff1f604fbd522918870d6a373d7))

- handle uncommitted Cargo.lock changes ([baa9f7c](https://github.com/d-o-hub/rust-self-learning-memory/commit/baa9f7c74f5209190d88e55c0c1127688721c68d))

- resolve failed parse server response error ([5c797d8](https://github.com/d-o-hub/rust-self-learning-memory/commit/5c797d819d27bade28176945df8328f9c555eb0a))

- resolve failed parse server response error ([ac2d5ef](https://github.com/d-o-hub/rust-self-learning-memory/commit/ac2d5efec5a8c418e13afd58e6c76d71c5765c1b))

- Fix YAML trailing spaces in benchmarks workflow ([d33e0b7](https://github.com/d-o-hub/rust-self-learning-memory/commit/d33e0b7eddae9a298d2114d5208db4d66c81d1c9))


### Maintenance

- bump sysinfo from 0.30.13 to 0.37.2 (#109) ([77f36ae](https://github.com/d-o-hub/rust-self-learning-memory/commit/77f36aead58cf31e2df429f118f388897c64628e))

- remove opencode-worktree submodule reference (#110) ([ba29459](https://github.com/d-o-hub/rust-self-learning-memory/commit/ba29459fdc34719d4420c3fc84680d183fb0288e))

- bump toml from 0.8.23 to 0.9.8 (#125) ([1395408](https://github.com/d-o-hub/rust-self-learning-memory/commit/1395408638d284d186c58c574eecfdd2e48e451b))

- bump dialoguer from 0.11.0 to 0.12.0 (#124) ([62778da](https://github.com/d-o-hub/rust-self-learning-memory/commit/62778da428eee114484f32fd1de333daa19e3a6c))

- bump libsql from 0.9.27 to 0.9.29 (#123) ([193756e](https://github.com/d-o-hub/rust-self-learning-memory/commit/193756ef1b828404720b40828c9ddc2bff16a571))

- bump clap_complete from 4.5.60 to 4.5.61 (#121) ([aea5ffd](https://github.com/d-o-hub/rust-self-learning-memory/commit/aea5ffd0250739f41decabdf12972850121bf38a))

- bump rstest from 0.18.2 to 0.26.1 (#120) ([f846241](https://github.com/d-o-hub/rust-self-learning-memory/commit/f846241476f791a559eff51187318be214ef719e))

- bump clap from 4.5.51 to 4.5.53 (#119) ([0b5465e](https://github.com/d-o-hub/rust-self-learning-memory/commit/0b5465ecb3f21ba4dc3dcae83e109db2f2cac400))

- bump colored from 2.2.0 to 3.0.0 (#122) ([7c56c94](https://github.com/d-o-hub/rust-self-learning-memory/commit/7c56c9412f3b26c3a2c3fce4b2b45142f3327a65))

- bump tracing from 0.1.41 to 0.1.43 (#138) ([1b5e8cf](https://github.com/d-o-hub/rust-self-learning-memory/commit/1b5e8cf866ff199e6ed8cab84390e4741aa2b7a3))

- bump libc from 0.2.177 to 0.2.178 ([6acf590](https://github.com/d-o-hub/rust-self-learning-memory/commit/6acf5901e0a32c91a71a453bc60b31f70d0ac9c7))

- bump tracing-subscriber from 0.3.20 to 0.3.22 ([a979144](https://github.com/d-o-hub/rust-self-learning-memory/commit/a9791441f931f461e917888fe65153679af93f76))


### release

- v0.1.2 - Code quality improvements (#106) ([d7edbe3](https://github.com/d-o-hub/rust-self-learning-memory/commit/d7edbe3d3c4740b9eb011504f39254d0aa54ce2b))


## [0.1.1] - 2025-11-14



### Added

- v0.1.1 patch release - architectural improvements ([9b55c35](https://github.com/d-o-hub/rust-self-learning-memory/commit/9b55c353691dcf576b8b8f609c7bbc5f6f201d23))


### Documentation

- Update v0.1.0 release status documentation (#101) ([9235ac2](https://github.com/d-o-hub/rust-self-learning-memory/commit/9235ac23d9b100541b6f4daa8a760488f4b33945))


## [0.1.0] - 2025-11-14



### Added

- add comprehensive GOAP-based implementation plans for all 6 phases ([5a04bda](https://github.com/d-o-hub/rust-self-learning-memory/commit/5a04bda71a52b9c5543d0b5d293463c53f941e4e))

- initialize Rust workspace with core data structures ([f233bdc](https://github.com/d-o-hub/rust-self-learning-memory/commit/f233bdc20bae442997e9059621d3c4eb31cd5a4c))

- add goap-agent skill for intelligent task planning ([08c4629](https://github.com/d-o-hub/rust-self-learning-memory/commit/08c462999aecc5c3762064758d5a8ce78ebb04d8))

- implement complete self-learning memory system (Phases 3-5) ([9b96d24](https://github.com/d-o-hub/rust-self-learning-memory/commit/9b96d24695ed3a58ec76feff55a522a0da874c8c))

- implement zero-trust security architecture ([fe55c77](https://github.com/d-o-hub/rust-self-learning-memory/commit/fe55c7732c71bbb5dcad5ca9a8cff5ad4147b64b))

- complete all missing tasks from plans folder (Phases 3-5) (#41) ([1b889bf](https://github.com/d-o-hub/rust-self-learning-memory/commit/1b889bf1ce690baa251d49a9d4241a6656065457))

- complete package publishing preparation (Phase 1) (#43) ([b173a30](https://github.com/d-o-hub/rust-self-learning-memory/commit/b173a3013820c64234df0c6a0b32e01ceb3ebceb))

- add Analysis Swarm agent and skill for multi-perspective code analysis (#47) ([54d151c](https://github.com/d-o-hub/rust-self-learning-memory/commit/54d151c5769a74f8dacf9fa132ab89093b73f533))

- create web-search-researcher Claude Code skill (#52) ([b04c586](https://github.com/d-o-hub/rust-self-learning-memory/commit/b04c586d6f3a7ea473ca005432276a8b55b86f78))

- enhance web-search-researcher skill to 5/5 rating (#53) ([dca8d5b](https://github.com/d-o-hub/rust-self-learning-memory/commit/dca8d5bc823b24e63cdbdedaa5246a53de9f5520))

- add codebase-consolidation skill for comprehensive code analysis (#65) ([bf1705a](https://github.com/d-o-hub/rust-self-learning-memory/commit/bf1705af75126de2b0173620718ef6298cb0dc3b))

- enhance github-workflows skill with verification-first approach and release management (#81) ([4df2b8d](https://github.com/d-o-hub/rust-self-learning-memory/commit/4df2b8d520b5dbb823a5c7d345f50d614d313c80))

- make architecture-validator generic and self-learning (#85) ([7bb046d](https://github.com/d-o-hub/rust-self-learning-memory/commit/7bb046dad35f1922f8c7b94357332cc20bf2dc47))

- Complete Phase 1 implementation and organize project files (#84) ([a16a56d](https://github.com/d-o-hub/rust-self-learning-memory/commit/a16a56d5ad024a3193495df9510fa1412e9b81e5))

- Complete P0 security improvements (#86) ([043ff4d](https://github.com/d-o-hub/rust-self-learning-memory/commit/043ff4d3385a13ba45174ecb9fa9142703e534e6))

- Enhance path canonicalization to handle non-existent paths and improve symlink resolution (#89) ([60dfc1a](https://github.com/d-o-hub/rust-self-learning-memory/commit/60dfc1a1b3ad3d024a14a3a53943ebaa8ca680f2))

- Complete P0 blockers for v0.1.0 release (#94) ([ca9fdd5](https://github.com/d-o-hub/rust-self-learning-memory/commit/ca9fdd57c536da44118bdf1aadb3fbe82c8b23ad))

- update testing documentation and adjust code coverage requirement to >80% (#98) ([c2f3efa](https://github.com/d-o-hub/rust-self-learning-memory/commit/c2f3efaba68ac01bba523132ca7e9471e8f66e9e))


### CI/CD

- setup GitHub Actions workflows and contributing guidelines ([0601378](https://github.com/d-o-hub/rust-self-learning-memory/commit/0601378018a7d69d321811dfe71d1688d6f65bdb))

- fix GitHub Actions workflows and create github-workflows skill ([d607b3f](https://github.com/d-o-hub/rust-self-learning-memory/commit/d607b3f890654c311e9924e4099df7caa7daebd1))

- bump actions/checkout from 4 to 5 ([50448d7](https://github.com/d-o-hub/rust-self-learning-memory/commit/50448d78abc108cf74dd580b30566efe10584082))

- bump actions/download-artifact from 4 to 6 ([4f85260](https://github.com/d-o-hub/rust-self-learning-memory/commit/4f852608201f1ce3ff6eb96e0b0d339ac91a0a56))

- bump codecov/codecov-action from 4 to 5 ([60b6f43](https://github.com/d-o-hub/rust-self-learning-memory/commit/60b6f43048b92c5a97e05e47331d8f10ffdeb79b))

- use only stable Rust toolchain in test matrix (#34) ([1e36975](https://github.com/d-o-hub/rust-self-learning-memory/commit/1e36975620032064d347190a67674860a2664e95))

- add disk space cleanup to coverage and quality-gates jobs (#92) ([a65524f](https://github.com/d-o-hub/rust-self-learning-memory/commit/a65524fc6f9da5bef4d914697743420c2fa35e8f))

- bump actions/setup-python from 5 to 6 (#97) ([7d43911](https://github.com/d-o-hub/rust-self-learning-memory/commit/7d4391125bb9e84bcc847a0ca1c9635bcd6cfa80))


### Changed

- apply rustfmt formatting ([03283ae](https://github.com/d-o-hub/rust-self-learning-memory/commit/03283aee471c2f8e06137bed8b4e8b29cebb25b6))

- optimize CI pipeline with two-stage workflow (#36) ([4a9cd54](https://github.com/d-o-hub/rust-self-learning-memory/commit/4a9cd5483942de1cbfdfae651ae3b1984e11cda7))


### Documentation

- complete security review and fix hook syntax ([7a137e5](https://github.com/d-o-hub/rust-self-learning-memory/commit/7a137e5bb3a5f8d2a4a21a591aaaea9fe9620eab))

- enhance README with navigation, prerequisites, and configuration (#42) ([1dca33a](https://github.com/d-o-hub/rust-self-learning-memory/commit/1dca33a85bc7ba697afad76d0312c029de6d7262))

- Add opencode agent and skills documentation (#91) ([e06ab34](https://github.com/d-o-hub/rust-self-learning-memory/commit/e06ab34e58c10dd60aa89f0fa625d5e3c996e870))

- Update plans folder with GOAP gap analysis and production readiness plan (#93) ([70b7372](https://github.com/d-o-hub/rust-self-learning-memory/commit/70b7372c8cad7668844b2b3002c2564ee45173a4))


### Fixed

- improve workflow with toolchain matrix, coverage, and build timing ([aea406b](https://github.com/d-o-hub/rust-self-learning-memory/commit/aea406ba7b949165f2ac9d25564cb86f1092e78d))

- resolve GitHub Actions workflow failures ([9b1a120](https://github.com/d-o-hub/rust-self-learning-memory/commit/9b1a12000fef91948fdf1f4e7cb34451f86a1b92))

- update deny.toml to cargo-deny v2 configuration format ([0848d1b](https://github.com/d-o-hub/rust-self-learning-memory/commit/0848d1baa476e8f58ab45ca2f077d3d12fc66c81))

- resolve cargo-deny and Windows test failures ([a460d27](https://github.com/d-o-hub/rust-self-learning-memory/commit/a460d2737bbef3bf4e9f1cbea72c231988e4c21b))

- allow CDLA-Permissive-2.0 license for webpki-roots ([c8d8c1a](https://github.com/d-o-hub/rust-self-learning-memory/commit/c8d8c1a0765c22ba3a0091496b4bbfb925035a02))

- add version specifications to path dependencies ([b828e42](https://github.com/d-o-hub/rust-self-learning-memory/commit/b828e42d282fb765e4b30490c4ff6d0c4f4bf84e))

- fix health_check test by using query() instead of execute() ([e018dc4](https://github.com/d-o-hub/rust-self-learning-memory/commit/e018dc4d8ffd3fc81e8c7a1c61c77554285ac1c8))

- resolve import issues in isolation.rs (#37) ([d0f7422](https://github.com/d-o-hub/rust-self-learning-memory/commit/d0f74228e0c905de9b7cd24979a8563eb02ca9c1))

- resolve compiler warnings in isolation.rs (#48) ([d379b6b](https://github.com/d-o-hub/rust-self-learning-memory/commit/d379b6b2829ed8de369574385aa7cfb2809d2a71))

- clarify Skills vs Task Agents in agent-coordination skill (#49) ([8bef3d6](https://github.com/d-o-hub/rust-self-learning-memory/commit/8bef3d6e9c32490c6285b878e0b1a8477f7994f2))

- convert Criterion benchmark output to bencher format (#56) ([12b379b](https://github.com/d-o-hub/rust-self-learning-memory/commit/12b379b809ad9d42f24eca53ff4d7a8da51cec33))

- resolve GitHub Actions workflow failures (#59) ([284d32d](https://github.com/d-o-hub/rust-self-learning-memory/commit/284d32d7436686e84b0ef3d1b844c5fdbc76870f))

- add explicit CodeQL workflow for Rust language scanning (#61) ([7779e34](https://github.com/d-o-hub/rust-self-learning-memory/commit/7779e340b1f380a1b38b4fe5b9aef3b073b38563))

- correct Criterion benchmark output format for github-action-benchmark (#64) ([ff02543](https://github.com/d-o-hub/rust-self-learning-memory/commit/ff02543f2afc642d100ca23338ebd7d249aee7ee))

- resolve benchmark timeout issues and broken pipe errors (#79) ([5b01642](https://github.com/d-o-hub/rust-self-learning-memory/commit/5b01642d0409095a187ecfb3b087b46726c72760))

- resolve benchmark timeout issues and broken pipe errors (#78) ([31a1a7f](https://github.com/d-o-hub/rust-self-learning-memory/commit/31a1a7fd4a1a6bbc57dc30ee3330dd2b15d21cc5))

- remove invalid unwrap calls on start_episode return value (#80) ([a437c23](https://github.com/d-o-hub/rust-self-learning-memory/commit/a437c237c9c505627e4ddafedab5b38a1b59f328))

- Apply 2025 best practices and fix critical issues (#88) ([b7ea613](https://github.com/d-o-hub/rust-self-learning-memory/commit/b7ea613e1def3829dfff9a77824c3fa346a84c62))

- Make heuristic module public to fix failing doctests (#90) ([a2fab71](https://github.com/d-o-hub/rust-self-learning-memory/commit/a2fab715ace308f63cfbe8b1a94ee318097a77d6))

- correct YAML format in create-agent.md and add git-worktree-manager agent (#100) ([f28afa8](https://github.com/d-o-hub/rust-self-learning-memory/commit/f28afa8b106b71b8ee93e119943b08c0889f0c33))


### Maintenance

- bump libsql from 0.6.0 to 0.9.24 ([ec34fd4](https://github.com/d-o-hub/rust-self-learning-memory/commit/ec34fd421050791b7d5d185e19785792524e5674))

- bump thiserror from 1.0.69 to 2.0.17 ([460b609](https://github.com/d-o-hub/rust-self-learning-memory/commit/460b609878ed22d53bf00b9a91070a370c28d8eb))

- remove cargo-geiger from project (#57) ([bd5e388](https://github.com/d-o-hub/rust-self-learning-memory/commit/bd5e3882e5b58a4293a3377a2199d08531b10dd4))

- prepare v0.1.0 release with 2025 best practices (#62) ([de2bdaa](https://github.com/d-o-hub/rust-self-learning-memory/commit/de2bdaa7f6b44cc66947fbadcf0de368344f10bd))

- bump libsql from 0.9.24 to 0.9.27 (#73) ([70f7887](https://github.com/d-o-hub/rust-self-learning-memory/commit/70f788786aeab60a105f6f5b624d47c2af9ea965))

---
