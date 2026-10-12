use super::*;
use tempfile::TempDir;
use tokio::time::Duration;

async fn create_test_pool() -> (AdaptiveConnectionPool, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

    let config = AdaptivePoolConfig {
        min_connections: 2,
        max_connections: 10,
        scale_up_threshold: 0.8,
        scale_down_threshold: 0.3,
        scale_up_cooldown: Duration::from_secs(1),
        scale_down_cooldown: Duration::from_secs(1),
        scale_up_increment: 2,
        scale_down_decrement: 2,
        check_interval: Duration::from_secs(5),
    };

    let pool = AdaptiveConnectionPool::new_sync(Arc::new(db), config)
        .await
        .unwrap();
    (pool, dir)
}

/// Pool with more permits for tests that need 3+ concurrent connections
async fn create_large_test_pool() -> (AdaptiveConnectionPool, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

    let config = AdaptivePoolConfig {
        min_connections: 5,
        max_connections: 10,
        scale_up_threshold: 0.8,
        scale_down_threshold: 0.3,
        scale_up_cooldown: Duration::from_secs(1),
        scale_down_cooldown: Duration::from_secs(1),
        scale_up_increment: 2,
        scale_down_decrement: 2,
        check_interval: Duration::from_secs(5),
    };

    let pool = AdaptiveConnectionPool::new_sync(Arc::new(db), config)
        .await
        .unwrap();
    (pool, dir)
}

/// Base config for scaling tests: deterministic cooldowns, no monitor noise,
/// and a short enough acquire timeout that a broken scale-up fails fast.
fn adaptive_config() -> AdaptivePoolConfig {
    AdaptivePoolConfig {
        min_connections: 1,
        max_connections: 10,
        scale_up_threshold: 0.5,
        scale_down_threshold: 0.1,
        scale_up_cooldown: Duration::ZERO,
        scale_down_cooldown: Duration::ZERO,
        scale_up_increment: 1,
        scale_down_decrement: 1,
        check_interval: Duration::from_secs(30),
    }
}

async fn create_pool_with(config: AdaptivePoolConfig) -> (AdaptiveConnectionPool, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");
    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();
    let pool = AdaptiveConnectionPool::new_sync(Arc::new(db), config)
        .await
        .unwrap();
    (pool, dir)
}

/// Yield until a condition holds, so paused-clock tests do not depend on
/// task scheduling order between the monitor and the test body.
async fn yield_until(mut ready: impl FnMut() -> bool) {
    for _ in 0..256 {
        if ready() {
            return;
        }
        tokio::task::yield_now().await;
    }
    assert!(ready(), "condition not satisfied after yielding");
}

fn monitor_running(pool: &AdaptiveConnectionPool) -> bool {
    pool.monitor
        .lock()
        .as_ref()
        .is_some_and(|handle| !handle.task.is_finished())
}

#[tokio::test]
async fn test_adaptive_pool_creation() {
    let (pool, _dir) = create_test_pool().await;
    let metrics = pool.metrics();

    assert_eq!(metrics.active_connections, 0);
    assert_eq!(metrics.max_connections, 2);
}

#[tokio::test]
async fn test_connection_checkout() {
    let (pool, _dir) = create_test_pool().await;

    let conn = pool.get().await;
    assert!(conn.is_ok());

    let metrics = pool.metrics();
    assert_eq!(metrics.total_acquired, 1);
    assert_eq!(metrics.active_connections, 1);
}

#[tokio::test]
async fn test_connection_auto_return() {
    let (pool, _dir) = create_test_pool().await;

    {
        let _conn = pool.get().await.unwrap();
        let metrics = pool.metrics();
        assert_eq!(metrics.active_connections, 1);
    }

    let metrics = pool.metrics();
    assert_eq!(metrics.active_connections, 0);
}

#[tokio::test]
async fn test_utilization() {
    let (pool, _dir) = create_test_pool().await;

    let utilization = pool.utilization();
    assert_eq!(utilization, 0.0);

    let _conn = pool.get().await.unwrap();

    let utilization = pool.utilization();
    assert!(utilization > 0.0 && utilization <= 1.0);
}

#[tokio::test]
async fn test_available_connections() {
    let (pool, _dir) = create_test_pool().await;

    let available = pool.available_connections();
    assert_eq!(available, 2);

    let _conn1 = pool.get().await.unwrap();
    let available = pool.available_connections();
    assert_eq!(available, 1);

    let _conn2 = pool.get().await.unwrap();
    let available = pool.available_connections();
    assert_eq!(available, 0);
}

