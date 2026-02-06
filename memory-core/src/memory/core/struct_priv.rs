//! SelfLearningMemory struct definition and core fields.
//!
//! Contains the main memory struct with all its fields.

use crate::embeddings::{EmbeddingConfig, SemanticService};
use crate::episode::Episode;
use crate::pattern::PatternId;
use crate::storage::StorageBackend;
use crate::types::{MemoryConfig, TaskContext};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};

use super::super::step_buffer::StepBuffer;
use crate::learning::queue::{PatternExtractionQueue, QueueConfig};
use crate::monitoring::{AgentMetrics, AgentMonitor};
use crate::pattern::{Heuristic, Pattern};
use crate::patterns::extractors::HeuristicExtractor;
use crate::pre_storage::{QualityAssessor, SalientExtractor};
use crate::reflection::ReflectionGenerator;
use crate::reward::RewardCalculator;
use uuid::Uuid;

/// Main self-learning memory system with semantic search capabilities.
///
/// See `memory-core/src/memory/mod.rs` for full documentation.
#[derive(Clone)]
pub struct SelfLearningMemory {
    /// Configuration
    #[allow(dead_code)]
    config: MemoryConfig,
    /// Quality assessor for pre-storage reasoning
    pub(super) quality_assessor: QualityAssessor,
    /// Salient feature extractor for pre-storage reasoning
    pub(super) salient_extractor: SalientExtractor,
    /// Reward calculator
    pub(super) reward_calculator: RewardCalculator,
    /// Reflection generator
    pub(super) reflection_generator: ReflectionGenerator,
    /// Pattern extractor
    pub(super) pattern_extractor: PatternExtractor,
    /// Heuristic extractor
    pub(super) heuristic_extractor: HeuristicExtractor,
    /// Agent monitoring system
    pub(super) agent_monitor: AgentMonitor,
    /// Durable storage backend (Turso)
    pub(super) turso_storage: Option<Arc<dyn StorageBackend>>,
    /// Cache storage backend (redb)
    pub(super) cache_storage: Option<Arc<dyn StorageBackend>>,
    /// In-memory fallback for episodes (used when no storage configured)
    pub(super) episodes_fallback: Arc<RwLock<HashMap<Uuid, Arc<Episode>>>>,
    /// In-memory fallback for patterns (used when no storage configured)
    pub(super) patterns_fallback: Arc<RwLock<HashMap<PatternId, Pattern>>>,
    /// In-memory fallback for heuristics (used when no storage configured)
    pub(super) heuristics_fallback: Arc<RwLock<HashMap<Uuid, Heuristic>>>,
    /// Async pattern extraction queue (optional)
    pub(super) pattern_queue: Option<Arc<PatternExtractionQueue>>,
    /// Step buffers for batching I/O operations
    pub(super) step_buffers: Arc<RwLock<HashMap<Uuid, StepBuffer>>>,
    /// Semaphore to limit concurrent cache operations
    #[allow(dead_code)]
    pub(super) cache_semaphore: Arc<Semaphore>,
    /// Capacity manager for episodic storage (GENESIS)
    pub(super) capacity_manager: Option<crate::episodic::CapacityManager>,
    /// Semantic summarizer for episode compression (GENESIS)
    pub(super) semantic_summarizer: Option<crate::semantic::SemanticSummarizer>,
    /// Spatiotemporal index for hierarchical retrieval (Spatiotemporal)
    pub(super) spatiotemporal_index:
        Option<Arc<RwLock<crate::spatiotemporal::SpatiotemporalIndex>>>,
    /// Hierarchical retriever for efficient episode search
    pub(super) hierarchical_retriever: Option<crate::spatiotemporal::HierarchicalRetriever>,
    /// Diversity maximizer using MMR for result set optimization
    pub(super) diversity_maximizer: Option<crate::spatiotemporal::DiversityMaximizer>,
    /// Context-aware embeddings for task-specific similarity
    #[allow(dead_code)]
    pub(super) context_aware_embeddings: Option<crate::spatiotemporal::ContextAwareEmbeddings>,
    /// Semantic service for embedding generation and search
    #[allow(dead_code)]
    semantic_service: Option<Arc<SemanticService>>,
    /// Configuration for semantic search
    #[allow(dead_code)]
    semantic_config: EmbeddingConfig,
    /// Query cache for retrieval performance (v0.1.12)
    query_cache: Arc<crate::retrieval::QueryCache>,
    /// DBSCAN anomaly detector for identifying unusual episodes
    dbscan_detector: crate::patterns::DBSCANAnomalyDetector,
    /// Audit logger for security compliance and incident investigation
    pub(super) audit_logger: crate::security::audit::AuditLogger,
}
