//! DAG-based context assembler (WG-134).
//!
//! Assembles episode context by traversing the StateDag to
//! deduplicate shared attributes, achieving ~86% token reduction.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use super::node::StateNodeType;
use super::state::StateDag;

// ── Token estimation heuristics ──
// These constants calibrate the approximate token cost of various context
// elements.  They are intentionally simple heuristics, not precise counters;
// the goal is to guide deduplication decisions, not to match a specific
// tokenizer byte-for-byte.

/// Approximate characters per token (English text, ~4 chars/token).
pub(super) const CHARS_PER_TOKEN: usize = 4;

/// Base overhead tokens for a single episode's context fields
/// (language, domain, task_type, complexity, etc.).
pub(super) const DEFAULT_CONTEXT_TOKENS: usize = 20;

/// Estimated tokens consumed by a single tag.
pub(super) const TOKENS_PER_TAG: usize = 3;

/// Extra tokens added to the shared-context block header in token-optimized
/// format.
pub(super) const TOKEN_OPTIMIZED_HEADER_TOKENS: usize = 5;

/// Base tokens for one shared-context item (compact / full format).
pub(super) const SHARED_ITEM_BASE_TOKENS: usize = 2;

/// Base tokens for one shared-context item in token-optimised format.
pub(super) const SHARED_ITEM_TOKEN_OPT_BASE: usize = 3;

/// Per-episode overhead tokens for the unique-context section.
pub(super) const UNIQUE_ITEM_OVERHEAD_TOKENS: usize = 2;

/// Configuration for DAG context assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagAssemblyConfig {
    /// Maximum unique context items to include.
    pub max_unique_items: usize,
    /// Include shared context first (deduplicated).
    pub deduplicate_shared: bool,
    /// Include episode-unique context after shared.
    pub include_unique: bool,
    /// Minimum node reference count to consider "shared".
    pub min_shared_threshold: usize,
    /// Format: compact (minimal) vs full (verbose).
    pub format: AssemblyFormat,
}

/// Format for assembled context output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssemblyFormat {
    /// Compact format: node refs only.
    Compact,
    /// Full format: include node values.
    Full,
    /// Token-optimized: minimal tokens.
    TokenOptimized,
}

impl Default for DagAssemblyConfig {
    fn default() -> Self {
        Self {
            max_unique_items: 20,
            deduplicate_shared: true,
            include_unique: true,
            min_shared_threshold: 2,
            format: AssemblyFormat::TokenOptimized,
        }
    }
}

impl DagAssemblyConfig {
    /// Create config optimized for token efficiency.
    #[must_use]
    pub fn token_efficient() -> Self {
        Self {
            max_unique_items: 10,
            deduplicate_shared: true,
            include_unique: true,
            min_shared_threshold: 1,
            format: AssemblyFormat::TokenOptimized,
        }
    }

    /// Create config for full context (no deduplication).
    #[must_use]
    pub fn full_context() -> Self {
        Self {
            max_unique_items: 50,
            deduplicate_shared: false,
            include_unique: true,
            min_shared_threshold: 1,
            format: AssemblyFormat::Full,
        }
    }
}

/// Assembled context from DAG traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledContext {
    /// Shared context nodes (deduplicated).
    pub shared_context: Vec<SharedContextItem>,
    /// Episode-unique context (not shared).
    pub unique_context: Vec<UniqueContextItem>,
    /// Total estimated token count.
    pub estimated_tokens: usize,
    /// Token savings from deduplication.
    pub token_savings: usize,
    /// Episodes included.
    pub episode_ids: HashSet<Uuid>,
    /// Assembly timestamp.
    pub assembled_at: DateTime<Utc>,
}

/// A shared context item from the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContextItem {
    /// Node type.
    pub node_type: StateNodeType,
    /// Node value.
    pub value: String,
    /// Number of episodes sharing this.
    pub shared_count: usize,
    /// Node ID reference.
    pub node_id: Uuid,
}

/// An episode-unique context item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueContextItem {
    /// Episode ID.
    pub episode_id: Uuid,
    /// Task description (unique).
    pub task_description: String,
    /// Unique aspects not shared.
    pub unique_aspects: Vec<String>,
}

/// Assembler for DAG-based context.
///
/// Traverses StateDag to build minimal, deduplicated context.
pub struct DagContextAssembler {
    dag: StateDag,
    config: DagAssemblyConfig,
}

// Implementation lives in assembler_impl.rs to maintain ≤500 LOC
#[path = "assembler_impl.rs"]
mod assembler_impl;
