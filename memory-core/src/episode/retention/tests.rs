//! Tests for episode retention module

use super::*;
use crate::episode::Episode;
use crate::types::{TaskContext, TaskType};

fn create_test_episode() -> Episode {
    Episode::new(
        "test task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    )
}

#[test]
fn test_default_policy() {
    let policy = EpisodeRetentionPolicy::default();
    assert!(policy.validate().is_ok());
    assert_eq!(policy.max_age_days, DEFAULT_MAX_AGE_DAYS);
    assert_eq!(policy.max_episodes, DEFAULT_MAX_EPISODES);
}

#[test]
fn test_storage_limited_policy() {
    let policy = EpisodeRetentionPolicy::storage_limited();
    assert!(policy.validate().is_ok());
    assert!(policy.max_age_days < DEFAULT_MAX_AGE_DAYS);
    assert!(policy.max_episodes < DEFAULT_MAX_EPISODES);
}

#[test]
fn test_archival_policy() {
    let policy = EpisodeRetentionPolicy::archival();
    assert!(policy.validate().is_ok());
    assert_eq!(policy.criterion, RetentionCriterion::KeepAll);
}

#[test]
fn test_aggressive_policy() {
    let policy = EpisodeRetentionPolicy::aggressive();
    assert!(policy.validate().is_ok());
    assert!(policy.max_age_days <= 7);
    assert!(!policy.keep_pattern_sources);
}

#[test]
fn test_should_retain_keep_all() {
    let policy = EpisodeRetentionPolicy::archival();
    let episode = create_test_episode();
    assert!(policy.should_retain(&episode, Utc::now()));
}

#[test]
fn test_should_retain_age_based() {
    let policy = EpisodeRetentionPolicy::default()
        .with_criterion(RetentionCriterion::AgeBased)
        .with_max_age_days(30);

    // Recent episode - should keep
    let recent = create_test_episode();
    assert!(policy.should_retain(&recent, Utc::now()));

    // Old episode - would require modifying episode start_time
    // For now, just verify policy logic works with current episode
}

#[test]
fn test_should_retain_with_patterns() {
    let policy = EpisodeRetentionPolicy::default().with_criterion(RetentionCriterion::Unreferenced);

    let mut episode = create_test_episode();
    episode.patterns.push(uuid::Uuid::new_v4());

    // Episode with patterns should be kept
    assert!(policy.should_retain(&episode, Utc::now()));
}

#[test]
fn test_builder_methods() {
    let policy = EpisodeRetentionPolicy::new()
        .with_max_age_days(60)
        .with_min_reward_threshold(0.5)
        .with_max_episodes(5000)
        .with_cleanup_batch_size(50)
        .with_dry_run(true);

    assert_eq!(policy.max_age_days, 60);
    assert_eq!(policy.min_reward_threshold, 0.5);
    assert_eq!(policy.max_episodes, 5000);
    assert_eq!(policy.cleanup_batch_size, 50);
    assert!(policy.dry_run);
}

#[test]
fn test_invalid_policy() {
    let invalid_age = EpisodeRetentionPolicy::default().with_max_age_days(-1);
    assert!(invalid_age.validate().is_err());

    // Note: with_min_reward_threshold clamps values to 0.0-1.0, so validation passes
    // This tests the validate() function directly with an unclamped value
    let invalid_threshold = EpisodeRetentionPolicy {
        min_reward_threshold: 1.5,
        ..Default::default()
    };
    assert!(invalid_threshold.validate().is_err());
}

#[test]
fn test_cleanup_result() {
    let mut result = CleanupResult::new();
    result.evaluated = 10;
    result.add_deleted(uuid::Uuid::new_v4());
    result.add_deleted(uuid::Uuid::new_v4());
    result.add_kept(uuid::Uuid::new_v4());

    assert_eq!(result.deleted, 2);
    assert_eq!(result.kept, 1);
    assert_eq!(result.deleted_ids.len(), 2);
    assert!(!result.has_errors());
    assert_eq!(result.success_rate(), 1.0);
}

#[test]
fn test_cleanup_result_with_errors() {
    let mut result = CleanupResult::new();
    result.evaluated = 10;
    result.add_error("Failed to delete episode".to_string());

    assert!(result.has_errors());
    assert!(result.success_rate() < 1.0);
}

#[test]
fn test_plan_cleanup_enforces_max_episodes_and_batch_size() {
    let mut ep1 = create_test_episode();
    ep1.patterns.push(uuid::Uuid::new_v4());
    let mut ep2 = create_test_episode();
    ep2.patterns.push(uuid::Uuid::new_v4());
    let mut ep3 = create_test_episode();
    ep3.patterns.push(uuid::Uuid::new_v4());

    let episodes = vec![ep1, ep2, ep3];
    let policy = EpisodeRetentionPolicy::default()
        .with_criterion(RetentionCriterion::Unreferenced)
        .with_max_episodes(1)
        .with_cleanup_batch_size(1);

    assert_eq!(policy.count_candidates(&episodes, Utc::now()).unwrap(), 2);

    let (planned, to_delete) = policy.plan_cleanup(&episodes, Utc::now()).unwrap();
    assert_eq!(planned.evaluated, 3);
    assert_eq!(to_delete.len(), 1);
    assert_eq!(planned.kept, 2);

    let (dry, dry_to_delete) = policy
        .with_dry_run(true)
        .plan_cleanup(&episodes, Utc::now())
        .unwrap();
    assert!(
        dry_to_delete.is_empty(),
        "dry run must not return execution IDs"
    );
    assert_eq!(dry.deleted, 1);
    assert_eq!(dry.kept, 2);
}
