//! Integration tests for connection pool performance and functionality
//!
//! Tests connection pool behavior including concurrent operations, health checks,
//! utilization tracking, and graceful shutdown.

#![allow(clippy::float_cmp)]
#![allow(missing_docs)]

use do_memory_core::embeddings::EmbeddingStorageBackend;
use do_memory_storage_turso::{
    AdaptivePoolConfig, ConnectionPool, PoolConfig, TursoConfig, TursoStorage,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

async fn create_test_pool() -> anyhow::Result<(Arc<ConnectionPool>, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await?;

    let config = PoolConfig {
        min_connections: 1,
        max_connections: 10,
        connection_timeout: Duration::from_secs(5),
        enable_health_check: true,
        health_check_timeout: Duration::from_secs(2),
        acquire_timeout_ms: 5000,
        idle_timeout_ms: 0,
    };

    let pool = ConnectionPool::new(Arc::new(db), config).await?;
    Ok((Arc::new(pool), dir))
}

#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_pool_performance_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _dir) = create_test_pool().await?;

    let start = Instant::now();
    let mut handles = vec![];

    // Spawn 100 concurrent operations
    for _ in 0..100 {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let conn = pool_clone.get().await?;
            // Simulate database work
            let result = conn
                .connection()
                .ok_or_else(|| anyhow::anyhow!("Failed to get connection"))?
                .query("SELECT 1", ())
                .await;
            assert!(result.is_ok());
            tokio::time::sleep(Duration::from_millis(5)).await;
            anyhow::Ok(())
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await??;
    }

    let elapsed = start.elapsed();

    // Wait a bit for all drops to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    let stats = pool.statistics().await;

    println!("100 concurrent operations completed in: {elapsed:?}");
    println!("Total checkouts: {}", stats.total_checkouts);
    println!("Total created connections: {}", stats.total_created);
    println!("Avg wait time: {}ms", stats.avg_wait_time_ms);

    // Verify performance targets
    assert_eq!(stats.total_checkouts, 100);
    assert_eq!(stats.total_created, 100); // Creates a new connection for each request
    assert!(elapsed.as_millis() < 5000); // Should complete within 5 seconds (P95 target)

    // Verify concurrency was limited (no more than pool size active at once)
    // The semaphore ensures max 10 concurrent connections
    assert_eq!(stats.active_connections, 0); // All should be returned after test completes

    // Ensure pool is cleanly shutdown before TempDir drops to avoid Windows file handle races
    let _ = pool.shutdown().await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_pool_with_turso_storage() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await?;

    let config = PoolConfig {
        min_connections: 1,
        max_connections: 5,
        connection_timeout: Duration::from_secs(5),
        enable_health_check: true,
        health_check_timeout: Duration::from_secs(2),
        acquire_timeout_ms: 5000,
        idle_timeout_ms: 0,
    };

    let pool = ConnectionPool::new(Arc::new(db), config).await?;

    // Test multiple sequential operations
    for i in 0..10 {
        let conn = pool.get().await?;
        let result = conn
            .connection()
            .ok_or_else(|| anyhow::anyhow!("Failed to get connection"))?
            .query("SELECT 1", ())
            .await;
        assert!(result.is_ok(), "Query {i} failed");
    }

    let stats = pool.statistics().await;
    assert_eq!(stats.total_checkouts, 10);

    // Clean shutdown before TempDir is dropped
    let _ = pool.shutdown().await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_pool_utilization_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _dir) = create_test_pool().await?;

    // Initially no utilization
    assert_eq!(pool.utilization().await, 0.0);

    // Get connections and check utilization increases
    let conn1 = pool.get().await?;
    assert!(pool.utilization().await > 0.0);

    let conn2 = pool.get().await?;
    assert!(pool.utilization().await > 0.1);

    drop(conn1);
    drop(conn2);

    // Wait for drops to complete
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Utilization should be back to 0
    assert_eq!(pool.utilization().await, 0.0);

    // Clean shutdown
    let _ = pool.shutdown().await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_pool_health_checks() -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _dir) = create_test_pool().await?;

    // Get multiple connections, all should pass health checks
    for _ in 0..5 {
        let _conn = pool.get().await?;
    }

    let stats = pool.statistics().await;
    assert_eq!(stats.total_health_checks_passed, 5);
    assert_eq!(stats.total_health_checks_failed, 0);

    // Shutdown pool before TempDir cleanup
    let _ = pool.shutdown().await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_pool_graceful_shutdown() -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _dir) = create_test_pool().await?;

    // Perform some operations
    {
        let _conn1 = pool.get().await?;
        let _conn2 = pool.get().await?;
    }

    // Wait for connections to be returned
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Shutdown should complete cleanly (already called in test)
    let result = pool.shutdown().await;
    assert!(result.is_ok());

    // Small pause to ensure libsql releases handles on Windows before directory removal
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_pool_statistics_accuracy() -> Result<(), Box<dyn std::error::Error>> {
    let (pool, _dir) = create_test_pool().await?;

    // Get and use 3 connections
    for _ in 0..3 {
        let conn = pool.get().await?;
        let _result = conn
            .connection()
            .ok_or_else(|| anyhow::anyhow!("Failed to get connection"))?
            .query("SELECT 1", ())
            .await?;
        drop(conn);
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify statistics
    let stats = pool.statistics().await;
    assert_eq!(stats.total_checkouts, 3);
    assert!(stats.total_created >= 3);
    assert_eq!(stats.total_health_checks_passed, 3);
    assert_eq!(stats.active_connections, 0);

    // Ensure the pool is shut down before TempDir is dropped
    let _ = pool.shutdown().await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

/// A scoped checkout retains its pool permit for the whole operation: while a
/// slow operation is in flight, a saturated pool must reject the next checkout,
/// and capacity must return only after that operation has completed.
#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_scoped_checkout_retains_permit_until_operation_returns()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("scoped-saturation.db");
    let db = libsql::Builder::new_local(&db_path).build().await?;

    // One permit and a short checkout timeout: saturation is deterministic and
    // the test stays fast.
    let config = PoolConfig {
        min_connections: 1,
        max_connections: 1,
        connection_timeout: Duration::from_millis(250),
        enable_health_check: false,
        health_check_timeout: Duration::from_secs(1),
        acquire_timeout_ms: 250,
        idle_timeout_ms: 0,
    };
    let pool = Arc::new(ConnectionPool::new(Arc::new(db), config).await?);

    let (started_tx, mut started_rx) = tokio::sync::mpsc::channel::<()>(1);
    let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();

    // First operation holds the only permit until explicitly released.
    let slow_pool = Arc::clone(&pool);
    let slow = tokio::spawn(async move {
        slow_pool
            .with_connection(async move |conn| {
                conn.query("SELECT 1", ())
                    .await
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
                started_tx.send(()).await.ok();
                let _ = release_rx.await;
                conn.query("SELECT 1", ())
                    .await
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
                Ok(())
            })
            .await
    });

    // Wait until the first operation is inside its closure.
    started_rx
        .recv()
        .await
        .expect("first operation should start");

    // Permit still held: no capacity is reported and the next checkout times out
    // instead of being handed a connection whose permit was already released.
    assert_eq!(pool.available_connections().await, 0);
    let second = pool
        .with_connection(async |conn| {
            conn.query("SELECT 1", ())
                .await
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            Ok(())
        })
        .await;
    assert!(
        second.is_err(),
        "a saturated pool must reject the next checkout until the first returns"
    );
    assert_eq!(pool.statistics().await.active_connections, 1);

    // Releasing the first operation returns the permit.
    release_tx.send(()).expect("release signal");
    slow.await
        .map_err(|e| anyhow::anyhow!("slow task join failed: {e}"))?
        .map_err(|e| anyhow::anyhow!("slow operation failed: {e}"))?;

    assert_eq!(pool.statistics().await.active_connections, 0);
    assert_eq!(pool.available_connections().await, 1);

    // The freed permit is usable again.
    pool.with_connection(async |conn| {
        conn.query("SELECT 1", ())
            .await
            .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
        Ok(())
    })
    .await
    .map_err(|e| anyhow::anyhow!("post-release operation failed: {e}"))?;

    let _ = pool.shutdown().await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

/// `with_connection_with_id` clears the checkout's prepared-statement cache only
/// after the operation returns.
#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_with_connection_with_id_clears_cache_after_operation()
-> Result<(), Box<dyn std::error::Error>> {
    const SQL: &str = "SELECT 1";

    let db = libsql::Builder::new_local(":memory:").build().await?;
    let storage = TursoStorage::from_database(db)?;

    let (id_tx, mut id_rx) = tokio::sync::mpsc::unbounded_channel::<u64>();

    storage
        .with_connection_with_id(async |conn, conn_id| {
            // Record a cache entry for this checkout, as a real operation would.
            drop(
                storage
                    .prepared_cache()
                    .get_or_prepare_with_id(conn_id, conn, SQL)
                    .await
                    .map_err(|e| {
                        do_memory_core::Error::Storage(format!("Failed to prepare statement: {e}"))
                    })?,
            );
            assert!(
                storage.prepared_cache().is_cached(conn_id, SQL),
                "statement should be cached while the operation runs"
            );
            id_tx.send(conn_id).ok();
            Ok(())
        })
        .await
        .map_err(|e| anyhow::anyhow!("scoped operation failed: {e}"))?;

    let conn_id = id_rx
        .recv()
        .await
        .expect("closure should report its connection id");
    assert!(
        !storage.prepared_cache().is_cached(conn_id, SQL),
        "prepared cache must be cleared after the operation returns"
    );
    Ok(())
}

/// A panic inside the scoped operation must not leave the checkout's
/// prepared-statement cache entry behind: the guard clears it on unwinding, and
/// the storage stays usable afterwards.
#[cfg_attr(target_os = "windows", ignore)]
#[expect(
    clippy::panic,
    reason = "the unwinding path is the subject of this test"
)]
#[tokio::test]
async fn test_with_connection_with_id_clears_cache_when_operation_panics()
-> Result<(), Box<dyn std::error::Error>> {
    use futures::FutureExt;

    const SQL: &str = "SELECT 1";

    let db = libsql::Builder::new_local(":memory:").build().await?;
    let storage = TursoStorage::from_database(db)?;

    let (id_tx, mut id_rx) = tokio::sync::mpsc::unbounded_channel::<u64>();

    let panicked = std::panic::AssertUnwindSafe(storage.with_connection_with_id(
        async |conn, conn_id| -> do_memory_core::Result<()> {
            // Record a cache entry for this checkout, as a real operation would.
            drop(
                storage
                    .prepared_cache()
                    .get_or_prepare_with_id(conn_id, conn, SQL)
                    .await
                    .map_err(|e| {
                        do_memory_core::Error::Storage(format!("Failed to prepare statement: {e}"))
                    })?,
            );
            id_tx.send(conn_id).ok();
            panic!("operation failed after preparing a statement");
        },
    ))
    .catch_unwind()
    .await;

    assert!(
        panicked.is_err(),
        "the scoped operation should have panicked"
    );

    let conn_id = id_rx
        .recv()
        .await
        .expect("closure should report its connection id");
    assert!(
        !storage.prepared_cache().is_cached(conn_id, SQL),
        "a panicking operation must not leave its prepared statements cached"
    );

    // The checkout was released during unwinding, so the storage still answers.
    storage
        .with_connection(async |conn| {
            conn.query(SQL, ()).await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Storage unusable after panic: {e}"))
            })?;
            Ok(())
        })
        .await
        .map_err(|e| anyhow::anyhow!("scoped operation failed after panic: {e}"))?;

    Ok(())
}

