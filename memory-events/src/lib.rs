//! CloudEvents implementation for standardized agent lifecycle event emission.
//!
//! This crate provides the mapping between internal memory system events
//! and the CNCF CloudEvents v1.0.2 specification.

use chrono::Utc;
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Standardized memory event types for CloudEvents mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MemoryEvent {
    /// A task has started execution.
    TaskStarted {
        task_id: Uuid,
        agent_id: String,
        metadata: serde_json::Value,
    },
    /// A task has completed execution.
    TaskCompleted {
        task_id: Uuid,
        duration_ms: u64,
        success: bool,
    },
    /// a reward score has been calculated for a task.
    RewardScored {
        task_id: Uuid,
        score: f64,
        reason: String,
    },
    /// A reflection has been updated for an episode.
    ReflectionUpdated {
        episode_id: Uuid,
        reflection_type: String,
    },
    /// A skill has evolved or a new pattern has been promoted.
    SkillEvolved {
        skill_name: String,
        from_version: u32,
        to_version: u32,
    },
    /// An episode has been successfully stored in a backend.
    EpisodeStored { episode_id: Uuid, backend: String },
}

/// Maps a internal `MemoryEvent` to a `cloudevents::Event`.
///
/// # Arguments
///
/// * `event` - The memory event to map
/// * `source` - The source identifier (e.g., "memory-core", "memory-storage-turso")
///
/// # Returns
///
/// A constructed and validated `cloudevents::Event`
pub fn to_cloud_event(event: &MemoryEvent, source: &str) -> Event {
    let (ce_type, data) = match event {
        MemoryEvent::TaskStarted {
            task_id,
            agent_id,
            metadata,
        } => (
            "dev.d-o-hub.memory.task.started",
            serde_json::json!({
                "task_id": task_id,
                "agent_id": agent_id,
                "metadata": metadata
            }),
        ),
        MemoryEvent::TaskCompleted {
            task_id,
            duration_ms,
            success,
        } => (
            "dev.d-o-hub.memory.task.completed",
            serde_json::json!({
                "task_id": task_id,
                "duration_ms": duration_ms,
                "success": success
            }),
        ),
        MemoryEvent::RewardScored {
            task_id,
            score,
            reason,
        } => (
            "dev.d-o-hub.memory.reward.scored",
            serde_json::json!({
                "task_id": task_id,
                "score": score,
                "reason": reason
            }),
        ),
        MemoryEvent::ReflectionUpdated {
            episode_id,
            reflection_type,
        } => (
            "dev.d-o-hub.memory.reflection.updated",
            serde_json::json!({
                "episode_id": episode_id,
                "reflection_type": reflection_type
            }),
        ),
        MemoryEvent::SkillEvolved {
            skill_name,
            from_version,
            to_version,
        } => (
            "dev.d-o-hub.memory.skill.evolved",
            serde_json::json!({
                "skill_name": skill_name,
                "from_version": from_version,
                "to_version": to_version
            }),
        ),
        MemoryEvent::EpisodeStored {
            episode_id,
            backend,
        } => (
            "dev.d-o-hub.memory.episode.stored",
            serde_json::json!({
                "episode_id": episode_id,
                "backend": backend
            }),
        ),
    };

    EventBuilderV10::new()
        .id(Uuid::new_v4().to_string())
        .source(source)
        .ty(ce_type)
        .time(Utc::now())
        .data("application/json", data)
        .build()
        .expect("valid CloudEvent")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloudevents::{AttributesReader, Data};

    fn get_data_json(data: &Data) -> serde_json::Value {
        match data {
            Data::Json(v) => v.clone(),
            Data::Binary(v) => serde_json::from_slice(v).unwrap(),
            Data::String(v) => serde_json::from_str(v).unwrap(),
        }
    }

    #[test]
    fn test_to_cloud_event_task_started() {
        let task_id = Uuid::new_v4();
        let event = MemoryEvent::TaskStarted {
            task_id,
            agent_id: "test-agent".to_string(),
            metadata: serde_json::json!({ "foo": "bar" }),
        };

        let ce = to_cloud_event(&event, "memory-core");
        assert_eq!(ce.ty(), "dev.d-o-hub.memory.task.started");
        assert_eq!(ce.source().to_string(), "memory-core");

        let data = get_data_json(ce.data().unwrap());
        assert_eq!(data["task_id"], task_id.to_string());
        assert_eq!(data["agent_id"], "test-agent");
    }

    #[test]
    fn test_to_cloud_event_reward_scored() {
        let task_id = Uuid::new_v4();
        let event = MemoryEvent::RewardScored {
            task_id,
            score: 0.95,
            reason: "efficient execution".to_string(),
        };

        let ce = to_cloud_event(&event, "memory-core");
        assert_eq!(ce.ty(), "dev.d-o-hub.memory.reward.scored");

        let data = get_data_json(ce.data().unwrap());
        assert_eq!(data["score"], 0.95);
        assert_eq!(data["reason"], "efficient execution");
    }
}
