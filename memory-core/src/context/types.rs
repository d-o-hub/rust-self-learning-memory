//! Types for bounded context assembly.
//!
//! Provides the data structures for accumulating context items (episodes and patterns)
//! with bounded capacity and priority scoring.

use crate::episode::Episode;
use crate::pattern::Pattern;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// An item that can be accumulated into a context bundle.
///
/// Wraps either an episode or a pattern with associated scoring metadata
/// for bounded accumulation decisions.
///
/// # Examples
///
/// ```
/// use do_memory_core::context::{ContextItem, ContextItemType};
/// use do_memory_core::episode::Episode;
/// use do_memory_core::TaskContext;
/// use do_memory_core::types::TaskType;
/// use std::sync::Arc;
///
/// // Create an episode context item
/// let episode = Episode::new(
///     "Fix authentication bug".to_string(),
///     TaskContext::default(),
///     TaskType::Debugging,
/// );
/// let item = ContextItem::from_episode(Arc::new(episode), 0.85);
///
/// assert_eq!(item.item_type(), ContextItemType::Episode);
/// assert_eq!(item.salience(), 0.85);
/// ```
#[derive(Debug, Clone)]
pub struct ContextItem {
    /// The underlying item (episode or pattern)
    inner: ContextItemInner,
    /// Timestamp when item was created/added to memory
    timestamp: DateTime<Utc>,
    /// Salience score (importance/quality) from retrieval (0.0-1.0)
    salience: f32,
    /// Combined priority score (recency + salience weighted)
    priority: f32,
}

/// Inner content of a context item.
#[derive(Debug, Clone)]
enum ContextItemInner {
    /// An episode from episodic memory
    Episode(Arc<Episode>),
    /// A pattern from pattern memory
    Pattern(Arc<Pattern>),
}

/// Type of the context item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextItemType {
    /// Episode from episodic memory
    Episode,
    /// Pattern from pattern memory
    Pattern,
}

impl ContextItem {
    /// Create a context item from an episode.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to wrap
    /// * `salience` - Salience score from retrieval (0.0-1.0)
    ///
    /// # Returns
    ///
    /// A new `ContextItem` wrapping the episode
    #[must_use]
    pub fn from_episode(episode: Arc<Episode>, salience: f32) -> Self {
        let timestamp = episode.start_time;
        Self {
            inner: ContextItemInner::Episode(episode),
            timestamp,
            salience: salience.clamp(0.0, 1.0),
            priority: 0.0, // Computed later by accumulator
        }
    }

    /// Create a context item from a pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern to wrap
    /// * `salience` - Salience score from retrieval (0.0-1.0)
    ///
    /// # Returns
    ///
    /// A new `ContextItem` wrapping the pattern
    #[must_use]
    pub fn from_pattern(pattern: Arc<Pattern>, salience: f32) -> Self {
        let timestamp = pattern.effectiveness().created_at;
        Self {
            inner: ContextItemInner::Pattern(pattern),
            timestamp,
            salience: salience.clamp(0.0, 1.0),
            priority: 0.0,
        }
    }

    /// Get the type of this context item.
    #[must_use]
    pub fn item_type(&self) -> ContextItemType {
        match &self.inner {
            ContextItemInner::Episode(_) => ContextItemType::Episode,
            ContextItemInner::Pattern(_) => ContextItemType::Pattern,
        }
    }

    /// Get the salience score.
    #[must_use]
    pub fn salience(&self) -> f32 {
        self.salience
    }

    /// Get the priority score.
    #[must_use]
    pub fn priority(&self) -> f32 {
        self.priority
    }

    /// Set the priority score (computed by accumulator).
    pub fn set_priority(&mut self, priority: f32) {
        self.priority = priority.clamp(0.0, 1.0);
    }

    /// Get the timestamp.
    #[must_use]
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get the episode if this is an episode item.
    #[must_use]
    pub fn as_episode(&self) -> Option<&Arc<Episode>> {
        match &self.inner {
            ContextItemInner::Episode(ep) => Some(ep),
            ContextItemInner::Pattern(_) => None,
        }
    }

