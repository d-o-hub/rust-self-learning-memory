//! Connection pool tests

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::Duration;

    async fn create_test_pool() -> (ConnectionPool, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

        let config = PoolConfig {
            max_connections: 5,
            connection_timeout: Duration::from_secs(5),
            enable_health_check: true,
            health_check_timeout: Duration::from_secs(2),
        };

        let pool = ConnectionPool::new(Arc::new(db), config).await.unwrap();
        (pool, dir)
    }

    #[tokio::test]
    async fn test_pool_creation() {
        let (pool, _dir) = create_test_pool().await;
        let stats = pool.statistics().await;

        // Pool should be created but no connections yet
        assert_eq!(stats.total_checkouts, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_connection_checkout() {
        let (pool, _dir) = create_test_pool().await;

        let conn = pool.get().await;
        assert!(conn.is_ok());

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 1);
        assert_eq!(stats.active_connections, 1);
        assert!(stats.total_created >= 1);
    }

    #[tokio::test]
    async fn test_connection_auto_return() {
        let (pool, _dir) = create_test_pool().await;

        {
            let _conn = pool.get().await.unwrap();
            let stats = pool.statistics().await;
            assert_eq!(stats.active_connections, 1);
        }

        // Wait for drop to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        let stats = pool.statistics().await;
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_concurrent_checkouts() {
        let (pool, _dir) = create_test_pool().await;
        let pool = Arc::new(pool);

        let mut handles = vec![];

        for i in 0..3 {
            let pool_clone = Arc::clone(&pool);
            let handle = tokio::spawn(async move {
                let conn = pool_clone.get().await;
                assert!(conn.is_ok(), "Checkout {} failed", i);

                // Simulate work
                tokio::time::sleep(Duration::from_millis(10)).await;

                conn
            });
            handles.push(handle);
        }

        // Wait for all checkouts to complete
        for handle in handles {
            let _ = handle.await.unwrap();
        }

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 3);
        assert!(stats.total_created >= 3);
    }

    #[tokio::test]
    async fn test_pool_statistics() {
        let (pool, _dir) = create_test_pool().await;

        let _conn = pool.get().await.unwrap();

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 1);
        assert_eq!(stats.active_connections, 1);
        assert!(stats.total_created >= 1);
        // total_wait_time_ms is u64, always >= 0
    }

    #[tokio::test]
    async fn test_average_wait_time() {
        let (pool, _dir) = create_test_pool().await;

        let _conn1 = pool.get().await.unwrap();
        let _conn2 = pool.get().await.unwrap();

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 2);
        // avg_wait_time_ms is u64, always >= 0
    }

    #[tokio::test]
    async fn test_pool_utilization() {
        let (pool, _dir) = create_test_pool().await;

        let utilization = pool.utilization().await;
        assert_eq!(utilization, 0.0);

        let _conn = pool.get().await.unwrap();

        let utilization = pool.utilization().await;
        assert!(utilization > 0.0 && utilization <= 1.0);
    }

    #[tokio::test]
    async fn test_available_connections() {
        let (pool, _dir) = create_test_pool().await;

        let available = pool.available_connections().await;
        assert_eq!(available, 5);

        let _conn1 = pool.get().await.unwrap();
        let available = pool.available_connections().await;
        assert_eq!(available, 4);

        let _conn2 = pool.get().await.unwrap();
        let available = pool.available_connections().await;
        assert_eq!(available, 3);
    }

    #[tokio::test]
    async fn test_has_capacity() {
        let (pool, _dir) = create_test_pool().await;

        assert!(pool.has_capacity().await);

        let _conns: Vec<_> = futures::future::join_all((0..5).map(|_| pool.get())).await;

        assert!(!pool.has_capacity().await);
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let (pool, _dir) = create_test_pool().await;

        let _conn = pool.get().await.unwrap();
        drop(_conn);

        // Wait for drop to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = pool.shutdown().await;
        assert!(result.is_ok());

        let stats = pool.statistics().await;
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_connection_timeout() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

        let config = PoolConfig {
            max_connections: 1,
            connection_timeout: Duration::from_millis(100),
            enable_health_check: false,
            health_check_timeout: Duration::from_secs(2),
        };

        let pool = Arc::new(ConnectionPool::new(Arc::new(db), config).await.unwrap());

        // Get the only available connection
        let _conn1 = pool.get().await.unwrap();

        // Try to get another connection - should timeout
        let result = pool.get().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let (pool, _dir) = create_test_pool().await;

        let conn = pool.get().await.unwrap();

        let stats = pool.statistics().await;
        assert_eq!(stats.total_health_checks_passed, 1);
        assert_eq!(stats.total_health_checks_failed, 0);

        drop(conn);
    }

    #[tokio::test]
    async fn test_connection_usage() {
        let (pool, _dir) = create_test_pool().await;

        let conn = pool.get().await.unwrap();

        // Use the connection
        let result = conn.connection().unwrap().query("SELECT 1", ()).await;
        assert!(result.is_ok());

        drop(conn);
    }

    #[tokio::test]
    async fn test_high_concurrency() {
        let (pool, _dir) = create_test_pool().await;
        let pool = Arc::new(pool);

        let mut handles = vec![];

        // Spawn 20 concurrent tasks (more than pool size of 5)
        for i in 0..20 {
            let pool_clone = Arc::clone(&pool);
            let handle = tokio::spawn(async move {
                let conn = pool_clone.get().await;
                assert!(conn.is_ok(), "Checkout {} failed", i);

                // Simulate work
                tokio::time::sleep(Duration::from_millis(5)).await;

                conn
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 20);
        assert!(stats.total_created >= 5);
    }
}
