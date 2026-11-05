//! ANN index rebuild command.

use crate::config::Config;
use anyhow::Result;

/// Rebuild the ANN index from all episodes
pub async fn rebuild_index(
    memory: &do_memory_core::SelfLearningMemory,
    config: &Config,
) -> Result<()> {
    println!("🔄 Rebuilding ANN Index");
    println!("{}", "=".repeat(60));
    println!();

    if !config.embeddings.enabled {
        return Err(anyhow::anyhow!(
            "Embeddings are disabled. Enable them in config first."
        ));
    }

    crate::commands::embedding::activate_configured_semantic_service(memory, config).await?;
    let semantic = memory
        .live_semantic_service()
        .await
        .ok_or_else(|| anyhow::anyhow!("Semantic service unavailable after activation"))?;

    println!("📥 Fetching all episodes...");
    let episodes = memory.get_all_episodes().await?;
    println!("✅ Found {} episodes.", episodes.len());
    println!();

    println!("🧠 Generating embeddings and updating index...");
    let mut count = 0;
    for episode in episodes {
        // Missing embeddings are generated and stored by the service; a
        // provider failure aborts the rebuild instead of reporting a count it
        // never reached.
        semantic.embed_episode(&episode).await?;

        let embeddings = semantic.get_embeddings_batch(&[episode.episode_id]).await?;
        let embedding = embeddings
            .first()
            .and_then(|entry| entry.as_ref())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Embedding for episode {} missing after generation",
                    episode.episode_id
                )
            })?;

        if let Some(retriever) = &memory.semantic_retriever() {
            retriever.upsert(&episode.episode_id.to_string(), embedding.clone())?;
        }
        count += 1;
    }

    println!("✅ Rebuilt index with {} episodes.", count);

    println!("💾 Saving ANN snapshot...");
    memory.save_ann_snapshot()?;
    println!("✅ Snapshot saved.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{
        CliConfig, Config, DatabaseConfig, EmbeddingsConfig, StorageConfig,
    };
    use do_memory_core::{SelfLearningMemory, TaskContext, TaskType};

    fn config(enabled: bool) -> Config {
        Config {
            database: DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some(":memory:".to_string()),
                storage_mode: None,
                db_path: None,
            },
            storage: StorageConfig {
                max_episodes_cache: 100,
                cache_ttl_seconds: 3600,
                pool_size: 5,
                storage_mode: None,
            },
            cli: CliConfig {
                default_format: "human".to_string(),
                progress_bars: false,
                batch_size: 50,
            },
            embeddings: EmbeddingsConfig {
                enabled,
                provider: "local".to_string(),
                model: "test".to_string(),
                dimension: 384,
                similarity_threshold: 0.7,
                batch_size: 10,
                cache_embeddings: true,
                timeout_seconds: 30,
                base_url: None,
                api_key_env: None,
            },
        }
    }

    #[tokio::test]
    async fn rebuild_index_rejects_disabled_embeddings() {
        let memory = SelfLearningMemory::new();
        let err = rebuild_index(&memory, &config(false))
            .await
            .expect_err("disabled");
        assert!(err.to_string().contains("Embeddings are disabled"));
    }

    #[tokio::test]
    async fn rebuild_index_rejects_unactivatible_provider() {
        let memory = SelfLearningMemory::new();
        let mut cfg = config(true);
        cfg.embeddings.provider = "openai".to_string();
        cfg.embeddings.model = "text-embedding-3-small".to_string();
        cfg.embeddings.api_key_env = None;

        let err = rebuild_index(&memory, &cfg)
            .await
            .expect_err("a provider that cannot be activated must fail the rebuild");
        assert!(
            err.to_string().contains("api_key_env") || err.to_string().contains("not set"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn rebuild_index_fails_closed_without_provider_feature() {
        let memory = SelfLearningMemory::new();
        let _id = memory
            .start_episode(
                "rebuild coverage".into(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        let err = rebuild_index(&memory, &config(true))
            .await
            .expect_err("local provider needs the local-embeddings feature");
        assert!(
            err.to_string().contains("Local embeddings not available")
                || err.to_string().contains("Failed to load")
                || err.to_string().contains("model"),
            "unexpected error: {err}"
        );
    }
}
