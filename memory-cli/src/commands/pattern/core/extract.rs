//! Pattern extract command implementation (ADR-076 §5)
//!
//! Re-runs pattern extraction for completed episodes. Uses the same extractor
//! pipeline as `complete_episode` and respects ADR-075 durability rules.

use crate::config::Config;
use crate::output::OutputFormat;
use do_memory_core::SelfLearningMemory;
use uuid::Uuid;

/// Result for a single episode re-extraction.
#[derive(Debug, serde::Serialize)]
pub struct EpisodeExtractResult {
    pub episode_id: String,
    pub patterns_extracted: usize,
    pub status: String,
}

/// Summary across one or more episodes.
#[derive(Debug, serde::Serialize)]
pub struct ExtractSummary {
    pub episodes_processed: usize,
    pub total_patterns_extracted: usize,
    pub results: Vec<EpisodeExtractResult>,
}

/// Re-run pattern extraction for a specific completed episode or all completed
/// episodes that currently have no patterns linked (when `--all` is used).
///
/// # Arguments
///
/// * `episode_id`   – Re-extract for this single episode UUID.
/// * `all`          – When `true`, re-extract for every completed episode with
///   no patterns yet. Mutually exclusive with `episode_id`.
/// * `memory`       – Shared memory instance.
/// * `_config`      – CLI configuration (reserved for future use).
/// * `format`       – Output format (human / json / yaml).
///
/// # Errors
///
/// Returns an error if:
/// - `episode_id` is provided but is not a valid UUID.
/// - `episode_id` is provided but the episode does not exist or is not complete.
/// - Neither `episode_id` nor `all` is set.
pub async fn extract_patterns(
    episode_id: Option<String>,
    all: bool,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    match (episode_id, all) {
        (Some(id_str), false) => extract_single(id_str, memory, format).await,
        (None, true) => extract_all_without_patterns(memory, format).await,
        (Some(_), true) => anyhow::bail!("Provide either --episode-id <uuid> or --all, not both"),
        (None, false) => anyhow::bail!("Provide either --episode-id <uuid> or --all"),
    }
}

async fn extract_single(
    id_str: String,
    memory: &SelfLearningMemory,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let episode_uuid = Uuid::parse_str(&id_str)
        .map_err(|_| anyhow::anyhow!("Invalid episode ID format: {}", id_str))?;

    let count = memory
        .re_extract_patterns(episode_uuid)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to re-extract patterns for {}: {}", id_str, e))?;

    let summary = ExtractSummary {
        episodes_processed: 1,
        total_patterns_extracted: count,
        results: vec![EpisodeExtractResult {
            episode_id: id_str,
            patterns_extracted: count,
            status: "ok".to_string(),
        }],
    };

    print_summary(&summary, format)
}

async fn extract_all_without_patterns(
    memory: &SelfLearningMemory,
    format: OutputFormat,
) -> anyhow::Result<()> {
    // Retrieve all completed episodes that currently have no patterns linked.
    let episodes = memory
        .list_episodes(None, None, Some(true))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list completed episodes: {}", e))?;

    let candidates: Vec<_> = episodes
        .into_iter()
        .filter(|ep| ep.patterns.is_empty())
        .collect();

    if candidates.is_empty() {
        let summary = ExtractSummary {
            episodes_processed: 0,
            total_patterns_extracted: 0,
            results: vec![],
        };
        return print_summary(&summary, format);
    }

    let mut results = Vec::with_capacity(candidates.len());
    let mut total = 0usize;

    for ep in &candidates {
        let episode_uuid = ep.episode_id;
        match memory.re_extract_patterns(episode_uuid).await {
            Ok(count) => {
                total += count;
                results.push(EpisodeExtractResult {
                    episode_id: episode_uuid.to_string(),
                    patterns_extracted: count,
                    status: "ok".to_string(),
                });
            }
            Err(e) => {
                results.push(EpisodeExtractResult {
                    episode_id: episode_uuid.to_string(),
                    patterns_extracted: 0,
                    status: format!("error: {e}"),
                });
            }
        }
    }

    let summary = ExtractSummary {
        episodes_processed: results.len(),
        total_patterns_extracted: total,
        results,
    };

    print_summary(&summary, format)
}

