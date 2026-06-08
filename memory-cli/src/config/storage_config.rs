//! Memory configuration creation logic.
//!
//! Separated from storage.rs to keep file sizes within the 500 LOC limit.

use super::types::Config;
use do_memory_core::MemoryConfig;

/// Create memory system configuration
pub fn create_memory_config(config: &Config) -> MemoryConfig {
    MemoryConfig {
        storage: do_memory_core::StorageConfig {
            max_episodes_cache: config.storage.max_episodes_cache,
            sync_interval_secs: 300, // 5 minutes default
            enable_compression: false,
        },
        enable_embeddings: config.embeddings.enabled, // Use config value
        pattern_extraction_threshold: 0.1,
        quality_threshold: 0.0, // Allow CLI workflows to complete minimal episodes
        batch_config: None,     // Disable batching for CLI - each command is a separate process
        concurrency: do_memory_core::ConcurrencyConfig::default(),
        // Phase 2 (GENESIS) - Capacity management
        max_episodes: None, // No capacity limit by default
        eviction_policy: Some(do_memory_core::episodic::EvictionPolicy::RelevanceWeighted),
        // Phase 2 (GENESIS) - Semantic summarization
        enable_summarization: true,
        summary_min_length: 100,
        summary_max_length: 200,
        // Phase 3 (Spatiotemporal) - Hierarchical retrieval
        enable_spatiotemporal_indexing: true,
        enable_diversity_maximization: true,
        diversity_lambda: 0.7,
        temporal_bias_weight: 0.3,
        max_clusters_to_search: 5,
        // Semantic search configuration
        semantic_search_mode: "hybrid".to_string(),
        enable_query_embedding_cache: true,
        semantic_similarity_threshold: 0.7,
        // Audit configuration
        audit_config: do_memory_core::AuditConfig::default(),
        // CloudEvents EventEmitter (WG-149)
        event_emitter_mode: do_memory_core::types::emitter::EventEmitterMode::default(),
    }
}
