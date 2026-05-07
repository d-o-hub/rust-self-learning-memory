//! CloudEvents implementation for standardized agent lifecycle event emission.
//!
//! This crate provides the mapping between internal memory system events
//! and the CNCF CloudEvents v1.0.2 specification.

use async_trait::async_trait;
use chrono::Utc;
use cloudevents::{Event, EventBuilder, EventBuilderV10};
pub use do_memory_core::types::event::MemoryEvent;
use do_memory_core::types::event::EventEmitter;
use uuid::Uuid;

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
        MemoryEvent::EpisodeCreated { id, task, .. } => (
            "dev.d-o-hub.memory.episode.created",
            serde_json::json!({
                "id": id,
                "task": task
            }),
        ),
        MemoryEvent::EpisodeCompleted { id, reward, .. } => (
            "dev.d-o-hub.memory.episode.completed",
            serde_json::json!({
                "id": id,
                "reward": reward
            }),
        ),
        MemoryEvent::EpisodeGarbageCollected { id, reason, .. } => (
            "dev.d-o-hub.memory.episode.collected",
            serde_json::json!({
                "id": id,
                "reason": reason
            }),
        ),
        MemoryEvent::PatternExtracted {
            id,
            source_episodes,
            ..
        } => (
            "dev.d-o-hub.memory.pattern.extracted",
            serde_json::json!({
                "id": id,
                "source_episodes": source_episodes
            }),
        ),
        MemoryEvent::TaskStarted {
            task_id,
            agent_id,
            metadata,
            ..
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
            ..
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
            ..
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
            ..
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
            ..
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
            ..
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

/// An event emitter that converts internal events to CloudEvents
/// and passes them to a nested emitter or callback.
pub struct CloudEventEmitter<F>
where
    F: Fn(Event) + Send + Sync,
{
    source: String,
    handler: F,
}

impl<F> CloudEventEmitter<F>
where
    F: Fn(Event) + Send + Sync,
{
    /// Create a new CloudEventEmitter with a source and handler function.
    pub fn new(source: impl Into<String>, handler: F) -> Self {
        Self {
            source: source.into(),
            handler,
        }
    }
}

#[async_trait]
impl<F> EventEmitter for CloudEventEmitter<F>
where
    F: Fn(Event) + Send + Sync,
{
    async fn emit(&self, event: MemoryEvent) {
        let ce = to_cloud_event(&event, &self.source);
        (self.handler)(ce);
    }
}

/// An event emitter that sends CloudEvents to an HTTP endpoint.
#[cfg(feature = "events-http")]
pub struct HttpEventEmitter {
    source: String,
    client: reqwest::Client,
    url: String,
}

#[cfg(feature = "events-http")]
impl HttpEventEmitter {
    /// Create a new HttpEventEmitter with a source and target URL.
    pub fn new(source: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            client: reqwest::Client::new(),
            url: url.into(),
        }
    }
}

#[cfg(feature = "events-http")]
#[async_trait]
impl EventEmitter for HttpEventEmitter {
    async fn emit(&self, event: MemoryEvent) {
        let ce = to_cloud_event(&event, &self.source);

        // Best-effort emission
        let _ = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/cloudevents+json")
            .json(&ce)
            .send()
            .await;
    }
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
            timestamp: 12345,
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
            timestamp: 12345,
        };

        let ce = to_cloud_event(&event, "memory-core");
        assert_eq!(ce.ty(), "dev.d-o-hub.memory.reward.scored");

        let data = get_data_json(ce.data().unwrap());
        assert_eq!(data["score"], 0.95);
        assert_eq!(data["reason"], "efficient execution");
    }
}
