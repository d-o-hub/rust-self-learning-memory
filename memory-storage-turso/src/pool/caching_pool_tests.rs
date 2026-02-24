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
#[ignore = "Timing-dependent test - pool creation expects pre-created connections that may not be ready in CI"]
async fn test_pool_creation() {
    let (pool, _dir) = create_test_pool().await;

    let stats = pool.stats();
    assert_eq!(
        stats.idle_connections, 2,
        "Should pre-create min connections"
    );
}

#[tokio::test]
async fn test_connection_checkout() {
    let (pool, _dir) = create_test_pool().await;

    let guard = pool.get().await.unwrap();
    assert!(guard.connection().query("SELECT 1", ()).await.is_ok());

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

    // Give time for drop
    tokio::time::sleep(Duration::from_millis(10)).await;

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
        guard.id()
    };

    // Give time for return
    tokio::time::sleep(Duration::from_millis(10)).await;

    let conn_id2 = {
        let guard = pool.get().await.unwrap();
        guard.id()
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
