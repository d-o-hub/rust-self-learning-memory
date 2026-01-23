# SelfLearningMemory Examples

This document contains detailed examples for using `SelfLearningMemory`.

## Basic Usage (In-Memory)

```rust
use memory_core::{SelfLearningMemory, TaskContext, TaskType, TaskOutcome, ExecutionStep, ExecutionResult};

async fn example() {
    let memory = SelfLearningMemory::new();

    // Start tracking a task
    let episode_id = memory.start_episode(
        "Implement file parser".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    ).await;

    // Log execution steps
    let mut step = ExecutionStep::new(1, "parser".to_string(), "Parse TOML file".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Parsed successfully".to_string(),
    });
    memory.log_step(episode_id, step).await;

    // Complete and learn
    memory.complete_episode(
        episode_id,
        TaskOutcome::Success {
            verdict: "Parser implemented with tests".to_string(),
            artifacts: vec!["parser.rs".to_string()],
        },
    ).await.unwrap();

    // Later: retrieve for similar tasks
    let relevant = memory.retrieve_relevant_context(
        "Parse JSON file".to_string(),
        TaskContext::default(),
        5,
    ).await;
}
```

## With External Storage

```rust,no_run
use memory_core::{SelfLearningMemory, MemoryConfig};
use std::sync::Arc;

async fn example() -> anyhow::Result<()> {
    // In practice, use storage backends like:
    // - memory_storage_turso::TursoStorage for durable SQL storage
    // - memory_storage_redb::RedbStorage for fast key-value cache
    //
    // Example setup:
    // let turso_url = std::env::var("TURSO_URL")?;
    // let turso_backend = memory_storage_turso::TursoStorage::new(&turso_url).await?;
    // let redb_backend = memory_storage_redb::RedbStorage::new("cache.redb").await?;

    // For this example, we assume the backends are already configured
    # let turso_backend: Arc<dyn memory_core::StorageBackend> = todo!("Configure TursoStorage backend");
    # let redb_backend: Arc<dyn memory_core::StorageBackend> = todo!("Configure RedbStorage backend");
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        turso_backend,   // Durable storage
        redb_backend,    // Fast cache
    );
    # Ok(())
    # }
}
```

## Agent Monitoring

```rust,no_run
use memory_core::SelfLearningMemory;
use std::time::Instant;

async fn example() -> anyhow::Result<()> {
    let memory = SelfLearningMemory::new();

    // Track agent execution
    let start = Instant::now();
    // ... agent work ...
    let duration = start.elapsed();

    memory.record_agent_execution("feature-implementer", true, duration).await?;
    # Ok(())
    # }
}
```

## Semantic Pattern Search

```rust,no_run
use memory_core::{SelfLearningMemory, TaskContext};

async fn example() -> anyhow::Result<()> {
    let memory = SelfLearningMemory::new();
    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "web-api".to_string(),
        tags: vec!["rest".to_string()],
        ..Default::default()
    };

    let results = memory.search_patterns_semantic(
        "How to handle API rate limiting with retries",
        context,
        5,
    ).await?;

    for result in results {
        println!("Pattern: {:?}", result.pattern);
        println!("Relevance: {:.2}", result.relevance_score);
        println!("Breakdown: {:?}", result.score_breakdown);
    }
    # Ok(())
    # }
}
```

## Pattern Recommendations

```rust,no_run
use memory_core::{SelfLearningMemory, TaskContext};

async fn example() -> anyhow::Result<()> {
    let memory = SelfLearningMemory::new();
    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "web-api".to_string(),
        tags: vec!["async".to_string()],
        ..Default::default()
    };

    let recommendations = memory.recommend_patterns_for_task(
        "Build an async HTTP client with connection pooling",
        context,
        3,
    ).await?;

    for rec in recommendations {
        println!("Recommended: {:?}", rec.pattern);
        println!("Relevance: {:.2}", rec.relevance_score);
    }
    # Ok(())
    # }
}
```

## Advanced Filtering

```rust,no_run
use memory_core::{SelfLearningMemory, EpisodeFilter, TaskType};

async fn example() -> anyhow::Result<()> {
    let memory = SelfLearningMemory::new();

    // Get successful episodes with specific tags
    let filter = EpisodeFilter::builder()
        .with_any_tags(vec!["async".to_string()])
        .success_only(true)
        .build();

    let episodes = memory.list_episodes_filtered(filter, Some(10), None).await?;
    # Ok(())
    # }
}
```

## Cross-Domain Pattern Discovery

```rust,no_run
use memory_core::{SelfLearningMemory, TaskContext};

async fn example() -> anyhow::Result<()> {
    let memory = SelfLearningMemory::new();
    let target = TaskContext {
        language: Some("rust".to_string()),
        domain: "web-api".to_string(),
        tags: vec![],
        ..Default::default()
    };

    // Find patterns from CLI work that might apply to web APIs
    let analogous = memory.discover_analogous_patterns(
        "cli",
        target,
        5,
    ).await?;
    # Ok(())
    # }
}
```

For more examples, see the [memory/examples](https://github.com/your-repo/memory-core/tree/main/examples) directory.