/// The adaptive pool exposes the same scoped contract: a saturated pool rejects
/// the next checkout until the in-flight operation releases its permit.
#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_adaptive_pool_shares_scoped_checkout_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("adaptive-scoped.db");

    let adaptive = AdaptivePoolConfig {
        min_connections: 1,
        max_connections: 1,
        check_interval: Duration::from_millis(250),
        ..Default::default()
    };
    let storage = Arc::new(
        TursoStorage::new_with_adaptive_pool(
            &format!("file:{}", db_path.display()),
            "",
            TursoConfig::default(),
            adaptive,
        )
        .await?,
    );

    let (started_tx, mut started_rx) = tokio::sync::mpsc::channel::<()>(1);
    let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();

    let slow_storage = Arc::clone(&storage);
    let slow = tokio::spawn(async move {
        slow_storage
            .with_connection(async move |conn| {
                conn.query("SELECT 1", ())
                    .await
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
                started_tx.send(()).await.ok();
                let _ = release_rx.await;
                Ok(())
            })
            .await
    });

    started_rx
        .recv()
        .await
        .expect("first operation should start");
    assert_eq!(
        storage.adaptive_pool_size().map(|(active, _)| active),
        Some(1)
    );

    let second = storage
        .with_connection(async |conn| {
            conn.query("SELECT 1", ())
                .await
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            Ok(())
        })
        .await;
    assert!(
        second.is_err(),
        "adaptive pool must reject a saturated checkout"
    );

    release_tx.send(()).expect("release signal");
    slow.await
        .map_err(|e| anyhow::anyhow!("slow task join failed: {e}"))?
        .map_err(|e| anyhow::anyhow!("slow operation failed: {e}"))?;
    assert_eq!(
        storage.adaptive_pool_size().map(|(active, _)| active),
        Some(0)
    );
    Ok(())
}

