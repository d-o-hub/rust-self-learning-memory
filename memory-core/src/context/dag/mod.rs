//! DAG-based state management for episode context (WG-134).
//!
//! Inspired by arXiv:2602.22398: DAG-based state management achieves 86% token
//! reduction by deduplicating shared context between episodes.
//!
//! ## Key Concepts
//!
//! Instead of storing full context for each episode, we:
//! 1. Create DAG nodes for shared context attributes (language, domain, task_type)
//! 2. Episodes reference nodes by ID instead of storing full context strings
//! 3. Context assembly traverses DAG to deduplicate shared attributes
//!
//! ## Architecture
//!
//! ```text
//! Episodes ──► StateDag ──► Deduplicated Context
//!    │           │              │
//!    │           │              │
//!    └──► StateNode (shared) ──► Shared attribute once
//!    │   - language="rust"
//!    │   - domain="web-api"
//!    │   - task_type="Debugging"
//!    │
//!    └──► Episode-unique state stored separately
//! ```
//!
//! ## Token Reduction
//!
//! For N episodes sharing the same language/domain/task_type:
//! - Old: N × (language + domain + task_type) tokens
//! - New: 1 × (language + domain + task_type) + N × node_id refs
//! - Reduction: ~86% when N > 5 and shared context is large

pub mod assembler;
pub mod edge;
pub mod format;
pub mod node;
pub mod state;

#[cfg(test)]
mod tests;

pub use assembler::{AssembledContext, DagAssemblyConfig, DagContextAssembler};
pub use edge::{EdgeMetadata, EdgeType, StateEdge};
pub use node::{NodeId, StateNode, StateNodeType};
pub use state::{DagStats, StateDag};
