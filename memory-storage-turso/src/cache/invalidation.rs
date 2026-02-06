//! Cache Invalidation Strategies
//!
//! Provides multiple invalidation strategies for the query cache:
//! - Time-based expiration (TTL)
//! - Dependency-based invalidation (table changes)
//! - Event-driven invalidation (CRUD operations)
//! - Manual invalidation
//! - Batch invalidation
//! - Pattern-based invalidation

use super::query_cache::{
    AdvancedCacheStats, AdvancedQueryCache, InvalidationMessage, QueryKey, QueryType,
    TableDependency,
};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, info, trace, warn};

/// Invalidation strategy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InvalidationStrategy {
    /// Only time-based expiration
    TimeBased,
    /// Dependency-based invalidation
    DependencyBased,
    /// Event-driven invalidation
    EventDriven,
    /// Combined strategy (all of the above)
    #[default]
    Combined,
    /// Custom strategy with specific rules
    Custom,
}

/// Invalidation rule for pattern-based invalidation
#[derive(Debug, Clone)]
pub struct InvalidationRule {
    /// Pattern to match (SQL LIKE pattern)
    pub pattern: String,
    /// Tables that trigger invalidation when changed
    pub dependencies: Vec<TableDependency>,
    /// TTL override for matching queries
    pub ttl_override: Option<Duration>,
    /// Priority (higher = checked first)
    pub priority: u32,
}

impl InvalidationRule {
    /// Create a new invalidation rule
    pub fn new(pattern: impl Into<String>, dependencies: Vec<TableDependency>) -> Self {
        Self {
            pattern: pattern.into(),
            dependencies,
            ttl_override: None,
            priority: 0,
        }
    }

    /// Set TTL override
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl_override = Some(ttl);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Check if a SQL query matches this rule
    pub fn matches(&self, sql: &str) -> bool {
        let sql_lower = sql.to_lowercase();
        let pattern_lower = self.pattern.to_lowercase();

        // Simple LIKE pattern matching
        if pattern_lower.contains('%') {
            self.like_match(&sql_lower, &pattern_lower)
        } else {
            sql_lower.contains(&pattern_lower)
        }
    }

    /// Simple LIKE pattern matching
    fn like_match(&self, text: &str, pattern: &str) -> bool {
        let parts: Vec<&str> = pattern.split('%').collect();

        if parts.is_empty() {
            return true;
        }

        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            if i == 0 && !text.starts_with(part) {
                return false;
            }

            if let Some(found) = text[pos..].find(part) {
                pos += found + part.len();
            } else {
                return false;
            }
        }

        // Check if pattern ends with %
        if !pattern.ends_with('%') && pos < text.len() {
            return text.ends_with(parts.last().unwrap());
        }

        true
    }
}

/// Configuration for invalidation manager
#[derive(Debug, Clone)]
pub struct InvalidationConfig {
    /// Invalidation strategy
    pub strategy: InvalidationStrategy,
    /// Enable background cleanup
    pub enable_background_cleanup: bool,
    /// Cleanup interval
    pub cleanup_interval: Duration,
    /// Enable event listening
    pub enable_event_listening: bool,
    /// Maximum rules for pattern matching
    pub max_rules: usize,
    /// Batch invalidation size
    pub batch_size: usize,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for InvalidationConfig {
    fn default() -> Self {
        Self {
            strategy: InvalidationStrategy::Combined,
            enable_background_cleanup: true,
            cleanup_interval: Duration::from_secs(60),
            enable_event_listening: true,
            max_rules: 100,
            batch_size: 100,
            enable_metrics: true,
        }
    }
}

/// Invalidation event types
#[derive(Debug, Clone)]
pub enum InvalidationEvent {
    /// Table was modified (INSERT, UPDATE, DELETE)
    TableModified {
        table: TableDependency,
        operation: CrudOperation,
        affected_rows: u64,
    },
    /// Batch operation completed
    BatchCompleted {
        tables: Vec<TableDependency>,
        operation_count: u64,
    },
    /// Schema changed
    SchemaChanged {
        table: TableDependency,
        change_type: SchemaChangeType,
    },
    /// Manual invalidation request
    ManualInvalidation {
        target: InvalidationTarget,
        reason: String,
    },
}

/// CRUD operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrudOperation {
    Insert,
    Update,
    Delete,
    Upsert,
}

