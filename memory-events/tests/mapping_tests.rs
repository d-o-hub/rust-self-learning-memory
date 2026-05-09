use cloudevents::AttributesReader;
use do_memory_core::types::event::{MemoryEvent, unix_now_secs, EventEmitter};
use do_memory_events::{CloudEventEmitter, to_cloud_event};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[tokio::test]
async fn test_cloud_event_mapping_integration() {
    let task_id = Uuid::new_v4().to_string();
    let event = MemoryEvent::RewardScored {
        task_id: task_id.clone(),
        score: 0.88,
        reason: "test mapping".to_string(),
        timestamp: unix_now_secs(),
    };

    let ce = to_cloud_event(&event, "test-source").expect("valid mapping");
    assert_eq!(ce.ty(), "dev.d-o-hub.memory.reward.scored");
    assert_eq!(ce.source().to_string(), "test-source");
    let data = match ce.data() {
        Some(cloudevents::Data::Json(v)) => v,
        _ => panic!("Expected JSON data"),
    };
    assert_eq!(data["task_id"], task_id);
}

#[tokio::test]
async fn test_cloud_event_emitter_integration() {
    let received_events = Arc::new(Mutex::new(Vec::new()));
    let handler_received = received_events.clone();
    let emitter = CloudEventEmitter::new("memory-core", move |event| {
        handler_received.lock().unwrap().push(event);
        Ok(())
    });
    let task_id = Uuid::new_v4().to_string();
    let event = MemoryEvent::TaskStarted {
        task_id: task_id.clone(),
        agent_id: "integration-agent".to_string(),
        metadata: serde_json::json!({"test": true}),
        timestamp: unix_now_secs(),
    };
    let _ = emitter.emit(event).await;
    let events = received_events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].ty(), "dev.d-o-hub.memory.task.started");
}
