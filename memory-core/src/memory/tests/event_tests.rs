use crate::memory::SelfLearningMemory;
use crate::types::event::{EmitResult, EventEmitter, MemoryEvent};
use crate::types::{TaskContext, TaskOutcome, TaskType};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

struct TestEmitter {
    events: Arc<Mutex<Vec<MemoryEvent>>>,
}

#[async_trait]
impl EventEmitter for TestEmitter {
    async fn emit(&self, event: MemoryEvent) -> EmitResult {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

#[tokio::test]
async fn test_event_emission_lifecycle() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let emitter = Arc::new(TestEmitter {
        events: events.clone(),
    });

    // Low quality threshold for testing
    let mut config = crate::types::MemoryConfig::default();
    config.quality_threshold = 0.0;
    let memory = crate::memory::init::with_config(config)
        .with_event_emitter(Arc::clone(&emitter) as Arc<dyn EventEmitter>);

    let episode_id = memory
        .start_episode(
            "test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Give some time for async emission
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    {
        let evs = events.lock().unwrap();
        assert!(
            evs.iter()
                .any(|e| matches!(e, MemoryEvent::TaskStarted { .. }))
        );
    }

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Give some time for async emission
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    {
        let evs = events.lock().unwrap();
        assert!(
            evs.iter()
                .any(|e| matches!(e, MemoryEvent::RewardScored { .. }))
        );
        assert!(
            evs.iter()
                .any(|e| matches!(e, MemoryEvent::ReflectionUpdated { .. }))
        );
        assert!(
            evs.iter()
                .any(|e| matches!(e, MemoryEvent::TaskCompleted { .. }))
        );
    }
}