/// Schema change types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaChangeType {
    Create,
    Alter,
    Drop,
    Index,
}

/// Invalidation target
#[derive(Debug, Clone)]
pub enum InvalidationTarget {
    All,
    Table(TableDependency),
    Query(QueryKey),
    Pattern(String),
    Type(QueryType),
}

/// Metrics for invalidation operations
#[derive(Debug, Clone, Default)]
pub struct InvalidationMetrics {
    /// Total invalidations triggered
    pub total_invalidations: u64,
    /// Invalidations by table
    pub by_table: HashMap<TableDependency, u64>,
    /// Invalidations by operation
    pub by_operation: HashMap<CrudOperation, u64>,
    /// Batch invalidations performed
    pub batch_count: u64,
    /// Average invalidation time (microseconds)
    pub avg_invalidation_time_us: u64,
    /// Total entries invalidated
    pub entries_invalidated: u64,
}

impl InvalidationMetrics {
    /// Record an invalidation
    pub fn record(&mut self, table: &TableDependency, operation: CrudOperation, entries: u64) {
        self.total_invalidations += 1;
        *self.by_table.entry(table.clone()).or_default() += 1;
        *self.by_operation.entry(operation).or_default() += 1;
        self.entries_invalidated += entries;
    }

    /// Record batch invalidation
    pub fn record_batch(&mut self, count: u64) {
        self.batch_count += 1;
        self.entries_invalidated += count;
    }
}

/// Invalidation manager for query cache
pub struct InvalidationManager {
    /// Configuration
    config: InvalidationConfig,
    /// Reference to query cache
    cache: AdvancedQueryCache,
    /// Invalidation rules
    rules: Arc<RwLock<Vec<InvalidationRule>>>,
    /// Event receiver
    event_rx: mpsc::UnboundedReceiver<InvalidationEvent>,
    /// Event sender (for external use)
    event_tx: mpsc::UnboundedSender<InvalidationEvent>,
    /// Invalidation receiver from cache
    invalidation_rx: mpsc::UnboundedReceiver<InvalidationMessage>,
    /// Metrics
    metrics: Arc<RwLock<InvalidationMetrics>>,
    /// Pending invalidations (for batching)
    pending: Arc<RwLock<VecDeque<InvalidationEvent>>>,
    /// Last batch time
    last_batch_time: Arc<RwLock<Instant>>,
}

impl InvalidationManager {
    /// Create a new invalidation manager
    pub fn new(
        config: InvalidationConfig,
        cache: AdvancedQueryCache,
    ) -> (Self, mpsc::UnboundedSender<InvalidationEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let invalidation_rx = {
            // We need to create a new channel since we can't extract from cache
            let (_tx, rx) = mpsc::unbounded_channel();
            rx
        };

        let manager = Self {
            config,
            cache,
            rules: Arc::new(RwLock::new(Vec::new())),
            event_rx,
            event_tx: event_tx.clone(),
            invalidation_rx,
            metrics: Arc::new(RwLock::new(InvalidationMetrics::default())),
            pending: Arc::new(RwLock::new(VecDeque::new())),
            last_batch_time: Arc::new(RwLock::new(Instant::now())),
        };

        (manager, event_tx)
    }

    /// Create with default configuration
    pub fn default(cache: AdvancedQueryCache) -> (Self, mpsc::UnboundedSender<InvalidationEvent>) {
        Self::new(InvalidationConfig::default(), cache)
    }

