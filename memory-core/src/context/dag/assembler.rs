//! DAG-based context assembler (WG-134) — assembles episode context by
//! traversing StateDag to deduplicate shared attributes (~86% token reduction).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use super::node::StateNodeType;
use super::state::StateDag;

use crate::episode::Episode;
use std::sync::Arc;

// ── Token estimation heuristics ──
// These constants calibrate the approximate token cost of context elements.
// They are simple heuristics, not precise counters — the goal is to guide
// deduplication decisions, not to match a specific tokenizer byte-for-byte.

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

impl DagContextAssembler {
    /// Create a new assembler with given DAG.
    pub fn new(dag: StateDag) -> Self {
        Self {
            dag,
            config: DagAssemblyConfig::default(),
        }
    }

    /// Create assembler with specific config.
    pub fn with_config(dag: StateDag, config: DagAssemblyConfig) -> Self {
        Self { dag, config }
    }

    /// Create a new assembler with default empty DAG.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            dag: StateDag::new(),
            config: DagAssemblyConfig::default(),
        }
    }

    /// Get the underlying DAG.
    #[must_use]
    pub fn dag(&self) -> &StateDag {
        &self.dag
    }

    /// Get mutable DAG for registration.
    pub fn dag_mut(&mut self) -> &mut StateDag {
        &mut self.dag
    }

    /// Register episodes in the DAG.
    pub fn register_episodes(&mut self, episodes: &[Arc<Episode>]) {
        for episode in episodes {
            self.dag.register_episode(episode.as_ref());
        }
    }

    /// Assemble context for a set of episodes.
    ///
    /// Traverses DAG to deduplicate shared context and
    /// build minimal representation.
    #[must_use]
    pub fn assemble(&self, episodes: &[Arc<Episode>]) -> AssembledContext {
        let episode_ids: HashSet<Uuid> = episodes.iter().map(|e| e.episode_id).collect();

        // Find shared context
        let shared_context = if self.config.deduplicate_shared {
            self.extract_shared_context(&episode_ids)
        } else {
            Vec::new()
        };

        // Extract unique context per episode
        let unique_context = if self.config.include_unique {
            self.extract_unique_context(episodes, &shared_context)
        } else {
            Vec::new()
        };

        // Calculate tokens
        let shared_tokens = self.estimate_shared_tokens(&shared_context);
        let unique_tokens = self.estimate_unique_tokens(&unique_context);
        let estimated_tokens = shared_tokens + unique_tokens;

        // Calculate savings (use saturating_sub to avoid underflow
        // when estimation heuristics differ)
        let original_tokens = self.estimate_original_tokens(episodes);
        let token_savings = original_tokens.saturating_sub(estimated_tokens);

        AssembledContext {
            shared_context,
            unique_context,
            estimated_tokens,
            token_savings,
            episode_ids,
            assembled_at: Utc::now(),
        }
    }

    /// Extract shared context from DAG.
    fn extract_shared_context(&self, episode_ids: &HashSet<Uuid>) -> Vec<SharedContextItem> {
        let ids: Vec<Uuid> = episode_ids.iter().copied().collect();
        let shared_nodes = self.dag.get_shared_context(&ids);

        shared_nodes
            .into_iter()
            .filter(|n| n.ref_count() >= self.config.min_shared_threshold)
            .map(|n| SharedContextItem {
                node_type: n.node_type,
                value: n.value.clone(),
                shared_count: n.ref_count(),
                node_id: n.node_id,
            })
            .collect()
    }

    /// Extract unique context per episode.
    fn extract_unique_context(
        &self,
        episodes: &[Arc<Episode>],
        shared_context: &[SharedContextItem],
    ) -> Vec<UniqueContextItem> {
        let shared_values: HashSet<String> =
            shared_context.iter().map(|s| s.value.clone()).collect();

        episodes
            .iter()
            .take(self.config.max_unique_items)
            .map(|ep| {
                let unique_aspects = self.find_unique_aspects(ep.as_ref(), &shared_values);
                UniqueContextItem {
                    episode_id: ep.episode_id,
                    task_description: ep.task_description.clone(),
                    unique_aspects,
                }
            })
            .collect()
    }

    /// Find aspects unique to this episode (not in shared context).
    fn find_unique_aspects(
        &self,
        episode: &Episode,
        shared_values: &HashSet<String>,
    ) -> Vec<String> {
        let mut unique = Vec::new();

        // Check each context field
        if let Some(ref lang) = episode.context.language
            && !shared_values.contains(lang)
        {
            unique.push(format!("language:{lang}"));
        }

        if let Some(ref fw) = episode.context.framework
            && !shared_values.contains(fw)
        {
            unique.push(format!("framework:{fw}"));
        }

        if !shared_values.contains(&episode.context.domain) {
            let domain = &episode.context.domain;
            unique.push(format!("domain:{domain}"));
        }

        let complexity_str = format!("{:?}", episode.context.complexity);
        if !shared_values.contains(&complexity_str) {
            unique.push(format!("complexity:{complexity_str}"));
        }

        for tag in &episode.context.tags {
            if !shared_values.contains(tag) {
                unique.push(format!("tag:{tag}"));
            }
        }

        unique
    }

    /// Estimate tokens for shared context.
    fn estimate_shared_tokens(&self, items: &[SharedContextItem]) -> usize {
        match self.config.format {
            AssemblyFormat::Compact => {
                // Just node IDs: ~1 token per ref
                items.len()
            }
            AssemblyFormat::Full => {
                // Full values: base tokens + value-chars estimate per item
                items
                    .iter()
                    .map(|i| i.value.len() / CHARS_PER_TOKEN + SHARED_ITEM_BASE_TOKENS)
                    .sum()
            }
            AssemblyFormat::TokenOptimized => {
                if items.is_empty() {
                    return 0;
                }
                // One shared block: type + value per item + header overhead
                items
                    .iter()
                    .map(|i| i.value.len() / CHARS_PER_TOKEN + SHARED_ITEM_TOKEN_OPT_BASE)
                    .sum::<usize>()
                    + TOKEN_OPTIMIZED_HEADER_TOKENS
            }
        }
    }

    /// Estimate tokens for unique context.
    fn estimate_unique_tokens(&self, items: &[UniqueContextItem]) -> usize {
        items
            .iter()
            .map(|i| {
                let desc_tokens = i.task_description.len() / CHARS_PER_TOKEN;
                let unique_tokens = i.unique_aspects.len() * TOKENS_PER_TAG;
                desc_tokens + unique_tokens + UNIQUE_ITEM_OVERHEAD_TOKENS
            })
            .sum()
    }

    /// Estimate original tokens (without deduplication), using the module-level
    /// heuristics (`DEFAULT_CONTEXT_TOKENS`, `CHARS_PER_TOKEN`, `TOKENS_PER_TAG`) to
    /// approximate the token cost of every episode's full context without sharing.
    fn estimate_original_tokens(&self, episodes: &[Arc<Episode>]) -> usize {
        episodes
            .iter()
            .map(|ep| {
                // Full context per episode: see
                //   Episode.context.language, .domain, .task_type, .complexity,
                //   .tags, .task_description
                let context_tokens = DEFAULT_CONTEXT_TOKENS;
                let desc_tokens = ep.task_description.len() / CHARS_PER_TOKEN;
                let tag_tokens = ep.context.tags.len() * TOKENS_PER_TAG;
                context_tokens + desc_tokens + tag_tokens
            })
            .sum()
    }

    /// Format assembled context as string for prompt.
    #[must_use]
    pub fn format_for_prompt(&self, assembled: &AssembledContext) -> String {
        super::format::format_for_prompt(self.config.format, assembled)
    }

    /// Calculate reduction percentage.
    #[must_use]
    pub fn reduction_percentage(&self, assembled: &AssembledContext) -> f32 {
        super::format::reduction_percentage(assembled)
    }
}
