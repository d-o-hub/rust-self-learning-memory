//! Tests for the `complete` episode command (split out to keep the source file under the LOC ceiling).

use super::*;
use crate::config::Config;
use crate::output::OutputFormat;
use do_memory_core::TaskOutcome as CoreTaskOutcome;
use do_memory_core::{MemoryConfig, SelfLearningMemory, TaskContext, TaskType};

fn test_memory() -> SelfLearningMemory {
    // Match CLI config: quality_threshold 0.0 so minimal episodes complete.
    let config = MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 1.0,
        enable_summarization: false,
        enable_embeddings: false,
        ..Default::default()
    };
    SelfLearningMemory::with_config(config)
}

async fn start_test_episode(memory: &SelfLearningMemory, task: &str) -> uuid::Uuid {
    memory
        .start_episode(task.to_string(), TaskContext::default(), TaskType::Testing)
        .await
}

#[test]
fn map_cli_outcome_success() {
    let mapped = map_cli_outcome(TaskOutcome::Success);
    assert!(matches!(mapped, CoreTaskOutcome::Success { .. }));
}

#[test]
fn map_cli_outcome_partial() {
    let mapped = map_cli_outcome(TaskOutcome::PartialSuccess);
    assert!(matches!(mapped, CoreTaskOutcome::PartialSuccess { .. }));
}

#[test]
fn map_cli_outcome_failure() {
    let mapped = map_cli_outcome(TaskOutcome::Failure);
    assert!(matches!(mapped, CoreTaskOutcome::Failure { .. }));
}

#[test]
fn outcome_kind_matches_success() {
    let actual = CoreTaskOutcome::Success {
        verdict: "ok".into(),
        artifacts: vec![],
    };
    assert!(outcome_kind_matches(TaskOutcome::Success, &actual));
    assert!(!outcome_kind_matches(TaskOutcome::Failure, &actual));
}

#[test]
fn outcome_kind_matches_failure() {
    let actual = CoreTaskOutcome::Failure {
        reason: "boom".into(),
        error_details: None,
    };
    assert!(outcome_kind_matches(TaskOutcome::Failure, &actual));
    assert!(!outcome_kind_matches(TaskOutcome::Success, &actual));
    assert!(!outcome_kind_matches(TaskOutcome::PartialSuccess, &actual));
}

#[test]
fn outcome_kind_matches_partial() {
    let actual = CoreTaskOutcome::PartialSuccess {
        verdict: "half".into(),
        completed: vec![],
        failed: vec![],
    };
    assert!(outcome_kind_matches(TaskOutcome::PartialSuccess, &actual));
    assert!(!outcome_kind_matches(TaskOutcome::Success, &actual));
}

#[test]
fn fail_maps_to_failure_outcome() {
    // episode fail reuses complete_episode with TaskOutcome::Failure
    let mapped = map_cli_outcome(TaskOutcome::Failure);
    assert!(matches!(
        mapped,
        CoreTaskOutcome::Failure {
            reason,
            error_details: Some(_)
        } if reason.contains("CLI")
    ));
}

// --- print_complete_success formats (default features = not turso) ---