    /// Add an invalidation rule
    pub fn add_rule(&self, rule: InvalidationRule) {
        let mut rules = self.rules.write();

        if rules.len() >= self.config.max_rules {
            warn!("Max rules reached, removing lowest priority rule");
            rules.sort_by(|a, b| a.priority.cmp(&b.priority));
            rules.remove(0);
        }

        rules.push(rule);
        rules.sort_by(|a, b| b.priority.cmp(&a.priority)); // Higher priority first

        debug!("Added invalidation rule, total rules: {}", rules.len());
    }

    /// Remove all rules
    pub fn clear_rules(&self) {
        self.rules.write().clear();
        debug!("Cleared all invalidation rules");
    }

    /// Get rules matching a SQL query
    pub fn get_matching_rules(&self, sql: &str) -> Vec<InvalidationRule> {
        self.rules
            .read()
            .iter()
            .filter(|rule| rule.matches(sql))
            .cloned()
            .collect()
    }

    /// Handle an invalidation event
    pub fn handle_event(&self, event: InvalidationEvent) {
        let start = Instant::now();

        match &event {
            InvalidationEvent::TableModified {
                table,
                operation,
                affected_rows,
            } => {
                debug!(
                    "Handling table modification: {:?} {:?}, {} rows affected",
                    table, operation, affected_rows
                );

                // Invalidate by table dependency
                self.cache.invalidate_by_table(table);

                // Update metrics
                let mut metrics = self.metrics.write();
                metrics.total_invalidations += 1;
                *metrics.by_table.entry(table.clone()).or_default() += 1;
                *metrics.by_operation.entry(*operation).or_default() += 1;
                metrics.entries_invalidated += *affected_rows;
            }

            InvalidationEvent::BatchCompleted {
                tables,
                operation_count,
            } => {
                debug!(
                    "Handling batch completion: {} tables, {} operations",
                    tables.len(),
                    operation_count
                );

                for table in tables {
                    self.cache.invalidate_by_table(table);
                }

                self.metrics.write().batch_count += 1;
                self.metrics.write().entries_invalidated += *operation_count;
            }

            InvalidationEvent::SchemaChanged { table, change_type } => {
                info!("Schema changed on {:?}: {:?}", table, change_type);
                // Invalidate all queries for this table on schema changes
                self.cache.invalidate_by_table(table);
            }

            InvalidationEvent::ManualInvalidation { target, reason } => {
                info!("Manual invalidation: {} - {:?}", reason, target);
                self.handle_manual_invalidation(target);
            }
        }

        // Record timing
        let elapsed = start.elapsed().as_micros() as u64;
        let mut metrics = self.metrics.write();
        let total = metrics.total_invalidations;
        metrics.avg_invalidation_time_us =
            (metrics.avg_invalidation_time_us * (total - 1) + elapsed) / total.max(1);
    }

    /// Handle manual invalidation
    fn handle_manual_invalidation(&self, target: &InvalidationTarget) {
        match target {
            InvalidationTarget::All => {
                self.cache.clear();
            }
            InvalidationTarget::Table(table) => {
                self.cache.invalidate_by_table(table);
            }
            InvalidationTarget::Query(key) => {
                self.cache.invalidate_key(key);
            }
            InvalidationTarget::Pattern(pattern) => {
                // Find and invalidate queries matching pattern
                self.invalidate_by_pattern(pattern);
            }
            InvalidationTarget::Type(query_type) => {
                self.invalidate_by_type(*query_type);
            }
        }
    }

    /// Invalidate queries matching a pattern
    fn invalidate_by_pattern(&self, pattern: &str) {
        // This would require access to the query keys in the cache
        // For now, we clear all as a safe fallback
        warn!(
            "Pattern-based invalidation not fully implemented, clearing all: {}",
            pattern
        );
        self.cache.clear();
    }

    /// Invalidate queries by type
    fn invalidate_by_type(&self, query_type: QueryType) {
        // This would require type-based indexing in the cache
        // For now, we rely on table-based invalidation
        debug!("Type-based invalidation requested for: {:?}", query_type);
    }

