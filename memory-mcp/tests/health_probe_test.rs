//! End-to-end probes for the health surface (`#1085`).
//!
//! The handler these tests exercise used to answer from the environment: a Turso URL present
//! in the process meant "connected" and a redb path that existed meant "connected". Each test
//! below sets those variables to values that *contradict* the attached backends, so the old
//! inference cannot pass.

use async_trait::async_trait;
use do_memory_core::episode::PatternId;
use do_memory_core::{
    Episode, Error, Heuristic, MemoryConfig, Pattern, Result, SelfLearningMemory, StorageBackend,
};
use do_memory_mcp::cache::{QueryCache, QueryMemoryKey};
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

/// Point every variable the removed inference used at something that is not a connection.
#[allow(unsafe_code)]
fn configure_dead_environment(dir: &tempfile::TempDir) -> String {
    let missing_redb = dir.path().join("never-created.redb").display().to_string();
    // SAFETY: test-only env var manipulation; nextest runs each test in its own process.
    unsafe {
        std::env::set_var("TURSO_DATABASE_URL", format!("libsql://{DEAD_HOST}:5001"));
        std::env::set_var("TURSO_AUTH_TOKEN", SECRET_TOKEN);
        std::env::set_var("REDB_CACHE_PATH", &missing_redb);
    }
    missing_redb
}

#[tokio::test]
async fn configured_environment_without_a_backend_is_not_a_connection() {
    let dir = tempfile::TempDir::new().unwrap();
    let missing_redb = configure_dead_environment(&dir);

    // Nothing is attached, so there is nothing to probe — regardless of the environment.
    let memory = SelfLearningMemory::new();
    let cache = Arc::new(QueryCache::new());
    let monitoring = MonitoringSystem::new(MonitoringConfig::default());

    let response = build_health_response(&memory, &cache, &monitoring, BUDGET).await;

    assert_eq!(
        response.storage.turso_status,
        BackendStatus::NotConfigured,
        "the probe reported a connection the server does not have"
    );
    assert!(
        !response.storage.turso_connected,
        "TURSO_DATABASE_URL is configuration, not connectivity"
    );
    assert!(
        !response.storage.redb_connected,
        "no cache backend is attached, so nothing may claim one"
    );
    assert_eq!(response.status, "degraded");

    let rendered = serde_json::to_string(&response).unwrap();
    for secret in [SECRET_TOKEN, DEAD_HOST, "libsql", &missing_redb] {
        assert!(
            !rendered.contains(secret),
            "health output leaked {secret:?}: {rendered}"
        );
    }
}

#[tokio::test]
async fn attached_backends_are_measured_even_when_the_environment_points_elsewhere() {
    let dir = tempfile::TempDir::new().unwrap();
    let missing_redb = configure_dead_environment(&dir);

    // Both slots hold real, reachable databases: the durable path production wires when no
    // Turso is available (`server_impl/storage.rs`), the cache slot a redb file. The old
    // handler would have answered `redb_connected: false` here, because it only looked at
    // whether `REDB_CACHE_PATH` existed.
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        redb_backend(&dir, "durable.redb").await,
        redb_backend(&dir, "cache.redb").await,
    );
    let cache = Arc::new(QueryCache::new());
    let monitoring = MonitoringSystem::new(MonitoringConfig::default());

    let key = QueryMemoryKey::new("axum".to_string(), "rust".to_string(), None, 5);
    cache.put_query_memory(key.clone(), serde_json::json!([]));
    assert!(
        cache.get_query_memory(&key).is_some(),
        "the fixture must register a hit"
    );

    // Uptime is read from the live monitoring system, so it has to move with the clock.
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    let response = build_health_response(&memory, &cache, &monitoring, BUDGET).await;

    assert_eq!(
        response.status, "healthy",
        "both probes answered: {:?}",
        response.storage
    );
    assert!(
        response.storage.redb_connected,
        "an attached cache backend must outrank a missing REDB_CACHE_PATH"
    );
    assert_eq!(response.storage.redb_status, BackendStatus::Healthy);
    assert_eq!(response.storage.turso_status, BackendStatus::Healthy);

    // Acceptance 2: the cache and uptime sections come from the running server.
    assert_eq!(
        response.cache.hits, 1,
        "cache hits were not read from the live cache"
    );
    assert_eq!(response.cache.size, 1);
    assert_eq!(response.cache.max_size, cache.stats().max_entries);
    assert!(
        response.uptime_seconds >= 1,
        "uptime is still the placeholder constant: {}",
        response.uptime_seconds
    );

    let rendered = serde_json::to_string(&response).unwrap();
    for secret in [SECRET_TOKEN, DEAD_HOST, "libsql", &missing_redb] {
        assert!(
            !rendered.contains(secret),
            "health output leaked {secret:?}: {rendered}"
        );
    }
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
    configure_dead_environment(&dir);

    let memory = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::new(BrokenBackend),
        redb_backend(&dir, "cache.redb").await,
    );
    let cache = Arc::new(QueryCache::new());
    let monitoring = MonitoringSystem::new(MonitoringConfig::default());

    let response = build_health_response(&memory, &cache, &monitoring, BUDGET).await;

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
    for secret in [SECRET_DRIVER_DETAIL, "pa55w0rd", "db.internal"] {
        assert!(
            !rendered.contains(secret),
            "health output repeated a backend error: {rendered}"
        );
    }
}
