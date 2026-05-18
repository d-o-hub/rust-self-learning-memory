//! CloudEvents EventEmitter trait for external interoperability (WG-149).
//!
//! Implements the CloudEvents 1.0 specification for standardizing memory
//! lifecycle events. The `EventEmitter` trait provides a pluggable interface
//! for routing events to external sinks (logging, HTTP, message queues, etc.).
//!
//! # References
//! - CloudEvents 1.0 Specification: `<https://github.com/cloudevents/spec>`
//! - ADR-054: CloudEvents EventEmitter Integration

use crate::types::MemoryEvent;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// A CloudEvent following the CloudEvents 1.0 specification.
///
/// CloudEvents provides a standardized format for describing event data,
/// enabling interoperability across services, platforms, and systems.
///
/// # Required Attributes (per CloudEvents 1.0)
///
/// - `id`: Unique identifier for the event
/// - `source`: Identifies the context in which the event happened
/// - `specversion`: The version of the CloudEvents specification (always "1.0")
/// - `type`: Describes the type of event related to the originating occurrence
///
/// # Examples
///
/// ```
/// use do_memory_core::types::emitter::CloudEvent;
///
/// let event = CloudEvent::new(
///     "com.do-memory.episode.created".to_string(),
///     "do-memory://core".to_string(),
///     serde_json::json!({"episode_id": "abc-123"}),
/// );
/// println!("{}", event.event_type);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEvent {
    /// CloudEvents specification version (always "1.0")
    pub specversion: String,
    /// Unique identifier for this event
    pub id: String,
    /// Event type (e.g., "com.do-memory.episode.created")
    #[serde(rename = "type")]
    pub event_type: String,
    /// Source of the event (e.g., "do-memory://core")
    pub source: String,
    /// Subject of the event (e.g., episode ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Timestamp of when the event occurred (RFC 3339)
    pub time: String,
    /// Content type of the data payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datacontenttype: Option<String>,
    /// Schema reference for the data payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataschema: Option<String>,
    /// Event data payload
    pub data: serde_json::Value,
}

/// Well-known CloudEvents metadata.
pub mod metadata {
    /// Default source for all memory events.
    pub const DEFAULT_SOURCE: &str = "do-memory://core";
    /// CloudEvents specification version used.
    pub const SPEC_VERSION: &str = "1.0";
    /// Default content type for event data.
    pub const DEFAULT_CONTENT_TYPE: &str = "application/json";
}

impl CloudEvent {
    /// Create a new CloudEvent with auto-generated ID and current timestamp.
    #[must_use]
    pub fn new(event_type: String, source: String, data: serde_json::Value) -> Self {
        Self {
            specversion: metadata::SPEC_VERSION.to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            source,
            subject: None,
            time: Utc::now().to_rfc3339(),
            datacontenttype: Some(metadata::DEFAULT_CONTENT_TYPE.to_string()),
            dataschema: None,
            data,
        }
    }

    /// Create a new CloudEvent from a `MemoryEvent` with the default source.
    #[must_use]
    pub fn from_memory_event(event: &MemoryEvent) -> Self {
        let (event_type, subject, data) = MemoryEventMapping::map_event(event);
        let mut cloud_event = Self::new(event_type, metadata::DEFAULT_SOURCE.to_string(), data);
        cloud_event.subject = Some(subject);
        cloud_event
    }

    /// Set the subject of this event.
    #[must_use]
    pub fn with_subject(mut self, subject: String) -> Self {
        self.subject = Some(subject);
        self
    }

    /// Set the data schema reference.
    #[must_use]
    pub fn with_schema(mut self, schema: String) -> Self {
        self.dataschema = Some(schema);
        self
    }
}

/// Mapping from internal `MemoryEvent` variants to CloudEvent attributes.
///
/// This provides a canonical mapping as defined in ADR-054.
pub struct MemoryEventMapping;

