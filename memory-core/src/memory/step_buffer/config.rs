//! Batch configuration for step buffering
//!
//! Controls when buffered steps are flushed to storage.

/// Configuration for step batching and auto-flush behavior.
///
/// Controls when buffered execution steps are flushed to persistent storage.
/// Steps can be flushed based on size thresholds, time intervals, or manual triggers.
///
/// # Examples
///
/// ```
/// use memory_core::BatchConfig;
///
/// // Default configuration (50 steps, 5 seconds)
/// let config = BatchConfig::default();
/// assert_eq!(config.max_batch_size, 50);
/// assert_eq!(config.flush_interval_ms, 5000);
///
/// // High-frequency episodes (20 steps, 2 seconds)
/// let high_freq = BatchConfig::high_frequency();
///
/// // Low-frequency episodes (100 steps, 10 seconds)
/// let low_freq = BatchConfig::low_frequency();
///
/// // Manual flush only (no auto-flush)
/// let manual = BatchConfig::manual_only();
/// assert!(!manual.auto_flush);
///
/// // Custom configuration
/// let custom = BatchConfig::new(30, 3000, true);
/// assert_eq!(custom.max_batch_size, 30);
/// assert_eq!(custom.flush_interval_ms, 3000);
/// assert!(custom.auto_flush);
/// ```
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of steps to buffer before auto-flush (default: 50)
    pub max_batch_size: usize,
    /// Time interval in milliseconds between auto-flushes (default: 5000)
    pub flush_interval_ms: u64,
    /// Whether to enable automatic flushing (default: true)
    pub auto_flush: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 50,
            flush_interval_ms: 5000,
            auto_flush: true,
        }
    }
}

impl BatchConfig {
    /// Create a new batch configuration with custom values.
    ///
    /// # Arguments
    ///
    /// * `max_batch_size` - Maximum steps to buffer before flush
    /// * `flush_interval_ms` - Milliseconds between time-based flushes
    /// * `auto_flush` - Enable automatic flushing on thresholds
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::BatchConfig;
    ///
    /// let config = BatchConfig::new(100, 10000, true);
    /// assert_eq!(config.max_batch_size, 100);
    /// assert_eq!(config.flush_interval_ms, 10000);
    /// assert!(config.auto_flush);
    /// ```
    #[must_use]
    pub fn new(max_batch_size: usize, flush_interval_ms: u64, auto_flush: bool) -> Self {
        Self {
            max_batch_size,
            flush_interval_ms,
            auto_flush,
        }
    }

    /// Create a configuration for high-frequency episodes.
    ///
    /// Uses smaller buffer (20 steps) and shorter interval (2 seconds)
    /// for episodes where steps are logged rapidly.
    #[must_use]
    pub fn high_frequency() -> Self {
        Self {
            max_batch_size: 20,
            flush_interval_ms: 2000,
            auto_flush: true,
        }
    }

    /// Create a configuration for low-frequency episodes.
    ///
    /// Uses larger buffer (100 steps) and longer interval (10 seconds)
    /// for episodes where steps are logged slowly.
    #[must_use]
    pub fn low_frequency() -> Self {
        Self {
            max_batch_size: 100,
            flush_interval_ms: 10000,
            auto_flush: true,
        }
    }

    /// Create a configuration with manual flush only.
    ///
    /// Disables all automatic flushing. Steps are only persisted
    /// when explicitly flushed or when the episode completes.
    #[must_use]
    pub fn manual_only() -> Self {
        Self {
            max_batch_size: usize::MAX,
            flush_interval_ms: u64::MAX,
            auto_flush: false,
        }
    }
}
