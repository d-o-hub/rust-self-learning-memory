//! Operations for step buffer management
//!
//! Provides step buffer operations including `take_steps` and related functionality.

use crate::episode::ExecutionStep;
use std::time::Instant;
use tracing::debug;

use super::buffer::StepBuffer;

impl StepBuffer {
    /// Take all buffered steps and reset the buffer.
    ///
    /// Removes and returns all steps from the buffer, resetting it to empty state.
    /// Updates the last flush timestamp. This is typically called after
    /// `should_flush()` returns true to retrieve steps for persistence.
    ///
    /// # Returns
    ///
    /// Vector of all buffered steps. Empty if buffer was empty.
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
    /// // Add some steps
    /// buffer.add_step(ExecutionStep::new(1, "tool".to_string(), "action".to_string()))?;
    /// buffer.add_step(ExecutionStep::new(2, "tool".to_string(), "action".to_string()))?;
    ///
    /// // Take steps
    /// let steps = buffer.take_steps();
    /// assert_eq!(steps.len(), 2);
    ///
    /// // Buffer is now empty
    /// assert!(buffer.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn take_steps(&mut self) -> Vec<ExecutionStep> {
        let step_count = self.len();

        if step_count > 0 {
            debug!(
                episode_id = %self.episode_id(),
                step_count = step_count,
                total_processed = self.total_steps_processed(),
                "Flushing buffered steps"
            );
        }

        // Take ownership of steps, leaving empty vec
        let steps = std::mem::take(self.steps_mut());

        // Update flush timestamp
        *self.last_flush_mut() = Instant::now();

        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BatchConfig;
    use std::time::Duration;
    use uuid::Uuid;

    #[test]
    fn test_take_steps() {
        let episode_id = Uuid::new_v4();
        let mut buffer = StepBuffer::new(episode_id, BatchConfig::default());

        // Add steps
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
        buffer
            .add_step(ExecutionStep::new(
                3,
                "tool".to_string(),
                "action".to_string(),
            ))
            .unwrap();

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.total_steps_processed(), 3);

        // Take steps
        let steps = buffer.take_steps();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].step_number, 1);
        assert_eq!(steps[1].step_number, 2);
        assert_eq!(steps[2].step_number, 3);

        // Buffer should be empty
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        // Total processed should still be 3
        assert_eq!(buffer.total_steps_processed(), 3);
    }

    #[test]
    fn test_take_steps_resets_timer() {
        let episode_id = Uuid::new_v4();
        let config = BatchConfig {
            max_batch_size: 100,
            flush_interval_ms: 100,
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

        // Wait a bit
        std::thread::sleep(Duration::from_millis(60));

        // Take steps (resets timer)
        let _ = buffer.take_steps();

        // Add another step
        buffer
            .add_step(ExecutionStep::new(
                2,
                "tool".to_string(),
                "action".to_string(),
            ))
            .unwrap();

        // Should not flush yet (timer was reset)
        assert!(!buffer.should_flush());
    }

    #[test]
    fn test_take_empty_buffer() {
        let episode_id = Uuid::new_v4();
        let mut buffer = StepBuffer::new(episode_id, BatchConfig::default());

        // Take from empty buffer
        let steps = buffer.take_steps();
        assert!(steps.is_empty());

        // Buffer should still be empty
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_take_steps_multiple_times() {
        let episode_id = Uuid::new_v4();
        let mut buffer = StepBuffer::new(episode_id, BatchConfig::default());

        // Add and take first batch
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

        let first_batch = buffer.take_steps();
        assert_eq!(first_batch.len(), 2);

        // Add and take second batch
        buffer
            .add_step(ExecutionStep::new(
                3,
                "tool".to_string(),
                "action".to_string(),
            ))
            .unwrap();

        let second_batch = buffer.take_steps();
        assert_eq!(second_batch.len(), 1);

        // Verify order
        assert_eq!(first_batch[0].step_number, 1);
        assert_eq!(first_batch[1].step_number, 2);
        assert_eq!(second_batch[0].step_number, 3);
    }
}
