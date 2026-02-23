//! Query cache types: QueryKey, QueryType, TableDependency, CachedResult
//!
//! This module provides the core types used by the query cache system.

use parking_lot::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Table dependency for cache invalidation tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TableDependency {
    /// Episodes table
    Episodes,
    /// Steps table
    Steps,
    /// Patterns table
    Patterns,
    /// Heuristics table
    Heuristics,
    /// Embeddings table
    Embeddings,
    /// Tags table
    Tags,
    /// Custom table name
    Custom(String),
}

impl TableDependency {
    /// Get the table name as a string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Episodes => "episodes",
            Self::Steps => "steps",
            Self::Patterns => "patterns",
            Self::Heuristics => "heuristics",
            Self::Embeddings => "embeddings",
            Self::Tags => "tags",
            Self::Custom(name) => name.as_str(),
        }
    }

    /// Parse a table name from a SQL query
    pub fn from_query(sql: &str) -> Vec<Self> {
        let sql_lower = sql.to_lowercase();
        let mut tables = Vec::new();

        // Simple table detection from common query patterns
        if sql_lower.contains("from episodes") || sql_lower.contains("join episodes") {
            tables.push(Self::Episodes);
        }
        if sql_lower.contains("from steps") || sql_lower.contains("join steps") {
            tables.push(Self::Steps);
        }
        if sql_lower.contains("from patterns") || sql_lower.contains("join patterns") {
            tables.push(Self::Patterns);
        }
        if sql_lower.contains("from heuristics") || sql_lower.contains("join heuristics") {
            tables.push(Self::Heuristics);
        }
        if sql_lower.contains("from embeddings") || sql_lower.contains("join embeddings") {
            tables.push(Self::Embeddings);
        }
        if sql_lower.contains("from tags") || sql_lower.contains("join tags") {
            tables.push(Self::Tags);
        }

        tables
    }
}

/// Query key for cache lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryKey {
    /// Normalized SQL query hash
    pub sql_hash: u64,
    /// Parameter hashes (for parameterized queries)
    pub param_hashes: Vec<u64>,
    /// Query type for TTL configuration
    pub query_type: QueryType,
}

impl QueryKey {
    /// Create a query key from SQL and parameters
    pub fn new(sql: &str, params: &[&dyn ToString]) -> Self {
        let normalized = Self::normalize_sql(sql);
        let sql_hash = Self::hash_string(&normalized);

        let param_hashes: Vec<u64> = params
            .iter()
            .map(|p| Self::hash_string(&p.to_string()))
            .collect();

        let query_type = QueryType::from_sql(&normalized);

        Self {
            sql_hash,
            param_hashes,
            query_type,
        }
    }

    /// Create a query key from SQL only (no parameters)
    pub fn from_sql(sql: &str) -> Self {
        Self::new(sql, &[])
    }

    /// Normalize SQL for consistent hashing
    /// - Remove extra whitespace
    /// - Convert to lowercase
    /// - Remove comments
    fn normalize_sql(sql: &str) -> String {
        let mut result = String::with_capacity(sql.len());
        let mut in_comment = false;
        let mut prev_char = ' ';

        for ch in sql.chars() {
            // Handle SQL comments
            if ch == '-' && prev_char == '-' {
                in_comment = true;
            }
            if ch == '\n' {
                in_comment = false;
            }

            if !in_comment {
                result.push(ch.to_ascii_lowercase());
            }
            prev_char = ch;
        }

        // Normalize whitespace
        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Hash a string using DefaultHasher
    fn hash_string(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

/// Query type for TTL configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryType {
    /// Episode queries
    Episode,
    /// Pattern queries
    Pattern,
    /// Heuristic queries
    Heuristic,
    /// Embedding queries
    Embedding,
    /// Statistics queries
    Statistics,
    /// Search queries
    Search,
    /// Generic queries
    Generic,
}

impl QueryType {
    /// Determine query type from SQL
    pub fn from_sql(sql: &str) -> Self {
        let sql_lower = sql.to_lowercase();

        if sql_lower.contains("episode") {
            Self::Episode
        } else if sql_lower.contains("pattern") {
            Self::Pattern
        } else if sql_lower.contains("heuristic") {
            Self::Heuristic
        } else if sql_lower.contains("embedding") {
            Self::Embedding
        } else if sql_lower.contains("count") || sql_lower.contains("stats") {
            Self::Statistics
        } else if sql_lower.contains("search") || sql_lower.contains("similar") {
            Self::Search
        } else {
            Self::Generic
        }
    }

    /// Get default TTL for this query type
    pub fn default_ttl(&self) -> Duration {
        match self {
            Self::Episode => Duration::from_secs(300),    // 5 minutes
            Self::Pattern => Duration::from_secs(600),    // 10 minutes
            Self::Heuristic => Duration::from_secs(600),  // 10 minutes
            Self::Embedding => Duration::from_secs(1800), // 30 minutes
            Self::Statistics => Duration::from_secs(60),  // 1 minute
            Self::Search => Duration::from_secs(120),     // 2 minutes
            Self::Generic => Duration::from_secs(300),    // 5 minutes
        }
    }
}

/// A cached query result with metadata
pub struct CachedResult {
    /// Serialized result data
    pub data: Vec<u8>,
    /// When the result was cached
    pub created_at: Instant,
    /// TTL for this result
    pub ttl: Duration,
    /// Table dependencies for invalidation
    pub dependencies: Vec<TableDependency>,
    /// Number of times accessed
    pub access_count: AtomicU64,
    /// Last access time
    pub last_accessed: RwLock<Instant>,
    /// Query type
    pub query_type: QueryType,
}

impl std::fmt::Debug for CachedResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedResult")
            .field("data_len", &self.data.len())
            .field("created_at", &self.created_at)
            .field("ttl", &self.ttl)
            .field("dependencies", &self.dependencies)
            .field(
                "access_count",
                &self.access_count.load(std::sync::atomic::Ordering::Relaxed),
            )
            .field("query_type", &self.query_type)
            .finish()
    }
}