    /// Get the pattern if this is a pattern item.
    #[must_use]
    pub fn as_pattern(&self) -> Option<&Arc<Pattern>> {
        match &self.inner {
            ContextItemInner::Episode(_) => None,
            ContextItemInner::Pattern(p) => Some(p),
        }
    }

    /// Get the unique identifier for this item.
    #[must_use]
    pub fn id(&self) -> Uuid {
        match &self.inner {
            ContextItemInner::Episode(ep) => ep.episode_id,
            ContextItemInner::Pattern(p) => p.id(),
        }
    }

    /// Get a summary string for this item (for logging/display).
    #[must_use]
    pub fn summary(&self) -> String {
        match &self.inner {
            ContextItemInner::Episode(ep) => {
                format!(
                    "[Episode {}] {} (salience={:.2}, priority={:.2})",
                    ep.episode_id,
                    ep.task_description.chars().take(50).collect::<String>(),
                    self.salience,
                    self.priority
                )
            }
            ContextItemInner::Pattern(p) => {
                format!(
                    "[Pattern {}] {} (salience={:.2}, priority={:.2})",
                    p.id(),
                    p.similarity_key().chars().take(50).collect::<String>(),
                    self.salience,
                    self.priority
                )
            }
        }
    }
}

/// Configuration for bounded context accumulation.
///
/// Controls how the `BundleAccumulator` manages its sliding window
/// of context items, including capacity limits and scoring weights.
///
/// # Examples
///
/// ```
/// use do_memory_core::context::BundleConfig;
///
/// // Default configuration (20 items, balanced weights)
/// let config = BundleConfig::default();
///
/// // Custom configuration for token-sensitive prompts
/// let token_aware = BundleConfig {
///     max_items: 10,
///     recency_weight: 0.3,
///     salience_weight: 0.5,
///     min_salience_threshold: 0.4,
///     recency_half_life_days: 14.0,
/// };
///
/// // Configuration favoring recent items
/// let recency_focused = BundleConfig {
///     max_items: 15,
///     recency_weight: 0.7,
///     salience_weight: 0.2,
///     min_salience_threshold: 0.0,
///     recency_half_life_days: 30.0,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleConfig {
    /// Maximum number of items to accumulate (sliding window capacity)
    pub max_items: usize,
    /// Weight for recency in priority scoring (0.0-1.0)
    /// Higher values favor more recent items
    pub recency_weight: f32,
    /// Weight for salience in priority scoring (0.0-1.0)
    /// Higher values favor items with higher retrieval scores
    pub salience_weight: f32,
    /// Minimum salience threshold for admission (0.0-1.0)
    /// Items below this threshold are rejected
    pub min_salience_threshold: f32,
    /// Half-life in days for recency decay
    /// Items older than this get progressively lower recency scores
    pub recency_half_life_days: f32,
}

impl Default for BundleConfig {
    fn default() -> Self {
        Self {
            max_items: 20,
            recency_weight: 0.4,
            salience_weight: 0.4,
            min_salience_threshold: 0.2,
            recency_half_life_days: 30.0,
        }
    }
}

impl BundleConfig {
    /// Create a new bundle configuration.
    ///
    /// # Arguments
    ///
    /// * `max_items` - Maximum number of items in the bundle
    /// * `recency_weight` - Weight for recency scoring (should sum with salience to ~1.0)
    /// * `salience_weight` - Weight for salience scoring
    ///
    /// # Returns
    ///
    /// A new `BundleConfig` with specified parameters
    #[must_use]
    pub fn new(max_items: usize, recency_weight: f32, salience_weight: f32) -> Self {
        Self {
            max_items,
            recency_weight: recency_weight.clamp(0.0, 1.0),
            salience_weight: salience_weight.clamp(0.0, 1.0),
            min_salience_threshold: 0.2,
            recency_half_life_days: 30.0,
        }
    }

