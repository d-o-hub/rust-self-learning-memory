//! `SelfLearningMemory` constructors and initialization logic.
//!
//! This module contains the factory methods and configuration for creating
//! `SelfLearningMemory` instances with various storage backends.

use crate::embeddings::EmbeddingConfig;
use crate::extraction::PatternExtractor;
use crate::learning::queue::{PatternExtractionQueue, QueueConfig};
use crate::monitoring::{AgentMonitor, MonitoringConfig, storage::SimpleMonitoringStorage};
use crate::pre_storage::{QualityAssessor, QualityConfig, SalientExtractor};
use crate::reflection::ReflectionGenerator;
use crate::reward::RewardCalculator;
use crate::security::audit::AuditLogger;
use crate::types::{DEFAULT_EVENT_CHANNEL_CAPACITY, MemoryConfig};
use std::collections::HashMap;
use std::path::PathBuf;
#[allow(unused_imports)]
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore, broadcast};

/// Get the default database path in the OS-appropriate data directory.
pub fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("do-memory")
        .join("memory.db")
}

/// Create a memory system with custom configuration (in-memory only)
#[must_use]
pub fn with_config(config: MemoryConfig) -> super::SelfLearningMemory {
    let pattern_extractor =
        PatternExtractor::with_thresholds(config.pattern_extraction_threshold, 5);

    // Initialize quality assessor with configured threshold
    let quality_config = QualityConfig::new(config.quality_threshold);
    let quality_assessor = QualityAssessor::new(quality_config);

    // Initialize salient feature extractor
    let salient_extractor = SalientExtractor::new();

    // Phase 2 (GENESIS) - Initialize capacity manager if max_episodes is configured
    let capacity_manager = if let Some(max_episodes) = config.max_episodes {
        let eviction_policy = config
            .eviction_policy
            .unwrap_or(crate::episode::EvictionPolicy::RelevanceWeighted);
        Some(crate::episode::CapacityManager::new(
            max_episodes,
            eviction_policy,
        ))
    } else {
        None
    };

    // Phase 2 (GENESIS) - Initialize semantic summarizer if enabled
    let semantic_summarizer = if config.enable_summarization {
        Some(crate::semantic::SemanticSummarizer::with_config(
            config.summary_min_length,
            config.summary_max_length,
            5,
        ))
    } else {
        None
    };

    // Phase 3 (Spatiotemporal) - Initialize components if enabled
    let spatiotemporal_index = if config.enable_spatiotemporal_indexing {
        Some(Arc::new(RwLock::new(
            crate::spatiotemporal::SpatiotemporalIndex::new(),
        )))
    } else {
        None
    };

    let hierarchical_retriever = if config.enable_spatiotemporal_indexing {
        Some(crate::spatiotemporal::HierarchicalRetriever::with_config(
            config.temporal_bias_weight,
            config.max_clusters_to_search,
        ))
    } else {
        None
    };

    let diversity_maximizer = if config.enable_diversity_maximization {
        Some(crate::spatiotemporal::DiversityMaximizer::new(
            config.diversity_lambda,
        ))
    } else {
        None
    };

    // Initialize ANN-backed semantic retriever
    let semantic_retriever = if let Some(path) = &config.ann_index_path {
        if let Ok(index) = crate::embeddings::SimpleVectorIndex::load(path) {
            Some(Arc::new(crate::retrieval::SemanticRetriever::new(
                config.clone(),
                Box::new(index),
            )))
        } else {
            Some(Arc::new(crate::retrieval::SemanticRetriever::new(
                config.clone(),
                Box::new(crate::embeddings::SimpleVectorIndex::new()),
            )))
        }
    } else {
        Some(Arc::new(crate::retrieval::SemanticRetriever::new(
            config.clone(),
            Box::new(crate::embeddings::SimpleVectorIndex::new()),
        )))
    };

    // Initialize semantic config (service will be initialized on first use if needed)
    let semantic_config = EmbeddingConfig::default();

    // Semantic service initialized to None (will be created lazily if needed)
    let semantic_service: Option<Arc<crate::embeddings::SemanticService>> = None;

    // Initialize query cache with default settings
    let query_cache = Arc::new(crate::retrieval::QueryCache::new());

    // Phase 3 (DBSCAN) - Initialize anomaly detector
    let dbscan_detector = crate::patterns::DBSCANAnomalyDetector::new();

    // Initialize event broadcast channel
    let (event_sender, _) = broadcast::channel(DEFAULT_EVENT_CHANNEL_CAPACITY);

    // Initialize CloudEvents event emitter based on config mode
    let event_emitter: Arc<dyn crate::types::emitter::EventEmitter> =
        config.event_emitter_mode.build();

    super::SelfLearningMemory {
        config: config.clone(),
        quality_assessor,
        salient_extractor,
        reward_calculator: RewardCalculator::new(),
        reflection_generator: ReflectionGenerator::new(),
        pattern_extractor,
        heuristic_extractor: crate::patterns::extractors::HeuristicExtractor::new(),
        agent_monitor: AgentMonitor::new(),
        turso_storage: None,
        cache_storage: None,
        episodes_fallback: Arc::new(RwLock::new(HashMap::new())),
        patterns_fallback: Arc::new(RwLock::new(HashMap::new())),
        heuristics_fallback: Arc::new(RwLock::new(HashMap::new())),
        procedural_fallback: Arc::new(RwLock::new(HashMap::new())),
        relationships_fallback: Arc::new(RwLock::new(HashMap::new())),
        pattern_queue: None,
        step_buffers: Arc::new(RwLock::new(HashMap::new())),
        cache_semaphore: Arc::new(Semaphore::new(config.concurrency.max_concurrent_cache_ops)),
        event_emitter_semaphore: Arc::new(Semaphore::new(10)),
        capacity_manager,
        semantic_summarizer,
        spatiotemporal_index,
        hierarchical_retriever,
        diversity_maximizer,
        semantic_retriever,
        context_aware_embeddings: None,
        semantic_service,
        semantic_config,
        query_cache,
        dbscan_detector,
        audit_logger: AuditLogger::new(config.audit_config.clone()),
        playbook_generator: super::playbook::PlaybookGenerator::new(),
        summaries_fallback: Arc::new(RwLock::new(HashMap::new())),
        recommendation_tracker: super::attribution::RecommendationTracker::new(),
        event_sender,
        event_emitter,
        pending_eviction_failures: Arc::new(RwLock::new(Vec::new())),
    }
}