    /// Queue an event for batch processing
    pub fn queue_event(&self, event: InvalidationEvent) {
        self.pending.write().push_back(event);

        // Check if we should process the batch
        let should_process = {
            let pending = self.pending.read();
            pending.len() >= self.config.batch_size
                || self.last_batch_time.read().elapsed() > Duration::from_secs(5)
        };

        if should_process {
            self.process_batch();
        }
    }

    /// Process pending invalidation events in batch
    pub fn process_batch(&self) {
        let mut pending = self.pending.write();

        if pending.is_empty() {
            return;
        }

        debug!("Processing {} pending invalidation events", pending.len());

        // Group events by table for efficiency
        let mut by_table: HashMap<TableDependency, Vec<InvalidationEvent>> = HashMap::new();

        for event in pending.drain(..) {
            if let Some(table) = Self::get_event_table(&event) {
                by_table.entry(table).or_default().push(event);
            }
        }

        // Process each table's events
        for (table, events) in by_table {
            let total_rows: u64 = events
                .iter()
                .filter_map(|e| match e {
                    InvalidationEvent::TableModified { affected_rows, .. } => Some(*affected_rows),
                    _ => None,
                })
                .sum();

            // Single invalidation for all events on this table
            self.cache.invalidate_by_table(&table);

            // Update metrics
            let mut metrics = self.metrics.write();
            metrics.total_invalidations += 1;
            *metrics.by_table.entry(table.clone()).or_default() += 1;
            *metrics
                .by_operation
                .entry(CrudOperation::Update)
                .or_default() += 1;
            metrics.entries_invalidated += total_rows;
        }

        *self.last_batch_time.write() = Instant::now();
    }

    /// Get table from event
    fn get_event_table(event: &InvalidationEvent) -> Option<TableDependency> {
        match event {
            InvalidationEvent::TableModified { table, .. } => Some(table.clone()),
            InvalidationEvent::SchemaChanged { table, .. } => Some(table.clone()),
            _ => None,
        }
    }

    /// Get current metrics
    pub fn metrics(&self) -> InvalidationMetrics {
        self.metrics.read().clone()
    }

    /// Clear metrics
    pub fn clear_metrics(&self) {
        *self.metrics.write() = InvalidationMetrics::default();
    }

    /// Get cache stats
    pub fn cache_stats(&self) -> AdvancedCacheStats {
        self.cache.stats()
    }

    /// Clear expired cache entries
    pub fn clear_expired(&self) -> usize {
        self.cache.clear_expired()
    }

    /// Start the invalidation manager
    pub async fn run(mut self) {
        info!(
            "Starting invalidation manager with {:?} strategy",
            self.config.strategy
        );

        let mut cleanup_interval = interval(self.config.cleanup_interval);

        loop {
            tokio::select! {
                // Handle invalidation events
                Some(event) = self.event_rx.recv() => {
                    if self.config.enable_event_listening {
                        self.handle_event(event);
                    }
                }

                // Handle messages from cache
                Some(message) = self.invalidation_rx.recv() => {
                    self.handle_invalidation_message(message);
                }

                // Periodic cleanup
                _ = cleanup_interval.tick() => {
                    if self.config.enable_background_cleanup {
                        self.perform_cleanup();
                    }
                }

                // Shutdown signal
                else => {
                    info!("Invalidation manager shutting down");
                    break;
                }
            }
        }
    }

    /// Handle invalidation message from cache
    fn handle_invalidation_message(&self, message: InvalidationMessage) {
        match message {
            InvalidationMessage::TableChanged(table) => {
                self.cache.invalidate_by_table(&table);
            }
            InvalidationMessage::InvalidateKey(key) => {
                self.cache.invalidate_key(&key);
            }
            InvalidationMessage::InvalidateAll => {
                self.cache.clear();
            }
            InvalidationMessage::Shutdown => {
                info!("Received shutdown signal");
            }
        }
    }