#[tokio::test]
async fn test_active_connections() {
    let (pool, _dir) = create_test_pool().await;

    let active = pool.active_connections();
    assert_eq!(active, 0);

    let _conn = pool.get().await.unwrap();

    let active = pool.active_connections();
    assert_eq!(active, 1);
}

#[tokio::test]
async fn test_max_connections() {
    let (pool, _dir) = create_test_pool().await;

    let max = pool.max_connections();
    assert_eq!(max, 2);
}

#[tokio::test]
async fn test_scale_up_adds_permits_for_extra_checkouts() {
    let config = AdaptivePoolConfig {
        max_connections: 5,
        scale_up_increment: 4,
        ..adaptive_config()
    };
    let (pool, _dir) = create_pool_with(config).await;

    assert_eq!(pool.available_connections(), 1);
    let conn1 = pool.get().await.unwrap();
    assert_eq!(pool.available_connections(), 0);

    // Utilization is 1/1, so an explicit tick grows the pool.
    assert!(pool.check_and_scale().await);
    assert_eq!(pool.max_connections(), 5);
    assert_eq!(pool.available_connections(), 4);
    assert_eq!(pool.metrics().scale_up_count, 1);

    // The extra capacity is real: these checkouts would time out without it.
    let conn2 = pool.get().await.unwrap();
    let conn3 = pool.get().await.unwrap();
    let conn4 = pool.get().await.unwrap();
    let conn5 = pool.get().await.unwrap();
    assert_eq!(pool.active_connections(), 5);
    assert_eq!(pool.available_connections(), 0);

    drop((conn1, conn2, conn3, conn4, conn5));
    assert_eq!(pool.active_connections(), 0);
    assert_eq!(pool.available_connections(), 5);
}

#[tokio::test(start_paused = true)]
async fn test_scale_up_respects_cooldown() {
    let config = AdaptivePoolConfig {
        scale_up_threshold: 0.1,
        scale_down_threshold: 0.05,
        scale_up_cooldown: Duration::from_secs(10),
        check_interval: Duration::from_secs(3600),
        ..adaptive_config()
    };
    let (pool, _dir) = create_pool_with(config).await;

    let _conn = pool.get().await.unwrap(); // utilization 1/1 keeps scaling eligible

    // The first eligible resize happens immediately.
    assert!(pool.check_and_scale().await);
    assert_eq!(pool.max_connections(), 2);
    assert_eq!(pool.metrics().scale_up_count, 1);

    // Still inside the cooldown: no second resize, even though utilization is
    // still above the scale-up threshold.
    assert!(!pool.check_and_scale().await);
    assert_eq!(pool.max_connections(), 2);
    assert_eq!(pool.metrics().scale_up_count, 1);

    // The first resize eligible after the cooldown happens as soon as time moves.
    tokio::time::advance(Duration::from_secs(10)).await;
    assert!(pool.check_and_scale().await);
    assert_eq!(pool.max_connections(), 3);
    assert_eq!(pool.metrics().scale_up_count, 2);

    // The new resize starts a fresh cooldown.
    assert!(!pool.check_and_scale().await);
    assert_eq!(pool.max_connections(), 3);
    assert_eq!(pool.metrics().scale_up_count, 2);
}

#[tokio::test]
async fn test_scale_down_reclaims_only_idle_permits() {
    let config = AdaptivePoolConfig {
        max_connections: 10,
        scale_down_threshold: 0.6,
        scale_up_increment: 9,
        scale_down_decrement: 8,
        ..adaptive_config()
    };
    let (pool, _dir) = create_pool_with(config).await;

    let conn1 = pool.get().await.unwrap(); // utilization 1/1
    assert!(pool.check_and_scale().await);
    assert_eq!(pool.max_connections(), 10);
    assert_eq!(pool.available_connections(), 9);

    let conn2 = pool.get().await.unwrap();
    let conn3 = pool.get().await.unwrap();
    assert_eq!(pool.active_connections(), 3);
    assert_eq!(pool.available_connections(), 7);

    // The target drop is 8 permits, but only 7 are idle: the reduction is
    // partial so the three checked-out connections are never stranded.
    assert!(pool.check_and_scale().await);
    assert_eq!(pool.max_connections(), 3);
    assert_eq!(pool.available_connections(), 0);
    assert_eq!(pool.active_connections(), 3);
    assert_eq!(pool.metrics().scale_down_count, 1);

    // Checked-out connections survive the shrink and stay usable.
    for conn in [&conn1, &conn2, &conn3] {
        assert!(
            conn.connection()
                .unwrap()
                .query("SELECT 1", ())
                .await
                .is_ok()
        );
    }

    // Dropping them releases exactly the remaining capacity, never more.
    drop((conn1, conn2, conn3));
    assert_eq!(pool.active_connections(), 0);
    assert_eq!(pool.available_connections(), 3);
    assert!(pool.available_connections() <= pool.max_connections() as usize);
}

