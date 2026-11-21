//! End-to-end persistence and playbook synthesis tests for semantic `EpisodeSummary`.
//!
//! Verifies that:
//! 1. `complete_episode` persists generated semantic summaries to backends that
//!    advertise `supports_episode_summaries()` and skips backends that do not.
//! 2. `get_episode_summary` resolves from memory, cache, or durable storage and
//!    hydrates the cache/fallback on a durable hit.
//! 3. `retrieve_playbooks` on a restarted `SelfLearningMemory` uses the
//!    persisted `EpisodeSummary` (including extracted `key_concepts`) rather
//!    than fabricating an empty-concept summary when episode tags are empty.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use async_trait::async_trait;
use do_memory_core::episode::PatternId;
use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::semantic::EpisodeSummary;
use do_memory_core::{
    Episode, ExecutionResult, ExecutionStep, Heuristic, MemoryConfig, Pattern, Result,
    StorageBackend, StorageBackendCapabilities, TaskContext, TaskOutcome, TaskType,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

struct SummaryRecordingBackend {
    advertises: bool,
    episodes: Mutex<HashMap<Uuid, Episode>>,
    patterns: Mutex<HashMap<PatternId, Pattern>>,
    summaries: Mutex<HashMap<Uuid, EpisodeSummary>>,
    summary_store_calls: Mutex<Vec<Uuid>>,
}

impl SummaryRecordingBackend {
    fn new(advertises: bool) -> Self {
        Self {
            advertises,
            episodes: Mutex::new(HashMap::new()),
            patterns: Mutex::new(HashMap::new()),
            summaries: Mutex::new(HashMap::new()),
            summary_store_calls: Mutex::new(Vec::new()),
        }
    }

    fn summary_store_calls(&self) -> Vec<Uuid> {
        self.summary_store_calls.lock().clone()
    }
}

impl StorageBackendCapabilities for SummaryRecordingBackend {
    fn supports_episode_summaries(&self) -> bool {
        self.advertises
    }
}

#[async_trait]
impl StorageBackend for SummaryRecordingBackend {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.episodes
            .lock()
            .insert(episode.episode_id, episode.clone());
        Ok(())
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        Ok(self.episodes.lock().get(&id).cloned())
    }

    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.episodes.lock().remove(&id);
        self.summaries.lock().remove(&id);
        Ok(())
    }

    async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        self.patterns.lock().insert(pattern.id(), pattern.clone());
        Ok(())
    }

    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>> {
        Ok(self.patterns.lock().get(&id).cloned())
    }

    async fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        Ok(self.patterns.lock().values().cloned().collect())
    }

    async fn store_heuristic(&self, _heuristic: &Heuristic) -> Result<()> {
        Ok(())
    }

    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<Heuristic>> {
        Ok(None)
    }

    async fn query_episodes_since(
        &self,
        _since: chrono::DateTime<chrono::Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(self.episodes.lock().values().cloned().collect())
    }

    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(vec![])
    }

    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        Ok(())
    }

    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        Ok(true)
    }

    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Ok(())
    }

    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Ok(vec![])
    }

    async fn store_episode_summary(&self, summary: &EpisodeSummary) -> Result<()> {
        self.summary_store_calls.lock().push(summary.episode_id);
        self.summaries
            .lock()
            .insert(summary.episode_id, summary.clone());
        Ok(())
    }

    async fn get_episode_summary(&self, episode_id: Uuid) -> Result<Option<EpisodeSummary>> {
        Ok(self.summaries.lock().get(&episode_id).cloned())
    }
}

fn permissive_config() -> MemoryConfig {
    MemoryConfig {
        quality_threshold: 0.0,
        ..MemoryConfig::default()
    }
}

#[tokio::test]
async fn complete_episode_persists_summary_only_to_advertising_backends() {
    let durable = Arc::new(SummaryRecordingBackend::new(true));
    let non_advertising_cache = Arc::new(SummaryRecordingBackend::new(false));
    let memory = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&non_advertising_cache) as Arc<dyn StorageBackend>,
    );

    let context = TaskContext {
        domain: "authentication".to_string(),
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        ..TaskContext::default()
    };
    let episode_id = memory
        .start_episode(
            "Implement JWT token validation middleware".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    let mut step = ExecutionStep::new(
        1,
        "validator".to_string(),
        "Verify signature and expiration claims".to_string(),
    );
    step.result = Some(ExecutionResult::Success {
        output: "Validated RS256 signature".to_string(),
    });
    memory.log_step(episode_id, step).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "JWT middleware verified".to_string(),
                artifacts: vec!["src/auth/jwt.rs".to_string()],
            },
        )
        .await
        .expect("episode completion must succeed");

    assert_eq!(
        durable.summary_store_calls(),
        vec![episode_id],
        "advertising durable backend must receive the generated EpisodeSummary"
    );
    assert!(
        non_advertising_cache.summary_store_calls().is_empty(),
        "non-advertising cache backend must be skipped"
    );

    // Cold-process simulation: create a new SelfLearningMemory with an empty cache
    // that advertises summary support, backed by the populated durable store.
    let fresh_cache = Arc::new(SummaryRecordingBackend::new(true));
    let restarted = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&fresh_cache) as Arc<dyn StorageBackend>,
    );

    let loaded = restarted
        .get_episode_summary(episode_id)
        .await
        .expect("get_episode_summary should not error")
        .expect("summary must resolve from durable storage after restart");
    assert_eq!(loaded.episode_id, episode_id);
    assert!(
        !loaded.key_concepts.is_empty(),
        "SemanticSummarizer must extract non-empty key_concepts"
    );
    assert_eq!(
        fresh_cache.summary_store_calls(),
        vec![episode_id],
        "durable summary hit must hydrate an advertising cache backend"
    );
}

