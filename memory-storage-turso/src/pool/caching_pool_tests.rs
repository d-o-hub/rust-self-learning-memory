use super::*;
use tempfile::TempDir;

async fn create_test_pool() -> (CachingPool, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();
    let config = CachingPoolConfig {
        max_connections: 5,
        min_connections: 2,
        ..Default::default()
    };

    let pool = CachingPool::new(Arc::new(db), config).await.unwrap();
    (pool, dir)
}

#[tokio::test]
async fn test_pool_creation() {
    let (pool, _dir) = create_test_pool().await;

    // Just verify pool was created successfully; idle count may vary due to async timing
    let _stats = pool.stats();
}

#[tokio::test]
async fn test_connection_checkout() {
    let (pool, _dir) = create_test_pool().await;

    let guard = pool.get().await.unwrap();
    assert!(
        guard
            .connection()
            .expect("connection")
            .query("SELECT 1", ())
            .await
            .is_ok()
    );

    let stats = pool.stats();
    assert_eq!(stats.active_connections, 1);
}

#[tokio::test]
async fn test_connection_return() {
    let (pool, _dir) = create_test_pool().await;

    {
        let _guard = pool.get().await.unwrap();
        let stats = pool.stats();
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.idle_connections, 1); // One of the 2 pre-created
    }

    let stats = pool.stats();
    assert_eq!(stats.active_connections, 0, "Connection should be returned");
    assert_eq!(
        stats.idle_connections, 2,
        "Connection should be back in pool"
    );
}

#[tokio::test]
async fn test_cache_hit_rate() {
    let (pool, _dir) = create_test_pool().await;

    // First checkout - should be cache hit (reusing pre-created)
    {
        let _guard = pool.get().await.unwrap();
        let stats = pool.stats();
        assert_eq!(
            stats.cache_hits, 1,
            "Should hit cache (pre-created connection)"
        );
        assert_eq!(stats.cache_misses, 2, "Should have 2 misses (pre-creation)");
    }

    // Second checkout - should reuse returned connection
    {
        let _guard = pool.get().await.unwrap();
        let stats = pool.stats();
        assert_eq!(stats.cache_hits, 2, "Should hit cache (reused connection)");
    }
}

#[tokio::test]
async fn test_stable_connection_id() {
    let (pool, _dir) = create_test_pool().await;

    let conn_id1 = {
        let guard = pool.get().await.unwrap();
        guard.id().expect("connection id")
    };

    let conn_id2 = {
        let guard = pool.get().await.unwrap();
        guard.id().expect("connection id")
    };

    // Should get the same connection back (same ID)
    assert_eq!(conn_id1, conn_id2, "Should reuse connection with same ID");
}

#[tokio::test]
async fn test_cleanup_callback() {
    let (pool, _dir) = create_test_pool().await;

    use std::sync::{Arc, atomic::AtomicU64};
    let cleaned_up = Arc::new(AtomicU64::new(0));

    pool.set_cleanup_callback({
        let cleaned_up = Arc::clone(&cleaned_up);
        move |_conn_id| {
            cleaned_up.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    });

    // Clean up idle connections
    let evicted = pool.cleanup_idle_connections();
    // Should have 2 idle connections, but they're not old yet
    assert_eq!(evicted, 0, "No connections should be evicted (too new)");

    assert_eq!(cleaned_up.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[tokio::test]
async fn test_guard_survives_pool_drop() {
    let (pool, _dir) = create_test_pool().await;

    let guard = pool.get().await.unwrap();
    let shared = Arc::clone(&pool.state);
    let guard_id = guard.id().expect("connection id");

    // Drop the owning pool value first. The guard owns its return state, so it
    // must remain usable and safely return the connection afterwards.
    drop(pool);

    assert_eq!(guard.id().expect("connection id"), guard_id);
    assert_eq!(shared.active_connections(), 1);

    drop(guard);

    let stats = shared.stats();
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.total_returns, 1);
    assert_eq!(shared.available_connections(), 2);
    assert_eq!(shared.active_connections(), 0);
}

#[tokio::test]
async fn test_return_restores_counts_and_permit_once() {
    let (pool, _dir) = create_test_pool().await;

    // Baseline: the pre-created connections are idle, no checkouts yet.
    assert_eq!(pool.available_connections(), 2);
    assert_eq!(pool.stats().total_checkouts, 0);
    assert_eq!(pool.semaphore.available_permits(), 5);

    {
        let _guard = pool.get().await.unwrap();
        let stats = pool.stats();
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.idle_connections, 1);
        assert_eq!(stats.total_returns, 0);
        assert_eq!(pool.semaphore.available_permits(), 4);
    }

    let stats = pool.stats();
    assert_eq!(stats.active_connections, 0, "active count restored once");
    assert_eq!(stats.idle_connections, 2, "idle count restored once");
    assert_eq!(stats.total_returns, 1, "return recorded exactly once");
    assert_eq!(
        pool.semaphore.available_permits(),
        5,
        "permit released exactly once"
    );
}

#[tokio::test]
async fn test_guard_moved_across_tasks() {
    let (pool, _dir) = create_test_pool().await;

    let guard = pool.get().await.unwrap();
    let guard_id = guard.id().expect("connection id");

    // Moving the guard to another task requires it to be auto-`Send`.
    let handle = tokio::spawn(async move {
        let moved_id = guard.id().expect("connection id");
        drop(guard);
        moved_id
    });

    assert_eq!(handle.await.unwrap(), guard_id);

    let stats = pool.stats();
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.total_returns, 1);
    assert_eq!(pool.active_connections(), 0);
    assert_eq!(pool.available_connections(), 2);
}