#[tokio::test]
async fn test_scale_down_defers_while_all_permits_are_active() {
    // Thresholds that only ever allow shrinking; capacity is grown directly so
    // the scale-down path can be isolated.
    let config = AdaptivePoolConfig {
        min_connections: 2,
        max_connections: 6,
        scale_up_threshold: 1.1,
        scale_down_threshold: 1.0,
        scale_up_increment: 4,
        scale_down_decrement: 4,
        ..adaptive_config()
    };
    let (pool, _dir) = create_pool_with(config).await;

    // White-box: grow capacity without touching the scaling thresholds.
    assert!(pool.core.scale_up());
    assert_eq!(pool.max_connections(), 6);

    let conns: Vec<_> = {
        let mut conns = Vec::new();
        for _ in 0..6 {
            conns.push(pool.get().await.unwrap());
        }
        conns
    };
    assert_eq!(pool.active_connections(), 6);
    assert_eq!(pool.available_connections(), 0);

    // Every permit is checked out: the reduction must be deferred, not forced.
    assert!(!pool.check_and_scale().await);
    assert_eq!(pool.max_connections(), 6);
    assert_eq!(pool.available_connections(), 0);
    assert_eq!(pool.metrics().scale_down_count, 0);

    // Once the work drains, the next tick reclaims idle permits.
    drop(conns);
    assert!(pool.check_and_scale().await);
    assert_eq!(pool.max_connections(), 2);
    assert_eq!(pool.available_connections(), 2);
    assert_eq!(pool.metrics().scale_down_count, 1);
}

#[tokio::test(start_paused = true)]
async fn test_monitor_runs_on_check_interval_and_stops_on_shutdown() {
    let config = AdaptivePoolConfig {
        scale_up_cooldown: Duration::from_secs(1),
        check_interval: Duration::from_secs(5),
        ..adaptive_config()
    };
    let (pool, _dir) = create_pool_with(config).await;

    let _conn = pool.get().await.unwrap(); // utilization 1/1
    // Let the spawned monitor poll once so its interval timer is registered at
    // the test's t = 0 before any time is advanced.
    tokio::task::yield_now().await;
    tokio::task::yield_now().await;
    assert!(monitor_running(&pool));
    assert_eq!(pool.max_connections(), 1);

    // Before the configured interval elapses the monitor has done nothing.
    tokio::time::advance(Duration::from_secs(4)).await;
    tokio::task::yield_now().await;
    assert_eq!(pool.metrics().scale_up_count, 0);
    assert_eq!(pool.max_connections(), 1);

    // Crossing the interval lets the monitor perform the resize itself.
    tokio::time::sleep(Duration::from_secs(1)).await;
    yield_until(|| pool.metrics().scale_up_count >= 1).await;
    assert_eq!(pool.max_connections(), 2);
    assert!(pool.available_connections() >= 1);

    // Shutdown stops the monitor: no further resize happens as time advances.
    pool.shutdown().await;
    assert!(!monitor_running(&pool));

    let count = pool.metrics().scale_up_count;
    let capacity = pool.max_connections();
    tokio::time::sleep(Duration::from_secs(60)).await;
    tokio::task::yield_now().await;
    assert_eq!(pool.metrics().scale_up_count, count);
    assert_eq!(pool.max_connections(), capacity);
}

#[tokio::test]
async fn test_shutdown() {
    let (pool, _dir) = create_test_pool().await;

    let _conn = pool.get().await.unwrap();
    drop(_conn);

    pool.shutdown().await;
}

