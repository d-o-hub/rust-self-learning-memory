//! Bounded, secret-free storage probes for the health endpoint.
//!
//! The JSON-RPC `health` method used to infer connectivity from the environment: a Turso URL
//! existing in the process meant "connected", a redb path existing meant "connected", and the
//! cache counters were zeros (`#1085`). Everything reported here is measured instead, through
//! [`do_memory_core::StorageBackend::health_check`] under a deadline, and no backend error
//! text, URL or filesystem path is ever copied into the response.

use super::types::{BackendStatus, CacheHealth, HealthResponse, StorageHealth, SyncHealth};
use crate::cache::QueryCache;
use crate::monitoring::MonitoringSystem;
use do_memory_core::SelfLearningMemory;
use do_memory_core::storage::StorageBackend;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// Budget for one backend probe.
///
/// Comfortably longer than a healthy local round trip, short enough that a black-holed
/// database cannot keep the health response open.
pub const PROBE_TIMEOUT: Duration = Duration::from_millis(1_500);

/// Reported by the sync section while no synchronizer runs in this process.
///
/// There is no background Turso↔redb task inside the MCP server, so claiming a last-sync time
/// would be invented; the honest value is that nothing is configured to sync.
pub const SYNC_NOT_CONFIGURED: &str = "no synchronizer configured";

/// Map a probe outcome onto one backend's status.
///
/// Kept generic over the probe so the timeout and error branches are testable without a full
/// [`StorageBackend`] implementation.
pub async fn probe_outcome<F, Fut>(configured: bool, probe: F, budget: Duration) -> BackendStatus
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = do_memory_core::Result<()>>,
{
    if !configured {
        return BackendStatus::NotConfigured;
    }
    match tokio::time::timeout(budget, probe()).await {
        Ok(Ok(())) => BackendStatus::Healthy,
        Ok(Err(_)) => BackendStatus::Unavailable,
        Err(_) => BackendStatus::Degraded,
    }
}

/// Probe one configured backend within `budget`.
pub async fn probe_backend(
    backend: Option<&Arc<dyn StorageBackend>>,
    budget: Duration,
) -> BackendStatus {
    match backend {
        None => BackendStatus::NotConfigured,
        Some(backend) => probe_outcome(true, || backend.health_check(), budget).await,
    }
}

/// Overall status: healthy only when every configured backend answered.
///
/// A server with no durable backend at all is degraded rather than healthy — it can still
/// serve from memory, but nothing it accepts survives the process.
#[must_use]
pub fn overall_status(turso: BackendStatus, redb: BackendStatus) -> &'static str {
    let configured = [turso, redb]
        .into_iter()
        .filter(|status| *status != BackendStatus::NotConfigured)
        .collect::<Vec<_>>();
    if configured.is_empty() {
        return "degraded";
    }
    if configured
        .iter()
        .all(|status| matches!(status, BackendStatus::Healthy))
    {
        "healthy"
    } else if configured
        .iter()
        .any(|status| matches!(status, BackendStatus::Healthy))
    {
        "degraded"
    } else {
        "unhealthy"
    }
}

