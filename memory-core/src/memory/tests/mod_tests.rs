//! Tests for SelfLearningMemory constructors and accessors in memory/mod.rs + init.rs
use crate::memory::SelfLearningMemory;
use crate::memory::init::default_db_path;
use crate::types::MemoryConfig;

#[test]
fn test_default_db_path_is_non_empty() {
    let path = default_db_path();
    assert!(path.to_string_lossy().contains("memory.db"));
}

#[test]
fn test_new_default_has_no_storage_backends() {
    let memory = SelfLearningMemory::new();
    let (primary, cache) = memory.storage_backends();
    assert!(primary.is_none());
    assert!(cache.is_none());
}

#[test]
fn test_with_config_respects_quality_threshold() {
    let config = MemoryConfig {
        quality_threshold: 0.42,
        ..MemoryConfig::default()
    };
    let memory = SelfLearningMemory::with_config(config);
    assert!((memory.quality_threshold() - 0.42).abs() < f32::EPSILON);
}

#[test]
fn test_with_semantic_config_does_not_panic() {
    let config = MemoryConfig::default();
    let sem_config = crate::embeddings::EmbeddingConfig::default();
    let _ = SelfLearningMemory::with_semantic_config(config, sem_config);
}

#[test]
fn test_batch_config_some_by_default() {
    let memory = SelfLearningMemory::new();
    assert!(memory.batch_config().is_some());
}

#[test]
fn test_default_impl() {
    // Exercises the Default trait (calls new())
    let _: SelfLearningMemory = SelfLearningMemory::default();
}
