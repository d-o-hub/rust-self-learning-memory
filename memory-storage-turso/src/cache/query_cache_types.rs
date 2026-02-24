use parking_lot::RwLock;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const DEFAULT_MAX_QUERIES: usize = 1000;
const DEFAULT_QUERY_TTL: Duration = Duration::from_secs(300);
const DEFAULT_HOT_THRESHOLD: u64 = 5;
const DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TableDependency {
    Episodes,
    Steps,
    Patterns,
    Heuristics,
    Embeddings,
    Tags,
    Custom(String),
}

impl TableDependency {
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

    pub fn from_query(sql: &str) -> Vec<Self> {
        let sql_lower = sql.to_lowercase();
        let mut tables = Vec::new();

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryKey {
    pub sql_hash: u64,
    pub param_hashes: Vec<u64>,
    pub query_type: QueryType,
}

impl QueryKey {
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

    pub fn from_sql(sql: &str) -> Self {
        Self::new(sql, &[])
    }

    fn normalize_sql(sql: &str) -> String {
        let mut result = String::with_capacity(sql.len());
        let mut in_comment = false;
        let mut prev_char = ' ';

        for ch in sql.chars() {
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

        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn hash_string(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryType {
    Episode,
    Pattern,
    Heuristic,
    Embedding,
    Statistics,
    Search,
    Generic,
}

impl QueryType {
    fn from_sql(sql: &str) -> Self {
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

    pub fn default_ttl(&self) -> Duration {
        match self {
            Self::Episode => Duration::from_secs(300),
            Self::Pattern => Duration::from_secs(600),
            Self::Heuristic => Duration::from_secs(600),
            Self::Embedding => Duration::from_secs(1800),
            Self::Statistics => Duration::from_secs(60),
            Self::Search => Duration::from_secs(120),
            Self::Generic => Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdvancedQueryCacheConfig {
    pub max_queries: usize,
    pub default_ttl: Duration,
    pub ttl_overrides: HashMap<QueryType, Duration>,
    pub hot_threshold: u64,
    pub refresh_interval: Duration,
    pub enable_background_refresh: bool,
    pub enable_dependency_tracking: bool,
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
    pub fn ttl_for_type(&self, query_type: QueryType) -> Duration {
        self.ttl_overrides
            .get(&query_type)
            .copied()
            .unwrap_or_else(|| query_type.default_ttl())
    }
}

pub struct CachedResult {
    pub data: Vec<u8>,
    pub created_at: Instant,
    pub ttl: Duration,
    pub dependencies: Vec<TableDependency>,
    pub access_count: AtomicU64,
    pub last_accessed: RwLock<Instant>,
    pub query_type: QueryType,
}

impl std::fmt::Debug for CachedResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedResult")
            .field("data_len", &self.data.len())
            .field("created_at", &self.created_at)
            .field("ttl", &self.ttl)
            .field("dependencies", &self.dependencies)
            .field("access_count", &self.access_count.load(Ordering::Relaxed))
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
            access_count: AtomicU64::new(self.access_count.load(Ordering::Relaxed)),
            last_accessed: RwLock::new(*self.last_accessed.read()),
            query_type: self.query_type,
        }
    }
}

impl CachedResult {
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

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    pub fn should_refresh(&self, hot_threshold: u64, refresh_interval: Duration) -> bool {
        let access_count = self.access_count.load(Ordering::Relaxed);
        let time_until_expiry = self.ttl.saturating_sub(self.created_at.elapsed());

        access_count >= hot_threshold && time_until_expiry < refresh_interval
    }

    pub fn record_access(&self) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        *self.last_accessed.write() = Instant::now();
    }

    pub fn access_count(&self) -> u64 {
        self.access_count.load(Ordering::Relaxed)
    }

    pub fn depends_on(&self, table: &TableDependency) -> bool {
        self.dependencies.contains(table)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AdvancedCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub expirations: u64,
    pub invalidations: u64,
    pub current_size: usize,
    pub hot_queries: usize,
    pub refreshes: u64,
    pub hit_rate_by_type: HashMap<QueryType, f64>,
}

impl AdvancedCacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn merge(&mut self, other: &AdvancedCacheStats) {
        self.hits += other.hits;
        self.misses += other.misses;
        self.evictions += other.evictions;
        self.expirations += other.expirations;
        self.invalidations += other.invalidations;
        self.refreshes += other.refreshes;
    }
}

#[derive(Debug, Clone)]
pub enum InvalidationMessage {
    TableChanged(TableDependency),
    InvalidateKey(QueryKey),
    InvalidateAll,
    Shutdown,
}
