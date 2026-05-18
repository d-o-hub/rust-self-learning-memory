use super::*;
use crate::memory::playbook::PlaybookStep;
use crate::types::TaskContext;
use uuid::Uuid;

#[test]
fn test_procedural_memory_creation() {
    let context = TaskContext::default();
    let steps = vec![
        PlaybookStep::new(1, "Step 1".to_string()),
        PlaybookStep::new(2, "Step 2".to_string()),
    ];
    let procedural = ProceduralMemory::new(
        "Test Skill".to_string(),
        "Description".to_string(),
        context,
        steps,
    );

    assert_eq!(procedural.name, "Test Skill");
    assert_eq!(procedural.steps.len(), 2);
    assert_eq!(procedural.effectiveness.times_retrieved, 0);
}

#[test]
fn test_relevance() {
    let context = TaskContext {
        domain: "rust".to_string(),
        tags: vec!["async".to_string()],
        ..TaskContext::default()
    };

    let procedural = ProceduralMemory::new(
        "Rust Async".to_string(),
        "How to do async in rust".to_string(),
        context,
        vec![],
    );

    let mut query_context = TaskContext {
        domain: "rust".to_string(),
        ..TaskContext::default()
    };
    assert!(procedural.is_relevant_to(&query_context));

    query_context.domain = "python".to_string();
    assert!(!procedural.is_relevant_to(&query_context));

    query_context.tags = vec!["async".to_string()];
    assert!(procedural.is_relevant_to(&query_context));
}

#[test]
fn test_effectiveness_recording() {
    let mut procedural = ProceduralMemory::new(
        "Test".to_string(),
        "Test".to_string(),
        TaskContext::default(),
        vec![],
    );

    procedural.record_retrieval();
    assert_eq!(procedural.effectiveness.times_retrieved, 1);

    procedural.record_application(true, 0.5);
    assert_eq!(procedural.effectiveness.times_applied, 1);
    assert_eq!(procedural.effectiveness.success_when_applied, 1);
    assert_eq!(procedural.effectiveness.avg_reward_delta, 0.5);
}

#[test]
fn test_sources() {
    let mut procedural = ProceduralMemory::new(
        "Test".to_string(),
        "Test".to_string(),
        TaskContext::default(),
        vec![],
    );

    let ep_id = Uuid::new_v4();
    procedural.add_source_episode(ep_id);
    procedural.add_source_episode(ep_id); // Duplicate
    assert_eq!(procedural.source_episodes.len(), 1);

    let pat_id = Uuid::new_v4();
    procedural.add_source_pattern(pat_id);
    assert_eq!(procedural.source_patterns.len(), 1);
}
