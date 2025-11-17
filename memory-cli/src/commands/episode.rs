use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;
use std::path::PathBuf;

use crate::config::Config;
use crate::output::{Output, OutputFormat};

#[derive(Subcommand)]
pub enum EpisodeCommands {
    /// Create a new episode
    Create {
        /// Task description
        #[arg(short, long)]
        task: String,

        /// Context file (JSON)
        #[arg(short, long, value_name = "FILE")]
        context: Option<PathBuf>,
    },

    /// List episodes
    List {
        /// Filter by task type
        #[arg(short, long)]
        task_type: Option<String>,

        /// Maximum number of episodes to return
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Filter by status
        #[arg(short, long)]
        status: Option<EpisodeStatus>,
    },

    /// View episode details
    View {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,
    },

    /// Complete an episode
    Complete {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Task outcome
        #[arg(value_enum)]
        outcome: TaskOutcome,
    },

    /// Search episodes
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Log an execution step
    LogStep {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Tool name
        #[arg(short, long)]
        tool: String,

        /// Action description
        #[arg(short, long)]
        action: String,

        /// Whether the step was successful
        #[arg(long)]
        success: bool,

        /// Latency in milliseconds
        #[arg(long)]
        latency_ms: Option<u64>,

        /// Token count
        #[arg(long)]
        tokens: Option<u32>,

        /// Step observation
        #[arg(short, long)]
        observation: Option<String>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum EpisodeStatus {
    /// Episode is currently in progress
    InProgress,
    /// Episode has been completed
    Completed,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TaskOutcome {
    /// Task completed successfully
    Success,
    /// Task completed with partial success
    PartialSuccess,
    /// Task failed
    Failure,
}

#[derive(Debug, Serialize)]
pub struct EpisodeSummary {
    pub episode_id: String,
    pub task_description: String,
    pub status: String,
    pub created_at: String,
    pub duration_ms: Option<u64>,
    pub steps_count: usize,
}

impl Output for EpisodeSummary {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(writer, "Episode: {}", self.episode_id)?;
        writeln!(writer, "Task: {}", self.task_description)?;
        writeln!(writer, "Status: {}", self.status)?;
        writeln!(writer, "Created: {}", self.created_at)?;
        if let Some(duration) = self.duration_ms {
            writeln!(writer, "Duration: {}ms", duration)?;
        }
        writeln!(writer, "Steps: {}", self.steps_count)?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct EpisodeList {
    pub episodes: Vec<EpisodeSummary>,
    pub total_count: usize,
}

impl Output for EpisodeList {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{} episodes (showing {})", self.total_count, self.episodes.len())?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for episode in &self.episodes {
            let status_color = match episode.status.as_str() {
                "completed" => Color::Green,
                "in_progress" => Color::Yellow,
                _ => Color::Red,
            };

            writeln!(writer, "{} {} {}",
                episode.episode_id[..8].to_string().dimmed(),
                episode.task_description.chars().take(50).collect::<String>(),
                episode.status.color(status_color).bold()
            )?;
        }

        Ok(())
    }
}

// Command implementations
pub async fn create_episode(
    task: String,
    context: Option<PathBuf>,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    use memory_core::{SelfLearningMemory, TaskContext, TaskType, MemoryConfig};
    #[cfg(feature = "turso")]
    use memory_storage_turso::TursoStorage;
    #[cfg(feature = "redb")]
    use memory_storage_redb::RedbStorage;
    use std::sync::Arc;

    if dry_run {
        println!("Would create episode with task: {}", task);
        if let Some(context_path) = context {
            println!("Would load context from: {}", context_path.display());
        }
        return Ok(());
    }

    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!("Turso storage feature not enabled. Use --features turso to enable."));

    #[cfg(feature = "turso")]
    {
        // Load context from file if provided
        let context_data = if let Some(context_path) = context {
            let content = std::fs::read_to_string(&context_path)
                .with_context(|| format!("Failed to read context file: {}", context_path.display()))?;

            // Try to parse as JSON first, then YAML
            if let Ok(ctx) = serde_json::from_str::<TaskContext>(&content) {
                ctx
            } else {
                serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse context file as JSON or YAML: {}", context_path.display()))?
            }
        } else {
            TaskContext::default()
        };

        // Create storage backends
        let turso_url = config.database.turso_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Turso database URL not configured"))?;
        let token = config.database.turso_token.as_deref().unwrap_or("");

        let turso_storage = TursoStorage::new(turso_url, token).await?;
        let turso = Arc::new(turso_storage);

        #[cfg(feature = "redb")]
        let cache_storage = if let Some(path) = &config.database.redb_path {
            Some(Arc::new(RedbStorage::new(std::path::Path::new(path)).await?))
        } else {
            None
        };

        // Create memory system
        let memory_config = MemoryConfig::default();
        #[cfg(feature = "redb")]
        let memory = if let Some(cache) = cache_storage {
            SelfLearningMemory::with_storage(memory_config, turso, cache)
        } else {
            SelfLearningMemory::with_storage(memory_config, turso, Arc::new(RedbStorage::new(std::path::Path::new(":memory:")).await?))
        };

        #[cfg(not(feature = "redb"))]
        let memory = SelfLearningMemory::new(); // Fallback to in-memory only

        // Start the episode
        let episode_id = memory.start_episode(task.clone(), context_data, TaskType::CodeGeneration).await;

        // Output the result
        #[derive(Debug, serde::Serialize)]
        struct CreateResult {
            episode_id: String,
            task: String,
            status: String,
        }

        impl Output for CreateResult {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;
                writeln!(writer, "{}", "Episode Created".green().bold())?;
                writeln!(writer, "ID: {}", self.episode_id.dimmed())?;
                writeln!(writer, "Task: {}", self.task)?;
                writeln!(writer, "Status: {}", self.status.green())?;
                Ok(())
            }
        }

        let result = CreateResult {
            episode_id: episode_id.to_string(),
            task: task.clone(),
            status: "created".to_string(),
        };

        format.print_output(&result)
    }
}

