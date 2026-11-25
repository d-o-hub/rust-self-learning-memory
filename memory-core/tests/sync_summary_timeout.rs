//! Regression coverage for best-effort episode-summary cache timeouts.
use async_trait::async_trait;
use do_memory_core::{
    Episode, Error, Heuristic, Pattern, StorageBackend, StorageBackendCapabilities,
    SyncWatermarkBackend, TaskContext, TaskType,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use uuid::Uuid;

struct TimeoutBackend {
    episode: Episode,
    summary: do_memory_core::semantic::EpisodeSummary,
    stall_summary_write: bool,
    summary_write_attempted: Arc<AtomicBool>,
}

fn unsupported<T>() -> do_memory_core::Result<T> {
    Err(Error::capability_unavailable(
        "unused timeout-test operation",
    ))
}

impl StorageBackendCapabilities for TimeoutBackend {
    fn supports_episode_summaries(&self) -> bool {
        true
    }
}

#[async_trait]
impl StorageBackend for TimeoutBackend {
    async fn store_episode(&self, _: &Episode) -> do_memory_core::Result<()> {
        Ok(())
    }

    async fn get_episode(&self, id: Uuid) -> do_memory_core::Result<Option<Episode>> {
        Ok((self.episode.episode_id == id).then(|| self.episode.clone()))
    }

    async fn delete_episode(&self, _: Uuid) -> do_memory_core::Result<()> {
        unsupported()
    }

    async fn store_pattern(&self, _: &Pattern) -> do_memory_core::Result<()> {
        unsupported()
    }

    async fn get_pattern(
        &self,
        _: do_memory_core::episode::PatternId,
    ) -> do_memory_core::Result<Option<Pattern>> {
        unsupported()
    }

    async fn store_heuristic(&self, _: &Heuristic) -> do_memory_core::Result<()> {
        unsupported()
    }

    async fn get_heuristic(&self, _: Uuid) -> do_memory_core::Result<Option<Heuristic>> {
        unsupported()
    }

    async fn query_episodes_since(
        &self,
        _: chrono::DateTime<chrono::Utc>,
        _: Option<usize>,
    ) -> do_memory_core::Result<Vec<Episode>> {
        Ok(vec![self.episode.clone()])
    }

    async fn query_episodes_by_metadata(
        &self,
        _: &str,
        _: &str,
        _: Option<usize>,
    ) -> do_memory_core::Result<Vec<Episode>> {
        unsupported()
    }

    async fn store_embedding(&self, _: &str, _: Vec<f32>) -> do_memory_core::Result<()> {
        unsupported()
    }

    async fn get_embedding(&self, _: &str) -> do_memory_core::Result<Option<Vec<f32>>> {
        unsupported()
    }

    async fn delete_embedding(&self, _: &str) -> do_memory_core::Result<bool> {
        unsupported()
    }

    async fn store_embeddings_batch(
        &self,
        _: Vec<(String, Vec<f32>)>,
    ) -> do_memory_core::Result<()> {
        unsupported()
    }

    async fn get_embeddings_batch(
        &self,
        _: &[String],
    ) -> do_memory_core::Result<Vec<Option<Vec<f32>>>> {
        unsupported()
    }

    async fn store_episode_summary(
        &self,
        _: &do_memory_core::semantic::EpisodeSummary,
    ) -> do_memory_core::Result<()> {
        self.summary_write_attempted.store(true, Ordering::SeqCst);
        if self.stall_summary_write {
            std::future::pending().await
        } else {
            Ok(())
        }
    }

    async fn get_episode_summary(
        &self,
        _: Uuid,
    ) -> do_memory_core::Result<Option<do_memory_core::semantic::EpisodeSummary>> {
        Ok(Some(self.summary.clone()))
    }
}

impl SyncWatermarkBackend for TimeoutBackend {}

#[tokio::test(start_paused = true)]
async fn summary_cache_timeout_is_best_effort() {
    let episode = Episode::new(
        "summary timeout fixture".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    let episode_id = episode.episode_id;
    let summary = do_memory_core::semantic::EpisodeSummary::new(episode_id);
    let summary_write_attempted = Arc::new(AtomicBool::new(false));
    let source = Arc::new(TimeoutBackend {
        episode: episode.clone(),
        summary: summary.clone(),
        stall_summary_write: false,
        summary_write_attempted: Arc::new(AtomicBool::new(false)),
    });
    let cache = Arc::new(TimeoutBackend {
        episode,
        summary,
        stall_summary_write: true,
        summary_write_attempted: Arc::clone(&summary_write_attempted),
    });
    let synchronizer = do_memory_core::sync::StorageSynchronizer::new(source, cache);

    let result = synchronizer.sync_episode_to_cache(episode_id).await;

    assert!(
        result.is_ok(),
        "summary cache timeout must not fail episode sync"
    );
    assert!(
        summary_write_attempted.load(Ordering::SeqCst),
        "the cache summary write must reach its timeout"
    );
}
