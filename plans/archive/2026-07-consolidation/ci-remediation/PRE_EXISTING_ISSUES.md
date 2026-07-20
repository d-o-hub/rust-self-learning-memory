# Pre-Existing Issues

Documented: 2026-06-09 | Updated after PR #611 fix session
Previous: 2026-05-16 | See `plans/GOAP_PRE_EXISTING_ISSUES_FOLLOWUP_2026-06-09.md` for detailed plan

## Blocking Issues

None currently. All clippy `-D warnings` violations across the PR #611 affected
crates (`do-memory-core`, `do-memory-mcp`, `do-memory-cli`, `do-memory-storage-turso`,
`do-memory-storage-redb`, `do-memory-test-utils`, `do-memory-benches`) have been fixed.

## Recently Fixed (2026-06-09)

| Issue | File(s) | Fix |
|-------|---------|-----|
| `rand::thread_rng` removed in rand 0.10 | `benches/cosine_similarity_benchmark.rs` | Changed to `rand::rng()` + `RngExt::random_range()` |
| `criterion::black_box` deprecated | `benches/cosine_similarity_benchmark.rs` | Changed to `std::hint::black_box` |
| `uninlined_format_args` (6 locations across 4 files) | `memory-mcp/tests/tool_contract_parity.rs`, `memory-mcp/src/bin/server_impl/tools/memory_handlers_tests.rs`, `memory-core/src/extraction/tests.rs`, `examples/local_memory.rs` | Inlined variables in format strings |
| `field_reassign_with_default` | `examples/local_memory.rs` | Replaced with struct literal `..Default::default()` |
| `doc_markdown` | `examples/local_memory.rs`, `benches/cosine_similarity_benchmark.rs` | Added `#![allow(clippy::doc_markdown)]` |
| Config validation — unrecognized `storage_mode` | `memory-cli/src/config/validator/rules/database.rs` | Added validation rule + 2 tests |

## Non-Blocking Warnings

### 1. GOAP Missing Plan Files (OUTDATED — all files exist)
**Severity**: Informational — quality gate needs updating
**Details**: `GOAP_AGENT_IMPROVEMENT_PLAN.md`, `GOAP_PERFORMANCE_PLAN.md`, and
`GOAP_SECURITY_PLAN.md` all exist in `plans/`. The quality gate check is stale.

### 2. Root `llms.txt` Location
**Severity**: Non-blocking (informational)
**Details**: Per llms.txt standard, root placement is correct for AI agent discovery.

### 3. Docs Integrity — Broken Links
**Severity**: Non-blocking (informational)
**Details**: The `check-docs-integrity.sh` script reports broken internal links across
documentation and plan files. Scheduled for a dedicated docs cleanup sprint (v0.1.33 candidate).

### 4. Test File Sizes (>500 LOC)
**Severity**: Non-blocking (tests exempt from LOC gate)
**Details**: 54 test files exceed the 500 LOC threshold. Expected for comprehensive test suites.

### 5. Sprint Backlog (v0.1.32 open WGs)
**Severity**: Non-blocking for PR #611
**Source**: `plans/GOAP_STATE.md`
**Details**: 6 WGs remain open from the v0.1.32 missing-implementation sprint:
- WG-156: `pattern_match_score` hard-coded 0.8
- WG-157: `memory_usage_mb` hard-coded 50.0
- WG-158: `episode_success_rate` hard-coded 99.0
- WG-160: Turso cache query_hits/evictions = 0
- WG-161: Cascade `analyze_query` stub
- WG-162: `generate_simple_embedding` placeholder

**Follow-up plan**: `plans/GOAP_PRE_EXISTING_ISSUES_FOLLOWUP_2026-06-09.md`
