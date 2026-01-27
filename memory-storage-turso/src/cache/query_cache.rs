//! Query Result Caching
//!
//! Provides caching for query results to reduce database round trips.
//! This is particularly useful for repeated queries with the same parameters.

use memory_core::{Episode, Pattern};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, trace};

/// Maximum number of cached query results
const DEFAULT_MAX_QUERIES: usize = 1000;
/// Default TTL for query results (5 minutes)
const DEFAULT_QUERY_TTL: Duration = Duration::from_secs(300);

/// A query key for caching
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueryKey {
    /// Query episodes since timestamp
    EpisodesSince(i64),
    /// Query episodes by metadata
    EpisodesByMetadata(String, String),
    /// Query episodes by domain
    EpisodesByDomain(String),
    /// Query patterns by effectiveness
    PatternsByEffectiveness(String),
    /// Custom query with SQL hash
    Custom(String),
}

impl QueryKey {
    /// Create a query key from a SQL string
    pub fn from_sql(sql: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        let hash = hasher.finish();
        Self::Custom(format!("{:x}", hash))
    }
}

/// A cached query result
#[derive(Clone)]
struct CachedQueryResult<T> {
    /// The cached result
    result: T,
    /// When the result was cached
    cached_at: Instant,
    /// TTL for this result
    ttl: Duration,
    /// Number of times accessed
    access_count: u64,
}

impl<T> CachedQueryResult<T> {
    fn new(result: T, ttl: Duration) -> Self {
        Self {
            result,
            cached_at: Instant::now(),
            ttl,
            access_count: 0,
        }
    }

    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    fn increment_access(&mut self) {
        self.access_count += 1;
    }
}

/// Statistics for query cache
#[derive(Debug, Clone, Default)]
pub struct QueryCacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total evictions due to capacity
    pub evictions: u64,
    /// Total expirations due to TTL
    pub expirations: u64,
    /// Current cache size
    pub current_size: usize,
}

impl QueryCacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Query result cache for episodes and patterns
pub struct QueryCache {
    /// Cache for episode query results
    episode_queries: Arc<RwLock<HashMap<QueryKey, CachedQueryResult<Vec<Episode>>>>>,
    /// Cache for pattern query results
    pattern_queries: Arc<RwLock<HashMap<QueryKey, CachedQueryResult<Vec<Pattern>>>>>,
    /// Configuration
    max_queries: usize,
    default_ttl: Duration,
    /// Statistics
    stats: Arc<RwLock<QueryCacheStats>>,
}

impl QueryCache {
    /// Create a new query cache
    pub fn new(max_queries: usize, default_ttl: Duration) -> Self {
        Self {
            episode_queries: Arc::new(RwLock::new(HashMap::new())),
            pattern_queries: Arc::new(RwLock::new(HashMap::new())),
            max_queries,
            default_ttl,
            stats: Arc::new(RwLock::new(QueryCacheStats::default())),
        }
    }

    /// Create with default configuration
    #[allow(dead_code)]
    pub fn with_default_config() -> Self {
        Self::new(DEFAULT_MAX_QUERIES, DEFAULT_QUERY_TTL)
    }

    /// Get cached episode query result
    pub fn get_episodes(&self, key: &QueryKey) -> Option<Vec<Episode>> {
        let mut cache = self.episode_queries.write();

        if let Some(cached) = cache.get_mut(key) {
            if cached.is_expired() {
                trace!("Query cache expired for key: {:?}", key);
                cache.remove(key);
                let mut stats = self.stats.write();
                stats.misses += 1;
                stats.expirations += 1;
                stats.current_size = cache.len();
                None
            } else {
                debug!("Query cache hit for key: {:?}", key);
                cached.increment_access();
                let mut stats = self.stats.write();
                stats.hits += 1;
                Some(cached.result.clone())
            }
        } else {
            trace!("Query cache miss for key: {:?}", key);
            let mut stats = self.stats.write();
            stats.misses += 1;
            None
        }
    }

    /// Cache episode query result
    pub fn cache_episodes(&self, key: QueryKey, result: Vec<Episode>) {
        let mut cache = self.episode_queries.write();

        // Evict oldest entry if at capacity
        if cache.len() >= self.max_queries {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.cached_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
                let mut stats = self.stats.write();
                stats.evictions += 1;
            }
        }