#[test]
fn print_complete_success_json() {
    let result = print_complete_success(
        "ep-json",
        TaskOutcome::Success,
        "committed",
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn print_complete_success_yaml() {
    let result = print_complete_success(
        "ep-yaml",
        TaskOutcome::PartialSuccess,
        "committed",
        OutputFormat::Yaml,
    );
    assert!(result.is_ok());
}

#[test]
fn print_complete_success_human() {
    let result = print_complete_success(
        "ep-human",
        TaskOutcome::Failure,
        "local",
        OutputFormat::Human,
    );
    assert!(result.is_ok());
}

// --- async complete / fail paths with real SelfLearningMemory ---

#[tokio::test(flavor = "multi_thread")]
async fn complete_episode_success_happy_path() {
    let memory = test_memory();
    let config = Config::default();
    let episode_id = start_test_episode(&memory, "CLI complete success").await;

    let result = complete_episode(
        episode_id.to_string(),
        TaskOutcome::Success,
        &memory,
        &config,
        OutputFormat::Human,
        false,
        30,
    )
    .await;
    assert!(result.is_ok(), "{result:?}");

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(matches!(
        episode.outcome,
        Some(CoreTaskOutcome::Success { .. })
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn complete_episode_partial_success_json() {
    let memory = test_memory();
    let config = Config::default();
    let episode_id = start_test_episode(&memory, "CLI complete partial").await;

    let result = complete_episode(
        episode_id.to_string(),
        TaskOutcome::PartialSuccess,
        &memory,
        &config,
        OutputFormat::Json,
        false,
        30,
    )
    .await;
    assert!(result.is_ok(), "{result:?}");

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(matches!(
        episode.outcome,
        Some(CoreTaskOutcome::PartialSuccess { .. })
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn fail_episode_happy_path() {
    let memory = test_memory();
    let config = Config::default();
    let episode_id = start_test_episode(&memory, "CLI fail episode").await;

    let result = fail_episode(
        episode_id.to_string(),
        &memory,
        &config,
        OutputFormat::Yaml,
        false,
        30,
    )
    .await;
    assert!(result.is_ok(), "{result:?}");

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(matches!(
        episode.outcome,
        Some(CoreTaskOutcome::Failure { .. })
    ));
}

#[tokio::test]
async fn complete_episode_dry_run_skips_write() {
    let memory = test_memory();
    let config = Config::default();
    let result = complete_episode(
        "00000000-0000-0000-0000-000000000001".to_string(),
        TaskOutcome::Success,
        &memory,
        &config,
        OutputFormat::Human,
        true,
        30,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn complete_episode_invalid_uuid() {
    let memory = test_memory();
    let config = Config::default();
    let err = complete_episode(
        "not-a-uuid".to_string(),
        TaskOutcome::Success,
        &memory,
        &config,
        OutputFormat::Human,
        false,
        30,
    )
    .await
    .expect_err("invalid uuid");
    assert!(err.to_string().contains("Invalid episode ID"));
}

#[tokio::test]
async fn complete_episode_not_found() {
    let memory = test_memory();
    let config = Config::default();
    let err = complete_episode(
        "00000000-0000-0000-0000-000000000099".to_string(),
        TaskOutcome::Failure,
        &memory,
        &config,
        OutputFormat::Json,
        false,
        30,
    )
    .await
    .expect_err("missing episode");
    assert!(err.to_string().contains("Episode not found"));
}

#[tokio::test]
async fn fail_episode_dry_run() {
    let memory = test_memory();
    let config = Config::default();
    let result = fail_episode(
        "00000000-0000-0000-0000-000000000002".to_string(),
        &memory,
        &config,
        OutputFormat::Human,
        true,
        30,
    )
    .await;
    assert!(result.is_ok());
}

// --- durable-drain gate (#1081) ---

/// Memory with the durable write queue enabled, optionally with its
/// background workers running (the CLI drains what the workers push).
#[cfg(feature = "redb")]
async fn test_memory_with_queue(start_workers: bool) -> (SelfLearningMemory, tempfile::TempDir) {
    use do_memory_core::WriteQueueConfig;

    let dir = tempfile::tempdir().expect("tempdir");
    let durable = std::sync::Arc::new(
        do_memory_storage_redb::RedbStorage::new(&dir.path().join("durable.redb"))
            .await
            .expect("durable backend"),
    );
    let cache = std::sync::Arc::new(
        do_memory_storage_redb::RedbStorage::new(&dir.path().join("cache.redb"))
            .await
            .expect("cache backend"),
    );
    let config = MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 1.0,
        enable_summarization: false,
        enable_embeddings: false,
        durable_write_queue: Some(WriteQueueConfig {
            poll_interval_ms: 10,
            retry_base_delay_ms: 10,
            retry_max_delay_ms: 50,
            ..WriteQueueConfig::default()
        }),
        ..Default::default()
    };
    let memory = SelfLearningMemory::with_storage(config, durable, cache);
    if start_workers {
        memory.start_durable_workers();
    }
    (memory, dir)
}

#[cfg(feature = "redb")]
#[tokio::test(flavor = "multi_thread")]
async fn complete_episode_drains_queued_durable_write() {
    let (memory, _dir) = test_memory_with_queue(true).await;
    let config = Config::default();
    let episode_id = start_test_episode(&memory, "queued drain").await;

    let result = complete_episode(
        episode_id.to_string(),
        TaskOutcome::Success,
        &memory,
        &config,
        OutputFormat::Human,
        false,
        10,
    )
    .await;

    assert!(result.is_ok(), "{result:?}");
    let stats = memory
        .durable_write_stats()
        .await
        .expect("queue must be wired");
    assert!(
        stats.total_written >= 1,
        "success output requires the queued write to have committed: {stats:?}"
    );
    assert_eq!(stats.total_failed, 0);
}

#[cfg(feature = "redb")]
#[tokio::test(flavor = "multi_thread")]
async fn complete_episode_reports_drain_timeout_without_workers() {
    let (memory, _dir) = test_memory_with_queue(false).await;
    let config = Config::default();
    let episode_id = start_test_episode(&memory, "queued timeout").await;

    let err = complete_episode(
        episode_id.to_string(),
        TaskOutcome::Success,
        &memory,
        &config,
        OutputFormat::Human,
        false,
        1,
    )
    .await
    .expect_err("an undrained queue must fail the command");

    assert!(
        err.to_string().contains("did not reach durable completion"),
        "unexpected error: {err}"
    );
    // The local completion still happened; only durability is unproven.
    assert!(memory.get_episode(episode_id).await.unwrap().is_complete());
}
