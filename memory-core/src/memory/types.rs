use crate::embeddings::{EmbeddingConfig, SemanticService};
use crate::episode::{Episode, EpisodeRelationship, PatternId};
use crate::extraction::PatternExtractor;
use crate::learning::queue::PatternExtractionQueue;
use crate::monitoring::AgentMonitor;
use crate::pattern::{Heuristic, Pattern};
use crate::patterns::extractors::HeuristicExtractor;
use crate::pre_storage::{QualityAssessor, SalientExtractor};
use crate::reflection::ReflectionGenerator;
use crate::reward::RewardCalculator;
use crate::security::audit::AuditLogger;
use crate::storage::StorageBackend;
use crate::types::MemoryConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use super::step_buffer::StepBuffer;

/// Main self-learning memory system with semantic search capabilities.
///
/// `SelfLearningMemory` is the primary interface for episodic learning. It manages:
/// - **Episode lifecycle**: Create, track, and complete task executions
/// - **Pre-storage reasoning**: Quality assessment and salient feature extraction (`PREMem`)
/// - **Learning analysis**: Calculate rewards, generate reflections, extract patterns
/// - **Pattern storage**: Persist learnings to durable (Turso) and cache (redb) storage
/// - **Context retrieval**: Find relevant past episodes for new tasks using semantic search
/// - **Agent monitoring**: Track agent utilization, performance, and task completion rates
///
/// # Semantic Search
///
/// When available, `SelfLearningMemory` uses semantic embeddings to find
/// contextually relevant episodes:
/// - **Local provider**: Offline, privacy-preserving (default)
/// - **`OpenAI` provider**: Higher accuracy, requires API key
/// - **Fallback**: Automatic fallback chain (Local → `OpenAI` → Mock)
///
/// # Architecture
///
/// The system uses a dual-storage approach:
/// - **Turso (libSQL)**: Durable, queryable storage for long-term retention
/// - **redb**: Fast embedded cache for hot data and quick lookups
/// - **In-memory**: Fallback when external storage is not configured
///
/// # Learning Cycle
///
/// 1. **Start Episode** - [`start_episode()`](SelfLearningMemory::start_episode) creates a new task record
/// 2. **Log Steps** - [`log_step()`](SelfLearningMemory::log_step) tracks execution steps
/// 3. **Complete** - [`complete_episode()`](SelfLearningMemory::complete_episode) finalizes and analyzes
/// 4. **Retrieve** - [`retrieve_relevant_context()`](SelfLearningMemory::retrieve_relevant_context) queries for similar episodes
/// 5. **Monitor** - [`record_agent_execution()`](SelfLearningMemory::record_agent_execution) tracks agent performance
#[derive(Clone)]
pub struct SelfLearningMemory {
    /// Configuration
    pub(super) config: MemoryConfig,
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
    /// Uses `Arc<Episode>` to avoid cloning when sharing episodes across operations
    pub(super) episodes_fallback: Arc<RwLock<HashMap<Uuid, Arc<Episode>>>>,
    /// In-memory fallback for patterns (used when no storage configured)
    pub(super) patterns_fallback: Arc<RwLock<HashMap<PatternId, Pattern>>>,
    /// In-memory fallback for heuristics (used when no storage configured)
    pub(super) heuristics_fallback: Arc<RwLock<HashMap<Uuid, Heuristic>>>,
    /// In-memory fallback for relationships (used when no storage configured)
    pub(super) relationships_fallback: Arc<RwLock<HashMap<Uuid, EpisodeRelationship>>>,
    /// Async pattern extraction queue (optional)
    pub(super) pattern_queue: Option<Arc<PatternExtractionQueue>>,
    /// Step buffers for batching I/O operations
    pub(super) step_buffers: Arc<RwLock<HashMap<Uuid, StepBuffer>>>,
    /// Semaphore to limit concurrent cache operations and prevent async runtime blocking
    #[allow(dead_code)]
    pub(super) cache_semaphore: Arc<Semaphore>,

    // Phase 2 (GENESIS) - Capacity management
    /// Capacity manager for episodic storage with eviction policies
    pub(super) capacity_manager: Option<crate::episodic::CapacityManager>,

    // Phase 2 (GENESIS) - Semantic summarization
    /// Semantic summarizer for episode compression
    pub(super) semantic_summarizer: Option<crate::semantic::SemanticSummarizer>,

    // Phase 3 (Spatiotemporal) - Hierarchical retrieval and indexing
    /// Spatiotemporal index for domain -> `task_type` -> temporal clustering
    pub(super) spatiotemporal_index:
        Option<Arc<RwLock<crate::spatiotemporal::SpatiotemporalIndex>>>,
    /// Hierarchical retriever for efficient episode search
    pub(super) hierarchical_retriever: Option<crate::spatiotemporal::HierarchicalRetriever>,
    /// Diversity maximizer using MMR for result set optimization
    pub(super) diversity_maximizer: Option<crate::spatiotemporal::DiversityMaximizer>,
    /// Context-aware embeddings for task-specific similarity (future)
    #[allow(dead_code)]
    pub(super) context_aware_embeddings: Option<crate::spatiotemporal::ContextAwareEmbeddings>,

    // Semantic Search Integration
    /// Semantic service for embedding generation and search
    #[allow(dead_code)]
    pub(super) semantic_service: Option<Arc<SemanticService>>,
    /// Configuration for semantic search
    #[allow(dead_code)]
    pub(super) semantic_config: EmbeddingConfig,

    // v0.1.12: Query Caching
    /// Query cache for retrieval performance (LRU + TTL)
    pub(super) query_cache: Arc<crate::retrieval::QueryCache>,

    // Phase 3 (DBSCAN) - Anomaly Detection
    /// DBSCAN anomaly detector for identifying unusual episodes
    #[allow(dead_code)]
    pub(super) dbscan_detector: crate::patterns::DBSCANAnomalyDetector,

    // Security - Audit logging
    /// Audit logger for security compliance and incident investigation
    pub(super) audit_logger: AuditLogger,
}
