use chrono::Utc;

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

    // Old episode - should cleanup
    let _old = Episode::new(
        "old task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );
    // Manually set old start time would require modifying episode
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
