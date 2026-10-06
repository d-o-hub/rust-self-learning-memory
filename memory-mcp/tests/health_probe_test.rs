//! End-to-end probes for the health surface (`#1085`).
//!
//! The handler these tests exercise used to answer from the environment: a Turso URL present
//! in the process meant "connected" and a redb path that existed meant "connected". Health is
//! now derived only from the backends attached to the running server, so every test below
//! controls that attachment directly instead of mutating the process environment.

use async_trait::async_trait;
use do_memory_core::episode::PatternId;
use do_memory_core::{
    Episode, Error, Heuristic, MemoryConfig, Pattern, Result, SelfLearningMemory, StorageBackend,
};
use do_memory_mcp::monitoring::types::BackendStatus;
use do_memory_mcp::monitoring::{MonitoringConfig, MonitoringSystem, build_health_response};
use do_memory_storage_redb::RedbStorage;
use do_memory_storage_turso::TursoStorage;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Token material that must never appear in health output.
const SECRET_TOKEN: &str = "srv-secret-token-must-not-leak";
/// Host part of a database URL that cannot resolve.
const DEAD_HOST: &str = "health-probe-unreachable.invalid";
/// Probe budget used by these tests.
const BUDGET: Duration = Duration::from_millis(1_500);
/// Credential material inside a backend error message, which health output must not repeat.
const SECRET_DRIVER_DETAIL: &str = "libsql://user:pa55w0rd@db.internal:9003";

/// A backend that is attached but cannot answer reads.
///
/// Only the required methods are implemented, so liveness runs through
/// [`StorageBackend::health_check`]'s default body — which is what a backend without a native
/// ping gets.
struct BrokenBackend;

#[async_trait]
impl StorageBackend for BrokenBackend {
    async fn store_episode(&self, _episode: &Episode) -> Result<()> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn delete_episode(&self, _id: Uuid) -> Result<()> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn store_pattern(&self, _pattern: &Pattern) -> Result<()> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn get_pattern(&self, _id: PatternId) -> Result<Option<Pattern>> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn store_heuristic(&self, _heuristic: &Heuristic) -> Result<()> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<Heuristic>> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn query_episodes_since(
        &self,
        _since: chrono::DateTime<chrono::Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Err(Error::Storage(SECRET_DRIVER_DETAIL.to_string()))
    }
}

async fn redb_backend(dir: &tempfile::TempDir, name: &str) -> Arc<dyn StorageBackend> {
    let storage = RedbStorage::new(&dir.path().join(name))
        .await
        .expect("open redb cache");
    Arc::new(storage)
}

/// Assert that health output stayed free of the given fixture material.
///
/// The panic message names the fixture's label rather than interpolating its value: a leak
/// check that prints the material it is guarding is the same `rust/cleartext-logging` defect
/// it is testing for. The rendered response is included, so a failure is still diagnosable.
fn assert_no_fixture_leak(rendered: &str, fixtures: &[(&str, &str)]) {
    for (label, fixture) in fixtures {
        assert!(
            !rendered.contains(fixture),
            "health output leaked the {label} fixture: {rendered}"
        );
    }
}

/// Unattached backends are reported as not configured, and the details are fixed literals.
#[tokio::test]
async fn unattached_backends_are_not_configured_and_never_connected() {
    // Nothing is attached, so there is nothing to probe.
    let memory = SelfLearningMemory::new();
    let monitoring = MonitoringSystem::new(MonitoringConfig::default());

    let response = build_health_response(&memory, &monitoring, BUDGET).await;

    assert_eq!(
        response.storage.turso_status,
        BackendStatus::NotConfigured,
        "the probe reported a connection the server does not have"
    );
    assert!(
        !response.storage.turso_connected,
        "a URL in a config file is not connectivity"
    );
    assert!(
        !response.storage.redb_connected,
        "no cache backend is attached, so nothing may claim one"
    );
    assert_eq!(response.status, "degraded");

    // Details come from the fixed `BackendStatus::detail()` table only, so no configuration
    // value (URL, path, token) has a path into the response.
    let not_configured = BackendStatus::NotConfigured.detail();
    assert_eq!(
        response.storage.turso_details.as_deref(),
        Some(not_configured)
    );
    assert_eq!(response.storage.redb_details.as_deref(), Some(not_configured));
}

