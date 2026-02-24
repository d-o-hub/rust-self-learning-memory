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
    AdvancedCacheStats, AdvancedQueryCache, InvalidationMessage, QueryType, TableDependency,
};
#[path = "invalidation_types.rs"]
mod types;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, info, trace, warn};
pub use types::{
    CrudOperation, InvalidationConfig, InvalidationEvent, InvalidationMetrics, InvalidationRule,
    InvalidationRuleBuilder, InvalidationStrategy, InvalidationTarget, SchemaChangeType, utils,
};

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
                metrics.total_invalidations, metrics.entries_invalidated
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

#[cfg(test)]
#[path = "invalidation_tests.rs"]
mod tests;