    /// Perform periodic cleanup
    fn perform_cleanup(&self) {
        // Process any pending batch events
        self.process_batch();

        // Clear expired cache entries
        let cleared = self.cache.clear_expired();
        if cleared > 0 {
            debug!("Cleared {} expired cache entries during cleanup", cleared);
        }

        // Log metrics
        let metrics = self.metrics();
        if metrics.total_invalidations > 0 {
            trace!(
                "Invalidation metrics: {} total, {} entries invalidated",
                metrics.total_invalidations,
                metrics.entries_invalidated
            );
        }
    }
}

impl Clone for InvalidationManager {
    fn clone(&self) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let invalidation_rx = {
            let (_tx, rx) = mpsc::unbounded_channel();
            rx
        };

        Self {
            config: self.config.clone(),
            cache: self.cache.clone(),
            rules: Arc::clone(&self.rules),
            event_rx,
            event_tx,
            invalidation_rx,
            metrics: Arc::clone(&self.metrics),
            pending: Arc::clone(&self.pending),
            last_batch_time: Arc::clone(&self.last_batch_time),
        }
    }
}

/// Builder for invalidation rules
pub struct InvalidationRuleBuilder {
    pattern: String,
    dependencies: Vec<TableDependency>,
    ttl_override: Option<Duration>,
    priority: u32,
}

impl InvalidationRuleBuilder {
    /// Create a new rule builder
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            dependencies: Vec::new(),
            ttl_override: None,
            priority: 0,
        }
    }

    /// Add a table dependency
    pub fn depends_on(mut self, table: TableDependency) -> Self {
        self.dependencies.push(table);
        self
    }

    /// Add multiple dependencies
    pub fn depends_on_many(mut self, tables: Vec<TableDependency>) -> Self {
        self.dependencies.extend(tables);
        self
    }

    /// Set TTL override
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl_override = Some(ttl);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Build the rule
    pub fn build(self) -> InvalidationRule {
        InvalidationRule {
            pattern: self.pattern,
            dependencies: self.dependencies,
            ttl_override: self.ttl_override,
            priority: self.priority,
        }
    }
}

/// Utility functions for invalidation
pub mod utils {
    use super::*;

    /// Create a rule for episode queries
    pub fn episode_queries_rule() -> InvalidationRule {
        InvalidationRuleBuilder::new("%episodes%")
            .depends_on(TableDependency::Episodes)
            .depends_on(TableDependency::Steps)
            .with_priority(10)
            .build()
    }

    /// Create a rule for pattern queries
    pub fn pattern_queries_rule() -> InvalidationRule {
        InvalidationRuleBuilder::new("%patterns%")
            .depends_on(TableDependency::Patterns)
            .with_priority(10)
            .build()
    }

    /// Create a rule for statistics queries
    pub fn statistics_queries_rule() -> InvalidationRule {
        InvalidationRuleBuilder::new("%count%")
            .depends_on(TableDependency::Episodes)
            .depends_on(TableDependency::Patterns)
            .depends_on(TableDependency::Steps)
            .with_ttl(Duration::from_secs(30))
            .with_priority(5)
            .build()
    }

    /// Create a rule for search queries
    pub fn search_queries_rule() -> InvalidationRule {
        InvalidationRuleBuilder::new("%search%")
            .depends_on(TableDependency::Episodes)
            .depends_on(TableDependency::Patterns)
            .depends_on(TableDependency::Embeddings)
            .with_ttl(Duration::from_secs(120))
            .with_priority(8)
            .build()
    }

