//! Query result caching for MCP operations
//!
//! This module provides caching functionality for expensive MCP operations
//! to improve performance and reduce redundant computations.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime};

/// Configuration for query caching
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Cache TTL in seconds (default: 7 minutes = 420 seconds)
    pub ttl_seconds: u64,
    /// Maximum number of cached entries
    pub max_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 420, // 7 minutes
            max_entries: 1000,
        }
    }
}

/// Cache entry with timestamp and data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry<T> {
    /// Cached data
    data: T,
    /// Timestamp when entry was created
    created_at: SystemTime,
    /// TTL for this entry
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            created_at: SystemTime::now(),
            ttl,
        }
    }

    /// Check if entry is expired
    fn is_expired(&self) -> bool {
        self.created_at.elapsed().unwrap_or(Duration::MAX) > self.ttl
    }
}

/// Cache key for query_memory operations
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct QueryMemoryKey {
    pub query: String,
    pub domain: String,
    pub task_type: Option<String>,
    pub limit: usize,
}

impl QueryMemoryKey {
    pub fn new(query: String, domain: String, task_type: Option<String>, limit: usize) -> Self {
        Self {
            query,
            domain,
            task_type,
            limit,
        }
    }
}

/// Cache key for analyze_patterns operations
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct AnalyzePatternsKey {
    pub task_type: String,
    pub min_success_rate: u32, // Store as integer for hashing
    pub limit: usize,
}

impl AnalyzePatternsKey {
    pub fn new(task_type: String, min_success_rate: f32, limit: usize) -> Self {
        Self {
            task_type,
            min_success_rate: (min_success_rate * 100.0) as u32, // Convert to integer for hashing
            limit,
        }
    }
}

/// Cache key for execute_agent_code operations
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ExecuteCodeKey {
    pub code_hash: u64, // Hash of the code for caching
    pub context_task: String,
    pub context_input_hash: u64, // Hash of input JSON
}

impl ExecuteCodeKey {
    pub fn new(code: &str, context: &super::ExecutionContext) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        code.hash(&mut hasher);
        let code_hash = hasher.finish();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        context.input.to_string().hash(&mut hasher);
        let context_input_hash = hasher.finish();

        Self {
            code_hash,
            context_task: context.task.clone(),
            context_input_hash,
        }
    }
}

/// Query result cache for MCP operations
pub struct QueryCache {
    config: CacheConfig,
    /// Cache for query_memory results
    query_memory_cache: RwLock<HashMap<QueryMemoryKey, CacheEntry<serde_json::Value>>>,
    /// Cache for analyze_patterns results
    analyze_patterns_cache: RwLock<HashMap<AnalyzePatternsKey, CacheEntry<serde_json::Value>>>,
    /// Cache for execute_agent_code results
    execute_code_cache: RwLock<HashMap<ExecuteCodeKey, CacheEntry<super::ExecutionResult>>>,
}