/// The brute-force similarity fallback runs on the already-held checkout: with a
/// single-permit pool the search must return results instead of timing out on a
/// nested checkout.
#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_similarity_search_fallback_uses_held_checkout()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("similarity-single-permit.db");

    let adaptive = AdaptivePoolConfig {
        min_connections: 1,
        max_connections: 1,
        check_interval: Duration::from_millis(250),
        ..Default::default()
    };
    let storage = TursoStorage::new_with_adaptive_pool(
        &format!("file:{}", db_path.display()),
        "",
        TursoConfig::default(),
        adaptive,
    )
    .await?;
    storage.initialize_schema().await?;

    // Empty store: the native vector path is unavailable, so the search falls
    // back to brute force on the same checked-out connection.
    let results = storage
        .find_similar_episodes(vec![0.1; 384], 10, 0.5)
        .await
        .map_err(|e| anyhow::anyhow!("similarity search failed: {e}"))?;
    assert_eq!(results.len(), 0, "an empty store must return no neighbours");

    // The single permit was released once the operation returned.
    assert_eq!(
        storage.adaptive_pool_size().map(|(active, _)| active),
        Some(0)
    );
    Ok(())
}

/// N concurrent similarity searches on an N-permit pool must not self-occupy on
/// nested checkouts (the fallback path runs on the held connection).
#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_concurrent_similarity_searches_do_not_self_occupy()
-> Result<(), Box<dyn std::error::Error>> {
    const CONCURRENCY: u32 = 4;
    let concurrency = usize::try_from(CONCURRENCY).expect("concurrency fits in usize");

    let dir = TempDir::new()?;
    let db_path = dir.path().join("similarity-concurrent.db");

    let adaptive = AdaptivePoolConfig {
        min_connections: CONCURRENCY,
        max_connections: CONCURRENCY,
        check_interval: Duration::from_millis(500),
        ..Default::default()
    };
    let storage = Arc::new(
        TursoStorage::new_with_adaptive_pool(
            &format!("file:{}", db_path.display()),
            "",
            TursoConfig::default(),
            adaptive,
        )
        .await?,
    );
    storage.initialize_schema().await?;

    // Start every search at the same instant so they all hold a permit while the
    // fallback runs.
    let barrier = Arc::new(tokio::sync::Barrier::new(concurrency));
    let mut handles = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let storage = Arc::clone(&storage);
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            storage.find_similar_episodes(vec![0.1; 384], 10, 0.5).await
        }));
    }

    for handle in handles {
        let results = handle
            .await
            .map_err(|e| anyhow::anyhow!("search task join failed: {e}"))?
            .map_err(|e| anyhow::anyhow!("search failed: {e}"))?;
        assert_eq!(results.len(), 0, "an empty store must return no neighbours");
    }

    assert_eq!(
        storage.adaptive_pool_size().map(|(active, _)| active),
        Some(0)
    );
    Ok(())
}

