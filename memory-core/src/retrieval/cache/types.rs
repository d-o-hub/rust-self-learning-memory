//! # Query Cache Types
//!
//! Types for the query cache with LRU eviction and TTL.

use crate::episode::Episode;
use crate::types::{ComplexityLevel, TaskContext};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Default cache TTL (60 seconds)
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(60);

/// Default maximum cache entries (10,000 queries)
pub const DEFAULT_MAX_ENTRIES: usize = 10_000;

/// Cache key combining query parameters.
///
/// Identity includes all `TaskContext` fields that affect ranking so that
/// context-distinct requests cannot share a cache entry incorrectly
/// (ADR-074 partial / GOAP S1.2).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Query text or description
    pub query: String,
    /// Task domain filter (optional)
    pub domain: Option<Arc<str>>,
    /// Programming language filter (optional)
    pub language: Option<Arc<str>>,
    /// Framework filter (optional)
    pub framework: Option<Arc<str>>,
    /// Complexity level as a stable string (optional)
    pub complexity: Option<String>,
    /// Normalized tags (trimmed, non-empty, sorted, deduped)
    pub tags: Vec<String>,
    /// Task type filter (optional)
    pub task_type: Option<String>,
    /// Time range start (unix timestamp, optional)
    pub time_start: Option<i64>,
    /// Time range end (unix timestamp, optional)
    pub time_end: Option<i64>,
    /// Maximum results to return
    pub limit: usize,
}

impl CacheKey {
    /// Create a new cache key
    #[must_use]
    pub fn new(query: String) -> Self {
        Self {
            query,
            domain: None,
            language: None,
            framework: None,
            complexity: None,
            tags: Vec::new(),
            task_type: None,
            time_start: None,
            time_end: None,
            limit: 10,
        }
    }

    /// Set domain filter
    #[must_use]
    pub fn with_domain(mut self, domain: Option<String>) -> Self {
        self.domain = normalize_optional_arc(domain);
        self
    }

    /// Set programming language filter
    #[must_use]
    pub fn with_language(mut self, language: Option<String>) -> Self {
        self.language = normalize_optional_arc(language);
        self
    }

    /// Set framework filter
    #[must_use]
    pub fn with_framework(mut self, framework: Option<String>) -> Self {
        self.framework = normalize_optional_arc(framework);
        self
    }

    /// Set complexity filter from a stable string representation
    #[must_use]
    pub fn with_complexity(mut self, complexity: Option<String>) -> Self {
        self.complexity = complexity.and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        self
    }

    /// Set complexity from [`ComplexityLevel`]
    #[must_use]
    pub fn with_complexity_level(self, complexity: ComplexityLevel) -> Self {
        self.with_complexity(Some(complexity_level_key(complexity).to_string()))
    }

    /// Set tags after canonical normalization (trim, drop empty, sort, dedupe)
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = normalize_tags(tags);
        self
    }

    /// Include all ranking-affecting fields from a [`TaskContext`]
    #[must_use]
    pub fn with_task_context(self, context: &TaskContext) -> Self {
        self.with_domain(Some(context.domain.clone()))
            .with_language(context.language.clone())
            .with_framework(context.framework.clone())
            .with_complexity_level(context.complexity)
            .with_tags(context.tags.clone())
    }

    /// Set task type filter
    #[must_use]
    pub fn with_task_type(mut self, task_type: Option<String>) -> Self {
        self.task_type = task_type.and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        self
    }

    /// Set time range filter
    #[must_use]
    pub fn with_time_range(mut self, start: Option<i64>, end: Option<i64>) -> Self {
        self.time_start = start;
        self.time_end = end;
        self
    }

    /// Set result limit
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Compute hash for this cache key
    #[must_use]
    pub fn compute_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }
}

/// Stable string key for [`ComplexityLevel`] identity.
#[must_use]
pub(crate) fn complexity_level_key(level: ComplexityLevel) -> &'static str {
    match level {
        ComplexityLevel::Simple => "Simple",
        ComplexityLevel::Moderate => "Moderate",
        ComplexityLevel::Complex => "Complex",
    }
}

/// Normalize optional string filters: trim and treat empty as `None`.
fn normalize_optional_arc(value: Option<String>) -> Option<Arc<str>> {
    value.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(Arc::from(trimmed))
        }
    })
}

/// Canonical tag normalization for cache identity (ADR-074).
///
/// Tags are trimmed, empty entries removed, sorted, and deduplicated so that
/// order and duplicates cannot create distinct cache entries.
#[must_use]
pub fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized: Vec<String> = tags
        .into_iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    normalized.sort_unstable();
    normalized.dedup();
    normalized
}

/// Cached query result with expiration time
#[derive(Debug, Clone)]
pub struct CachedResult {
    /// Cached episodes (Arc for zero-copy retrieval)
    pub episodes: Arc<[Arc<Episode>]>,
    /// Time when this entry was cached
    pub cached_at: Instant,
    /// Time-to-live for this entry
    pub ttl: Duration,
}

impl CachedResult {
    /// Check if this cached result has expired
    pub(crate) fn is_expired(&self) -> bool {
        self.cached_at.elapsed() >= self.ttl
    }
}

/// Cache metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total cache evictions
    pub evictions: u64,
    /// Total cache invalidations
    pub invalidations: u64,
    /// Current cache size (number of entries)
    pub size: usize,
    /// Maximum cache capacity
    pub capacity: usize,
}

impl CacheMetrics {
    /// Calculate cache hit rate (0.0 to 1.0)
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Check if cache is performing well (hit rate ≥ 40%)
    #[must_use]
    pub fn is_effective(&self) -> bool {
        self.hit_rate() >= 0.4
    }
}