#[tokio::test]
async fn retrieve_playbooks_uses_persisted_episode_summary_across_restart() {
    let durable = Arc::new(SummaryRecordingBackend::new(true));
    let cache = Arc::new(SummaryRecordingBackend::new(true));

    // Context deliberately has NO tags so the fallback summary would have
    // empty `key_concepts`, whereas the persisted summary carries a custom concept.
    let context = TaskContext {
        domain: "database-migration".to_string(),
        tags: vec![],
        ..TaskContext::default()
    };

    let memory = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let episode_id = memory
        .start_episode(
            "Run zero-downtime schema migration".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;
    let mut step = ExecutionStep::new(
        1,
        "migrator".to_string(),
        "Apply expand-contract column migration".to_string(),
    );
    step.result = Some(ExecutionResult::Success {
        output: "Migration applied cleanly".to_string(),
    });
    memory.log_step(episode_id, step).await;
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Migration complete".to_string(),
                artifacts: vec!["migrations/001.sql".to_string()],
            },
        )
        .await
        .unwrap();

    // Overwrite the durable summary with a distinctive concept marker to prove
    // `retrieve_playbooks` reads the persisted summary on a cold restart.
    let mut custom_summary = EpisodeSummary::new(episode_id);
    custom_summary.summary_text = "Zero-downtime expand-contract migration summary".to_string();
    custom_summary.key_concepts = vec!["expand-contract-sentinel".to_string()];
    durable
        .store_episode_summary(&custom_summary)
        .await
        .unwrap();

    let cold_cache = Arc::new(SummaryRecordingBackend::new(false));
    let restarted = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cold_cache) as Arc<dyn StorageBackend>,
    );

    let playbooks = restarted
        .retrieve_playbooks(
            "Run zero-downtime schema migration",
            "database-migration",
            TaskType::CodeGeneration,
            context,
            1,
            5,
        )
        .await;

    assert_eq!(playbooks.len(), 1);
    assert!(
        playbooks[0]
            .expected_outcome
            .contains("expand-contract-sentinel"),
        "expected playbook outcome to include concept from persisted EpisodeSummary, got: {}",
        playbooks[0].expected_outcome
    );
}

#[tokio::test]
async fn cleanup_episodes_prunes_failed_candidates_across_memory_and_storage() {
    use do_memory_core::episode::{EpisodeRetentionPolicy, RetentionCriterion};

    let durable = Arc::new(SummaryRecordingBackend::new(true));
    let cache = Arc::new(SummaryRecordingBackend::new(true));
    let memory = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let keep_id = memory
        .start_episode(
            "Successful release workflow".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        )
        .await;
    let mut step = ExecutionStep::new(1, "builder".to_string(), "Compile release".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "ok".to_string(),
    });
    memory.log_step(keep_id, step).await;
    memory
        .complete_episode(
            keep_id,
            TaskOutcome::Success {
                verdict: "Shipped".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let drop_id = memory
        .start_episode(
            "Aborted broken experiment".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    let mut bad_step = ExecutionStep::new(1, "runner".to_string(), "Run broken check".to_string());
    bad_step.result = Some(ExecutionResult::Error {
        message: "failed".to_string(),
    });
    memory.log_step(drop_id, bad_step).await;
    memory
        .complete_episode(
            drop_id,
            TaskOutcome::Failure {
                reason: "broken".to_string(),
                error_details: None,
            },
        )
        .await
        .unwrap();

    let policy = EpisodeRetentionPolicy::default().with_criterion(RetentionCriterion::FailedOnly);
    assert_eq!(memory.count_cleanup_candidates(&policy).await.unwrap(), 1);

    let dry_run = memory
        .cleanup_episodes(&policy.clone().with_dry_run(true))
        .await
        .unwrap();
    assert_eq!(dry_run.deleted_ids, vec![drop_id]);
    assert!(memory.get_episode(drop_id).await.is_ok());

    let executed = memory.cleanup_episodes(&policy).await.unwrap();
    assert_eq!(executed.deleted_ids, vec![drop_id]);
    assert_eq!(executed.kept_ids, vec![keep_id]);
    assert!(memory.get_episode(drop_id).await.is_err());
    assert!(memory.get_episode_summary(drop_id).await.unwrap().is_none());
    assert!(memory.get_episode(keep_id).await.is_ok());
}
