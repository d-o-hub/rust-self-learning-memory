//! Actionable Recommendation Playbooks (ADR-044 Feature 1)
//!
//! This module provides template-driven synthesis of actionable playbooks from
//! patterns, reflections, and summaries. Playbooks provide step-by-step guidance
//! for agents, including when to apply, when not to apply, and expected outcomes.
//!
//! # Overview
//!
//! Unlike raw patterns, playbooks synthesize multiple data sources to provide:
//! - Ordered, actionable steps
//! - Context-aware applicability guidance
//! - Expected outcomes and pitfalls to avoid
//!
//! # Template-Driven Synthesis
//!
//! Playbooks are generated using templates - NO LLM on the hot path. The synthesis
//! uses:
//! - Existing pattern types (ToolSequence, DecisionPoint, ErrorRecovery, ContextPattern)
//! - Episode reflections (successes, improvements, insights)
//! - Semantic summaries (key concepts, key steps)
//!
//! # Example
//!
//! ```no_run
//! use memory_core::memory::playbook::{PlaybookGenerator, PlaybookRequest};
//! use memory_core::{TaskContext, TaskType};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let generator = PlaybookGenerator::new();
//!
//! let request = PlaybookRequest {
//!     task_description: "Implement user authentication".to_string(),
//!     domain: "web-api".to_string(),
//!     task_type: TaskType::CodeGeneration,
//!     context: TaskContext::default(),
//!     max_steps: 5,
//! };
//!
//! let playbook = generator.generate(&request, &[], &[], &[])?;
//!
//! println!("Playbook: {}", playbook.playbook_id);
//! println!("Confidence: {:.1}%", playbook.confidence * 100.0);
//! # Ok(())
//! # }
//! ```

mod builder;
mod generator;
mod types;

pub use builder::ReflectionData;
pub use generator::PlaybookGenerator;
pub use types::{
    PlaybookPitfall, PlaybookRequest, PlaybookStep, PlaybookSynthesisSource, RecommendedPlaybook,
};

#[cfg(test)]
mod tests;
