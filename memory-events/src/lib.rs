//! CloudEvents implementation for standardized agent lifecycle event emission.

#![allow(clippy::uninlined_format_args)]

use async_trait::async_trait;
use chrono::Utc;
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use do_memory_core::types::event::{EmitResult, EventEmitter, MemoryEvent};
#[cfg(feature = "events-http")]
use std::time::Duration;
use uuid::Uuid;

pub fn to_cloud_event(event: &MemoryEvent, source: &str) -> Result<Event, String> {
    let (ce_type, data) = match event {
        MemoryEvent::EpisodeCreated { id, task, .. } => (
            "dev.d-o-hub.memory.episode.created",
            serde_json::json!({ "id": id, "task": task }),
        ),
        MemoryEvent::EpisodeCompleted { id, reward, .. } => (
            "dev.d-o-hub.memory.episode.completed",
            serde_json::json!({ "id": id, "reward": reward }),
        ),
        MemoryEvent::EpisodeGarbageCollected { id, reason, .. } => (
            "dev.d-o-hub.memory.episode.collected",
            serde_json::json!({ "id": id, "reason": reason }),
        ),
        MemoryEvent::PatternExtracted {
            id,
            source_episodes,
            ..
        } => (
            "dev.d-o-hub.memory.pattern.extracted",
            serde_json::json!({ "id": id, "source_episodes": source_episodes }),
        ),
        MemoryEvent::TaskStarted {
            task_id,
            agent_id,
            metadata,
            ..
        } => (
            "dev.d-o-hub.memory.task.started",
            serde_json::json!({ "task_id": task_id, "agent_id": agent_id, "metadata": metadata }),
        ),
        MemoryEvent::TaskCompleted {
            task_id,
            duration_ms,
            success,
            ..
        } => (
            "dev.d-o-hub.memory.task.completed",
            serde_json::json!({ "task_id": task_id, "duration_ms": duration_ms, "success": success }),
        ),
        MemoryEvent::RewardScored {
            task_id,
            score,
            reason,
            ..
        } => (
            "dev.d-o-hub.memory.reward.scored",
            serde_json::json!({ "task_id": task_id, "score": score, "reason": reason }),
        ),
        MemoryEvent::ReflectionUpdated {
            episode_id,
            reflection_type,
            ..
        } => (
            "dev.d-o-hub.memory.reflection.updated",
            serde_json::json!({ "episode_id": episode_id, "reflection_type": reflection_type }),
        ),
        MemoryEvent::SkillEvolved {
            skill_name,
            from_version,
            to_version,
            ..
        } => (
            "dev.d-o-hub.memory.skill.evolved",
            serde_json::json!({ "skill_name": skill_name, "from_version": from_version, "to_version": to_version }),
        ),
        MemoryEvent::EpisodeStored {
            episode_id,
            backend,
            ..
        } => (
            "dev.d-o-hub.memory.episode.stored",
            serde_json::json!({ "episode_id": episode_id, "backend": backend }),
        ),
    };

    EventBuilderV10::new()
        .id(Uuid::new_v4().to_string())
        .source(source)
        .ty(ce_type)
        .time(Utc::now())
        .data("application/json", data)
        .build()
        .map_err(|e| format!("Failed to build CloudEvent: {}", e))
}

pub struct CloudEventEmitter<F>
where
    F: Fn(Event) -> EmitResult + Send + Sync,
{
    source: String,
    handler: F,
}

impl<F> CloudEventEmitter<F>
where
    F: Fn(Event) -> EmitResult + Send + Sync,
{
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
    F: Fn(Event) -> EmitResult + Send + Sync,
{
    async fn emit(&self, event: MemoryEvent) -> EmitResult {
        let ce = to_cloud_event(&event, &self.source)?;
        (self.handler)(ce)
    }
}

#[cfg(feature = "events-http")]
pub struct HttpEventEmitter {
    source: String,
    client: reqwest::Client,
    url: String,
}

#[cfg(feature = "events-http")]
impl HttpEventEmitter {
    pub fn new(source: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            url: url.into(),
        }
    }
}

#[cfg(feature = "events-http")]
#[async_trait]
impl EventEmitter for HttpEventEmitter {
    async fn emit(&self, event: MemoryEvent) -> EmitResult {
        let ce = to_cloud_event(&event, &self.source)?;
        let body = serde_json::to_string(&ce)
            .map_err(|e| format!("Failed to serialize CloudEvent: {}", e))?;

        match self
            .client
            .post(&self.url)
            .header("Content-Type", "application/cloudevents+json")
            .body(body)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => Err(format!(
                "HTTP emission failed with status: {}",
                resp.status()
            )),
            Err(e) => Err(format!("HTTP emission failed: {}", e)),
        }
    }
}