#[tokio::test]
async fn attached_backends_are_measured_and_report_live_counters() {
    let dir = tempfile::TempDir::new().unwrap();

    // Both slots hold real, reachable databases: the durable path production wires when no
    // Turso is available (`server_impl/storage.rs`), the cache slot a redb file. The old
    // handler would have answered `redb_connected: false` here, because it only looked at
    // whether a configured cache path existed.
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        redb_backend(&dir, "durable.redb").await,
        redb_backend(&dir, "cache.redb").await,
    );
    let cache_metrics = memory.get_cache_metrics();
    let monitoring = MonitoringSystem::new(MonitoringConfig::default());

    // Uptime is read from the live monitoring system, so it has to move with the clock.
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    let response = build_health_response(&memory, &monitoring, BUDGET).await;

    assert_eq!(
        response.status, "healthy",
        "both probes answered: {:?}",
        response.storage
    );
    assert!(
        response.storage.redb_connected,
        "an attached cache backend must be measured, not inferred from a path"
    );
    assert_eq!(response.storage.redb_status, BackendStatus::Healthy);
    assert_eq!(response.storage.turso_status, BackendStatus::Healthy);

    // Acceptance 2: the cache and uptime sections come from the running server's retrieval
    // cache — the one `get_cache_metrics()` measures — not from constants or a dead cache.
    assert_eq!(
        (response.cache.hits, response.cache.misses),
        (cache_metrics.hits, cache_metrics.misses),
        "cache counters are not bound to the live retrieval cache"
    );
    assert_eq!(response.cache.size, cache_metrics.size);
    assert_eq!(response.cache.max_size, cache_metrics.capacity);
    assert!(
        response.cache.max_size > 0,
        "the live retrieval cache always advertises a capacity"
    );
    assert_eq!(response.cache.hit_rate, cache_metrics.hit_rate() * 100.0);
    assert!(
        response.uptime_seconds >= 1,
        "uptime is still the placeholder constant: {}",
        response.uptime_seconds
    );

    let rendered = serde_json::to_string(&response).unwrap();
    let cache_path = dir.path().display().to_string();
    assert_no_fixture_leak(
        &rendered,
        &[
            ("token", SECRET_TOKEN),
            ("dead host", DEAD_HOST),
            ("url scheme", "libsql"),
            ("cache path", &cache_path),
        ],
    );
}

/// A remote URL that cannot be reached never becomes a usable handle, which is precisely why
/// inferring connectivity from configuration was wrong.
#[tokio::test]
async fn unreachable_turso_url_cannot_be_constructed_into_a_connection() {
    let url = format!("libsql://{DEAD_HOST}:5001");
    let built = TursoStorage::new(&url, SECRET_TOKEN).await;
    assert!(
        built.is_err(),
        "a dead URL must not produce a storage handle, so health cannot probe one"
    );
}

/// Acceptance 1 and 4: an attached backend that fails its probe is reported as unavailable,
/// and its driver error text stays out of the response.
#[tokio::test]
async fn attached_but_failing_backend_is_unavailable_and_stays_redacted() {
    let dir = tempfile::TempDir::new().unwrap();

    let memory = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::new(BrokenBackend),
        redb_backend(&dir, "cache.redb").await,
    );
    let monitoring = MonitoringSystem::new(MonitoringConfig::default());

    let response = build_health_response(&memory, &monitoring, BUDGET).await;

    assert_eq!(
        response.storage.turso_status,
        BackendStatus::Unavailable,
        "a backend that fails its probe must not be reported healthy: {:?}",
        response.storage
    );
    assert!(
        !response.storage.turso_connected,
        "attaching a broken backend is not the same as reaching one"
    );
    assert_eq!(
        response.storage.redb_status,
        BackendStatus::Healthy,
        "the reachable cache backend must still be measured healthy"
    );
    assert_eq!(response.status, "degraded");

    let rendered = serde_json::to_string(&response).unwrap();
    assert_no_fixture_leak(
        &rendered,
        &[
            ("driver detail", SECRET_DRIVER_DETAIL),
            ("password", "pa55w0rd"),
            ("internal host", "db.internal"),
        ],
    );
}