/// The keep-alive pool shares the same scoped contract: a saturated pool rejects
/// the next checkout until the in-flight operation releases its permit.
#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_keepalive_pool_shares_scoped_checkout_contract()
-> Result<(), Box<dyn std::error::Error>> {
    use do_memory_storage_turso::pool::KeepAlivePool;

    let dir = TempDir::new()?;
    let db_path = dir.path().join("keepalive-scoped.db");
    let db = libsql::Builder::new_local(&db_path).build().await?;

    let config = PoolConfig {
        min_connections: 1,
        max_connections: 1,
        connection_timeout: Duration::from_millis(250),
        enable_health_check: false,
        health_check_timeout: Duration::from_secs(1),
        acquire_timeout_ms: 250,
        idle_timeout_ms: 0,
    };
    let pool = Arc::new(ConnectionPool::new(Arc::new(db), config).await?);
    let keepalive = Arc::new(KeepAlivePool::new(Arc::clone(&pool), None).await?);

    let (started_tx, mut started_rx) = tokio::sync::mpsc::channel::<()>(1);
    let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();

    let slow_keepalive = Arc::clone(&keepalive);
    let slow = tokio::spawn(async move {
        slow_keepalive
            .with_connection(async move |conn| {
                conn.query("SELECT 1", ())
                    .await
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
                started_tx.send(()).await.ok();
                let _ = release_rx.await;
                Ok(())
            })
            .await
    });

    started_rx
        .recv()
        .await
        .expect("first operation should start");
    assert_eq!(pool.statistics().await.active_connections, 1);

    let second = keepalive
        .with_connection(async |conn| {
            conn.query("SELECT 1", ())
                .await
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            Ok(())
        })
        .await;
    assert!(
        second.is_err(),
        "a saturated keep-alive pool must reject the next checkout"
    );

    release_tx.send(()).expect("release signal");
    slow.await
        .map_err(|e| anyhow::anyhow!("slow task join failed: {e}"))?
        .map_err(|e| anyhow::anyhow!("slow operation failed: {e}"))?;

    assert_eq!(pool.statistics().await.active_connections, 0);
    let _ = pool.shutdown().await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

/// The raw-returning helpers are direct-connection mode only and must reject a
/// pooled storage instead of handing out a connection without its permit.
#[cfg_attr(target_os = "windows", ignore)]
#[tokio::test]
async fn test_raw_checkout_is_rejected_when_pooled() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("pooled-raw-checkout.db");

    let adaptive = AdaptivePoolConfig {
        min_connections: 1,
        max_connections: 2,
        ..Default::default()
    };
    let storage = TursoStorage::new_with_adaptive_pool(
        &format!("file:{}", db_path.display()),
        "",
        TursoConfig::default(),
        adaptive,
    )
    .await?;

    assert!(
        storage.get_connection().await.is_err(),
        "get_connection must be rejected when a pool is configured"
    );
    assert!(
        storage.get_connection_with_id().await.is_err(),
        "get_connection_with_id must be rejected when a pool is configured"
    );
    Ok(())
}