impl QueryCache {
    /// Create a new query cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new query cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            config,
            query_memory_cache: RwLock::new(HashMap::new()),
            analyze_patterns_cache: RwLock::new(HashMap::new()),
            execute_code_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get cached query_memory result
    pub fn get_query_memory(&self, key: &QueryMemoryKey) -> Option<serde_json::Value> {
        if !self.config.enabled {
            return None;
        }

        let cache = self.query_memory_cache.read();
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Cache query_memory result
    pub fn put_query_memory(&self, key: QueryMemoryKey, result: serde_json::Value) {
        if !self.config.enabled {
            return;
        }

        let mut cache = self.query_memory_cache.write();
        self.evict_expired_entries(&mut cache);

        // Evict oldest entries if at capacity
        if cache.len() >= self.config.max_entries {
            self.evict_oldest(&mut cache);
        }

        let ttl = Duration::from_secs(self.config.ttl_seconds);
        cache.insert(key, CacheEntry::new(result, ttl));
    }

    /// Get cached analyze_patterns result
    pub fn get_analyze_patterns(&self, key: &AnalyzePatternsKey) -> Option<serde_json::Value> {
        if !self.config.enabled {
            return None;
        }

        let cache = self.analyze_patterns_cache.read();
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Cache analyze_patterns result
    pub fn put_analyze_patterns(&self, key: AnalyzePatternsKey, result: serde_json::Value) {
        if !self.config.enabled {
            return;
        }

        let mut cache = self.analyze_patterns_cache.write();
        self.evict_expired_entries(&mut cache);

        // Evict oldest entries if at capacity
        if cache.len() >= self.config.max_entries {
            self.evict_oldest(&mut cache);
        }

        let ttl = Duration::from_secs(self.config.ttl_seconds);
        cache.insert(key, CacheEntry::new(result, ttl));
    }

    /// Get cached execute_agent_code result
    pub fn get_execute_code(&self, key: &ExecuteCodeKey) -> Option<super::ExecutionResult> {
        if !self.config.enabled {
            return None;
        }

        let cache = self.execute_code_cache.read();
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Cache execute_agent_code result
    pub fn put_execute_code(&self, key: ExecuteCodeKey, result: super::ExecutionResult) {
        if !self.config.enabled {
            return;
        }

        let mut cache = self.execute_code_cache.write();
        self.evict_expired_entries(&mut cache);

        // Evict oldest entries if at capacity
        if cache.len() >= self.config.max_entries {
            self.evict_oldest(&mut cache);
        }

        let ttl = Duration::from_secs(self.config.ttl_seconds);
        cache.insert(key, CacheEntry::new(result, ttl));
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        self.query_memory_cache.write().clear();
        self.analyze_patterns_cache.write().clear();
        self.execute_code_cache.write().clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let query_memory = self.query_memory_cache.read();
        let analyze_patterns = self.analyze_patterns_cache.read();
        let execute_code = self.execute_code_cache.read();

        CacheStats {
            query_memory_entries: query_memory.len(),
            analyze_patterns_entries: analyze_patterns.len(),
            execute_code_entries: execute_code.len(),
            total_entries: query_memory.len() + analyze_patterns.len() + execute_code.len(),
            max_entries: self.config.max_entries,
            enabled: self.config.enabled,
            ttl_seconds: self.config.ttl_seconds,
        }
    }

    /// Evict expired entries from a cache
    fn evict_expired_entries<T, U>(&self, cache: &mut HashMap<T, CacheEntry<U>>)
    where
        T: Eq + Hash + Clone,
        U: Clone,
    {
        cache.retain(|_, entry| !entry.is_expired());
    }

    /// Evict oldest entries when at capacity (LRU-style)
    fn evict_oldest<T, U>(&self, cache: &mut HashMap<T, CacheEntry<U>>)
    where
        T: Eq + Hash + Clone,
        U: Clone,
    {
        if cache.is_empty() {
            return;
        }

        // Find the oldest entry
        let mut oldest_key = None;
        let mut oldest_time = SystemTime::now();

        for (key, entry) in cache.iter() {
            if entry.created_at < oldest_time {
                oldest_time = entry.created_at;
                oldest_key = Some(key.clone());
            }
        }

        if let Some(key) = oldest_key {
            cache.remove(&key);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub query_memory_entries: usize,
    pub analyze_patterns_entries: usize,
    pub execute_code_entries: usize,
    pub total_entries: usize,
    pub max_entries: usize,
    pub enabled: bool,
    pub ttl_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.ttl_seconds, 420); // 7 minutes
        assert_eq!(config.max_entries, 1000);
    }

    #[test]
    fn test_query_memory_cache() {
        let cache = QueryCache::new();

        let key = QueryMemoryKey::new(
            "test query".to_string(),
            "web-api".to_string(),
            Some("code_generation".to_string()),
            10,
        );

        let result = json!({"test": "data"});

        // Should not be cached initially
        assert!(cache.get_query_memory(&key).is_none());

        // Cache the result
        cache.put_query_memory(key.clone(), result.clone());

        // Should be cached now
        let cached = cache.get_query_memory(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), result);
    }

    #[test]
    fn test_analyze_patterns_cache() {
        let cache = QueryCache::new();

        let key = AnalyzePatternsKey::new("code_generation".to_string(), 0.7, 20);
        let result = json!({"patterns": []});

        // Should not be cached initially
        assert!(cache.get_analyze_patterns(&key).is_none());

        // Cache the result
        cache.put_analyze_patterns(key.clone(), result.clone());

        // Should be cached now
        let cached = cache.get_analyze_patterns(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), result);
    }

    #[test]
    fn test_execute_code_cache() {
        let cache = QueryCache::new();

        let context = crate::ExecutionContext::new("test".to_string(), json!({}));
        let key = ExecuteCodeKey::new("console.log('test')", &context);

        let result = crate::ExecutionResult::Success {
            output: "test output".to_string(),
            stdout: "test".to_string(),
            stderr: "".to_string(),
            execution_time_ms: 100,
        };

        // Should not be cached initially
        assert!(cache.get_execute_code(&key).is_none());

        // Cache the result
        cache.put_execute_code(key.clone(), result.clone());

        // Should be cached now
        let cached = cache.get_execute_code(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), result);
    }

    #[test]
    fn test_cache_disabled() {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };
        let cache = QueryCache::with_config(config);

        let key = QueryMemoryKey::new("test".to_string(), "test".to_string(), None, 10);
        let result = json!({"test": true});

        // Should not cache when disabled
        cache.put_query_memory(key.clone(), result);
        assert!(cache.get_query_memory(&key).is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = QueryCache::new();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 0);
        assert!(stats.enabled);
        assert_eq!(stats.ttl_seconds, 420);

        // Add some entries
        let key1 = QueryMemoryKey::new("test1".to_string(), "domain1".to_string(), None, 10);
        let key2 = AnalyzePatternsKey::new("task1".to_string(), 0.8, 15);

        cache.put_query_memory(key1, json!({"test": 1}));
        cache.put_analyze_patterns(key2, json!({"patterns": 2}));

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.query_memory_entries, 1);
        assert_eq!(stats.analyze_patterns_entries, 1);
        assert_eq!(stats.execute_code_entries, 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = QueryCache::new();

        let key = QueryMemoryKey::new("test".to_string(), "test".to_string(), None, 10);
        cache.put_query_memory(key.clone(), json!({"test": true}));

        assert!(cache.get_query_memory(&key).is_some());

        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 0);
        assert!(cache.get_query_memory(&key).is_none());
    }
}
