//! Memory event types for broadcast channel notification.
//!
//! Events are emitted when significant lifecycle operations occur in the memory system,
//! allowing external subscribers to react to changes without coupling to internal state.

use serde::{Deserialize, Serialize};

/// Events emitted by the memory system for lifecycle notifications.
///
/// Uses `tokio::sync::broadcast` channel for efficient fan-out to multiple subscribers.
/// Subscribers can use `memory.subscribe()` to receive events.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl MemoryEvent {
    /// Get the timestamp of the event.
    #[must_use]
    pub fn timestamp(&self) -> u64 {
        match self {
            Self::EpisodeCreated { timestamp, .. }
            | Self::EpisodeCompleted { timestamp, .. }
            | Self::EpisodeGarbageCollected { timestamp, .. }
            | Self::PatternExtracted { timestamp, .. } => *timestamp,
        }
    }

    /// Get the entity ID associated with the event.
    #[must_use]
    pub fn entity_id(&self) -> &str {
        match self {
            Self::EpisodeCreated { id, .. }
            | Self::EpisodeCompleted { id, .. }
            | Self::EpisodeGarbageCollected { id, .. } => id,
            Self::PatternExtracted { id, .. } => id,
        }
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
