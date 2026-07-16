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
