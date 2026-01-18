//! Buffer implementation for step batching
//!
//! Provides the StepBuffer struct and core buffer operations
//! for efficient I/O in episodic memory.

use crate::episode::ExecutionStep;
use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::{debug, trace};
use uuid::Uuid;

use super::config::BatchConfig;

/// In-memory buffer for execution steps pending flush to storage.
///
/// Accumulates steps for an episode and determines when they should be
/// flushed to persistent storage based on size and time thresholds.
///
/// # Thread Safety
///
/// `StepBuffer` is not thread-safe on its own. The parent `SelfLearningMemory`
/// manages concurrent access using `Arc<RwLock<HashMap<Uuid, StepBuffer>>>`.
///
/// # Flush Conditions
///
/// Steps are flushed when:
/// 1. Buffer size >= `max_batch_size`, OR
/// 2. Time since last flush >= `flush_interval_ms`, OR
/// 3. Manual `take_steps()` is called
///
/// # Examples
///
/// ```
/// use memory_core::memory::step_buffer::StepBuffer;
/// use memory_core::{BatchConfig, ExecutionStep};
/// use uuid::Uuid;
///
/// # fn example() -> anyhow::Result<()> {
/// let episode_id = Uuid::new_v4();
/// let config = BatchConfig::default();
/// let mut buffer = StepBuffer::new(episode_id, config);
///
/// // Add steps
/// let step1 = ExecutionStep::new(1, "tool_a".to_string(), "Action 1".to_string());
/// buffer.add_step(step1)?;
///
/// let step2 = ExecutionStep::new(2, "tool_b".to_string(), "Action 2".to_string());
/// buffer.add_step(step2)?;
///
/// // Check if flush is needed
/// if buffer.should_flush() {
///     let steps_to_persist = buffer.take_steps();
///     // ... persist steps to storage ...
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct StepBuffer {
    /// Episode ID this buffer is for
    episode_id: Uuid,

    /// Buffered steps pending flush
    pub(super) steps: Vec<ExecutionStep>,

    /// Configuration controlling flush behavior
    config: BatchConfig,

    /// Timestamp of last flush operation
    pub(super) last_flush: Instant,

    /// Total number of steps processed (including flushed)
    total_steps_processed: usize,
}

impl StepBuffer {
    /// Create a new step buffer for an episode.
    ///
    /// Initializes an empty buffer with the given configuration.
    /// The flush timer starts from creation time.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode this buffer is for
    /// * `config` - Batch configuration controlling flush behavior
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::memory::step_buffer::StepBuffer;
    /// use memory_core::BatchConfig;
    /// use uuid::Uuid;
    ///
    /// let episode_id = Uuid::new_v4();
    /// let buffer = StepBuffer::new(episode_id, BatchConfig::default());
    ///
    /// assert!(buffer.is_empty());
    /// assert_eq!(buffer.len(), 0);
    /// ```
    pub fn new(episode_id: Uuid, config: BatchConfig) -> Self {
        debug!(
            episode_id = %episode_id,
            max_batch_size = config.max_batch_size,
            flush_interval_ms = config.flush_interval_ms,
            auto_flush = config.auto_flush,
            "Created new step buffer"
        );

        Self {
            episode_id,
            steps: Vec::new(),
            config,
            last_flush: Instant::now(),
            total_steps_processed: 0,
        }
    }

    /// Add a step to the buffer.
    ///
    /// Appends the step to the internal buffer. Does not immediately
    /// persist to storage - call `should_flush()` to check if flush is needed.
    ///
    /// # Arguments
    ///
    /// * `step` - Execution step to buffer
    ///
    /// # Returns
    ///
    /// `Ok(())` on success. Currently always succeeds but returns Result
    /// for future extensibility (e.g., validation, size limits).
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::memory::step_buffer::StepBuffer;
    /// use memory_core::{BatchConfig, ExecutionStep};
    /// use uuid::Uuid;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let episode_id = Uuid::new_v4();
    /// let mut buffer = StepBuffer::new(episode_id, BatchConfig::default());
    ///
    /// let step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    /// buffer.add_step(step)?;
    ///
    /// assert_eq!(buffer.len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_step(&mut self, step: ExecutionStep) -> Result<()> {
        trace!(
            episode_id = %self.episode_id,
            step_number = step.step_number,
            buffer_size = self.steps.len(),
            "Adding step to buffer"
        );

