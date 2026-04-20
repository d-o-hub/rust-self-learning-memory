//! Sliding window accumulator for bounded context assembly.
//!
//! The `BundleAccumulator` manages a bounded set of context items (episodes and patterns)
//! using a sliding window with recency-weighted and salience-based priority scoring.
//!
//! When capacity is exceeded, the lowest-priority item is evicted, ensuring the
//! bundle always contains the most relevant context for downstream prompts.

use crate::context::scoring::{calculate_priority_score, compare_by_priority};
use crate::context::types::{AddResult, BundleConfig, BundleStats, ContextItem};
use chrono::Utc;
use tracing::{debug, trace};
use uuid::Uuid;

/// Sliding window accumulator for bounded context assembly.
///
/// Maintains a bounded set of context items prioritized by recency and salience.
/// When capacity is exceeded, evicts the lowest-priority item.
///
/// # Examples
///
/// ```
/// use do_memory_core::context::{BundleAccumulator, BundleConfig, ContextItem};
/// use do_memory_core::episode::Episode;
/// use do_memory_core::TaskContext;
/// use do_memory_core::types::TaskType;
/// use std::sync::Arc;
///
/// // Create accumulator with default config
/// let mut accumulator = BundleAccumulator::new(BundleConfig::default());
///
/// // Add items (typically from retrieval results)
/// let episode = Episode::new(
///     "Fix bug in auth module".to_string(),
///     TaskContext::default(),
///     TaskType::Debugging,
/// );
/// let item = ContextItem::from_episode(Arc::new(episode), 0.85);
///
/// let result = accumulator.add(item);
/// assert!(result.accepted);
///
/// // Get final bundle for prompt
/// let bundle = accumulator.to_bundle();
/// println!("Bundle contains {} items", bundle.len());
/// ```
#[derive(Debug)]
pub struct BundleAccumulator {
    /// Configuration for the accumulator
    config: BundleConfig,
    /// Items currently in the bundle (sorted by priority on finalization)
    items: Vec<ContextItem>,
    /// Statistics tracking
    stats: BundleStats,
    /// Reference time for scoring (defaults to now)
    reference_time: chrono::DateTime<Utc>,
}

impl BundleAccumulator {
    /// Create a new accumulator with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for bounded accumulation
    ///
    /// # Returns
    ///
    /// A new `BundleAccumulator` instance
    #[must_use]
    pub fn new(config: BundleConfig) -> Self {
        Self {
            config,
            items: Vec::new(),
            stats: BundleStats::default(),
            reference_time: Utc::now(),
        }
    }

