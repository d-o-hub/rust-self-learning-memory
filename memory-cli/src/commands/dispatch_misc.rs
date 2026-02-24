use crate::config::Config;
use crate::output::OutputFormat;

use super::{EmbeddingCommands, TagCommands, embedding, tag};

pub async fn handle_embedding_command(
    command: EmbeddingCommands,
    _memory: &memory_core::SelfLearningMemory,
    config: &Config,
    _format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        EmbeddingCommands::Test => embedding::test_embeddings(config).await,
        EmbeddingCommands::Config => embedding::show_config(config),
        EmbeddingCommands::ListProviders => embedding::list_providers(),
        EmbeddingCommands::Benchmark => embedding::benchmark_embeddings(config).await,
        EmbeddingCommands::Enable => embedding::enable_embeddings(),
        EmbeddingCommands::Disable => embedding::disable_embeddings(),
    }
}

pub async fn handle_tag_command(
    command: TagCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    tag::handle_tag_command(command, memory, config, format, dry_run).await
}
