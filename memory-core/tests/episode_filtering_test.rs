//! Integration tests for episode filtering
//!
//! Tests the complete filtering workflow with real memory system operations.

use chrono::Utc;
use memory_core::{
    EpisodeFilter, ExecutionResult, ExecutionStep, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};

#[tokio::test]
async fn test_filter_by_tags_integration() {
    let memory = SelfLearningMemory::new();

    // Create episodes with different tags
    let ctx1 = TaskContext {
        domain: "web-api".to_string(),
        tags: vec!["async".to_string(), "http".to_string()],
        ..Default::default()
    };

    let ctx2 = TaskContext {
        domain: "cli".to_string(),
        tags: vec!["parsing".to_string()],
        ..Default::default()
    };

    let ep1 = memory
        .start_episode(
            "Build HTTP client".to_string(),
            ctx1,
            TaskType::CodeGeneration,
        )
        .await;

    let ep2 = memory
        .start_episode("Parse CLI args".to_string(), ctx2, TaskType::CodeGeneration)
        .await;

    // Add execution steps to pass quality validation (need multiple steps)
    for i in 1..=3 {
        let mut step = ExecutionStep::new(i, format!("agent_{}", i), format!("Step {}", i));
        step.result = Some(ExecutionResult::Success {
            output: format!("Step {} completed", i),
        });
        memory.log_step(ep1, step).await;
    }

    for i in 1..=3 {
        let mut step = ExecutionStep::new(i, format!("agent_{}", i), format!("Step {}", i));
        step.result = Some(ExecutionResult::Success {
            output: format!("Step {} completed", i),
        });
        memory.log_step(ep2, step).await;
    }

    // Complete the episodes
    memory
        .complete_episode(
            ep1,
            TaskOutcome::Success {
                verdict: "HTTP client built".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    memory
        .complete_episode(
            ep2,
            TaskOutcome::Success {
                verdict: "CLI parser built".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Filter by tags
    let filter = EpisodeFilter::builder()
        .with_any_tags(vec!["async".to_string()])
        .build();

    let filtered = memory
        .list_episodes_filtered(filter, None, None)
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].episode_id, ep1);
}

#[tokio::test]
async fn test_filter_by_success_only() {
    let memory = SelfLearningMemory::new();

    let ctx = TaskContext::default();

    // Create successful episode
    let ep_success = memory
        .start_episode("Task 1".to_string(), ctx.clone(), TaskType::Testing)
        .await;

    for i in 1..=3 {
        let mut step = ExecutionStep::new(i, format!("tester_{}", i), format!("Test {}", i));
        step.result = Some(ExecutionResult::Success {
            output: format!("Test {} passed", i),
        });
        memory.log_step(ep_success, step).await;
    }

    memory
        .complete_episode(
            ep_success,
            TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Create failed episode
    let ep_failure = memory
        .start_episode("Task 2".to_string(), ctx, TaskType::Testing)
        .await;

    for i in 1..=3 {
        let mut step = ExecutionStep::new(i, format!("tester_{}", i), format!("Test {}", i));
        step.result = Some(ExecutionResult::Success {
            output: format!("Test {} passed", i),
        });
        memory.log_step(ep_failure, step).await;
    }

    memory
        .complete_episode(
            ep_failure,
            TaskOutcome::Failure {
                reason: "Failed".to_string(),
                error_details: None,
            },
        )
        .await
        .unwrap();

    // Filter for success only
    let filter = EpisodeFilter::builder().success_only(true).build();

    let filtered = memory
        .list_episodes_filtered(filter, None, None)
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].episode_id, ep_success);
}

#[tokio::test]
async fn test_filter_by_task_type_and_domain() {
    let memory = SelfLearningMemory::new();

    // Create episodes with different task types and domains
    let ctx_web = TaskContext {
        domain: "web-api".to_string(),
        ..Default::default()
    };

    let ctx_cli = TaskContext {
        domain: "cli".to_string(),
        ..Default::default()
    };

    let ep1 = memory
        .start_episode("Web task".to_string(), ctx_web, TaskType::CodeGeneration)
        .await;

    let _ep2 = memory
        .start_episode("CLI task".to_string(), ctx_cli, TaskType::Debugging)
        .await;

    // Filter by task type and domain
    let filter = EpisodeFilter::builder()
        .task_types(vec![TaskType::CodeGeneration])
        .domains(vec!["web-api".to_string()])
        .build();

    let filtered = memory
        .list_episodes_filtered(filter, None, None)
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].episode_id, ep1);
}