impl MemoryEventMapping {
    /// Map a `MemoryEvent` to its CloudEvent type, subject, and data payload.
    fn map_event(event: &MemoryEvent) -> (String, String, serde_json::Value) {
        match event {
            MemoryEvent::EpisodeCreated {
                id,
                task,
                timestamp,
            } => (
                "com.do-memory.episode.created".to_string(),
                id.clone(),
                serde_json::json!({
                    "episode_id": id,
                    "task": task,
                    "creation_timestamp": timestamp,
                }),
            ),
            MemoryEvent::EpisodeCompleted {
                id,
                reward,
                timestamp,
            } => (
                "com.do-memory.episode.completed".to_string(),
                id.clone(),
                serde_json::json!({
                    "episode_id": id,
                    "reward_score": reward,
                    "completion_timestamp": timestamp,
                }),
            ),
            MemoryEvent::EpisodeGarbageCollected {
                id,
                reason,
                timestamp,
            } => (
                "com.do-memory.episode.gc".to_string(),
                id.clone(),
                serde_json::json!({
                    "episode_id": id,
                    "gc_reason": reason,
                    "gc_timestamp": timestamp,
                }),
            ),
            MemoryEvent::PatternExtracted {
                id,
                source_episodes,
                timestamp,
            } => (
                "com.do-memory.pattern.extracted".to_string(),
                id.clone(),
                serde_json::json!({
                    "pattern_id": id,
                    "source_episodes": source_episodes,
                    "extraction_timestamp": timestamp,
                }),
            ),
            MemoryEvent::ProceduralMemoryCreated {
                id,
                name,
                timestamp,
            } => (
                "com.do-memory.procedural.created".to_string(),
                id.clone(),
                serde_json::json!({
                    "procedural_id": id,
                    "name": name,
                    "timestamp": timestamp,
                }),
            ),
            MemoryEvent::ProceduralMemoryUpdated { id, timestamp } => (
                "com.do-memory.procedural.updated".to_string(),
                id.clone(),
                serde_json::json!({
                    "procedural_id": id,
                    "timestamp": timestamp,
                }),
            ),
        }
    }
}

/// Trait for emitting CloudEvents to external sinks.
///
/// Implementations can route events to logging, HTTP webhooks, message queues,
/// or any other destination. The default implementation uses `NoOpEmitter` which
/// discards all events.
///
/// # Examples
///
/// ```no_run
/// use do_memory_core::types::emitter::{CloudEvent, EventEmitter};
/// use async_trait::async_trait;
///
/// struct ConsoleEmitter;
///
/// #[async_trait]
/// impl EventEmitter for ConsoleEmitter {
///     async fn emit(&self, event: CloudEvent) -> anyhow::Result<()> {
///         println!("Event: {} - {}", event.event_type, event.subject.unwrap_or_default());
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait EventEmitter: Send + Sync {
    /// Emit a single CloudEvent to the configured sink.
    async fn emit(&self, event: CloudEvent) -> anyhow::Result<()>;

    /// Emit a batch of CloudEvents.
    ///
    /// Default implementation emits events sequentially.
    async fn emit_batch(&self, events: Vec<CloudEvent>) -> anyhow::Result<()> {
        for event in events {
            self.emit(event).await?;
        }
        Ok(())
    }
}

/// Configuration mode for selecting the CloudEvent emitter at construction.
///
/// Controls which `EventEmitter` implementation is used when creating a
/// `SelfLearningMemory`. The emitter is constructed based on this mode
/// so the runtime cost is only incurred if a non-NoOp mode is chosen.
///
/// # Examples
///
/// ```
/// use do_memory_core::types::emitter::EventEmitterMode;
///
/// // Default: NoOp (zero overhead)
/// let mode = EventEmitterMode::default();
/// assert!(matches!(mode, EventEmitterMode::NoOp));
///
/// // Log events via tracing
/// let log_mode = EventEmitterMode::Log;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EventEmitterMode {
    /// No operation — events are silently discarded (default, zero overhead).
    #[default]
    NoOp,
    /// Log events via the `tracing` crate at INFO level.
    Log,
    /// Send events to an HTTP webhook endpoint (requires `http-emitter` feature).
    #[cfg(feature = "http-emitter")]
    Http {
        /// Webhook endpoint URL for CloudEvents delivery.
        url: String,
    },
}