pub async fn list_episodes(
    task_type: Option<String>,
    limit: usize,
    status: Option<EpisodeStatus>,
    config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use memory_core::{SelfLearningMemory, MemoryConfig, TaskType as CoreTaskType};
    #[cfg(feature = "turso")]
    use memory_storage_turso::{TursoStorage, EpisodeQuery};
    use std::sync::Arc;

    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!("Turso storage feature not enabled. Use --features turso to enable."));

    #[cfg(feature = "turso")]
    {
        // Create storage backend
        let turso_url = config.database.turso_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Turso database URL not configured"))?;
        let token = config.database.turso_token.as_deref().unwrap_or("");

        let turso_storage = TursoStorage::new(turso_url, token).await?;
        let turso = Arc::new(turso_storage);

        // Build query
        let mut query = EpisodeQuery {
            limit: Some(limit),
            completed_only: matches!(status, Some(EpisodeStatus::Completed)),
            ..Default::default()
        };

        if let Some(task_type_str) = task_type {
            query.task_type = match task_type_str.as_str() {
                "code_generation" => Some(CoreTaskType::CodeGeneration),
                "debugging" => Some(CoreTaskType::Debugging),
                "testing" => Some(CoreTaskType::Testing),
                "analysis" => Some(CoreTaskType::Analysis),
                "documentation" => Some(CoreTaskType::Documentation),
                "refactoring" => Some(CoreTaskType::Refactoring),
                "other" => Some(CoreTaskType::Other),
                _ => return Err(anyhow::anyhow!("Invalid task type: {}. Valid types: code_generation, debugging, testing, analysis, documentation, refactoring, other", task_type_str)),
            };
        }

        // Query episodes
        let episodes = turso.query_episodes(&query).await?;

        // Convert to summary format
        let episode_summaries: Vec<EpisodeSummary> = episodes
            .into_iter()
            .map(|episode| {
                let status = if episode.is_complete() { "completed" } else { "in_progress" };
                let duration_ms = episode.end_time.map(|end| {
                    (end - episode.start_time).num_milliseconds() as u64
                });

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

        let list = EpisodeList {
            episodes: episode_summaries,
            total_count: turso.get_statistics().await?.episode_count,
        };

        format.print_output(&list)
    }
}

pub async fn view_episode(
    episode_id: String,
    config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use memory_core::SelfLearningMemory;
    #[cfg(feature = "turso")]
    use memory_storage_turso::TursoStorage;
    #[cfg(feature = "redb")]
    use memory_storage_redb::RedbStorage;
    use std::sync::Arc;
    use uuid::Uuid;

    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!("Turso storage feature not enabled. Use --features turso to enable."));

    #[cfg(feature = "turso")]
    {
        // Parse episode ID
        let episode_uuid = Uuid::parse_str(&episode_id)
            .with_context(|| format!("Invalid episode ID format: {}", episode_id))?;

        // Create storage backends
        let turso_url = config.database.turso_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Turso database URL not configured"))?;
        let token = config.database.turso_token.as_deref().unwrap_or("");

        let turso_storage = TursoStorage::new(turso_url, token).await?;
        let turso = Arc::new(turso_storage);

        #[cfg(feature = "redb")]
        let cache_storage = if let Some(path) = &config.database.redb_path {
            Some(Arc::new(RedbStorage::new(std::path::Path::new(path)).await?))
        } else {
            None
        };

        // Create memory system
        let memory_config = memory_core::MemoryConfig::default();
        #[cfg(feature = "redb")]
        let memory = if let Some(cache) = cache_storage {
            SelfLearningMemory::with_storage(memory_config, turso, cache)
        } else {
            SelfLearningMemory::with_storage(memory_config, turso, Arc::new(RedbStorage::new(std::path::Path::new(":memory:")).await?))
        };

        #[cfg(not(feature = "redb"))]
        let memory = SelfLearningMemory::with_storage(memory_config, turso, Arc::new(memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:")).await?));

        // Get the episode
        let episode = memory.get_episode(episode_uuid).await
            .with_context(|| format!("Episode not found: {}", episode_id))?;

        // Create a detailed view
        #[derive(Debug, serde::Serialize)]
        struct EpisodeDetail {
            episode_id: String,
            task_description: String,
            task_type: String,
            context: serde_json::Value,
            status: String,
            created_at: String,
            completed_at: Option<String>,
            duration_ms: Option<i64>,
            steps_count: usize,
            steps: Vec<serde_json::Value>,
            outcome: Option<serde_json::Value>,
            reward: Option<serde_json::Value>,
            reflection: Option<serde_json::Value>,
            patterns_count: usize,
            heuristics_count: usize,
        }

        impl Output for EpisodeDetail {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;

                writeln!(writer, "{}", "Episode Details".bold().underline())?;
                writeln!(writer, "ID: {}", self.episode_id.dimmed())?;
                writeln!(writer, "Task: {}", self.task_description)?;
                writeln!(writer, "Type: {}", self.task_type)?;
                writeln!(writer, "Status: {}", if self.completed_at.is_some() { "Completed".green() } else { "In Progress".yellow() })?;
                writeln!(writer, "Created: {}", self.created_at)?;

                if let Some(completed) = &self.completed_at {
                    writeln!(writer, "Completed: {}", completed)?;
                }

                if let Some(duration) = self.duration_ms {
                    writeln!(writer, "Duration: {}ms", duration)?;
                }

                writeln!(writer, "Steps: {}", self.steps_count)?;
                writeln!(writer, "Patterns: {}", self.patterns_count)?;
                writeln!(writer, "Heuristics: {}", self.heuristics_count)?;

                Ok(())
            }
        }

        let detail = EpisodeDetail {
            episode_id: episode.episode_id.to_string(),
            task_description: episode.task_description,
            task_type: episode.task_type.to_string(),
            context: serde_json::to_value(&episode.context)?,
            status: if episode.is_complete() { "completed" } else { "in_progress" }.to_string(),
            created_at: episode.start_time.to_rfc3339(),
            completed_at: episode.end_time.map(|t| t.to_rfc3339()),
            duration_ms: episode.end_time.map(|end| (end - episode.start_time).num_milliseconds()),
            steps_count: episode.steps.len(),
            steps: episode.steps.iter().map(|s| serde_json::to_value(s)).collect::<Result<Vec<_>, _>>()?,
            outcome: episode.outcome.as_ref().map(|o| serde_json::to_value(o)).transpose()?,
            reward: episode.reward.as_ref().map(|r| serde_json::to_value(r)).transpose()?,
            reflection: episode.reflection.as_ref().map(|r| serde_json::to_value(r)).transpose()?,
            patterns_count: episode.patterns.len(),
            heuristics_count: episode.heuristics.len(),
        };

        format.print_output(&detail)
    }
}

pub async fn complete_episode(
    episode_id: String,
    outcome: TaskOutcome,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    use memory_core::{SelfLearningMemory, MemoryConfig, TaskOutcome as CoreTaskOutcome};
    #[cfg(feature = "turso")]
    use memory_storage_turso::TursoStorage;
    #[cfg(feature = "redb")]
    use memory_storage_redb::RedbStorage;
    use std::sync::Arc;
    use uuid::Uuid;

    if dry_run {
        println!("Would complete episode {} with outcome: {:?}", episode_id, outcome);
        return Ok(());
    }

    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!("Turso storage feature not enabled. Use --features turso to enable."));

    #[cfg(feature = "turso")]
    {
        // Parse episode ID
        let episode_uuid = Uuid::parse_str(&episode_id)
            .with_context(|| format!("Invalid episode ID format: {}", episode_id))?;

        // Create storage backends
        let turso_url = config.database.turso_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Turso database URL not configured"))?;
        let token = config.database.turso_token.as_deref().unwrap_or("");

        let turso_storage = TursoStorage::new(turso_url, token).await?;
        let turso = Arc::new(turso_storage);

        #[cfg(feature = "redb")]
        let cache_storage = if let Some(path) = &config.database.redb_path {
            Some(Arc::new(RedbStorage::new(std::path::Path::new(path)).await?))
        } else {
            None
        };

        // Create memory system
        let memory_config = MemoryConfig::default();
        #[cfg(feature = "redb")]
        let memory = if let Some(cache) = cache_storage {
            SelfLearningMemory::with_storage(memory_config, turso, cache)
        } else {
            SelfLearningMemory::with_storage(memory_config, turso, Arc::new(RedbStorage::new(std::path::Path::new(":memory:")).await?))
        };

        #[cfg(not(feature = "redb"))]
        let memory = SelfLearningMemory::with_storage(memory_config, turso, Arc::new(memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:")).await?));

        // Map CLI outcome to core outcome
        let core_outcome = match outcome {
            TaskOutcome::Success => CoreTaskOutcome::Success {
                verdict: "Task completed successfully via CLI".to_string(),
                artifacts: vec![],
            },
            TaskOutcome::PartialSuccess => CoreTaskOutcome::PartialSuccess {
                verdict: "Task completed with partial success via CLI".to_string(),
                completed: vec!["Marked as partial success by user".to_string()],
                failed: vec![],
            },
            TaskOutcome::Failure => CoreTaskOutcome::Failure {
                reason: "Task failed via CLI".to_string(),
                error_details: Some("Marked as failed by user".to_string()),
            },
        };

        // Complete the episode
        memory.complete_episode(episode_uuid, core_outcome).await
            .with_context(|| format!("Failed to complete episode: {}", episode_id))?;

        // Return success
        #[derive(Debug, serde::Serialize)]
        struct CompleteResult {
            episode_id: String,
            status: String,
            outcome: String,
        }

        impl Output for CompleteResult {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;
                writeln!(writer, "{}", "Episode Completed".green().bold())?;
                writeln!(writer, "Episode: {}", self.episode_id.dimmed())?;
                writeln!(writer, "Status: {}", self.status.green())?;
                writeln!(writer, "Outcome: {}", self.outcome)?;
                Ok(())
            }
        }

        let result = CompleteResult {
            episode_id: episode_id.clone(),
            status: "completed".to_string(),
            outcome: format!("{:?}", outcome),
        };

        format.print_output(&result)
    }
}

pub async fn search_episodes(
    query: String,
    limit: usize,
    config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use memory_core::{SelfLearningMemory, MemoryConfig, TaskContext};
    #[cfg(feature = "turso")]
    use memory_storage_turso::TursoStorage;
    #[cfg(feature = "redb")]
    use memory_storage_redb::RedbStorage;
    use std::sync::Arc;

    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!("Turso storage feature not enabled. Use --features turso to enable."));

    #[cfg(feature = "turso")]
    {
        // Create storage backends
        let turso_url = config.database.turso_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Turso database URL not configured"))?;
        let token = config.database.turso_token.as_deref().unwrap_or("");

        let turso_storage = TursoStorage::new(turso_url, token).await?;
        let turso = Arc::new(turso_storage);

        #[cfg(feature = "redb")]
        let cache_storage = if let Some(path) = &config.database.redb_path {
            Some(Arc::new(RedbStorage::new(std::path::Path::new(path)).await?))
        } else {
            None
        };

        // Create memory system
        let memory_config = MemoryConfig::default();
        #[cfg(feature = "redb")]
        let memory = if let Some(cache) = cache_storage {
            SelfLearningMemory::with_storage(memory_config, turso, cache)
        } else {
            SelfLearningMemory::with_storage(memory_config, turso, Arc::new(RedbStorage::new(std::path::Path::new(":memory:")).await?))
        };

        #[cfg(not(feature = "redb"))]
        let memory = SelfLearningMemory::with_storage(memory_config, turso, Arc::new(memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:")).await?));

        // Search for relevant episodes
        let context = TaskContext::default(); // Use default context for search
        let episodes = memory.retrieve_relevant_context(query.clone(), context, limit).await;

        // Convert to summary format
        let episode_summaries: Vec<EpisodeSummary> = episodes
            .into_iter()
            .map(|episode| {
                let status = if episode.is_complete() { "completed" } else { "in_progress" };
                let duration_ms = episode.end_time.map(|end| {
                    (end - episode.start_time).num_milliseconds() as u64
                });

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

        let list = EpisodeList {
            episodes: episode_summaries,
            total_count: episodes.len(), // For search, we don't know total count
        };

        format.print_output(&list)
    }
}

pub async fn log_step(
    episode_id: String,
    tool: String,
    action: String,
    success: bool,
    latency_ms: Option<u64>,
    tokens: Option<u32>,
    observation: Option<String>,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    use memory_core::{SelfLearningMemory, MemoryConfig, ExecutionStep, ExecutionResult};
    #[cfg(feature = "turso")]
    use memory_storage_turso::TursoStorage;
    #[cfg(feature = "redb")]
    use memory_storage_redb::RedbStorage;
    use std::sync::Arc;
    use uuid::Uuid;

    if dry_run {
        println!("Would log step for episode {}: tool={}, action={}, success={}",
                episode_id, tool, action, success);
        return Ok(());
    }

    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!("Turso storage feature not enabled. Use --features turso to enable."));

    #[cfg(feature = "turso")]
    {
        // Parse episode ID
        let episode_uuid = Uuid::parse_str(&episode_id)
            .with_context(|| format!("Invalid episode ID format: {}", episode_id))?;

        // Create storage backends
        let turso_url = config.database.turso_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Turso database URL not configured"))?;
        let token = config.database.turso_token.as_deref().unwrap_or("");

        let turso_storage = TursoStorage::new(turso_url, token).await?;
        let turso = Arc::new(turso_storage);

        #[cfg(feature = "redb")]
        let cache_storage = if let Some(path) = &config.database.redb_path {
            Some(Arc::new(RedbStorage::new(std::path::Path::new(path)).await?))
        } else {
            None
        };

        // Create memory system
        let memory_config = MemoryConfig::default();
        #[cfg(feature = "redb")]
        let memory = if let Some(cache) = cache_storage {
            SelfLearningMemory::with_storage(memory_config, turso, cache)
        } else {
            SelfLearningMemory::with_storage(memory_config, turso, Arc::new(RedbStorage::new(std::path::Path::new(":memory:")).await?))
        };

        #[cfg(not(feature = "redb"))]
        let memory = SelfLearningMemory::with_storage(memory_config, turso, Arc::new(memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:")).await?));

        // Get the current episode to determine step number
        let episode = memory.get_episode(episode_uuid).await
            .with_context(|| format!("Failed to retrieve episode: {}", episode_id))?;

        let step_number = episode.steps.len() + 1;

        // Create execution step
        let mut step = ExecutionStep::new(step_number, tool, action);

        // Set result based on success flag
        step.result = Some(if success {
            ExecutionResult::Success {
                output: observation.unwrap_or_else(|| "Step completed successfully".to_string()),
            }
        } else {
            ExecutionResult::Error {
                message: observation.unwrap_or_else(|| "Step failed".to_string()),
            }
        });

        // Set optional metadata
        if let Some(latency) = latency_ms {
            step.metadata.insert("latency_ms".to_string(), latency.to_string());
        }
        if let Some(token_count) = tokens {
            step.metadata.insert("tokens".to_string(), token_count.to_string());
        }

        // Log the step
        memory.log_step(episode_uuid, step).await;

        // Return success
        #[derive(Debug, serde::Serialize)]
        struct LogStepResult {
            episode_id: String,
            step_number: usize,
            tool: String,
            action: String,
            success: bool,
            status: String,
        }

        impl Output for LogStepResult {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;
                writeln!(writer, "{}", "Step Logged".green().bold())?;
                writeln!(writer, "Episode: {}", self.episode_id.dimmed())?;
                writeln!(writer, "Step: {}", self.step_number)?;
                writeln!(writer, "Tool: {}", self.tool)?;
                writeln!(writer, "Action: {}", self.action)?;
                writeln!(writer, "Success: {}", if self.success { "Yes".green() } else { "No".red() })?;
                Ok(())
            }
        }

        let result = LogStepResult {
            episode_id: episode_id.clone(),
            step_number,
            tool: tool.clone(),
            action: action.clone(),
            success,
            status: "logged".to_string(),
        };

        format.print_output(&result)
    }
}