use super::MAX_STATEMENT_AGE;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PreparedCacheConfig {
    pub max_size: usize,
    pub enable_refresh: bool,
    pub refresh_threshold: u64,
    pub max_connections: usize,
}

impl Default for PreparedCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 100,
            enable_refresh: true,
            refresh_threshold: 1000,
            max_connections: 100,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreparedCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub prepared: u64,
    pub evictions: u64,
    pub current_size: usize,
    pub max_size_reached: usize,
    pub refreshes: u64,
    pub preparation_time_us: u64,
    pub avg_preparation_time_us: f64,
    pub active_connections: usize,
    pub connection_evictions: u64,
}

impl PreparedCacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    pub fn record_prepared(&mut self, time_us: u64) {
        self.prepared += 1;
        self.preparation_time_us += time_us;
        self.avg_preparation_time_us = self.preparation_time_us as f64 / self.prepared as f64;
    }

    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    pub fn update_size(&mut self, size: usize) {
        self.current_size = size;
        if size > self.max_size_reached {
            self.max_size_reached = size;
        }
    }

    pub fn record_refresh(&mut self) {
        self.refreshes += 1;
    }

    pub fn update_active_connections(&mut self, count: usize) {
        self.active_connections = count;
    }

    pub fn record_connection_eviction(&mut self) {
        self.connection_evictions += 1;
    }
}

pub(super) struct CachedStatementMetadata {
    prepared_at: Instant,
    use_count: AtomicU64,
    sql: String,
}

impl Clone for CachedStatementMetadata {
    fn clone(&self) -> Self {
        Self {
            prepared_at: self.prepared_at,
            use_count: AtomicU64::new(self.use_count.load(Ordering::Relaxed)),
            sql: self.sql.clone(),
        }
    }
}

impl CachedStatementMetadata {
    pub(super) fn new(sql: String) -> Self {
        Self {
            prepared_at: Instant::now(),
            use_count: AtomicU64::new(0),
            sql,
        }
    }

    pub(super) fn increment_use(&self) {
        self.use_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn use_count(&self) -> u64 {
        self.use_count.load(Ordering::Relaxed)
    }

    pub(super) fn needs_refresh(&self, config: &PreparedCacheConfig) -> bool {
        config.enable_refresh
            && (self.use_count() > config.refresh_threshold
                || self.prepared_at.elapsed() > MAX_STATEMENT_AGE)
    }
}

#[allow(dead_code)]
pub(super) struct ConnectionCache {
    pub(super) statements: HashMap<String, CachedStatementMetadata>,
    access_order: Vec<String>,
    created_at: Instant,
    pub(super) last_accessed: Instant,
}

impl ConnectionCache {
    pub(super) fn new() -> Self {
        let now = Instant::now();
        Self {
            statements: HashMap::new(),
            access_order: Vec::new(),
            created_at: now,
            last_accessed: now,
        }
    }

    pub(super) fn get(&mut self, sql: &str) -> Option<&CachedStatementMetadata> {
        if let Some(stmt) = self.statements.get(sql) {
            if let Some(pos) = self.access_order.iter().position(|s| s == sql) {
                let key = self.access_order.remove(pos);
                self.access_order.push(key);
            }
            self.last_accessed = Instant::now();
            Some(stmt)
        } else {
            None
        }
    }

    pub(super) fn insert(&mut self, sql: String, stmt: CachedStatementMetadata) {
        self.statements.insert(sql.clone(), stmt);
        self.access_order.push(sql);
        self.last_accessed = Instant::now();
    }

    pub(super) fn remove(&mut self, sql: &str) -> bool {
        if self.statements.remove(sql).is_some() {
            if let Some(pos) = self.access_order.iter().position(|s| s == sql) {
                self.access_order.remove(pos);
            }
            true
        } else {
            false
        }
    }

    pub(super) fn evict_lru(&mut self) -> Option<String> {
        if self.access_order.is_empty() {
            return None;
        }

        let lru_key = self.access_order.remove(0);
        self.statements.remove(&lru_key);
        Some(lru_key)
    }

    pub(super) fn len(&self) -> usize {
        self.statements.len()
    }

    #[allow(dead_code)]
    pub(super) fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    #[allow(dead_code)]
    pub(super) fn clear(&mut self) {
        self.statements.clear();
        self.access_order.clear();
    }

    pub(super) fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed()
    }
}