impl EventEmitterMode {
    /// Construct the appropriate `EventEmitter` for this mode.
    #[must_use]
    pub fn build(&self) -> std::sync::Arc<dyn EventEmitter> {
        match self {
            Self::NoOp => std::sync::Arc::new(super::sinks::NoOpEmitter),
            Self::Log => std::sync::Arc::new(super::sinks::LogEmitter),
            #[cfg(feature = "http-emitter")]
            Self::Http { url } => std::sync::Arc::new(super::sinks::HttpEmitter::new(url)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MemoryEvent;

    #[test]
    fn test_cloud_event_new() {
        let event = CloudEvent::new(
            "com.test.event".to_string(),
            "test://source".to_string(),
            serde_json::json!({"key": "value"}),
        );

        assert_eq!(event.specversion, "1.0");
        assert_eq!(event.event_type, "com.test.event");
        assert_eq!(event.source, "test://source");
        assert!(!event.id.is_empty());
        assert!(!event.time.is_empty());
        assert_eq!(event.data, serde_json::json!({"key": "value"}));
    }

    #[test]
    fn test_from_memory_event_episode_created() {
        let mem_event = MemoryEvent::EpisodeCreated {
            id: "ep-001".to_string(),
            task: "Implement auth".to_string(),
            timestamp: 1_714_500_000,
        };

        let cloud_event = CloudEvent::from_memory_event(&mem_event);

        assert_eq!(cloud_event.event_type, "com.do-memory.episode.created");
        assert_eq!(cloud_event.subject.as_deref(), Some("ep-001"));
        assert_eq!(cloud_event.source, metadata::DEFAULT_SOURCE);
        assert_eq!(cloud_event.data["episode_id"], "ep-001");
        assert_eq!(cloud_event.data["task"], "Implement auth");
    }

    #[test]
    fn test_from_memory_event_episode_completed() {
        let mem_event = MemoryEvent::EpisodeCompleted {
            id: "ep-002".to_string(),
            reward: 0.85,
            timestamp: 1_714_500_100,
        };

        let cloud_event = CloudEvent::from_memory_event(&mem_event);
        assert_eq!(cloud_event.event_type, "com.do-memory.episode.completed");
        assert!((cloud_event.data["reward_score"].as_f64().unwrap() - 0.85_f64).abs() < 1e-6);
    }

    #[test]
    fn test_from_memory_event_episode_gc() {
        let mem_event = MemoryEvent::EpisodeGarbageCollected {
            id: "ep-003".to_string(),
            reason: "ttl_expired".to_string(),
            timestamp: 1_714_500_200,
        };

        let cloud_event = CloudEvent::from_memory_event(&mem_event);

        assert_eq!(cloud_event.event_type, "com.do-memory.episode.gc");
        assert_eq!(cloud_event.data["gc_reason"], "ttl_expired");
    }

    #[test]
    fn test_from_memory_event_pattern_extracted() {
        let mem_event = MemoryEvent::PatternExtracted {
            id: "pat-001".to_string(),
            source_episodes: vec!["ep-001".to_string(), "ep-002".to_string()],
            timestamp: 1_714_500_300,
        };

        let cloud_event = CloudEvent::from_memory_event(&mem_event);

        assert_eq!(cloud_event.event_type, "com.do-memory.pattern.extracted");
        assert_eq!(
            cloud_event.data["source_episodes"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn test_event_emitter_mode_default() {
        let mode = EventEmitterMode::default();
        assert!(matches!(mode, EventEmitterMode::NoOp));
    }

    #[test]
    fn test_event_emitter_mode_build_noop() {
        let emitter = EventEmitterMode::NoOp.build();
        // Verify the emitter works (discards events silently)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let event = CloudEvent::new(
            "com.test".to_string(),
            "test://src".to_string(),
            serde_json::json!({"k": "v"}),
        );
        rt.block_on(async {
            let result = emitter.emit(event).await;
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_event_emitter_mode_build_log() {
        let emitter = EventEmitterMode::Log.build();
        // Verify the emitter works (logs but doesn't error)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let event = CloudEvent::new(
            "com.test".to_string(),
            "test://src".to_string(),
            serde_json::json!({"k": "v"}),
        );
        rt.block_on(async {
            let result = emitter.emit(event).await;
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_serialization() {
        let event = CloudEvent::new(
            "com.test".to_string(),
            "test://src".to_string(),
            serde_json::json!({"key": "value"}),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: CloudEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.specversion, "1.0");
        assert_eq!(deserialized.event_type, "com.test");
    }
}