        cache.insert(key, CachedQueryResult::new(result, self.default_ttl));
        let mut stats = self.stats.write();
        stats.current_size = cache.len();
    }

    /// Get cached pattern query result
    pub fn get_patterns(&self, key: &QueryKey) -> Option<Vec<Pattern>> {
        let mut cache = self.pattern_queries.write();

        if let Some(cached) = cache.get_mut(key) {
            if cached.is_expired() {
                trace!("Query cache expired for key: {:?}", key);
                cache.remove(key);
                let mut stats = self.stats.write();
                stats.misses += 1;
                stats.expirations += 1;
                stats.current_size = cache.len();
                None
            } else {
                debug!("Query cache hit for key: {:?}", key);
                cached.increment_access();
                let mut stats = self.stats.write();
                stats.hits += 1;
                Some(cached.result.clone())
            }
        } else {
            trace!("Query cache miss for key: {:?}", key);
            let mut stats = self.stats.write();
            stats.misses += 1;
            None
        }
    }

    /// Cache pattern query result
    pub fn cache_patterns(&self, key: QueryKey, result: Vec<Pattern>) {
        let mut cache = self.pattern_queries.write();

        // Evict oldest entry if at capacity
        if cache.len() >= self.max_queries {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.cached_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
                let mut stats = self.stats.write();
                stats.evictions += 1;
            }
        }

        cache.insert(key, CachedQueryResult::new(result, self.default_ttl));
        let mut stats = self.stats.write();
        stats.current_size = cache.len();
    }

    /// Clear all cached queries
    pub fn clear(&self) {
        self.episode_queries.write().clear();
        self.pattern_queries.write().clear();
        let mut stats = self.stats.write();
        stats.current_size = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> QueryCacheStats {
        self.stats.read().clone()
    }

    /// Clear expired entries
    pub fn clear_expired(&self) {
        let mut episode_cache = self.episode_queries.write();
        let mut pattern_cache = self.pattern_queries.write();

        let expired_episodes: Vec<_> = episode_cache
            .iter()
            .filter(|(_, v)| v.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        let expired_patterns: Vec<_> = pattern_cache
            .iter()
            .filter(|(_, v)| v.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_episodes {
            episode_cache.remove(&key);
        }

        for key in expired_patterns {
            pattern_cache.remove(&key);
        }

        let mut stats = self.stats.write();
        stats.current_size = episode_cache.len() + pattern_cache.len();
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_QUERIES, DEFAULT_QUERY_TTL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::{ComplexityLevel, Episode, TaskContext, TaskType};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_episode(id: Uuid) -> Episode {
        Episode {
            episode_id: id,
            task_type: TaskType::CodeGeneration,
            task_description: "Test task".to_string(),
            context: TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                domain: "test".to_string(),
                tags: vec![],
            },
            start_time: chrono::Utc::now(),
            end_time: None,
            steps: vec![],
            outcome: None,
            reward: None,
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            metadata: HashMap::new(),
            applied_patterns: vec![],
            salient_features: None,
            tags: vec![],
            tags: vec![],
        }
    }

    #[test]
    fn test_query_cache_basic() {
        let cache = QueryCache::default();
        let key = QueryKey::EpisodesByDomain("test".to_string());
        let episodes = vec![create_test_episode(Uuid::new_v4())];

        // Cache miss initially
        assert!(cache.get_episodes(&key).is_none());

        // Cache the result
        cache.cache_episodes(key.clone(), episodes.clone());

        // Cache hit now
        let cached = cache.get_episodes(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);

        // Verify stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_query_cache_expiration() {
        let cache = QueryCache::new(100, Duration::from_millis(50));
        let key = QueryKey::EpisodesByDomain("test".to_string());
        let episodes = vec![create_test_episode(Uuid::new_v4())];

        cache.cache_episodes(key.clone(), episodes);

        // Should be cached
        assert!(cache.get_episodes(&key).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(60));

        // Should be expired
        assert!(cache.get_episodes(&key).is_none());

        let stats = cache.stats();
        assert_eq!(stats.expirations, 1);
    }

    #[test]
    fn test_query_cache_eviction() {
        let cache = QueryCache::new(2, Duration::from_secs(60));

        let key1 = QueryKey::EpisodesByDomain("test1".to_string());
        let key2 = QueryKey::EpisodesByDomain("test2".to_string());
        let key3 = QueryKey::EpisodesByDomain("test3".to_string());

        let episodes = vec![create_test_episode(Uuid::new_v4())];

        // Fill cache to capacity
        cache.cache_episodes(key1.clone(), episodes.clone());
        cache.cache_episodes(key2.clone(), episodes.clone());

        // This should trigger eviction
        cache.cache_episodes(key3.clone(), episodes.clone());

        let stats = cache.stats();
        assert_eq!(stats.evictions, 1);
        assert_eq!(stats.current_size, 2);
    }
}
