# memory-core

[![Crates.io](https://img.shields.io/crates/v/memory-core.svg)](https://crates.io/crates/memory-core)
[![Documentation](https://docs.rs/memory-core/badge.svg)](https://docs.rs/memory-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Core episodic learning system for AI agents with pattern extraction, reward scoring, and dual storage backend.

## Overview

`memory-core` provides the foundation for building AI agents that learn from their execution history. It implements a complete episode lifecycle (start → execute → score → learn → retrieve) with intelligent pattern extraction and context-aware memory retrieval.

## Features

- **Episode Management**: Track AI agent execution from start to completion with detailed step logging
- **Pattern Extraction**: Automatically extract ToolSequences, DecisionPoints, ErrorRecovery, and ContextPatterns
- **Intelligent Reward Scoring**: Sophisticated reward calculation with efficiency, complexity, and quality bonuses
- **Smart Reflection**: Generate actionable insights from completed episodes
- **Dual Storage**: Seamless integration with Turso/libSQL (durable) and redb (cache) backends
- **Async Pattern Learning**: Queue-based pattern extraction with worker pool and backpressure handling

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
memory-core = "0.1"
memory-storage-turso = "0.1"
memory-storage-redb = "0.1"
```

### Basic Usage

```rust
use memory_core::{SelfLearningMemory, TaskContext, TaskType, ExecutionStep};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize memory system
    let memory = SelfLearningMemory::new(Default::default()).await?;

    // Start an episode
    let context = TaskContext {
        language: "rust".to_string(),
        domain: "web".to_string(),
        tags: vec!["api".to_string()],
    };

    let episode_id = memory.start_episode(
        "Build REST API endpoint".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    ).await;

    // Log execution steps
    let step = ExecutionStep {
        step_number: 1,
        timestamp: chrono::Utc::now(),
        tool: "rustc".to_string(),
        action: "compile".to_string(),
        parameters: serde_json::json!({}),
        result: Some("Compiled successfully".to_string()),
        latency_ms: 1250,
        tokens_used: Some(2500),
        metadata: Default::default(),
    };

    memory.log_step(episode_id, step).await;

    // Complete episode with scoring
    let outcome = TaskOutcome::Success {
        verdict: "Endpoint created successfully".to_string(),
        artifacts: vec![],
    };

    let completed = memory.complete_episode(episode_id, outcome).await?;

    println!("Episode completed with reward: {}", completed.reward.unwrap().total);

    // Retrieve similar past episodes
    let relevant = memory.retrieve_relevant_context(
        "Build REST endpoint".to_string(),
        context,
        5,
    ).await?;

    println!("Found {} relevant episodes", relevant.len());
    Ok(())
}
```

## Core Concepts

### Episodes

An episode represents a complete task execution with:
- Unique ID and timestamps
- Task context (language, domain, tags)
- Execution steps with tool usage and outcomes
- Reward score and reflection upon completion
- Extracted patterns for future learning

### Patterns

Four types of patterns are automatically extracted:
- **ToolSequence**: Common sequences of tool usage
- **DecisionPoint**: Critical decision moments and their outcomes
- **ErrorRecovery**: Successful error handling strategies
- **ContextPattern**: Recurring contextual features

### Reward & Reflection

Episodes are scored based on:
- Base reward from outcome (success/partial/failure)
- Efficiency multiplier (time + step count)
- Complexity bonus (task difficulty)
- Quality multipliers (code quality, test coverage, error handling)
- Learning bonuses (diverse tools, pattern usage, error recovery)

Reflections include:
- Success pattern identification
- Improvement opportunity analysis
- Key insight extraction
- Contextual recommendations

## Storage Backends

`memory-core` works with two storage backends:

- **[memory-storage-turso](https://crates.io/crates/memory-storage-turso)**: Turso/libSQL for durable, distributed SQL storage
- **[memory-storage-redb](https://crates.io/crates/memory-storage-redb)**: redb for fast embedded key-value caching

## Performance

All operations meet or exceed performance targets:

| Operation | Target (P95) | Typical Performance |
|-----------|-------------|---------------------|
| Episode Creation | < 50ms | ~2.5 µs (19,531x faster) |
| Step Logging | < 20ms | ~1.1 µs (17,699x faster) |
| Episode Completion | < 500ms | ~3.8 µs (130,890x faster) |
| Pattern Extraction | < 1000ms | ~10.4 µs (95,880x faster) |
| Memory Retrieval | < 100ms | ~721 µs (138x faster) |

## Documentation

Comprehensive API documentation is available at [docs.rs/memory-core](https://docs.rs/memory-core).

## Testing

Run the test suite:

```bash
cargo test -p memory-core
```

## License

Licensed under the MIT License. See [LICENSE](../LICENSE) for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## Project

This crate is part of the [rust-self-learning-memory](https://github.com/d-o-hub/rust-self-learning-memory) project.
