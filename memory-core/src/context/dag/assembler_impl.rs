//! Implementation of DagContextAssembler and SharedContextItem.
//!
//! Extracted from assembler.rs to maintain the ≤500 LOC invariant.

use std::fmt::Write;
use std::sync::Arc;

use super::super::state::StateDag;
use super::{
    AssembledContext, AssemblyFormat, DagAssemblyConfig, SharedContextItem, UniqueContextItem,
};
use crate::episode::Episode;
use chrono::Utc;
use uuid::Uuid;

// ── Token estimation heuristics ──
// These constants are defined as `pub(super)` in the parent `assembler`
// module.  We reference them via `super::*` to avoid redefinition.

impl super::DagContextAssembler {
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
        let episode_ids: std::collections::HashSet<Uuid> =
            episodes.iter().map(|e| e.episode_id).collect();

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
    fn extract_shared_context(
        &self,
        episode_ids: &std::collections::HashSet<Uuid>,
    ) -> Vec<SharedContextItem> {
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
        let shared_values: std::collections::HashSet<String> =
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
        shared_values: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut unique = Vec::new();

        // Check each context field
        if let Some(ref lang) = episode.context.language {
            if !shared_values.contains(lang) {
                unique.push(format!("language:{lang}"));
            }
        }

        if let Some(ref fw) = episode.context.framework {
            if !shared_values.contains(fw) {
                unique.push(format!("framework:{fw}"));
            }
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
                    .map(|i| {
                        i.value.len() / super::CHARS_PER_TOKEN + super::SHARED_ITEM_BASE_TOKENS
                    })
                    .sum()
            }
            AssemblyFormat::TokenOptimized => {
                if items.is_empty() {
                    return 0;
                }
                // One shared block: type + value per item + header overhead
                items
                    .iter()
                    .map(|i| {
                        i.value.len() / super::CHARS_PER_TOKEN + super::SHARED_ITEM_TOKEN_OPT_BASE
                    })
                    .sum::<usize>()
                    + super::TOKEN_OPTIMIZED_HEADER_TOKENS
            }
        }
    }

    /// Estimate tokens for unique context.
    fn estimate_unique_tokens(&self, items: &[UniqueContextItem]) -> usize {
        items
            .iter()
            .map(|i| {
                let desc_tokens = i.task_description.len() / super::CHARS_PER_TOKEN;
                let unique_tokens = i.unique_aspects.len() * super::TOKENS_PER_TAG;
                desc_tokens + unique_tokens + super::UNIQUE_ITEM_OVERHEAD_TOKENS
            })
            .sum()
    }

    /// Estimate original tokens (without deduplication).
    ///
    /// Uses the heuristics defined in the module-level constants
    /// (`DEFAULT_CONTEXT_TOKENS`, `CHARS_PER_TOKEN`, `TOKENS_PER_TAG`) to
    /// approximate the token cost of representing every episode's full context
    /// without any sharing.
    fn estimate_original_tokens(&self, episodes: &[Arc<Episode>]) -> usize {
        episodes
            .iter()
            .map(|ep| {
                // Full context per episode: see
                //   Episode.context.language, .domain, .task_type, .complexity,
                //   .tags, .task_description
                let context_tokens = super::DEFAULT_CONTEXT_TOKENS;
                let desc_tokens = ep.task_description.len() / super::CHARS_PER_TOKEN;
                let tag_tokens = ep.context.tags.len() * super::TOKENS_PER_TAG;
                context_tokens + desc_tokens + tag_tokens
            })
            .sum()
    }

    /// Format assembled context as string for prompt.
    #[must_use]
    pub fn format_for_prompt(&self, assembled: &AssembledContext) -> String {
        match self.config.format {
            AssemblyFormat::Compact => self.format_compact(assembled),
            AssemblyFormat::Full => self.format_full(assembled),
            AssemblyFormat::TokenOptimized => self.format_optimized(assembled),
        }
    }

    /// Format in compact mode (minimal).
    fn format_compact(&self, assembled: &AssembledContext) -> String {
        let shared = assembled
            .shared_context
            .iter()
            .map(|s| format!("{}:{}", s.node_type_name(), s.node_id))
            .collect::<Vec<_>>()
            .join(",");

        let unique = assembled
            .unique_context
            .iter()
            .map(|u| {
                format!(
                    "{}:{}",
                    u.episode_id,
                    u.task_description.chars().take(30).collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("|");

        format!("S:{shared}\nU:{unique}")
    }

    /// Format in full mode (verbose).
    fn format_full(&self, assembled: &AssembledContext) -> String {
        let mut output = String::new();

        output.push_str("## Shared Context\n");
        for item in &assembled.shared_context {
            writeln!(
                output,
                "- {} = {} (shared by {} episodes)",
                item.node_type_name(),
                item.value,
                item.shared_count
            )
            .unwrap();
        }

        output.push_str("\n## Episode Context\n");
        for item in &assembled.unique_context {
            writeln!(
                output,
                "- Episode {}: {}",
                item.episode_id, item.task_description
            )
            .unwrap();
            if !item.unique_aspects.is_empty() {
                writeln!(output, "  Unique: {}", item.unique_aspects.join(", ")).unwrap();
            }
        }

        output
    }

    /// Format in token-optimized mode.
    fn format_optimized(&self, assembled: &AssembledContext) -> String {
        let mut output = String::new();

        // Shared block (one copy, referenced)
        if !assembled.shared_context.is_empty() {
            output.push_str("SHARED:\n");
            for item in &assembled.shared_context {
                writeln!(output, "{}={}", item.node_type_name(), item.value).unwrap();
            }
        }

        // Unique per episode
        for item in &assembled.unique_context {
            writeln!(
                output,
                "EP:{}|{}",
                item.episode_id,
                item.task_description.chars().take(50).collect::<String>()
            )
            .unwrap();
        }

        output
    }

    /// Calculate reduction percentage.
    #[must_use]
    pub fn reduction_percentage(&self, assembled: &AssembledContext) -> f32 {
        if assembled.token_savings == 0 {
            return 0.0;
        }
        let original = assembled.estimated_tokens + assembled.token_savings;
        if original == 0 {
            return 0.0;
        }
        (assembled.token_savings as f32 / original as f32) * 100.0
    }
}

impl super::SharedContextItem {
    /// Get human-readable type name.
    #[must_use]
    pub fn node_type_name(&self) -> &'static str {
        use super::super::node::StateNodeType;
        match self.node_type {
            StateNodeType::Language => "lang",
            StateNodeType::Framework => "fw",
            StateNodeType::Domain => "domain",
            StateNodeType::TaskType => "type",
            StateNodeType::Complexity => "complexity",
            StateNodeType::Tag => "tag",
            StateNodeType::Composite => "composite",
        }
    }
}