#[tokio::test]
async fn test_connection_exposure() {
    let (pool, _dir) = create_test_pool().await;

    // Get a connection from the pool
    let pooled_conn = pool.get().await.unwrap();

    // Verify we can access the underlying connection
    let conn_ref = pooled_conn.connection();
    assert!(conn_ref.is_some(), "Connection should be exposed");

    // Verify the connection is usable by running a query
    let conn = conn_ref.unwrap();
    let result = conn.query("SELECT 1", ()).await;
    assert!(result.is_ok(), "Connection should be usable for queries");

    // Test into_inner to take ownership
    let conn = pooled_conn.into_inner();
    assert!(conn.is_some(), "into_inner should return the connection");
}

#[tokio::test]
async fn test_connection_query_after_into_inner() {
    let (pool, _dir) = create_test_pool().await;

    // Get a connection and take ownership
    let pooled_conn = pool.get().await.unwrap();
    let conn = pooled_conn.into_inner().unwrap();

    // Verify the connection is still usable
    let result = conn.query("SELECT 1 as value", ()).await;
    assert!(result.is_ok());

    let mut rows = result.unwrap();
    let row = rows.next().await.unwrap();
    assert!(row.is_some());

    let value: i32 = row.unwrap().get(0).unwrap();
    assert_eq!(value, 1);
}

#[tokio::test]
async fn test_connection_id_uniqueness() {
    let (pool, _dir) = create_large_test_pool().await;

    // Get multiple connections and verify they have unique IDs
    let conn1 = pool.get().await.unwrap();
    let conn2 = pool.get().await.unwrap();
    let conn3 = pool.get().await.unwrap();

    let id1 = conn1.connection_id();
    let id2 = conn2.connection_id();
    let id3 = conn3.connection_id();

    // All IDs should be unique
    assert_ne!(id1, id2, "Connection IDs should be unique");
    assert_ne!(id2, id3, "Connection IDs should be unique");
    assert_ne!(id1, id3, "Connection IDs should be unique");

    // IDs should be monotonically increasing
    assert!(id2 > id1, "Connection IDs should be increasing");
    assert!(id3 > id2, "Connection IDs should be increasing");
}

#[tokio::test]
async fn test_cleanup_callback_on_connection_drop() {
    let (pool, _dir) = create_large_test_pool().await;

    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    let cleanup_count = Arc::new(AtomicU64::new(0));
    let cleanup_count_clone = Arc::clone(&cleanup_count);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |_conn_id| {
        cleanup_count_clone.fetch_add(1, Ordering::Relaxed);
    }));

    // Create and drop a connection
    let _conn_id = {
        let conn = pool.get().await.unwrap();
        conn.connection_id()
    };
    // Connection is dropped here

    tokio::task::yield_now().await;

    // Verify callback was called
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 1);

    // Create and drop multiple connections
    {
        let _conn1 = pool.get().await.unwrap();
        let _conn2 = pool.get().await.unwrap();
        let _conn3 = pool.get().await.unwrap();
    }

    tokio::task::yield_now().await;

    // Should have 4 total cleanups (1 from before + 3 new)
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 4);
}

#[tokio::test]
async fn test_cleanup_callback_tracks_correct_connection_id() {
    let (pool, _dir) = create_test_pool().await;

    use std::sync::Arc;
    use std::sync::Mutex;

    let cleaned_ids = Arc::new(Mutex::new(Vec::new()));
    let cleaned_ids_clone = Arc::clone(&cleaned_ids);

    // Register cleanup callback that tracks connection IDs
    pool.set_cleanup_callback(Arc::new(move |conn_id| {
        cleaned_ids_clone.lock().unwrap().push(conn_id);
    }));

    // Create and drop connections
    let id1 = {
        let conn = pool.get().await.unwrap();
        conn.connection_id()
    };

    let id2 = {
        let conn = pool.get().await.unwrap();
        conn.connection_id()
    };

    let id3 = {
        let conn = pool.get().await.unwrap();
        conn.connection_id()
    };

    // Verify all connection IDs were tracked
    let ids = cleaned_ids.lock().unwrap();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
    assert!(ids.contains(&id3));
}

