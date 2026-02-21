//! Common utilities for memory system benchmarks

use memory_core::{
    memory::SelfLearningMemory,
    types::{ComplexityLevel, MemoryConfig, TaskContext},
};
use memory_storage_redb::RedbStorage;
use memory_storage_turso::TursoStorage;
use std::sync::Arc;
use tempfile::TempDir;

/// Setup helper for benchmarks that need a temporary redb storage
pub async fn setup_temp_memory() -> (SelfLearningMemory, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    // Use separate files for primary and cache to avoid contention and better simulate architecture
    let primary_path = temp_dir.path().join("benchmark_primary.redb");
    let cache_path = temp_dir.path().join("benchmark_cache.redb");

    let memory_config = MemoryConfig {
        quality_threshold: 0.3,
        ..Default::default()
    };

    let primary_storage = RedbStorage::new(&primary_path)
        .await
        .expect("Failed to create primary redb storage");
    let cache_storage = RedbStorage::new(&cache_path)
        .await
        .expect("Failed to create cache redb storage");
    let memory = SelfLearningMemory::with_storage(
        memory_config,
        Arc::new(primary_storage),
        Arc::new(cache_storage),
    );

    (memory, temp_dir)
}

/// Setup helper for benchmarks that need a temporary Turso storage
pub async fn setup_temp_turso_memory() -> (SelfLearningMemory, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("benchmark.db");

    let memory_config = MemoryConfig {
        quality_threshold: 0.3,
        ..Default::default()
    };

    let turso_storage = TursoStorage::new(&format!("file:{}", db_path.to_string_lossy()), "")
        .await
        .expect("Failed to create turso storage");
    turso_storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    let cache_storage = TursoStorage::new(&format!("file:{}", db_path.to_string_lossy()), "")
        .await
        .expect("Failed to create turso storage");
    // Schema already initialized

    let memory = SelfLearningMemory::with_storage(
        memory_config,
        Arc::new(turso_storage),
        Arc::new(cache_storage),
    );

    (memory, temp_dir)
}

/// Create a realistic task context for benchmarks
pub fn create_benchmark_context() -> TaskContext {
    TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        domain: "benchmark".to_string(),
        complexity: ComplexityLevel::Moderate,
        tags: vec!["performance".to_string(), "test".to_string()],
    }
}

/// Generate a realistic episode description
pub fn generate_episode_description(id: usize) -> String {
    format!(
        "Implement feature {} with async processing and error handling",
        id
    )
}

/// Generate realistic execution steps for an episode
pub fn generate_execution_steps(count: usize) -> Vec<memory_core::episode::ExecutionStep> {
    use memory_core::types::ExecutionResult;

    (0..count)
        .map(|i| {
            let mut step = memory_core::episode::ExecutionStep::new(
                i + 1,
                format!("tool_{}", i),
                format!("Execute step {} of the process", i),
            );
            step.result = Some(ExecutionResult::Success {
                output: format!("Step {} completed successfully with output data", i),
            });
            step.latency_ms = 50 + (i as u64 * 10);
            step.tokens_used = Some(100 + (i * 20));
            step
        })
        .collect()
}

/// Generate a large episode with many steps for memory pressure testing
pub fn generate_large_episode_description(id: usize) -> String {
    format!(
        "Implement comprehensive feature {} with extensive async processing, error handling, logging, monitoring, testing, and documentation covering multiple modules and integration points",
        id
    )
}

/// Generate many execution steps for memory pressure testing
pub fn generate_many_execution_steps(count: usize) -> Vec<memory_core::episode::ExecutionStep> {
    use memory_core::types::ExecutionResult;

    (0..count)
        .map(|i| {
            let mut step = memory_core::episode::ExecutionStep::new(
                i + 1,
                format!("comprehensive_tool_{}", i),
                format!("Execute comprehensive step {} involving multiple operations, validations, and transformations", i),
            );
            step.result = Some(ExecutionResult::Success {
                output: format!("Step {} completed successfully with comprehensive output data including metrics, logs, and transformed results from multiple subsystems", i),
            });
            step.latency_ms = 100 + (i as u64 * 5);
            step.tokens_used = Some(500 + (i * 50));
            step
        })
        .collect()
}
