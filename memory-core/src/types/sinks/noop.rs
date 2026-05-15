//! NoOpEmitter — Discards all events with zero overhead.
//!
//! This is the default emitter used when external eventing is not configured.
//! It satisfies the `EventEmitter` trait without performing any work.

use crate::types::emitter::{CloudEvent, EventEmitter};
use async_trait::async_trait;

/// An `EventEmitter` that discards all events.
///
/// Used as the default when no external event sink is configured.
/// Has effectively zero performance overhead.
///
/// # Examples
///
/// ```no_run
/// use do_memory_core::types::emitter::CloudEvent;
/// use do_memory_core::types::sinks::NoOpEmitter;
/// use do_memory_core::types::emitter::EventEmitter;
///
/// # async fn example() {
/// let emitter = NoOpEmitter;
/// let event = CloudEvent::new(
///     "com.test".to_string(),
///     "test://src".to_string(),
///     serde_json::json!({"key": "value"}),
/// );
/// // Silently discarded
/// emitter.emit(event).await.unwrap();
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpEmitter;

#[async_trait]
impl EventEmitter for NoOpEmitter {
    #[inline]
    async fn emit(&self, _event: CloudEvent) -> anyhow::Result<()> {
        // No operation — events are silently discarded
        Ok(())
    }

    #[inline]
    async fn emit_batch(&self, _events: Vec<CloudEvent>) -> anyhow::Result<()> {
        // No operation — all events are silently discarded
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_emitter_discards() {
        let emitter = NoOpEmitter;
        let event = CloudEvent::new(
            "com.test.event".to_string(),
            "test://source".to_string(),
            serde_json::json!({"test": true}),
        );

        // Should always succeed
        let result = emitter.emit(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_noop_emitter_batch() {
        let emitter = NoOpEmitter;
        let events = (0..100)
            .map(|i| {
                CloudEvent::new(
                    format!("com.test.event.{i}"),
                    "test://src".to_string(),
                    serde_json::json!({"index": i}),
                )
            })
            .collect();

        let result = emitter.emit_batch(events).await;
        assert!(result.is_ok());
    }
}