impl Clone for CachedResult {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            created_at: self.created_at,
            ttl: self.ttl,
            dependencies: self.dependencies.clone(),
            access_count: AtomicU64::new(
                self.access_count.load(std::sync::atomic::Ordering::Relaxed),
            ),
            last_accessed: RwLock::new(*self.last_accessed.read()),
            query_type: self.query_type,
        }
    }
}

impl CachedResult {
    /// Create a new cached result
    pub fn new(
        data: Vec<u8>,
        ttl: Duration,
        dependencies: Vec<TableDependency>,
        query_type: QueryType,
    ) -> Self {
        let now = Instant::now();
        Self {
            data,
            created_at: now,
            ttl,
            dependencies,
            access_count: AtomicU64::new(0),
            last_accessed: RwLock::new(now),
            query_type,
        }
    }

    /// Check if the result has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Check if this result should be refreshed (hot query nearing expiry)
    pub fn should_refresh(&self, hot_threshold: u64, refresh_interval: Duration) -> bool {
        let access_count = self.access_count.load(Ordering::Relaxed);
        let time_until_expiry = self.ttl.saturating_sub(self.created_at.elapsed());

        access_count >= hot_threshold && time_until_expiry < refresh_interval
    }

    /// Record an access
    pub fn record_access(&self) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        *self.last_accessed.write() = Instant::now();
    }

    /// Get access count
    pub fn access_count(&self) -> u64 {
        self.access_count.load(Ordering::Relaxed)
    }

    /// Check if this result depends on a specific table
    pub fn depends_on(&self, table: &TableDependency) -> bool {
        self.dependencies.contains(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_key_creation() {
        let sql = "SELECT * FROM episodes WHERE domain = ?";
        let key = QueryKey::new(sql, &[&"test_domain"]);

        assert_eq!(key.query_type, QueryType::Episode);
        assert!(!key.param_hashes.is_empty());
    }

    #[test]
    fn test_query_key_normalization() {
        let sql1 = "SELECT * FROM episodes WHERE id = 1";
        let sql2 = "select * from episodes where id = 1";
        let sql3 = "SELECT   *   FROM   episodes   WHERE   id   =   1";

        let key1 = QueryKey::from_sql(sql1);
        let key2 = QueryKey::from_sql(sql2);
        let key3 = QueryKey::from_sql(sql3);

        assert_eq!(key1.sql_hash, key2.sql_hash);
        assert_eq!(key2.sql_hash, key3.sql_hash);
    }

    #[test]
    fn test_table_dependency_detection() {
        let sql = "SELECT e.*, s.* FROM episodes e JOIN steps s ON e.episode_id = s.episode_id";
        let deps = TableDependency::from_query(sql);

        assert!(deps.contains(&TableDependency::Episodes));
        assert!(deps.contains(&TableDependency::Steps));
    }

    #[test]
    fn test_query_type_ttl() {
        assert_eq!(QueryType::Statistics.default_ttl(), Duration::from_secs(60));
        assert_eq!(QueryType::Episode.default_ttl(), Duration::from_secs(300));
        assert_eq!(
            QueryType::Embedding.default_ttl(),
            Duration::from_secs(1800)
        );
    }
}