#[tokio::test]
async fn test_cleanup_callback_removal() {
    let (pool, _dir) = create_test_pool().await;

    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    let cleanup_count = Arc::new(AtomicU64::new(0));
    let cleanup_count_clone = Arc::clone(&cleanup_count);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |_conn_id| {
        cleanup_count_clone.fetch_add(1, Ordering::Relaxed);
    }));

    // Create and drop a connection
    {
        let _conn = pool.get().await.unwrap();
    }

    assert_eq!(cleanup_count.load(Ordering::Relaxed), 1);

    // Remove the callback
    pool.remove_cleanup_callback();

    // Create and drop another connection
    {
        let _conn = pool.get().await.unwrap();
    }

    // Count should still be 1 (callback was removed)
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_connection_cache_integration() {
    let (pool, _dir) = create_large_test_pool().await;

    use crate::prepared::PreparedStatementCache;
    use std::sync::Arc;

    // Create a prepared statement cache
    let cache = Arc::new(PreparedStatementCache::new(10));
    let cache_clone = Arc::clone(&cache);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |conn_id| {
        cache_clone.clear_connection(conn_id);
    }));

    // Get a connection and record some cached statements
    let conn_id = {
        let conn = pool.get().await.unwrap();
        let id = conn.connection_id();

        // Record some cached statements
        cache.record_miss(id, "SELECT 1", 100);
        cache.record_miss(id, "SELECT 2", 100);
        cache.record_miss(id, "SELECT 3", 100);

        assert_eq!(cache.connection_size(id), 3);
        id
    };

    // Connection is dropped here, which should trigger cleanup
    tokio::task::yield_now().await;

    // Verify cache was cleared for this connection
    assert_eq!(cache.connection_size(conn_id), 0);
    assert_eq!(cache.connection_count(), 0);
}

#[tokio::test]
async fn test_connection_outlives_pool() {
    let (pool, _dir) = create_test_pool().await;

    let conn = pool.get().await.unwrap();
    // Clone the shared state the guard owns so we can observe it after the
    // pool value (and the allocations it owns) is gone.
    let metrics = Arc::clone(&conn.metrics);
    let current_max = Arc::clone(&conn.current_max);
    assert_eq!(metrics.active_connections.load(Ordering::Relaxed), 1);

    // Drop the pool first: the guard must not reference freed state.
    drop(pool);

    assert_eq!(metrics.total_released.load(Ordering::Relaxed), 0);

    // Dropping the checked-out connection now updates the owned state exactly once.
    drop(conn);

    assert_eq!(metrics.active_connections.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.total_released.load(Ordering::Relaxed), 1);
    assert_eq!(current_max.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn test_release_metrics_updated_exactly_once() {
    let (pool, _dir) = create_test_pool().await;

    let conn = pool.get().await.unwrap();
    assert_eq!(pool.active_connections(), 1);

    let before = pool.metrics();
    drop(conn);

    let after = pool.metrics();
    assert_eq!(after.active_connections, 0);
    assert_eq!(after.total_acquired, before.total_acquired);
    assert_eq!(after.total_released, before.total_released + 1);
    assert_eq!(pool.available_connections(), 2);
}

#[tokio::test]
async fn test_concurrent_checkout_drop_with_cleanup() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    let (pool, _dir) = create_large_test_pool().await;
    let pool = Arc::new(pool);

    let cleanup_count = Arc::new(AtomicU64::new(0));
    let cleanup_count_clone = Arc::clone(&cleanup_count);
    pool.set_cleanup_callback(Arc::new(move |_conn_id| {
        cleanup_count_clone.fetch_add(1, Ordering::Relaxed);
    }));

    let mut handles = Vec::new();
    for _ in 0..5 {
        let pool = Arc::clone(&pool);
        handles.push(tokio::spawn(async move {
            let conn = pool.get().await.unwrap();
            assert!(conn.connection().is_some());
            drop(conn);
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    let metrics = pool.metrics();
    assert_eq!(metrics.total_acquired, 5);
    assert_eq!(metrics.total_released, 5);
    assert_eq!(metrics.active_connections, 0);
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 5);
}

#[tokio::test]
async fn test_connection_moved_across_tasks() {
    let (pool, _dir) = create_test_pool().await;

    let conn = pool.get().await.unwrap();
    let conn_id = conn.connection_id();

    // Moving the guard to another task requires it to be auto-`Send`.
    let handle = tokio::spawn(async move {
        let moved_id = conn.connection_id();
        assert!(conn.connection().is_some());
        drop(conn);
        moved_id
    });

    assert_eq!(handle.await.unwrap(), conn_id);
    assert_eq!(pool.active_connections(), 0);
    assert_eq!(pool.metrics().total_released, 1);
}
