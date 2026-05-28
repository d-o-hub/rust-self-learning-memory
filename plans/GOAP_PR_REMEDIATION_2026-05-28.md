# GOAP PR Remediation — 2026-05-28

**Status**: Complete
**Goal**: Resolve all open PRs, address all unresolved review comments, get all CI green
**Result**: All 3 PRs fixed and pushed; CI re-triggered

---

## Open Inventory

| PR | Title | Author | Issues Resolved |
|----|-------|--------|-----------------|
| #589 | Optimize Turso batch embedding operations | Jules | ✅ Transaction leak (HIGH) — rollback on serde/get_or_prepare errors<br/>✅ IN clause limit (MED) — chunked 500/batch<br/>✅ Added large batch tests for >500 items |
| #590 | docs: sync README and agent docs | Jules | ✅ Inconsistent code examples fixed — unified to actual async API<br/>✅ TaskContext fields aligned with struct definition |
| #591 | feat: close residual placeholder WGs | d-o-hub | ✅ Codacy ErrorProne — unused `task_file` in `goap-orchestrator.sh`<br/>✅ Coverage CI disk space — increased root-reserve to 8GB, added docker/swigpl cleanup |

## Changes Made

### PR #589
- `memory-storage-turso/src/storage/embeddings_internal.rs`: Replaced `?` operators in `_store_embeddings_batch_internal` with explicit `match` + ROLLBACK. Added `chunk_item_ids`/`in_clause_placeholders` helpers (500/batch), applied to `_get_embeddings_batch_internal` and `_delete_embeddings_batch_internal`.
- `memory-storage-turso/src/storage/embeddings_multi.rs`: Applied chunking to `delete_embeddings_batch_dimension_aware`.
- `memory-storage-turso/src/tests.rs`: Added `test_get_embeddings_batch_large`, `test_delete_embeddings_batch_large`, `test_store_embeddings_batch_rollback_on_statement_error`.

### PR #590
- `README.md`: Fixed second code example — `SelfLearningMemory::new(Default::default()).await?` → `SelfLearningMemory::new()`. Added missing `complexity`, `framework` fields and `Some()` to `language`.

### PR #591
- `scripts/goap-orchestrator.sh`: Used `task_file` parameter in log output.
- `.github/workflows/coverage.yml`: Increased `root-reserve-mb` 512→8192, added `remove-docker-images` and `remove-swigpl`.