/// Create a memory system with storage backends
pub fn with_storage(
    config: MemoryConfig,
    turso: Arc<dyn crate::storage::StorageBackend>,
    cache: Arc<dyn crate::storage::StorageBackend>,
) -> super::SelfLearningMemory {
    let pattern_extractor =
        PatternExtractor::with_thresholds(config.pattern_extraction_threshold, 5);

    // Initialize quality assessor with configured threshold
    let quality_config = QualityConfig::new(config.quality_threshold);
    let quality_assessor = QualityAssessor::new(quality_config);

    // Initialize salient feature extractor
    let salient_extractor = SalientExtractor::new();

    // Configure agent monitor with storage backends
    let monitoring_config = MonitoringConfig {
        enabled: true,
        enable_persistence: true,
        max_records: 1000,
    };

    // Create monitoring storage that uses the primary storage backend
    let monitoring_storage = SimpleMonitoringStorage::new(Arc::clone(&turso));
    let agent_monitor = AgentMonitor::with_storage(monitoring_config, Arc::new(monitoring_storage));

    // Phase 2 (GENESIS) - Initialize capacity manager if max_episodes is configured
    let capacity_manager = if let Some(max_episodes) = config.max_episodes {
        let eviction_policy = config
            .eviction_policy
            .unwrap_or(crate::episode::EvictionPolicy::RelevanceWeighted);
        Some(crate::episode::CapacityManager::new(
            max_episodes,
            eviction_policy,
        ))
    } else {
        None
    };

    // Phase 2 (GENESIS) - Initialize semantic summarizer if enabled
    let semantic_summarizer = if config.enable_summarization {
        Some(crate::semantic::SemanticSummarizer::with_config(
            config.summary_min_length,
            config.summary_max_length,
            5,
        ))
    } else {
        None
    };

    // Phase 3 (Spatiotemporal) - Initialize components if enabled
    let spatiotemporal_index = if config.enable_spatiotemporal_indexing {
        Some(Arc::new(RwLock::new(
            crate::spatiotemporal::SpatiotemporalIndex::new(),
        )))
    } else {
        None
    };

    let hierarchical_retriever = if config.enable_spatiotemporal_indexing {
        Some(crate::spatiotemporal::HierarchicalRetriever::with_config(
            config.temporal_bias_weight,
            config.max_clusters_to_search,
        ))
    } else {
        None
    };

    let diversity_maximizer = if config.enable_diversity_maximization {
        Some(crate::spatiotemporal::DiversityMaximizer::new(
            config.diversity_lambda,
        ))
    } else {
        None
    };

    // Initialize ANN-backed semantic retriever
    let semantic_retriever = if let Some(path) = &config.ann_index_path {
        if let Ok(index) = crate::embeddings::SimpleVectorIndex::load(path) {
            Some(Arc::new(crate::retrieval::SemanticRetriever::new(
                config.clone(),
                Box::new(index),
            )))
        } else {
            Some(Arc::new(crate::retrieval::SemanticRetriever::new(
                config.clone(),
                Box::new(crate::embeddings::SimpleVectorIndex::new()),
            )))
        }
    } else {
        Some(Arc::new(crate::retrieval::SemanticRetriever::new(
            config.clone(),
            Box::new(crate::embeddings::SimpleVectorIndex::new()),
        )))
    };

    // Initialize semantic config (service will be initialized lazily if needed)
    let semantic_config = EmbeddingConfig::default();

    // Semantic service initialized to None (will be created lazily if needed)
    let semantic_service: Option<Arc<crate::embeddings::SemanticService>> = None;

    // Initialize query cache with default settings
    let query_cache = Arc::new(crate::retrieval::QueryCache::new());

    // Phase 3 (DBSCAN) - Initialize anomaly detector
    let dbscan_detector = crate::patterns::DBSCANAnomalyDetector::new();

    // Security - Initialize audit logger
    let audit_logger = AuditLogger::new(config.audit_config.clone());

    // Initialize event broadcast channel
    let (event_sender, _) = broadcast::channel(DEFAULT_EVENT_CHANNEL_CAPACITY);

    // Initialize CloudEvents event emitter based on config mode
    let event_emitter: Arc<dyn crate::types::emitter::EventEmitter> =
        config.event_emitter_mode.build();

    super::SelfLearningMemory {
        config: config.clone(),
        quality_assessor,
        salient_extractor,
        reward_calculator: RewardCalculator::new(),
        reflection_generator: ReflectionGenerator::new(),
        pattern_extractor,
        heuristic_extractor: crate::patterns::extractors::HeuristicExtractor::new(),
        agent_monitor,
        turso_storage: Some(turso),
        cache_storage: Some(cache),
        episodes_fallback: Arc::new(RwLock::new(HashMap::new())),
        patterns_fallback: Arc::new(RwLock::new(HashMap::new())),
        heuristics_fallback: Arc::new(RwLock::new(HashMap::new())),
        procedural_fallback: Arc::new(RwLock::new(HashMap::new())),
        relationships_fallback: Arc::new(RwLock::new(HashMap::new())),
        pattern_queue: None,
        step_buffers: Arc::new(RwLock::new(HashMap::new())),
        cache_semaphore: Arc::new(Semaphore::new(config.concurrency.max_concurrent_cache_ops)),
        event_emitter_semaphore: Arc::new(Semaphore::new(10)),
        capacity_manager,
        semantic_summarizer,
        spatiotemporal_index,
        hierarchical_retriever,
        diversity_maximizer,
        semantic_retriever,
        context_aware_embeddings: None,
        semantic_service,
        semantic_config,
        query_cache,
        dbscan_detector,
        audit_logger,
        playbook_generator: super::playbook::PlaybookGenerator::new(),
        summaries_fallback: Arc::new(RwLock::new(HashMap::new())),
        recommendation_tracker: super::attribution::RecommendationTracker::new(),
        event_sender,
        event_emitter,
        pending_eviction_failures: Arc::new(RwLock::new(Vec::new())),
    }
}
pub fn with_semantic_config(
    config: MemoryConfig,
    semantic_config: EmbeddingConfig,
) -> super::SelfLearningMemory {
    let mut memory = with_config(config);
    memory.semantic_config = semantic_config;
    memory
}
/// Enable async pattern extraction with a worker pool
#[must_use]
pub fn enable_async_extraction(
    memory: super::SelfLearningMemory,
    queue_config: QueueConfig,
) -> super::SelfLearningMemory {
    let memory_arc = Arc::new(memory.clone());
    let queue = Arc::new(PatternExtractionQueue::new(queue_config, memory_arc));
    let mut memory = memory;
    memory.pattern_queue = Some(queue);
    memory
}
/// Start async pattern extraction workers
pub async fn start_workers(memory: &super::SelfLearningMemory) {
    if let Some(queue) = &memory.pattern_queue {
        queue.start_workers().await;
    }
}

/// Stop async pattern extraction workers gracefully.
///
/// Waits for the queue to drain (workers finish processing current items),
/// then signals workers to shut down. Returns `true` if the queue emptied
/// within the timeout, `false` otherwise.
pub async fn stop_workers(memory: &super::SelfLearningMemory, timeout: Duration) -> bool {
    if let Some(queue) = &memory.pattern_queue {
        // Drain first: workers check the shutdown flag between items,
        // so we must let the queue empty before signaling shutdown.
        let drained = queue.wait_until_empty(timeout).await;
        queue.shutdown().await;
        drained
    } else {
        true
    }
}
