//! Tests for the filters module

use super::*;
use crate::Episode;
use crate::types::TaskOutcome;
use crate::types::TaskType;
use crate::types::{ComplexityLevel, RewardScore, TaskContext};

fn create_test_episode(
    task_type: TaskType,
    domain: &str,
    tags: Vec<String>,
    completed: bool,
    success: bool,
    reward: Option<f32>,
) -> Episode {
    let mut episode = Episode::new(
        "Test task".to_string(),
        TaskContext {
            language: None,
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: domain.to_string(),
            tags,
        },
        task_type,
    );

    if completed {
        let outcome = if success {
            TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            }
        } else {
            TaskOutcome::Failure {
                reason: "Failed".to_string(),
                error_details: None,
            }
        };
        episode.complete(outcome);
    }

    if let Some(r) = reward {
        episode.reward = Some(RewardScore {
            total: r,
            base: 1.0,
            efficiency: 1.0,
            complexity_bonus: 1.0,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
        });
    }

    episode
}

#[test]
fn test_filter_by_tags_any() {
    let ep1 = create_test_episode(
        TaskType::CodeGeneration,
        "web",
        vec!["async".to_string(), "http".to_string()],
        true,
        true,
        None,
    );

    let ep2 = create_test_episode(
        TaskType::Debugging,
        "web",
        vec!["sync".to_string()],
        true,
        true,
        None,
    );

    let filter = EpisodeFilter::builder()
        .with_any_tags(vec!["async".to_string()])
        .build();

    assert!(filter.matches(&ep1));
    assert!(!filter.matches(&ep2));
}

#[test]
fn test_filter_by_tags_all() {
    let ep1 = create_test_episode(
        TaskType::CodeGeneration,
        "web",
        vec!["async".to_string(), "http".to_string()],
        true,
        true,
        None,
    );

    let ep2 = create_test_episode(
        TaskType::Debugging,
        "web",
        vec!["async".to_string()],
        true,
        true,
        None,
    );

    let filter = EpisodeFilter::builder()
        .with_all_tags(vec!["async".to_string(), "http".to_string()])
        .build();

    assert!(filter.matches(&ep1));
    assert!(!filter.matches(&ep2));
}

#[test]
fn test_filter_by_task_type() {
    let ep1 = create_test_episode(TaskType::CodeGeneration, "web", vec![], true, true, None);

    let ep2 = create_test_episode(TaskType::Debugging, "web", vec![], true, true, None);

    let filter = EpisodeFilter::builder()
        .task_types(vec![TaskType::CodeGeneration])
        .build();

    assert!(filter.matches(&ep1));
    assert!(!filter.matches(&ep2));
}

#[test]
fn test_filter_by_completion() {
    let ep_complete =
        create_test_episode(TaskType::CodeGeneration, "web", vec![], true, true, None);

    let ep_incomplete =
        create_test_episode(TaskType::CodeGeneration, "web", vec![], false, false, None);

    let filter = EpisodeFilter::builder().completed_only(true).build();

    assert!(filter.matches(&ep_complete));
    assert!(!filter.matches(&ep_incomplete));
}

#[test]
fn test_filter_by_success() {
    let ep_success = create_test_episode(TaskType::CodeGeneration, "web", vec![], true, true, None);

    let ep_failure =
        create_test_episode(TaskType::CodeGeneration, "web", vec![], true, false, None);

    let filter = EpisodeFilter::builder().success_only(true).build();

    assert!(filter.matches(&ep_success));
    assert!(!filter.matches(&ep_failure));
}

#[test]
fn test_filter_by_reward() {
    let ep_high = create_test_episode(
        TaskType::CodeGeneration,
        "web",
        vec![],
        true,
        true,
        Some(1.5),
    );

    let ep_low = create_test_episode(
        TaskType::CodeGeneration,
        "web",
        vec![],
        true,
        true,
        Some(0.5),
    );

    let filter = EpisodeFilter::builder().min_reward(1.0).build();

    assert!(filter.matches(&ep_high));
    assert!(!filter.matches(&ep_low));
}

#[test]
fn test_filter_combined() {
    let ep = create_test_episode(
        TaskType::CodeGeneration,
        "web-api",
        vec!["async".to_string(), "rest".to_string()],
        true,
        true,
        Some(1.5),
    );

    let filter = EpisodeFilter::builder()
        .task_types(vec![TaskType::CodeGeneration])
        .domains(vec!["web-api".to_string()])
        .with_any_tags(vec!["async".to_string()])
        .completed_only(true)
        .success_only(true)
        .min_reward(1.0)
        .build();

    assert!(filter.matches(&ep));
}
