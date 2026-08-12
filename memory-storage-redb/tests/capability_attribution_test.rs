//! Attribution capability advertisement test (ADR-081 §2; historical #940
//! Codecov precedent for capability overrides).
//!
//! The compiled `RedbStorage` implementation (`src/backend_impl.rs`) must
//! advertise recommendation-attribution capability so the checked persistence
//! path counts it as durable. The uncompiled duplicate in `src/redb_cache.rs`
//! is intentionally untouched: `src/lib.rs` includes `backend_impl` only.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use do_memory_core::StorageBackend;
use do_memory_storage_redb::RedbStorage;
use tempfile::TempDir;

#[tokio::test]
async fn redb_storage_advertises_recommendation_attribution() {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("capability.redb");
    let storage = RedbStorage::new(&db_path).await.expect("create redb");
    assert!(
        storage.supports_recommendation_attribution(),
        "the compiled RedbStorage must advertise recommendation-attribution capability"
    );
}
