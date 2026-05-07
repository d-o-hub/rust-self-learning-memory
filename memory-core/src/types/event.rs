//! Memory event types for broadcast channel notification.
//!
//! Events are emitted when significant lifecycle operations occur in the memory system,
//! allowing external subscribers to react to changes without coupling to internal state.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Events emitted by the memory system for lifecycle notifications.
///
/// Uses `tokio::sync::broadcast` channel for efficient fan-out to multiple subscribers.
/// Subscribers can use `memory.subscribe()` to receive events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MemoryEvent {
    /// A new episode was created.
    EpisodeCreated {
        /// Episode ID
        id: String,
        /// Task description (truncated to first 100 chars)
        task: String,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
    /// An episode was completed.
    EpisodeCompleted {
        /// Episode ID
        id: String,
        /// Final reward score
        reward: f32,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
    /// An episode was garbage collected.
    EpisodeGarbageCollected {
        /// Episode ID
        id: String,
        /// Reason for GC (e.g., "ttl_expired", "capacity_eviction", "manual")
        reason: String,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
    /// A pattern was extracted and stored.
    PatternExtracted {
        /// Pattern ID
        id: String,
        /// Source episode IDs
        source_episodes: Vec<String>,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
    /// Standardized task started event (CloudEvents compatible).
    TaskStarted {
        /// Unique task identifier
        task_id: Uuid,
        /// Identifier of the agent performing the task
        agent_id: String,
        /// Flexible metadata associated with the task
        metadata: serde_json::Value,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
    /// Standardized task completed event (CloudEvents compatible).
    TaskCompleted {
        /// Unique task identifier
        task_id: Uuid,
        /// Execution duration in milliseconds
        duration_ms: u64,
        /// Whether the task was successful
        success: bool,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
    /// Standardized reward scored event (CloudEvents compatible).
    RewardScored {
        /// Unique task identifier
        task_id: Uuid,
        /// The assigned reward score
        score: f64,
        /// Qualitative reason for the score
        reason: String,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
    /// Standardized reflection updated event (CloudEvents compatible).
    ReflectionUpdated {
        /// Target episode identifier
        episode_id: Uuid,
        /// Type of reflection generated (e.g., "improvement", "insight")
        reflection_type: String,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
    /// Standardized skill evolved event (CloudEvents compatible).
    SkillEvolved {
        /// Name of the skill or pattern promoted
        skill_name: String,
        /// Previous version/confidence
        from_version: u32,
        /// New version/confidence
        to_version: u32,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
    /// Standardized episode stored event (CloudEvents compatible).
    EpisodeStored {
        /// Target episode identifier
        episode_id: Uuid,
        /// Storage backend used (e.g., "turso", "redb")
        backend: String,
        /// Unix timestamp in seconds
        timestamp: u64,
    },
}

impl MemoryEvent {
    /// Get the timestamp of the event.
    #[must_use]
    pub fn timestamp(&self) -> u64 {
        match self {
            Self::EpisodeCreated { timestamp, .. }
            | Self::EpisodeCompleted { timestamp, .. }
            | Self::EpisodeGarbageCollected { timestamp, .. }
            | Self::PatternExtracted { timestamp, .. }
            | Self::TaskStarted { timestamp, .. }
            | Self::TaskCompleted { timestamp, .. }
            | Self::RewardScored { timestamp, .. }
            | Self::ReflectionUpdated { timestamp, .. }
            | Self::SkillEvolved { timestamp, .. }
            | Self::EpisodeStored { timestamp, .. } => *timestamp,
        }
    }

    /// Get the entity ID associated with the event.
    #[must_use]
    pub fn entity_id(&self) -> String {
        match self {
            Self::EpisodeCreated { id, .. }
            | Self::EpisodeCompleted { id, .. }
            | Self::EpisodeGarbageCollected { id, .. }
            | Self::PatternExtracted { id, .. } => id.clone(),
            Self::TaskStarted { task_id, .. }
            | Self::TaskCompleted { task_id, .. }
            | Self::RewardScored { task_id, .. } => task_id.to_string(),
            Self::ReflectionUpdated { episode_id, .. } | Self::EpisodeStored { episode_id, .. } => {
                episode_id.to_string()
            }
            Self::SkillEvolved { skill_name, .. } => skill_name.clone(),
        }
    }
}

/// Trait for emitting memory events.
///
/// Pluggable interface for event emission, allowing standardized formats
/// like CloudEvents to be used for interoperability.
#[async_trait]
pub trait EventEmitter: Send + Sync {
    /// Emit a memory event.
    async fn emit(&self, event: MemoryEvent);

    /// Check if the emitter is enabled.
    fn is_enabled(&self) -> bool {
        true
    }
}

/// A zero-cost default event emitter that does nothing.
pub struct NullEmitter;

#[async_trait]
impl EventEmitter for NullEmitter {
    async fn emit(&self, _event: MemoryEvent) {
        // No-op
    }

    fn is_enabled(&self) -> bool {
        false
    }
}

/// Default capacity for the broadcast channel.
/// Allows up to 1024 events to be buffered before slow receivers miss events.
pub const DEFAULT_EVENT_CHANNEL_CAPACITY: usize = 1024;

/// Get the current Unix timestamp in seconds.
#[must_use]
pub fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_event_timestamp() {
        let event = MemoryEvent::EpisodeCreated {
            id: "test-id".to_string(),
            task: "test task".to_string(),
            timestamp: 12345,
        };
        assert_eq!(event.timestamp(), 12345);
    }

    #[test]
    fn test_memory_event_entity_id() {
        let event = MemoryEvent::EpisodeCompleted {
            id: "episode-123".to_string(),
            reward: 0.85,
            timestamp: 12345,
        };
        assert_eq!(event.entity_id(), "episode-123");
    }

    #[test]
    fn test_memory_event_serialization() {
        let event = MemoryEvent::EpisodeCreated {
            id: "test-id".to_string(),
            task: "test task".to_string(),
            timestamp: 12345,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: MemoryEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.entity_id(), deserialized.entity_id());
    }
}
