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

    println!("📥 Fetching all episodes...");
    let episodes = memory.get_all_episodes().await?;
    println!("✅ Found {} episodes.", episodes.len());
    println!();

    println!("🧠 Generating embeddings and updating index...");
    let mut count = 0;
    for episode in episodes {
        // Semantic service will handle embedding generation if missing
        if let Some(semantic) = memory.semantic_service() {
            match semantic.embed_episode(&episode).await {
                Ok(_) => {
                    if let Ok(embeddings) =
                        semantic.get_embeddings_batch(&[episode.episode_id]).await
                    {
                        if let Some(Some(embedding)) = embeddings.first() {
                            if let Some(retriever) = &memory.semantic_retriever() {
                                retriever
                                    .upsert(&episode.episode_id.to_string(), embedding.clone())?;
                                count += 1;
                            }
                        }
                    }
                }
                Err(e) => println!("⚠️  Failed to embed episode {}: {}", episode.episode_id, e),
            }
        }
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
    async fn rebuild_index_empty_memory_without_semantic_service() {
        // Enabled path with no semantic service: fetch episodes, skip embed loop body,
        // save snapshot (no-op without ANN path).
        let memory = SelfLearningMemory::new();
        rebuild_index(&memory, &config(true))
            .await
            .expect("empty rebuild succeeds");
    }

    #[tokio::test]
    async fn rebuild_index_with_episode_no_semantic_service() {
        let memory = SelfLearningMemory::new();
        let _id = memory
            .start_episode(
                "rebuild coverage".into(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        rebuild_index(&memory, &config(true))
            .await
            .expect("rebuild with episodes but no semantic service");
    }
}
