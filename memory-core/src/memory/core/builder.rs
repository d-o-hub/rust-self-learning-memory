//! Factory methods and builders for SelfLearningMemory.
//!
//! Contains static creation methods and initialization logic.

use super::struct_priv::SelfLearningMemory;
use crate::embeddings::EmbeddingConfig;
use crate::learning::queue::QueueConfig;
use crate::monitoring::AgentMonitor;
use crate::pattern_extractor::PatternExtractor;
use crate::patterns::extractors::HeuristicExtractor;
use crate::pre_storage::{QualityAssessor, SalientExtractor};
use crate::reflection::ReflectionGenerator;
use crate::reward::RewardCalculator;
use crate::storage::StorageBackend;
use crate::types::MemoryConfig;
use std::sync::Arc;

impl Default for SelfLearningMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfLearningMemory {
    /// Create a new self-learning memory system with default configuration (in-memory only)
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(MemoryConfig::default())
    }

    /// Create a memory system with custom configuration (in-memory only)
    #[must_use]
    pub fn with_config(config: MemoryConfig) -> Self {
        let quality_assessor = QualityAssessor::new(config.quality_config());
        let salient_extractor = SalientExtractor::new();
        let reward_calculator = RewardCalculator::new();
        let reflection_generator = ReflectionGenerator::new();
        let pattern_extractor = PatternExtractor::new();
        let heuristic_extractor = HeuristicExtractor::new();
        let agent_monitor = AgentMonitor::new();
        let dbscan_detector =
            crate::patterns::DBSCANAnomalyDetector::new(crate::patterns::DBSCANConfig::default());

        Self {
            config,
            quality_assessor,
            salient_extractor,
            reward_calculator,
            reflection_generator,
            pattern_extractor,
            heuristic_extractor,
            agent_monitor,
            turso_storage: None,
            cache_storage: None,
            episodes_fallback: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            patterns_fallback: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            heuristics_fallback: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            pattern_queue: None,
            step_buffers: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            cache_semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
            capacity_manager: None,
            semantic_summarizer: None,
            spatiotemporal_index: None,
            hierarchical_retriever: None,
            diversity_maximizer: None,
            context_aware_embeddings: None,
            semantic_service: None,
            semantic_config: EmbeddingConfig::default(),
            query_cache: Arc::new(crate::retrieval::QueryCache::new(
                crate::retrieval::CacheConfig::default(),
            )),
            dbscan_detector,
        }
    }

    /// Create a memory system with storage backends
    pub fn with_storage(
        config: MemoryConfig,
        turso: Arc<dyn StorageBackend>,
        cache: Arc<dyn StorageBackend>,
    ) -> Self {
        let mut memory = Self::with_config(config);
        memory.turso_storage = Some(turso);
        memory.cache_storage = Some(cache);
        memory
    }

    /// Create memory with custom semantic config
    #[must_use]
    pub fn with_semantic_config(config: MemoryConfig, semantic_config: EmbeddingConfig) -> Self {
        let mut memory = Self::with_config(config);
        memory.semantic_config = semantic_config;
        memory
    }

    /// Enable async pattern extraction with a worker pool
    #[must_use]
    pub fn enable_async_extraction(self, queue_config: QueueConfig) -> Self {
        use crate::learning::queue::PatternExtractionQueue;

        let queue = PatternExtractionQueue::new(queue_config, Arc::new(self.clone()));
        let memory = Arc::new(self);
        let queue = Arc::new(queue);

        // Store the queue reference in the memory
        let mut mem = (*memory).clone();
        mem.pattern_queue = Some(queue);

        // We need to re-arc everything since we cloned
        mem
    }

    /// Start async pattern extraction workers
    pub async fn start_workers(&self) {
        if let Some(queue) = &self.pattern_queue {
            queue.start_workers().await;
        }
    }
}