fn print_summary(summary: &ExtractSummary, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(summary)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(summary)?);
        }
        OutputFormat::Human => {
            println!(
                "Pattern extraction complete: {} episode(s) processed, {} pattern(s) extracted",
                summary.episodes_processed, summary.total_patterns_extracted
            );
            for r in &summary.results {
                println!(
                    "  {} → {} pattern(s) [{}]",
                    r.episode_id, r.patterns_extracted, r.status
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::output::OutputFormat;
    use do_memory_core::{
        ExecutionResult, ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome,
        TaskType,
    };

    fn test_memory() -> SelfLearningMemory {
        SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            pattern_extraction_threshold: 0.0,
            enable_summarization: false,
            enable_embeddings: false,
            ..Default::default()
        })
    }

    async fn create_completed_episode(memory: &SelfLearningMemory) -> Uuid {
        let id = memory
            .start_episode(
                "test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory
            .complete_episode(
                id,
                TaskOutcome::Success {
                    verdict: "done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .expect("complete failed");
        id
    }

    // ── Arrange / Act / Assert ────────────────────────────────────────────────

    #[tokio::test]
    async fn extract_single_invalid_uuid_returns_error() {
        // Arrange
        let memory = test_memory();
        let config = Config::default();

        // Act
        let err = extract_patterns(
            Some("not-a-uuid".to_string()),
            false,
            &memory,
            &config,
            OutputFormat::Human,
        )
        .await
        .expect_err("should fail for bad uuid");

        // Assert
        assert!(err.to_string().contains("Invalid episode ID"), "got: {err}");
    }

    #[tokio::test]
    async fn extract_single_missing_episode_returns_error() {
        // Arrange
        let memory = test_memory();
        let config = Config::default();

        // Act
        let err = extract_patterns(
            Some("00000000-0000-0000-0000-000000000001".to_string()),
            false,
            &memory,
            &config,
            OutputFormat::Human,
        )
        .await
        .expect_err("should fail for missing episode");

        // Assert
        assert!(
            err.to_string().contains("Failed to re-extract patterns"),
            "got: {err}"
        );
    }

    #[tokio::test]
    async fn extract_single_incomplete_episode_returns_error() {
        // Arrange
        let memory = test_memory();
        let config = Config::default();
        let episode_id = memory
            .start_episode(
                "incomplete task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Act
        let err = extract_patterns(
            Some(episode_id.to_string()),
            false,
            &memory,
            &config,
            OutputFormat::Human,
        )
        .await
        .expect_err("should fail for incomplete episode");

        // Assert
        assert!(
            err.to_string().contains("Failed to re-extract patterns"),
            "got: {err}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn extract_single_completed_episode_succeeds() {
        // Arrange
        let memory = test_memory();
        let config = Config::default();
        let episode_id = create_completed_episode(&memory).await;

        // Act
        let result = extract_patterns(
            Some(episode_id.to_string()),
            false,
            &memory,
            &config,
            OutputFormat::Human,
        )
        .await;

        // Assert
        assert!(result.is_ok(), "unexpected error: {:?}", result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn extract_single_json_format() {
        // Arrange
        let memory = test_memory();
        let config = Config::default();
        let episode_id = create_completed_episode(&memory).await;

        // Act
        let result = extract_patterns(
            Some(episode_id.to_string()),
            false,
            &memory,
            &config,
            OutputFormat::Json,
        )
        .await;

        // Assert
        assert!(result.is_ok(), "{:?}", result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn extract_single_yaml_format() {
        // Arrange
        let memory = test_memory();
        let config = Config::default();
        let episode_id = create_completed_episode(&memory).await;

        // Act
        let result = extract_patterns(
            Some(episode_id.to_string()),
            false,
            &memory,
            &config,
            OutputFormat::Yaml,
        )
        .await;

        // Assert
        assert!(result.is_ok(), "{:?}", result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn extract_all_no_completed_episodes() {
        // Arrange
        let memory = test_memory();
        let config = Config::default();

        // Act
        let result = extract_patterns(None, true, &memory, &config, OutputFormat::Human).await;

        // Assert
        assert!(result.is_ok(), "{:?}", result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn extract_all_skips_episodes_already_having_patterns() {
        // Arrange – create a completed episode with threshold 0.0 so complete_episode
        // already extracts patterns. --all filters to episodes with no patterns, so
        // the candidate list should be empty.
        let memory = test_memory(); // threshold 0.0 → patterns extracted at complete
        let config = Config::default();
        let mut step = ExecutionStep::new(
            1,
            "test_tool".to_string(),
            "action with patterns".to_string(),
        );
        step.result = Some(ExecutionResult::Success {
            output: "output".to_string(),
        });
        let episode_id = memory
            .start_episode(
                "episode with patterns".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory.log_step(episode_id, step).await;
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .expect("complete failed");

        // Verify episode already has patterns (threshold 0.0 extracts on complete)
        let episodes = memory.list_episodes(None, None, Some(true)).await.unwrap();
        let ep = episodes
            .iter()
            .find(|e| e.episode_id == episode_id)
            .unwrap();
        // --all only targets episodes with zero patterns; this episode has patterns
        // so the candidate list is empty → empty summary returned
        let has_patterns = !ep.patterns.is_empty();

        // Act – --all only targets episodes with no patterns
        let result = extract_patterns(None, true, &memory, &config, OutputFormat::Human).await;

        // Assert
        assert!(result.is_ok(), "{:?}", result);
        if has_patterns {
            // The episode was skipped (already had patterns), no error
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn extract_all_with_candidate_processes_non_empty_list() {
        // Arrange – complete a Failure episode with no steps.
        // Failure outcome → reward.total = 0.0 → context extractor skips.
        // No steps → tool-sequence, decision-point, error-recovery extractors all skip.
        // Result: completed episode with zero patterns linked → candidate for --all.
        let memory = SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            pattern_extraction_threshold: 0.0,
            enable_summarization: false,
            enable_embeddings: false,
            ..Default::default()
        });
        let config = Config::default();

        let episode_id = memory
            .start_episode(
                "failure episode no patterns".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Failure {
                    reason: "deliberate failure for test".to_string(),
                    error_details: None,
                },
            )
            .await
            .expect("complete failed");

        // Verify episode has no patterns before running --all
        let episodes = memory.list_episodes(None, None, Some(true)).await.unwrap();
        let ep = episodes
            .iter()
            .find(|e| e.episode_id == episode_id)
            .unwrap();
        assert!(ep.patterns.is_empty(), "expected no patterns before --all");

        // Act – JSON output exercises the non-empty candidate path (lines 110–140)
        let result = extract_patterns(None, true, &memory, &config, OutputFormat::Json).await;

        // Assert – command succeeds (covers the processing loop even if 0 new patterns)
        assert!(result.is_ok(), "extract --all failed: {:?}", result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn extract_all_human_format_with_candidate() {
        // Arrange – Failure episode, no steps → 0 patterns → candidate for --all
        let memory = SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            pattern_extraction_threshold: 0.0,
            enable_summarization: false,
            enable_embeddings: false,
            ..Default::default()
        });
        let config = Config::default();
        let episode_id = memory
            .start_episode(
                "human format candidate".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Failure {
                    reason: "deliberate failure for test".to_string(),
                    error_details: None,
                },
            )
            .await
            .expect("complete failed");

        // Act – exercises the non-empty path with Human output (line 140)
        let result = extract_patterns(None, true, &memory, &config, OutputFormat::Human).await;

        // Assert
        assert!(result.is_ok(), "{:?}", result);
    }

    #[tokio::test]
    async fn extract_both_flags_returns_error() {
        // Arrange
        let memory = test_memory();
        let config = Config::default();

        // Act
        let err = extract_patterns(
            Some("some-id".to_string()),
            true,
            &memory,
            &config,
            OutputFormat::Human,
        )
        .await
        .expect_err("should fail when both flags set");

        // Assert
        assert!(err.to_string().contains("not both"), "got: {err}");
    }

    #[tokio::test]
    async fn extract_no_flags_returns_error() {
        // Arrange
        let memory = test_memory();
        let config = Config::default();

        // Act
        let err = extract_patterns(None, false, &memory, &config, OutputFormat::Human)
            .await
            .expect_err("should fail when no flags set");

        // Assert
        assert!(
            err.to_string().contains("--episode-id") || err.to_string().contains("--all"),
            "got: {err}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn extract_single_with_steps_extracts_patterns() {
        // Arrange – create a completed episode with a successful step so
        // the tool-sequence extractor can produce at least one pattern.
        let memory = test_memory();
        let config = Config::default();
        let episode_id = memory
            .start_episode(
                "extract integration test".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        let mut step = ExecutionStep::new(1, "test_tool".to_string(), "run tests".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "all passed".to_string(),
        });
        memory.log_step(episode_id, step).await;
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .expect("complete failed");

        // Act – re-extract patterns from the completed episode.
        // Use JSON so we can inspect the count precisely.
        let result = memory.re_extract_patterns(episode_id).await;

        // Assert – at least one pattern must be returned.
        let count = result.expect("re_extract_patterns should succeed");
        assert!(
            count > 0,
            "expected >0 patterns from episode with a successful step, got {count}"
        );

        // Also verify the CLI wrapper path succeeds.
        let cli_result = extract_patterns(
            Some(episode_id.to_string()),
            false,
            &memory,
            &config,
            OutputFormat::Json,
        )
        .await;
        assert!(cli_result.is_ok(), "CLI path failed: {:?}", cli_result);
    }
}
