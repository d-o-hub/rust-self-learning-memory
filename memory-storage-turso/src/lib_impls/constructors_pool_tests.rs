//! Tests for the pool constructors (requires the `keepalive-pool` feature).

use crate::prepared::PreparedCacheConfig;
use crate::{KeepAliveConfig, PoolConfig, TursoConfig, TursoStorage};
use std::time::Duration;

fn config_with_capacity(max_size: usize) -> TursoConfig {
    TursoConfig {
        // The pool constructors build the pool themselves; this only keeps the
        // stored config consistent with the rest of the test.
        enable_pooling: false,
        prepared_cache_config: Some(PreparedCacheConfig {
            max_size,
            ..Default::default()
        }),
        ..TursoConfig::default()
    }
}

#[tokio::test]
async fn new_with_pool_config_honours_prepared_cache_capacity() {
    let storage = TursoStorage::new_with_pool_config(
        ":memory:",
        "",
        config_with_capacity(9),
        PoolConfig::default(),
    )
    .await
    .expect("pooled in-memory storage should build");

    assert_eq!(storage.prepared_cache.max_size(), 9);
}

#[tokio::test]
async fn new_with_keepalive_honours_prepared_cache_capacity() {
    let keepalive_config = KeepAliveConfig {
        keep_alive_interval: Duration::from_secs(60),
        stale_threshold: Duration::from_secs(120),
        enable_proactive_ping: false,
        ping_timeout: Duration::from_secs(5),
    };

    let storage = TursoStorage::new_with_keepalive(
        ":memory:",
        "",
        config_with_capacity(11),
        PoolConfig::default(),
        keepalive_config,
    )
    .await
    .expect("keep-alive in-memory storage should build");

    assert_eq!(storage.prepared_cache.max_size(), 11);
}