    /// Create an accumulator with default configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(BundleConfig::default())
    }

    /// Create an accumulator optimized for token efficiency.
    #[must_use]
    pub fn token_efficient() -> Self {
        Self::new(BundleConfig::token_efficient())
    }

    /// Create an accumulator optimized for comprehensive context.
    #[must_use]
    pub fn comprehensive() -> Self {
        Self::new(BundleConfig::comprehensive())
    }

    /// Set a custom reference time for scoring.
    ///
    /// Useful for deterministic scoring in tests or when processing
    /// historical data.
    pub fn set_reference_time(&mut self, time: chrono::DateTime<Utc>) {
        self.reference_time = time;
    }

    /// Get the current configuration.
    #[must_use]
    pub fn config(&self) -> &BundleConfig {
        &self.config
    }

    /// Get current statistics.
    #[must_use]
    pub fn stats(&self) -> &BundleStats {
        &self.stats
    }

    /// Get the number of items currently in the bundle.
    #[must_use]
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// Check if the bundle is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Check if the bundle is at capacity.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.config.max_items
    }

    /// Add an item to the bundle.
    ///
    /// Performs the following checks:
    /// 1. Salience threshold check (reject if below minimum)
    /// 2. Priority score computation
    /// 3. Capacity check (evict lowest-priority if full)
    ///
    /// # Arguments
    ///
    /// * `item` - The context item to add
    ///
    /// # Returns
    ///
    /// An `AddResult` indicating whether the item was accepted
    pub fn add(&mut self, mut item: ContextItem) -> AddResult {
        // 1. Salience threshold check
        if item.salience() < self.config.min_salience_threshold {
            self.stats.total_rejected += 1;
            debug!(
                item_id = %item.id(),
                salience = item.salience(),
                threshold = self.config.min_salience_threshold,
                "Rejected item below salience threshold"
            );
            return AddResult::rejected(
                self.items.len(),
                format!(
                    "Salience {} below threshold {}",
                    item.salience(),
                    self.config.min_salience_threshold
                ),
            );
        }

        // 2. Compute priority score
        let priority = calculate_priority_score(&item, &self.config, self.reference_time);
        item.set_priority(priority);

        // 3. Capacity management
        let evicted_id = if self.items.len() >= self.config.max_items {
            // Find and evict lowest-priority item
            self.evict_lowest_priority()
        } else {
            None
        };

        // 4. Add item (capture id before move)
        let added_id = item.id();
        self.items.push(item);
        self.stats.total_added += 1;

        if let Some(id) = evicted_id {
            self.stats.total_evicted += 1;
            debug!(
                added_id = %added_id,
                evicted_id = %id,
                bundle_size = self.items.len(),
                "Added item, evicted lowest priority"
            );
            AddResult::accepted_with_eviction(self.items.len(), id)
        } else {
            trace!(
                added_id = %added_id,
                bundle_size = self.items.len(),
                "Added item to bundle"
            );
            AddResult::accepted(self.items.len())
        }
    }

    /// Add multiple items to the bundle.
    ///
    /// More efficient than adding one-by-one because it computes priorities
    /// in bulk and uses O(n) top-k selection for finalization.
    ///
    /// # Arguments
    ///
    /// * `items` - Iterator of context items to add
    ///
    /// # Returns
    ///
    /// Vector of `AddResult` for each item
    pub fn add_batch(&mut self, items: impl IntoIterator<Item = ContextItem>) -> Vec<AddResult> {
        items.into_iter().map(|item| self.add(item)).collect()
    }

    /// Evict the lowest-priority item from the bundle.
    ///
    /// # Returns
    ///
    /// The ID of the evicted item, or `None` if bundle is empty
    fn evict_lowest_priority(&mut self) -> Option<Uuid> {
        if self.items.is_empty() {
            return None;
        }

        // Find item with lowest priority
        let min_idx = self
            .items
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.priority()
                    .partial_cmp(&b.priority())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx);

        if let Some(idx) = min_idx {
            let evicted = self.items.remove(idx);
            Some(evicted.id())
        } else {
            None
        }
    }

    /// Remove an item by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - UUID of the item to remove
    ///
    /// # Returns
    ///
    /// `true` if item was found and removed, `false` otherwise
    pub fn remove(&mut self, id: Uuid) -> bool {
        let idx = self.items.iter().position(|item| item.id() == id);
        if let Some(idx) = idx {
            self.items.remove(idx);
            true
        } else {
            false
        }
    }

    /// Check if an item with the given ID exists in the bundle.
    #[must_use]
    pub fn contains(&self, id: Uuid) -> bool {
        self.items.iter().any(|item| item.id() == id)
    }

    /// Get an item by ID.
    #[must_use]
    pub fn get(&self, id: Uuid) -> Option<&ContextItem> {
        self.items.iter().find(|item| item.id() == id)
    }

    /// Finalize the bundle, returning items sorted by priority.
    ///
    /// This method:
    /// 1. Recomputes priority scores (time may have passed)
    /// 2. Sorts by priority (highest first)
    /// 3. Truncates to max_items if somehow over capacity
    /// 4. Updates statistics
    ///
    /// # Returns
    ///
    /// Vector of context items sorted by priority (highest first)
    #[must_use]
    pub fn to_bundle(&mut self) -> Vec<ContextItem> {
        // Recompute priorities with current time
        self.recompute_priorities();

        // Sort by priority (descending)
        self.items.sort_by(compare_by_priority);

        // Ensure we don't exceed max_items
        self.items.truncate(self.config.max_items);

        // Update stats
        self.update_stats();

        self.items.clone()
    }

    /// Get items without finalization (for inspection).
    #[must_use]
    pub fn peek_items(&self) -> &[ContextItem] {
        &self.items
    }

    /// Get only episodes from the bundle.
    #[must_use]
    pub fn episodes_only(&self) -> Vec<&ContextItem> {
        self.items
            .iter()
            .filter(|item| item.item_type() == crate::context::types::ContextItemType::Episode)
            .collect()
    }

    /// Get only patterns from the bundle.
    #[must_use]
    pub fn patterns_only(&self) -> Vec<&ContextItem> {
        self.items
            .iter()
            .filter(|item| item.item_type() == crate::context::types::ContextItemType::Pattern)
            .collect()
    }

    /// Clear all items from the bundle.
    pub fn clear(&mut self) {
        self.items.clear();
        self.stats = BundleStats::default();
    }

    /// Recompute priority scores for all items.
    fn recompute_priorities(&mut self) {
        for item in &mut self.items {
            let priority = calculate_priority_score(item, &self.config, self.reference_time);
            item.set_priority(priority);
        }
    }

    /// Update statistics based on current bundle state.
    fn update_stats(&mut self) {
        if self.items.is_empty() {
            self.stats.average_salience = 0.0;
            self.stats.average_priority = 0.0;
            self.stats.oldest_timestamp = None;
            self.stats.newest_timestamp = None;
            return;
        }

        self.stats.current_size = self.items.len();

        // Average salience and priority
        let total_salience: f32 = self.items.iter().map(|i| i.salience()).sum();
        let total_priority: f32 = self.items.iter().map(|i| i.priority()).sum();
        self.stats.average_salience = total_salience / self.items.len() as f32;
        self.stats.average_priority = total_priority / self.items.len() as f32;

        // Timestamps
        self.stats.oldest_timestamp = self.items.iter().map(|i| i.timestamp()).min();
        self.stats.newest_timestamp = self.items.iter().map(|i| i.timestamp()).max();
    }

    /// Create a bundle from pre-retrieved episodes.
    ///
    /// Convenience method to convert retrieved episodes directly to a bounded bundle.
    ///
    /// # Arguments
    ///
    /// * `episodes` - Episodes from retrieval (with Arc wrapper)
    /// * `salience_fn` - Function to compute salience for each episode
    ///
    /// # Returns
    ///
    /// Vector of context items sorted by priority
    pub fn from_episodes(
        episodes: Vec<std::sync::Arc<crate::episode::Episode>>,
        salience_fn: impl Fn(&crate::episode::Episode) -> f32,
    ) -> Vec<ContextItem> {
        let mut accumulator = Self::default_config();

        for episode in episodes {
            let salience = salience_fn(&episode);
            let item = ContextItem::from_episode(episode, salience);
            accumulator.add(item);
        }

        accumulator.to_bundle()
    }

    /// Create a bundle with a custom configuration from episodes.
    pub fn from_episodes_with_config(
        episodes: Vec<std::sync::Arc<crate::episode::Episode>>,
        config: BundleConfig,
        salience_fn: impl Fn(&crate::episode::Episode) -> f32,
    ) -> Vec<ContextItem> {
        let mut accumulator = Self::new(config);

        for episode in episodes {
            let salience = salience_fn(&episode);
            let item = ContextItem::from_episode(episode, salience);
            accumulator.add(item);
        }

        accumulator.to_bundle()
    }
}

impl Default for BundleAccumulator {
    fn default() -> Self {
        Self::default_config()
    }
}
