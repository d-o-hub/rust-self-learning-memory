//! LogEmitter — Logs CloudEvents via the `tracing` crate.
//!
//! Useful for development, debugging, and environments where events should
//! be captured in structured logs rather than dispatched to external services.

use crate::types::emitter::{CloudEvent, EventEmitter};
use async_trait::async_trait;

/// An `EventEmitter` that logs CloudEvents using `tracing`.
///
/// Each event is logged at `INFO` level with structured fields for
/// the event type, source, and subject. The full data payload is
/// serialized as JSON in the log message.
///
/// # Examples
///
/// ```no_run
/// use do_memory_core::types::emitter::CloudEvent;
/// use do_memory_core::types::sinks::LogEmitter;
/// use do_memory_core::types::emitter::EventEmitter;
///
/// # async fn example() {
/// let emitter = LogEmitter;
/// let event = CloudEvent::new(
///     "com.test".to_string(),
///     "test://src".to_string(),
///     serde_json::json!({"key": "value"}),
/// );
/// emitter.emit(event).await.unwrap();
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct LogEmitter;

#[async_trait]
impl EventEmitter for LogEmitter {
    async fn emit(&self, event: CloudEvent) -> anyhow::Result<()> {
        let data_str = serde_json::to_string(&event.data).unwrap_or_default();
        tracing::info!(
            event_type = %event.event_type,
            event_id = %event.id,
            source = %event.source,
            subject = ?event.subject,
            time = %event.time,
            "CloudEvent emitted: {data}",
            data = data_str,
        );
        Ok(())
    }

    async fn emit_batch(&self, events: Vec<CloudEvent>) -> anyhow::Result<()> {
        let count = events.len();
        tracing::info!(batch_size = count, "Emitting batch of {count} CloudEvents");
        for event in events {
            self.emit(event).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_emitter_emit() {
        let emitter = LogEmitter;
        let event = CloudEvent::new(
            "com.test.event".to_string(),
            "test://source".to_string(),
            serde_json::json!({"test": true}),
        );

        // Should not error
        let result = emitter.emit(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_log_emitter_emit_batch() {
        let emitter = LogEmitter;
        let events = vec![
            CloudEvent::new(
                "com.test.a".to_string(),
                "test://src".to_string(),
                serde_json::json!({"n": 1}),
            ),
            CloudEvent::new(
                "com.test.b".to_string(),
                "test://src".to_string(),
                serde_json::json!({"n": 2}),
            ),
        ];

        let result = emitter.emit_batch(events).await;
        assert!(result.is_ok());
    }
}
