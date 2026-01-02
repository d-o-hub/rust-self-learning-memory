//! Search episodes command implementation

use crate::config::Config;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;
use memory_core::TaskContext;
use super::types::EpisodeSummary;

#[allow(clippy::too_many_arguments)]
pub async fn search_episodes(
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] query: String,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] limit: usize,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _semantic: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _enable_embeddings: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_provider: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_model: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
) -> anyhow::Result<()> {
    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!(
        "Turso storage feature not enabled. Use --features turso to enable."
    ));

    #[cfg(feature = "turso")]
    {
        // Use the pre-initialized memory system
        // Search for relevant episodes
        let context = TaskContext::default(); // Use default context for search
        let episodes = memory
            .retrieve_relevant_context(query.clone(), context, limit)
            .await;
        let total_count = episodes.len();

        // Convert to summary format
        let episode_summaries: Vec<EpisodeSummary> = episodes
            .into_iter()
            .map(|episode| {
                let status = if episode.is_complete() {
                    "completed"
                } else {
                    "in_progress"
                };
                let duration_ms = episode
                    .end_time
                    .map(|end| (end - episode.start_time).num_milliseconds() as u64);

                EpisodeSummary {
                    episode_id: episode.episode_id.to_string(),
                    task_description: episode.task_description,
                    status: status.to_string(),
                    created_at: episode.start_time.to_rfc3339(),
                    duration_ms,
                    steps_count: episode.steps.len(),
                }
            })
            .collect();

        let list = super::types::EpisodeList {
            episodes: episode_summaries,
            total_count, // For search, we don't know total count
        };

        format.print_output(&list)
    }
}