        self.steps.push(step);
        self.total_steps_processed += 1;

        Ok(())
    }

    /// Check if the buffer should be flushed.
    ///
    /// Returns true if any flush condition is met:
    /// - Buffer size >= `max_batch_size`
    /// - Time since last flush >= `flush_interval_ms`
    /// - Buffer is non-empty and `auto_flush` is disabled
    ///
    /// # Returns
    ///
    /// `true` if steps should be flushed to storage, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::memory::step_buffer::StepBuffer;
    /// use memory_core::{BatchConfig, ExecutionStep};
    /// use uuid::Uuid;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let episode_id = Uuid::new_v4();
    /// let config = BatchConfig {
    ///     max_batch_size: 2,
    ///     flush_interval_ms: 5000,
    ///     auto_flush: true,
    /// };
    /// let mut buffer = StepBuffer::new(episode_id, config);
    ///
    /// // Initially shouldn't flush
    /// assert!(!buffer.should_flush());
    ///
    /// // Add steps until threshold
    /// buffer.add_step(ExecutionStep::new(1, "tool".to_string(), "action".to_string()))?;
    /// buffer.add_step(ExecutionStep::new(2, "tool".to_string(), "action".to_string()))?;
    ///
    /// // Now should flush (size threshold reached)
    /// assert!(buffer.should_flush());
    /// # Ok(())
    /// # }
    /// ```
    pub fn should_flush(&self) -> bool {
        // Empty buffer never needs flush
        if self.steps.is_empty() {
            return false;
        }

        // If auto-flush disabled, don't auto-flush
        if !self.config.auto_flush {
            return false;
        }

        // Check size threshold
        if self.steps.len() >= self.config.max_batch_size {
            debug!(
                episode_id = %self.episode_id,
                buffer_size = self.steps.len(),
                max_batch_size = self.config.max_batch_size,
                "Buffer size threshold reached"
            );
            return true;
        }

        // Check time threshold
        let elapsed = self.last_flush.elapsed();
        let flush_interval = Duration::from_millis(self.config.flush_interval_ms);

        if elapsed >= flush_interval {
            debug!(
                episode_id = %self.episode_id,
                elapsed_ms = elapsed.as_millis(),
                flush_interval_ms = self.config.flush_interval_ms,
                "Buffer time threshold reached"
            );
            return true;
        }

        false
    }

    /// Get the episode ID this buffer is for.
    ///
    /// # Returns
    ///
    /// UUID of the episode.
    #[must_use]
    pub fn episode_id(&self) -> Uuid {
        self.episode_id
    }

    /// Get the number of buffered steps.
    ///
    /// # Returns
    ///
    /// Number of steps currently in the buffer (not yet flushed).
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::memory::step_buffer::StepBuffer;
    /// use memory_core::{BatchConfig, ExecutionStep};
    /// use uuid::Uuid;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let episode_id = Uuid::new_v4();
    /// let mut buffer = StepBuffer::new(episode_id, BatchConfig::default());
    ///
    /// assert_eq!(buffer.len(), 0);
    ///
    /// buffer.add_step(ExecutionStep::new(1, "tool".to_string(), "action".to_string()))?;
    /// assert_eq!(buffer.len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if the buffer is empty.
    ///
    /// # Returns
    ///
    /// `true` if no steps are buffered, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::memory::step_buffer::StepBuffer;
    /// use memory_core::BatchConfig;
    /// use uuid::Uuid;
    ///
    /// let episode_id = Uuid::new_v4();
    /// let buffer = StepBuffer::new(episode_id, BatchConfig::default());
    ///
    /// assert!(buffer.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get the total number of steps processed (including flushed).
    ///
    /// # Returns
    ///
    /// Total count of all steps added to this buffer, including those
    /// that have been flushed.
    #[must_use]
    pub fn total_steps_processed(&self) -> usize {
        self.total_steps_processed
    }

    /// Get time elapsed since last flush.
    ///
    /// # Returns
    ///
    /// Duration since the last `take_steps()` call or buffer creation.
    #[must_use]
    pub fn time_since_last_flush(&self) -> Duration {
        self.last_flush.elapsed()
    }

    /// Get mutable reference to steps for use by parent module.
    ///
    /// # Returns
    ///
    /// Mutable reference to the steps vector.
    pub(super) fn steps_mut(&mut self) -> &mut Vec<ExecutionStep> {
        &mut self.steps
    }

    /// Get mutable reference to last_flush for use by parent module.
    ///
    /// # Returns
    ///
    /// Mutable reference to the last_flush Instant.
    pub(super) fn last_flush_mut(&mut self) -> &mut Instant {
        &mut self.last_flush
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_buffer_creation() {
        let episode_id = Uuid::new_v4();
        let buffer = StepBuffer::new(episode_id, BatchConfig::default());

        assert_eq!(buffer.episode_id(), episode_id);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.total_steps_processed(), 0);
    }

    #[test]
    fn test_add_steps() {
        let episode_id = Uuid::new_v4();
        let mut buffer = StepBuffer::new(episode_id, BatchConfig::default());

        let step1 = ExecutionStep::new(1, "tool_a".to_string(), "Action 1".to_string());
        let step2 = ExecutionStep::new(2, "tool_b".to_string(), "Action 2".to_string());

        buffer.add_step(step1).unwrap();
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.total_steps_processed(), 1);

        buffer.add_step(step2).unwrap();
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.total_steps_processed(), 2);
    }

    #[test]
    fn test_should_flush_size_threshold() {
        let episode_id = Uuid::new_v4();
        let config = BatchConfig {
            max_batch_size: 3,
            flush_interval_ms: 10000,
            auto_flush: true,
        };
        let mut buffer = StepBuffer::new(episode_id, config);

        // Empty buffer shouldn't flush
        assert!(!buffer.should_flush());

        // Add steps below threshold
        buffer
            .add_step(ExecutionStep::new(
                1,
                "tool".to_string(),
                "action".to_string(),
            ))
            .unwrap();
        buffer
            .add_step(ExecutionStep::new(
                2,
                "tool".to_string(),
                "action".to_string(),
            ))
            .unwrap();
        assert!(!buffer.should_flush());

        // Reach threshold
        buffer
            .add_step(ExecutionStep::new(
                3,
                "tool".to_string(),
                "action".to_string(),
            ))
            .unwrap();
        assert!(buffer.should_flush());
    }

    #[test]
    fn test_should_flush_time_threshold() {
        let episode_id = Uuid::new_v4();
        let config = BatchConfig {
            max_batch_size: 100,
            flush_interval_ms: 50, // 50ms for testing
            auto_flush: true,
        };
        let mut buffer = StepBuffer::new(episode_id, config);

        buffer
            .add_step(ExecutionStep::new(
                1,
                "tool".to_string(),
                "action".to_string(),
            ))
            .unwrap();

        // Should not flush immediately
        assert!(!buffer.should_flush());

        // Wait for time threshold
        std::thread::sleep(Duration::from_millis(60));

        // Should now flush due to time
        assert!(buffer.should_flush());
    }

    #[test]
    fn test_should_flush_manual_only() {
        let episode_id = Uuid::new_v4();
        let config = BatchConfig::manual_only();
        let mut buffer = StepBuffer::new(episode_id, config);

        // Add many steps
        for i in 1..=10 {
            buffer
                .add_step(ExecutionStep::new(
                    i,
                    "tool".to_string(),
                    "action".to_string(),
                ))
                .unwrap();
        }

        // Should never auto-flush with manual_only config
        assert!(!buffer.should_flush());
    }
}