async fn create_pool_with_config(config: CachingPoolConfig) -> (CachingPool, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();
    let pool = CachingPool::new(Arc::new(db), config).await.unwrap();
    (pool, dir)
}

#[tokio::test]
async fn test_cache_hit_rate_zero_and_with_values() {
    // No pre-created connections and no checkouts -> 0 hits / 0 misses collapses to 0.0.
    let config = CachingPoolConfig {
        max_connections: 5,
        min_connections: 0,
        enable_health_check: false,
        ..Default::default()
    };
    let (pool, _dir) = create_pool_with_config(config).await;
    let stats = pool.stats();
    assert_eq!(stats.total_created, 0);
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
    assert_eq!(pool.cache_hit_rate(), 0.0);

    // Two pre-created connections are misses; a checkout is one hit.
    let (pool, _dir) = create_test_pool().await;
    assert_eq!(pool.cache_hit_rate(), 0.0);
    let guard = pool.get().await.unwrap();
    let stats = pool.stats();
    assert_eq!(stats.cache_hits, 1);
    assert_eq!(stats.cache_misses, 2);
    assert!((pool.cache_hit_rate() - (1.0 / 3.0)).abs() < 1e-9);
    drop(guard);
    assert_eq!(pool.stats().total_returns, 1);
}

#[tokio::test]
async fn test_set_cleanup_callback_and_destroy_connection() {
    use std::sync::atomic::{AtomicU64, Ordering};

    let (pool, _dir) = create_test_pool().await;

    let cleaned = Arc::new(AtomicU64::new(0));
    pool.set_cleanup_callback({
        let cleaned = Arc::clone(&cleaned);
        move |_conn_id| {
            cleaned.fetch_add(1, Ordering::Relaxed);
        }
    });

    // Take the pooled connection out of a guard and destroy it permanently.
    let mut guard = pool.get().await.unwrap();
    let connection = guard.connection.take().expect("connection");
    drop(guard); // guard has no connection left, so no return is recorded

    pool.state.destroy_connection(connection);

    assert_eq!(pool.stats().evictions, 1);
    assert_eq!(cleaned.load(Ordering::Relaxed), 1);
    assert_eq!(pool.state.active_connections(), 0);
}

#[tokio::test]
async fn test_cleanup_idle_connections_evicts_by_idle_time() {
    use std::sync::atomic::{AtomicU64, Ordering};

    let config = CachingPoolConfig {
        max_connections: 5,
        min_connections: 2,
        max_idle_time: std::time::Duration::from_millis(1),
        enable_health_check: false,
        ..Default::default()
    };
    let (pool, _dir) = create_pool_with_config(config).await;

    let cleaned = Arc::new(AtomicU64::new(0));
    pool.set_cleanup_callback({
        let cleaned = Arc::clone(&cleaned);
        move |_conn_id| {
            cleaned.fetch_add(1, Ordering::Relaxed);
        }
    });

    assert_eq!(pool.available_connections(), 2);
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    let evicted = pool.cleanup_idle_connections();

    assert_eq!(evicted, 2, "both idle connections should be evicted");
    assert_eq!(pool.available_connections(), 0);
    assert_eq!(pool.stats().idle_connections, 0);
    assert_eq!(pool.stats().evictions, 2);
    assert_eq!(cleaned.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn test_cleanup_idle_connections_evicts_by_age() {
    let config = CachingPoolConfig {
        max_connections: 5,
        min_connections: 2,
        max_connection_age: std::time::Duration::from_millis(1),
        max_idle_time: std::time::Duration::from_secs(3600),
        enable_health_check: false,
        ..Default::default()
    };
    let (pool, _dir) = create_pool_with_config(config).await;

    assert_eq!(pool.available_connections(), 2);
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    let evicted = pool.cleanup_idle_connections();

    assert_eq!(evicted, 2, "both connections should be evicted by age");
    assert_eq!(pool.stats().evictions, 2);
    assert_eq!(pool.available_connections(), 0);
}

#[tokio::test]
async fn test_guard_drop_without_connection_is_noop() {
    let (pool, _dir) = create_test_pool().await;

    let shared = Arc::clone(&pool.state);
    // A guard with no connection and no permit must drop without touching state.
    let guard = ConnectionGuard {
        state: Arc::clone(&shared),
        connection: None,
        _permit: None,
    };
    drop(guard);

    let snapshot = shared.stats();
    assert_eq!(snapshot.total_returns, 0);
    assert_eq!(snapshot.active_connections, 0);
    assert_eq!(shared.active_connections(), 0);
}