    /// Create a configuration optimized for token efficiency.
    ///
    /// Smaller bundle size with higher quality thresholds.
    #[must_use]
    pub fn token_efficient() -> Self {
        Self {
            max_items: 10,
            recency_weight: 0.3,
            salience_weight: 0.5,
            min_salience_threshold: 0.5,
            recency_half_life_days: 14.0,
        }
    }

    /// Create a configuration optimized for comprehensive context.
    ///
    /// Larger bundle size with lower thresholds.
    #[must_use]
    pub fn comprehensive() -> Self {
        Self {
            max_items: 50,
            recency_weight: 0.3,
            salience_weight: 0.5,
            min_salience_threshold: 0.1,
            recency_half_life_days: 60.0,
        }
    }

    /// Validate the configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if configuration is valid, `Err` with message if invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.max_items == 0 {
            return Err("max_items must be at least 1".to_string());
        }

        let weight_sum = self.recency_weight + self.salience_weight;
        if (weight_sum - 1.0).abs() > 0.2 {
            return Err(format!(
                "recency_weight + salience_weight should sum to ~1.0, got {weight_sum}"
            ));
        }

        if self.min_salience_threshold > 1.0 {
            return Err(format!(
                "min_salience_threshold must be <= 1.0, got {}",
                self.min_salience_threshold
            ));
        }

        if self.recency_half_life_days <= 0.0 {
            return Err(format!(
                "recency_half_life_days must be positive, got {}",
                self.recency_half_life_days
            ));
        }

        Ok(())
    }
}

/// Result of adding an item to the bundle.
#[derive(Debug, Clone)]
pub struct AddResult {
    /// Whether the item was accepted into the bundle
    pub accepted: bool,
    /// Number of items currently in the bundle
    pub current_size: usize,
    /// ID of item evicted (if capacity exceeded and an item was dropped)
    pub evicted_id: Option<Uuid>,
    /// Reason for rejection (if item was not accepted)
    pub rejection_reason: Option<String>,
}

impl AddResult {
    /// Create an accepted result.
    #[must_use]
    pub fn accepted(current_size: usize) -> Self {
        Self {
            accepted: true,
            current_size,
            evicted_id: None,
            rejection_reason: None,
        }
    }

    /// Create an accepted result with eviction.
    #[must_use]
    pub fn accepted_with_eviction(current_size: usize, evicted_id: Uuid) -> Self {
        Self {
            accepted: true,
            current_size,
            evicted_id: Some(evicted_id),
            rejection_reason: None,
        }
    }

    /// Create a rejected result.
    #[must_use]
    pub fn rejected(current_size: usize, reason: String) -> Self {
        Self {
            accepted: false,
            current_size,
            evicted_id: None,
            rejection_reason: Some(reason),
        }
    }
}

/// Statistics about the bundle accumulator.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleStats {
    /// Current number of items in the bundle
    pub current_size: usize,
    /// Total items ever added (accepted)
    pub total_added: u64,
    /// Total items rejected (below threshold)
    pub total_rejected: u64,
    /// Total items evicted (sliding window)
    pub total_evicted: u64,
    /// Average salience of items in bundle
    pub average_salience: f32,
    /// Average priority of items in bundle
    pub average_priority: f32,
    /// Oldest item timestamp in bundle
    pub oldest_timestamp: Option<DateTime<Utc>>,
    /// Newest item timestamp in bundle
    pub newest_timestamp: Option<DateTime<Utc>>,
}

impl BundleStats {
    /// Calculate the fill percentage of the bundle.
    #[must_use]
    pub fn fill_percentage(&self, max_items: usize) -> f32 {
        if max_items == 0 {
            return 0.0;
        }
        (self.current_size as f32 / max_items as f32) * 100.0
    }

    /// Calculate the acceptance rate.
    #[must_use]
    pub fn acceptance_rate(&self) -> f32 {
        let total = self.total_added + self.total_rejected;
        if total == 0 {
            return 0.0;
        }
        (self.total_added as f32 / total as f32) * 100.0
    }
}