/// Measure every live surface the health endpoint reports.
///
/// Both backends are probed concurrently, so one unreachable database costs one timeout rather
/// than two. Cache counters and uptime come from the running server's own state.
pub async fn build_health_response(
    memory: &SelfLearningMemory,
    cache: &Arc<QueryCache>,
    monitoring: &MonitoringSystem,
    budget: Duration,
) -> HealthResponse {
    let (turso_status, redb_status) = tokio::join!(
        probe_backend(memory.turso_storage(), budget),
        probe_backend(memory.cache_storage(), budget)
    );

    let stats = cache.stats();
    monitoring.update_uptime();

    HealthResponse {
        status: overall_status(turso_status, redb_status).to_string(),
        storage: StorageHealth {
            turso_connected: turso_status.connected(),
            turso_status,
            turso_details: Some(turso_status.detail().to_string()),
            redb_connected: redb_status.connected(),
            redb_status,
            redb_details: Some(redb_status.detail().to_string()),
        },
        cache: CacheHealth {
            enabled: stats.enabled,
            hits: stats.hits,
            misses: stats.misses,
            hit_rate: stats.hit_rate,
            size: stats.total_entries,
            max_size: stats.max_entries,
        },
        sync: SyncHealth {
            last_sync_timestamp: None,
            status: SYNC_NOT_CONFIGURED.to_string(),
            seconds_since_sync: None,
        },
        uptime_seconds: monitoring.get_stats().uptime_seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn probe_ok() -> do_memory_core::Result<()> {
        Ok(())
    }

    async fn probe_err() -> do_memory_core::Result<()> {
        Err(do_memory_core::Error::Storage(
            "driver said: see url".to_string(),
        ))
    }

    #[tokio::test]
    async fn unconfigured_backend_is_reported_as_such() {
        let status = probe_outcome(false, probe_ok, Duration::from_millis(50)).await;
        assert_eq!(status, BackendStatus::NotConfigured);
        assert!(!status.connected());
    }

    #[tokio::test]
    async fn answered_probe_is_healthy() {
        let status = probe_outcome(true, probe_ok, Duration::from_millis(50)).await;
        assert_eq!(status, BackendStatus::Healthy);
        assert!(status.connected());
    }

    /// Acceptance 1: a configured backend that fails is `unavailable`, never "connected".
    #[tokio::test]
    async fn failed_probe_is_unavailable_not_connected() {
        let status = probe_outcome(true, probe_err, Duration::from_millis(50)).await;
        assert_eq!(status, BackendStatus::Unavailable);
        assert!(
            !status.connected(),
            "a failed probe must not be reported as a connection"
        );
    }

    /// A probe that never answers is cut off by the budget instead of hanging health.
    #[tokio::test]
    async fn slow_probe_times_out_as_degraded() {
        let started = std::time::Instant::now();
        let status = probe_outcome(
            true,
            || async {
                tokio::time::sleep(Duration::from_secs(30)).await;
                Ok(())
            },
            Duration::from_millis(60),
        )
        .await;

        assert_eq!(status, BackendStatus::Degraded);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the probe ran {:.0?} instead of returning at its deadline",
            started.elapsed()
        );
    }

    /// Acceptance 4: the summary text is fixed, so no backend error can leak through it.
    #[tokio::test]
    async fn detail_strings_never_quote_the_backend_error() {
        for status in [
            BackendStatus::Healthy,
            BackendStatus::Degraded,
            BackendStatus::Unavailable,
            BackendStatus::NotConfigured,
        ] {
            let detail = status.detail();
            assert!(
                !detail.contains("driver") && !detail.contains("url"),
                "status {status:?} leaked a backend detail: {detail}"
            );
            assert!(!detail.is_empty(), "status {status:?} has no summary");
        }
        assert!(
            probe_err().await.is_err(),
            "the fixture must fail like a probe"
        );
    }

    #[test]
    fn overall_status_requires_every_backend() {
        assert_eq!(
            overall_status(BackendStatus::Healthy, BackendStatus::Healthy),
            "healthy"
        );
        assert_eq!(
            overall_status(BackendStatus::Healthy, BackendStatus::Unavailable),
            "degraded"
        );
        assert_eq!(
            overall_status(BackendStatus::Degraded, BackendStatus::Degraded),
            "unhealthy"
        );
        // An absent cache backend is not a failure of the durable one.
        assert_eq!(
            overall_status(BackendStatus::Healthy, BackendStatus::NotConfigured),
            "healthy"
        );
        assert_eq!(
            overall_status(BackendStatus::NotConfigured, BackendStatus::NotConfigured),
            "degraded"
        );
    }

    #[tokio::test]
    async fn response_reports_live_cache_counters_and_uptime() {
        let memory = SelfLearningMemory::new();
        let cache = Arc::new(QueryCache::new());
        let monitoring = MonitoringSystem::new(crate::monitoring::MonitoringConfig::default());
        let key =
            crate::cache::QueryMemoryKey::new("async".to_string(), "rust".to_string(), None, 10);

        cache.put_query_memory(key.clone(), serde_json::json!({"episodes": []}));
        assert!(
            cache.get_query_memory(&key).is_some(),
            "the fixture must register a hit"
        );
        assert!(
            cache
                .get_query_memory(&crate::cache::QueryMemoryKey::new(
                    "other".to_string(),
                    "rust".to_string(),
                    None,
                    10,
                ))
                .is_none(),
            "the fixture must register a miss"
        );

        let response = build_health_response(&memory, &cache, &monitoring, PROBE_TIMEOUT).await;
        let stats = cache.stats();

        assert_eq!(response.cache.size, stats.total_entries);
        assert_eq!(response.cache.max_size, stats.max_entries);
        assert_eq!(response.cache.enabled, stats.enabled);
        assert_eq!(
            (response.cache.hits, response.cache.misses),
            (stats.hits, stats.misses),
            "the cache section is a constant again"
        );
        assert!(
            response.cache.hits == 1 && response.cache.misses == 1,
            "the counters did not follow the traffic: {:?}",
            response.cache
        );
        assert_eq!(response.sync.status, SYNC_NOT_CONFIGURED);
        assert!(
            response.sync.last_sync_timestamp.is_none()
                && response.sync.seconds_since_sync.is_none(),
            "a server without a synchronizer must not invent sync times: {:?}",
            response.sync
        );
        assert_eq!(
            response.uptime_seconds,
            monitoring.get_stats().uptime_seconds
        );
        assert_eq!(
            response.status, "degraded",
            "a server with no configured backend cannot claim to be healthy"
        );
        assert_eq!(
            response.storage.turso_status,
            BackendStatus::NotConfigured,
            "no durable backend was configured, so the probe must not claim one"
        );
    }
}