#[tokio::test]
async fn test_filter_with_date_range() {
    let memory = SelfLearningMemory::new();

    let now = Utc::now();
    let ctx = TaskContext::default();

    // Create episode
    let ep1 = memory
        .start_episode("Recent task".to_string(), ctx, TaskType::Testing)
        .await;

    // Filter for episodes in the last hour
    let filter = EpisodeFilter::builder()
        .date_from(now - chrono::Duration::hours(1))
        .date_to(now + chrono::Duration::hours(1))
        .build();

    let filtered = memory
        .list_episodes_filtered(filter, None, None)
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].episode_id, ep1);

    // Filter for episodes from the future (should be empty)
    let future_filter = EpisodeFilter::builder()
        .date_from(now + chrono::Duration::days(1))
        .build();

    let future_filtered = memory
        .list_episodes_filtered(future_filter, None, None)
        .await
        .unwrap();

    assert_eq!(future_filtered.len(), 0);
}

#[tokio::test]
async fn test_filter_exclude_archived() {
    let memory = SelfLearningMemory::new();

    let ctx = TaskContext::default();

    // Create two episodes
    let ep1 = memory
        .start_episode("Task 1".to_string(), ctx.clone(), TaskType::Testing)
        .await;

    let ep2 = memory
        .start_episode("Task 2".to_string(), ctx, TaskType::Testing)
        .await;

    // Archive one episode
    memory.archive_episode(ep1).await.unwrap();

    // Filter excluding archived
    let filter = EpisodeFilter::builder().exclude_archived(true).build();

    let filtered = memory
        .list_episodes_filtered(filter, None, None)
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].episode_id, ep2);

    // Filter for archived only
    let archived_filter = EpisodeFilter::builder().archived_only(true).build();

    let archived = memory
        .list_episodes_filtered(archived_filter, None, None)
        .await
        .unwrap();

    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].episode_id, ep1);
}

#[tokio::test]
async fn test_filter_complex_query() {
    let memory = SelfLearningMemory::new();

    // Create episodes with various properties
    let ctx1 = TaskContext {
        domain: "web-api".to_string(),
        tags: vec!["async".to_string(), "rest".to_string()],
        ..Default::default()
    };

    let ctx2 = TaskContext {
        domain: "web-api".to_string(),
        tags: vec!["graphql".to_string()],
        ..Default::default()
    };

    let ctx3 = TaskContext {
        domain: "cli".to_string(),
        tags: vec!["async".to_string()],
        ..Default::default()
    };

    let ep1 = memory
        .start_episode("REST API".to_string(), ctx1, TaskType::CodeGeneration)
        .await;

    let _ep2 = memory
        .start_episode("GraphQL API".to_string(), ctx2, TaskType::CodeGeneration)
        .await;

    let _ep3 = memory
        .start_episode("CLI tool".to_string(), ctx3, TaskType::Debugging)
        .await;

    // Add steps to ep1
    for i in 1..=3 {
        let mut step = ExecutionStep::new(i, format!("builder_{}", i), format!("Build step {}", i));
        step.result = Some(ExecutionResult::Success {
            output: format!("Step {} completed", i),
        });
        memory.log_step(ep1, step).await;
    }

    // Complete ep1 successfully
    memory
        .complete_episode(
            ep1,
            TaskOutcome::Success {
                verdict: "API built".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Complex filter: web-api domain, has async tag, code generation, successful
    let filter = EpisodeFilter::builder()
        .domains(vec!["web-api".to_string()])
        .with_any_tags(vec!["async".to_string()])
        .task_types(vec![TaskType::CodeGeneration])
        .success_only(true)
        .completed_only(true)
        .build();

    let filtered = memory
        .list_episodes_filtered(filter, None, None)
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].episode_id, ep1);
}

#[tokio::test]
async fn test_filter_with_pagination() {
    let memory = SelfLearningMemory::new();

    let ctx = TaskContext::default();

    // Create 5 episodes
    for i in 1..=5 {
        memory
            .start_episode(format!("Task {}", i), ctx.clone(), TaskType::Testing)
            .await;
    }

    // Get first 2 episodes
    let filter = EpisodeFilter::new();
    let page1 = memory
        .list_episodes_filtered(filter.clone(), Some(2), None)
        .await
        .unwrap();

    assert_eq!(page1.len(), 2);

    // Get next 2 episodes (offset 2)
    let page2 = memory
        .list_episodes_filtered(filter, Some(2), Some(2))
        .await
        .unwrap();

    assert_eq!(page2.len(), 2);

    // Episodes should be different
    assert_ne!(page1[0].episode_id, page2[0].episode_id);
}