    /// Create default rules for common query patterns
    pub fn default_rules() -> Vec<InvalidationRule> {
        vec![
            episode_queries_rule(),
            pattern_queries_rule(),
            statistics_queries_rule(),
            search_queries_rule(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalidation_rule_matching() {
        let rule = InvalidationRule::new("%episodes%", vec![TableDependency::Episodes]);

        assert!(rule.matches("SELECT * FROM episodes"));
        assert!(rule.matches("SELECT * FROM episodes WHERE id = 1"));
        assert!(!rule.matches("SELECT * FROM patterns"));
    }

    #[test]
    fn test_invalidation_rule_priority() {
        let rule = InvalidationRule::new("test", vec![]).with_priority(5);
        assert_eq!(rule.priority, 5);
    }

    #[test]
    fn test_invalidation_rule_ttl() {
        let ttl = Duration::from_secs(60);
        let rule = InvalidationRule::new("test", vec![]).with_ttl(ttl);
        assert_eq!(rule.ttl_override, Some(ttl));
    }

    #[test]
    fn test_rule_builder() {
        let rule = InvalidationRuleBuilder::new("%episodes%")
            .depends_on(TableDependency::Episodes)
            .depends_on(TableDependency::Steps)
            .with_ttl(Duration::from_secs(300))
            .with_priority(10)
            .build();

        assert_eq!(rule.pattern, "%episodes%");
        assert_eq!(rule.dependencies.len(), 2);
        assert_eq!(rule.ttl_override, Some(Duration::from_secs(300)));
        assert_eq!(rule.priority, 10);
    }

    #[test]
    fn test_default_rules() {
        let rules = utils::default_rules();
        assert!(!rules.is_empty());

        let episode_rule = rules.iter().find(|r| r.pattern == "%episodes%");
        assert!(episode_rule.is_some());
    }

    #[test]
    fn test_invalidation_metrics() {
        let mut metrics = InvalidationMetrics::default();

        metrics.record(&TableDependency::Episodes, CrudOperation::Insert, 5);
        metrics.record(&TableDependency::Patterns, CrudOperation::Update, 3);
        metrics.record_batch(10);

        assert_eq!(metrics.total_invalidations, 2);
        assert_eq!(metrics.entries_invalidated, 18);
        assert_eq!(metrics.batch_count, 1);
    }

    #[test]
    fn test_invalidation_target() {
        let target_all = InvalidationTarget::All;
        let target_table = InvalidationTarget::Table(TableDependency::Episodes);
        let target_type = InvalidationTarget::Type(QueryType::Episode);

        // Just verify they can be created
        assert!(matches!(target_all, InvalidationTarget::All));
        assert!(matches!(target_table, InvalidationTarget::Table(_)));
        assert!(matches!(target_type, InvalidationTarget::Type(_)));
    }

    #[test]
    fn test_crud_operations() {
        assert_eq!(CrudOperation::Insert, CrudOperation::Insert);
        assert_ne!(CrudOperation::Insert, CrudOperation::Update);
    }

    #[test]
    fn test_schema_change_types() {
        assert_eq!(SchemaChangeType::Create, SchemaChangeType::Create);
        assert_ne!(SchemaChangeType::Create, SchemaChangeType::Drop);
    }

    #[test]
    fn test_invalidation_strategy_default() {
        let strategy: InvalidationStrategy = Default::default();
        assert_eq!(strategy, InvalidationStrategy::Combined);
    }

    #[test]
    fn test_invalidation_config_default() {
        let config = InvalidationConfig::default();
        assert_eq!(config.strategy, InvalidationStrategy::Combined);
        assert!(config.enable_background_cleanup);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_like_pattern_matching() {
        let rule = InvalidationRule::new("%episodes%", vec![]);

        // Test various patterns
        assert!(rule.matches("SELECT * FROM episodes"));
        assert!(rule.matches("UPDATE episodes SET x = 1"));
        assert!(rule.matches("DELETE FROM episodes WHERE id = 1"));
        assert!(!rule.matches("SELECT * FROM patterns"));

        // Test prefix pattern
        let prefix_rule = InvalidationRule::new("SELECT%", vec![]);
        assert!(prefix_rule.matches("SELECT * FROM episodes"));
        assert!(!prefix_rule.matches("INSERT INTO episodes"));

        // Test suffix pattern
        let suffix_rule = InvalidationRule::new("%episodes", vec![]);
        assert!(suffix_rule.matches("FROM episodes"));
        assert!(!suffix_rule.matches("FROM episodes WHERE"));
    }
}
