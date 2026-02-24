//! Configuration for the advanced query cache

use super::query_cache_types::QueryType;
use std::collections::HashMap;
use std::time::Duration;

/// Default maximum number of cached query results
pub const DEFAULT_MAX_QUERIES: usize = 1000;
/// Default TTL for query results (5 minutes)
pub const DEFAULT_QUERY_TTL: Duration = Duration::from_secs(300);
/// Default hot query threshold (accesses to trigger background refresh)
pub const DEFAULT_HOT_THRESHOLD: u64 = 5;
/// Default refresh interval for hot queries (30 seconds before expiry)
pub const DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

/// Configuration for the advanced query cache
#[derive(Debug, Clone)]
pub struct AdvancedQueryCacheConfig {
    /// Maximum number of cached queries
    pub max_queries: usize,
    /// Default TTL for cached results
    pub default_ttl: Duration,
    /// TTL overrides by query type
    pub ttl_overrides: HashMap<QueryType, Duration>,
    /// Hot query threshold for background refresh
    pub hot_threshold: u64,
    /// Refresh interval before expiry
    pub refresh_interval: Duration,
    /// Enable background refresh
    pub enable_background_refresh: bool,
    /// Enable dependency tracking
    pub enable_dependency_tracking: bool,
    /// Maximum dependency entries per query
    pub max_dependencies: usize,
}

impl Default for AdvancedQueryCacheConfig {
    fn default() -> Self {
        let mut ttl_overrides = HashMap::new();
        ttl_overrides.insert(QueryType::Statistics, Duration::from_secs(60));
        ttl_overrides.insert(QueryType::Embedding, Duration::from_secs(1800));

        Self {
            max_queries: DEFAULT_MAX_QUERIES,
            default_ttl: DEFAULT_QUERY_TTL,
            ttl_overrides,
            hot_threshold: DEFAULT_HOT_THRESHOLD,
            refresh_interval: DEFAULT_REFRESH_INTERVAL,
            enable_background_refresh: true,
            enable_dependency_tracking: true,
            max_dependencies: 10,
        }
    }
}

impl AdvancedQueryCacheConfig {
    /// Get TTL for a specific query type
    pub fn ttl_for_type(&self, query_type: QueryType) -> Duration {
        self.ttl_overrides
            .get(&query_type)
            .copied()
            .unwrap_or_else(|| query_type.default_ttl())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AdvancedQueryCacheConfig::default();
        assert_eq!(config.max_queries, 1000);
        assert_eq!(config.default_ttl, Duration::from_secs(300));
        assert!(config.enable_background_refresh);
        assert!(config.enable_dependency_tracking);
    }

    #[test]
    fn test_ttl_for_type() {
        let config = AdvancedQueryCacheConfig::default();
        assert_eq!(
            config.ttl_for_type(QueryType::Statistics),
            Duration::from_secs(60)
        );
        assert_eq!(
            config.ttl_for_type(QueryType::Embedding),
            Duration::from_secs(1800)
        );
    }
}
