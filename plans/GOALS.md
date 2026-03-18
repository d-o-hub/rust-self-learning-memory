# GOAP Goals Index

- **Last Updated**: 2026-03-11 (post-v0.1.17 revalidation)
- **Source ADR**: ADR-037
- **Status**: Active

## Current Goals

1. **WG-001**: Add docs integrity automation
   - Priority: P1
   - Owner: code-quality + documentation
   - Target: add `scripts/check-docs-integrity.sh` and wire into quality gates
   - Status: Complete

2. **WG-002**: Add release operations wrapper
   - Priority: P1
   - Owner: github-release-best-practices + release-guard
   - Target: add `scripts/release-manager.sh`
   - Status: Complete

3. **WG-003**: Introduce lightweight GOAP state index
   - Priority: P2
   - Owner: goap-agent
   - Target: establish `GOALS.md`, `ACTIONS.md`, `GOAP_STATE.md`
   - Status: Complete

4. **WG-004**: Add machine-readable architecture context contract
   - Priority: P2
   - Owner: codebase-analyzer + yaml-validator
   - Target: add `docs/architecture/context.yaml` and validation hook
   - Status: Complete

5. **WG-005**: Stabilize PR #334 checks after ADR-037 rollout
   - Priority: P1
   - Owner: github-workflows + code-quality
   - Target: clear format/yaml/quick-check chain failures on PR validation
   - Status: Complete (PR #334 merged 2026-03-06)

6. **WG-006**: Merge Dependabot dependency updates
   - Priority: P1
   - Owner: cicd-engineer
   - Target: merge PRs #344, #345, #346 once CI passes
   - Status: Partial (PR #344, #346 merged; #345 blocked by breaking changes)

7. **WG-007**: Fix rust-major breaking changes (PR #345)
   - Priority: P2
   - Owner: rust-expert
   - Target: resolve test/clippy failures from dependency update
   - Status: Complete (PR #345 merged with redb 3.x + rand 0.10 fixes)

## Next Goals (v0.1.18 Sprint)

8. **WG-008**: Triage 121 ignored tests
   - Priority: P0
   - Owner: test-runner + debug-troubleshoot
   - Target: reduce ignored tests to ≤30 (remove, fix, or document with tracking)
   - Status: Pending

9. **WG-009**: Resolve batch MCP tool state
   - Priority: P0
   - Owner: memory-mcp
   - Target: remove unreachable batch schemas or re-enable handlers
   - Status: ✅ Complete (PR #357 merged 2026-03-11)

10. **WG-010**: Error handling reduction (memory-core)
    - Priority: P1
    - Owner: code-quality
    - Target: reduce unwrap/expect in memory-core from 165 to ≤50
    - Status: ✅ Complete - Analysis revealed production code already follows best practices

11. **WG-011**: Dependency deduplication
     - Priority: P1
     - Owner: code-quality
     - Target: reduce duplicate dep roots from 134 to <100
     - Status: ✅ Complete - Removed unused libsql, target not achievable for architectural reasons

## Next Goals (v0.1.19 Sprint — ADR-040)

12. **WG-012**: Fix nightly test exclusion filter
    - Priority: P0
    - Owner: ci-engineer
    - Target: Add compression/keepalive/phase1 Turso tests to nightly exclusion filter
    - Status: ✅ Complete - Changed to binary() filters for integration test exclusion

13. **WG-013**: Fix changelog workflow
    - Priority: P0
    - Owner: ci-engineer
    - Target: Fix git-cliff install step; add checkout to notify-failure job
    - Status: ✅ Complete - Simplified to cargo install git-cliff --locked

14. **WG-014**: Disable ci-old ghost workflow
    - Priority: P0
    - Owner: ci-engineer
    - Target: Remove ghost workflow reference from GitHub
    - Status: ✅ Complete - Already disabled_manually via GitHub UI

15. **WG-015**: Upgrade Swatinem/rust-cache to v2.9+
    - Priority: P1
    - Owner: ci-engineer
    - Target: Update all 10 workflow references before Node.js 20 deprecation (June 2026)
    - Status: ✅ Complete - Already at v2.9.1 across all workflows

16. **WG-016**: ~~Wire rate limiter to production MCP server~~
    - Priority: P1
    - Status: ✅ N/A — Deep analysis confirmed already implemented (`server/mod.rs:83`, `RateLimiter::from_env()`)

17. **WG-017**: ~~Wire embedding config to production~~
    - Priority: P1
    - Status: ✅ N/A — Deep analysis confirmed already implemented (`jsonrpc.rs:28-128`)

18. **WG-018**: Audit and reduce dead_code attributes
    - Priority: P1
    - Owner: code-quality
    - Target: Reduce from 79 to ≤20 `#[allow(dead_code)]` in production source
    - Status: Pending — many attrs are on stale duplicate types in `types.rs`

19. **WG-019**: Remove stale TODO comments and duplicate modules
    - Priority: P1
    - Owner: code-quality
    - Target: Remove misleading TODOs on implemented features; remove duplicate `embedding.rs`
    - Status: Pending — `types.rs:22,81,138,315,332` have TODOs for features in `mcp/` submodule

20. **WG-020**: Fix stale #[ignore] reason on pattern CLI e2e test
    - Priority: P1
    - Owner: test-runner
    - Target: Update `cli_workflows.rs:554` — pattern CLI is fully implemented
    - Status: Pending

21. **WG-021**: Update ADR-039 "Not Built" table with corrections
    - Priority: P2
    - Owner: docs
    - Target: Correct ADR-039 — 5 of 6 "Not Built" items are actually implemented
    - Status: Pending

## Next Goals (v0.1.20 Sprint — ADR-041)

22. **WG-022**: Fix memory-storage-redb compilation errors
    - Priority: P0
    - Owner: code-quality
    - Target: `cargo build --all` compiles cleanly (4 errors + 3 warnings)
    - Root Cause: Missing `use crate::cache::{CacheConfig, CacheMetrics, LRUCache}` in lib.rs; broken `super::super::CacheConfig` path in adaptive/mod.rs
    - Status: ✅ Complete — build and clippy both pass

23. **WG-023**: Fix stale `#[ignore]` reasons and placeholder issue URLs
    - Priority: P1
    - Owner: test-runner
    - Target: All `#[ignore]` annotations have accurate reasons; 0 placeholder `issues/XXX` URLs remain
    - Status: ✅ Complete — commit `bf7abab` (ACT-021)

24. **WG-024**: Refactor nightly workflow exclusion filter
    - Priority: P1
    - Owner: ci-engineer
    - Target: Replace 18 per-test-name exclusions with category-based (crate/binary) filters
    - Status: ✅ Complete — nightly uses `package(memory-storage-turso)` filter (commit `c70db69`)

25. **WG-025**: Un-ignore fixable tests
    - Priority: P2
    - Owner: test-runner
    - Target: Reduce ignored tests from 119 to ≤112 (un-ignore pattern CLI e2e, fix sandbox timing, fix WASM binary data)
    - Status: 🟡 Partial — 119→118 (pattern CLI e2e un-ignored); 6 sandbox/WASM tests still pending

26. **WG-026**: Add ignored-test ceiling CI guard
    - Priority: P2
    - Owner: ci-engineer
    - Target: CI script that fails if `#[ignore]` count exceeds 125 (prevents silent growth)
    - Status: ✅ Complete — `scripts/check-ignored-tests.sh` (commit `e66f4e0`)

    ## Next Goals (v0.1.20 Sprint — ADR-042: Code Coverage Improvement)

    27. **WG-027**: Critical path coverage
    - Priority: P0
    - Owner: test-runner + quality-unit-testing
    - Target: All critical business logic modules have basic tests; no module below 50% coverage
    - Status: ✅ Complete — ACT-026, ACT-027, ACT-028, ACT-029

    28. **WG-028**: Property test expansion
    - Priority: P1
    - Owner: test-runner + quality-unit-testing
    - Target: Property tests for all serializable types; calculator properties validated; fuzz tests pass
    - Status: 🟡 Partial — ACT-030 (serialization) and ACT-031 (calculator) complete; ACT-032 (fuzz) pending

    29. **WG-029**: Integration coverage
    - Priority: P1
    - Owner: test-runner + quality-unit-testing
    - Target: CLI integration tests cover all commands; MCP tool tests cover all tools; storage tests cover error paths
    - Status: Pending — ACT-033, ACT-034, ACT-035

    30. **WG-030**: Coverage configuration and monitoring
    - Priority: P2
    - Owner: ci-engineer
    - Target: Realistic codecov targets; coverage monitoring script reporting coverage by crate
    - Status: ✅ Complete — ACT-036 (codecov config `be75d0a`), ACT-037 (`scripts/check-coverage.sh` `34d81f4`)

## Next Goals (v0.1.22 Sprint — Quality & Feature Polish)

31. **WG-040**: Fix failing doctests
    - Priority: P0
    - Owner: code-quality
    - Target: `cargo test --doc --all` passes (0 failures)
    - Root Cause: attribution doctest moves session value; playbook doctest `.await`s sync fn
    - Status: ✅ Complete — ACT-053, ACT-054 — Issue #374 closed

32. **WG-041**: Fix test timeout
    - Priority: P0
    - Owner: test-runner
    - Target: `cargo nextest run --all` passes with 0 timeouts
    - Root Cause: `quality_gate_no_clippy_warnings` runs full clippy internally (>120s)
    - Status: ✅ Complete — ACT-055 — Issue #375 closed

33. **WG-042**: Split production files >500 LOC
    - Priority: P0
    - Owner: code-quality
    - Target: 0 production source files >500 LOC
    - Files: `generator.rs` (631→500), `memory_handlers.rs` (608→426), `management.rs` (504→370)
    - Status: ✅ Complete — ACT-056, ACT-057, ACT-058 — Issue #376 closed

34. **WG-043**: Reduce dead_code annotations
    - Priority: P1
    - Owner: code-quality
    - Target: ≤40 `#[allow(dead_code)]` in production code (from 70)
    - Status: ⏳ Pending — ACT-059, ACT-060, ACT-061

35. **WG-044**: Fix broken markdown links
    - Priority: P1
    - Owner: docs
    - Target: ≤80 broken links (from 149)
    - Status: ⏳ Pending — ACT-062, ACT-063

36. **WG-045**: Add snapshot tests for new features
    - Priority: P1
    - Owner: test-runner
    - Target: ≥80 snapshots (from 65); cover new MCP tools + CLI commands
    - Status: ⏳ Pending — ACT-064, ACT-065

37. **WG-046**: Add property tests for new features
    - Priority: P1
    - Owner: test-runner
    - Target: ≥13 property test files (from 10)
    - Status: ⏳ Pending — ACT-066, ACT-067, ACT-068

38. **WG-047**: MCP tool contract parity for new tools
    - Priority: P2
    - Owner: memory-mcp
    - Target: All new tools verified in tool_contract_parity.rs
    - Status: ⏳ Pending — ACT-069, ACT-070

39. **WG-048**: Integration tests for new features
    - Priority: P2
    - Owner: test-runner
    - Target: End-to-end tests for attribution + checkpoint flows
    - Status: ⏳ Pending — ACT-071, ACT-072

40. **WG-049**: Changelog automation (git-cliff)
    - Priority: P2
    - Owner: ci-engineer
    - Target: Auto-generate changelog entries on release
    - Status: ⏳ Pending — ACT-073

41. **WG-050**: Documentation for new features
    - Priority: P2
    - Owner: docs
    - Target: Usage examples for playbook, checkpoint, attribution in docs/
    - Status: ⏳ Pending — ACT-074, ACT-075
