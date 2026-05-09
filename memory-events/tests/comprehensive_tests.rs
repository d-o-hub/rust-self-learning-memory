use cloudevents::AttributesReader;
use do_memory_core::types::event::{EventEmitter, MemoryEvent, unix_now_secs};
use do_memory_events::{CloudEventEmitter, to_cloud_event};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[tokio::test]
async fn test_all_variants_mapping() {
    let source = "test-source";
    let now = unix_now_secs();

    // EpisodeCreated
    let event = MemoryEvent::EpisodeCreated {
        id: "ep-1".to_string(),
        task: "task-1".to_string(),
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.episode.created");

    // EpisodeCompleted
    let event = MemoryEvent::EpisodeCompleted {
        id: "ep-1".to_string(),
        reward: 1.5,
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.episode.completed");

    // EpisodeGarbageCollected
    let event = MemoryEvent::EpisodeGarbageCollected {
        id: "ep-1".to_string(),
        reason: "ttl".to_string(),
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.episode.collected");

    // PatternExtracted
    let event = MemoryEvent::PatternExtracted {
        id: "pat-1".to_string(),
        source_episodes: vec!["ep-1".to_string()],
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.pattern.extracted");

    // TaskStarted
    let event = MemoryEvent::TaskStarted {
        task_id: "task-1".to_string(),
        agent_id: "agent-1".to_string(),
        metadata: serde_json::json!({}),
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.task.started");

    // TaskCompleted
    let event = MemoryEvent::TaskCompleted {
        task_id: "task-1".to_string(),
        duration_ms: 100,
        success: true,
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.task.completed");

    // RewardScored
    let event = MemoryEvent::RewardScored {
        task_id: "task-1".to_string(),
        score: 0.5,
        reason: "good".to_string(),
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.reward.scored");

    // ReflectionUpdated
    let event = MemoryEvent::ReflectionUpdated {
        episode_id: "ep-1".to_string(),
        reflection_type: "type".to_string(),
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.reflection.updated");

    // SkillEvolved
    let event = MemoryEvent::SkillEvolved {
        skill_name: "skill".to_string(),
        from_version: 1,
        to_version: 2,
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.skill.evolved");

    // EpisodeStored
    let event = MemoryEvent::EpisodeStored {
        episode_id: "ep-1".to_string(),
        backend: "turso".to_string(),
        timestamp: now,
    };
    let ce = to_cloud_event(&event, source).unwrap();
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.episode.stored");
}

#[cfg(feature = "events-http")]
#[tokio::test]
async fn test_http_emitter_failure_handling() {
    use do_memory_events::HttpEventEmitter;
    let emitter = HttpEventEmitter::new("test", "http://invalid-url-that-does-not-exist.local");
    let event = MemoryEvent::EpisodeCreated {
        id: "id".to_string(),
        task: "task".to_string(),
        timestamp: 0,
    };
    let result = emitter.emit(event).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("HTTP emission failed"));
}
