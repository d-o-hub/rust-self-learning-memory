//! Execution-signature retrieval (WG-121).
//!
//! Inspired by APEX-EM (arXiv:2603.29093): rank episodes by execution
//! metadata (tools, errors, step structure) alongside embeddings.
//!
//! Matches query signatures against episode signatures for retrieval
//! that considers execution patterns, not just semantic similarity.

mod matcher;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

pub use matcher::SignatureMatcher;

/// Configuration for execution-signature matching.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SignatureConfig {
    /// Weight for tool overlap in signature matching.
    pub tool_weight: f32,
    /// Weight for error pattern overlap.
    pub error_weight: f32,
    /// Weight for step structure similarity.
    pub structure_weight: f32,
    /// Minimum overlap threshold for matching.
    pub min_overlap_threshold: f32,
    /// Whether to normalize scores.
    pub normalize_scores: bool,
}

impl Default for SignatureConfig {
    fn default() -> Self {
        Self {
            tool_weight: 0.4,
            error_weight: 0.3,
            structure_weight: 0.3,
            min_overlap_threshold: 0.2,
            normalize_scores: true,
        }
    }
}

impl SignatureConfig {
    /// Create a config favoring tool matching.
    #[must_use]
    pub fn tool_focused() -> Self {
        Self {
            tool_weight: 0.6,
            error_weight: 0.2,
            structure_weight: 0.2,
            min_overlap_threshold: 0.3,
            normalize_scores: true,
        }
    }

    /// Create a config favoring error pattern matching.
    #[must_use]
    pub fn error_focused() -> Self {
        Self {
            tool_weight: 0.2,
            error_weight: 0.6,
            structure_weight: 0.2,
            min_overlap_threshold: 0.2,
            normalize_scores: true,
        }
    }

    /// Create a balanced config.
    #[must_use]
    pub fn balanced() -> Self {
        Self {
            tool_weight: 0.33,
            error_weight: 0.33,
            structure_weight: 0.34,
            min_overlap_threshold: 0.25,
            normalize_scores: true,
        }
    }
}

/// Execution signature capturing tool, error, and structure patterns.
///
/// Represents the execution characteristics of an episode that can
/// be matched against query signatures for retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSignature {
    /// Episode ID this signature belongs to.
    pub episode_id: Uuid,
    /// Set of tools used in execution.
    pub tools: HashSet<String>,
    /// Error types encountered (normalized).
    pub error_types: HashSet<String>,
    /// Step success pattern (sequence of success/failure).
    pub step_pattern: StepPattern,
    /// Total number of steps.
    pub total_steps: usize,
    /// Number of successful steps.
    pub successful_steps: usize,
    /// Number of failed steps.
    pub failed_steps: usize,
    /// Average latency per step (ms).
    pub avg_latency_ms: u64,
}

impl ExecutionSignature {
    /// Create a new execution signature.
    #[must_use]
    pub fn new(episode_id: Uuid) -> Self {
        Self {
            episode_id,
            tools: HashSet::new(),
            error_types: HashSet::new(),
            step_pattern: StepPattern::default(),
            total_steps: 0,
            successful_steps: 0,
            failed_steps: 0,
            avg_latency_ms: 0,
        }
    }

    /// Add a tool to the signature.
    pub fn add_tool(&mut self, tool: &str) {
        self.tools.insert(tool.to_lowercase());
    }

    /// Add an error type to the signature.
    pub fn add_error(&mut self, error_type: &str) {
        self.error_types.insert(normalize_error_type(error_type));
    }

    /// Record a step outcome (success/failure).
    pub fn record_step(&mut self, success: bool) {
        self.total_steps += 1;
        if success {
            self.successful_steps += 1;
            self.step_pattern.success_count += 1;
        } else {
            self.failed_steps += 1;
            self.step_pattern.failure_count += 1;
        }
    }

    /// Set the average latency.
    pub fn set_avg_latency(&mut self, latency_ms: u64) {
        self.avg_latency_ms = latency_ms;
    }

    /// Get success rate (0.0 to 1.0).
    #[must_use]
    pub fn success_rate(&self) -> f32 {
        if self.total_steps == 0 {
            return 0.0;
        }
        self.successful_steps as f32 / self.total_steps as f32
    }

    /// Get tool count.
    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Get error count.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.error_types.len()
    }

    /// Check if signature has any tools.
    #[must_use]
    pub fn has_tools(&self) -> bool {
        !self.tools.is_empty()
    }

    /// Check if signature has any errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.error_types.is_empty()
    }
}

/// Step pattern capturing the execution structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepPattern {
    /// Number of successful steps.
    pub success_count: usize,
    /// Number of failed steps.
    pub failure_count: usize,
    /// Whether execution ended in success.
    pub final_success: bool,
    /// Pattern code: compact representation of success/failure sequence.
    pub pattern_code: String,
}

impl StepPattern {
    /// Create a step pattern from a sequence of outcomes.
    #[must_use]
    pub fn from_outcomes(outcomes: &[bool]) -> Self {
        let success_count = outcomes.iter().filter(|&s| *s).count();
        let failure_count = outcomes.len() - success_count;
        let final_success = outcomes.last().copied().unwrap_or(false);

        // Create compact pattern code: S for success, F for failure
        let pattern_code = outcomes
            .iter()
            .map(|s| if *s { 'S' } else { 'F' })
            .collect::<String>();

        Self {
            success_count,
            failure_count,
            final_success,
            pattern_code,
        }
    }

    /// Get pattern similarity with another pattern.
    ///
    /// Compares the ratio of matching positions.
    #[must_use]
    pub fn similarity(&self, other: &StepPattern) -> f32 {
        if self.pattern_code.is_empty() || other.pattern_code.is_empty() {
            return 0.0;
        }

        // Compare lengths first - similar lengths score higher
        let len_ratio = if self.pattern_code.len() > other.pattern_code.len() {
            other.pattern_code.len() as f32 / self.pattern_code.len() as f32
        } else {
            self.pattern_code.len() as f32 / other.pattern_code.len() as f32
        };

        // Compare success/failure ratios
        let self_success_ratio =
            self.success_count as f32 / (self.success_count + self.failure_count).max(1) as f32;
        let other_success_ratio =
            other.success_count as f32 / (other.success_count + other.failure_count).max(1) as f32;
        let ratio_similarity = 1.0 - (self_success_ratio - other_success_ratio).abs();

        // Compare final outcome
        let final_match = if self.final_success == other.final_success {
            1.0
        } else {
            0.0
        };

        // Combine factors
        len_ratio * 0.3 + ratio_similarity * 0.5 + final_match * 0.2
    }
}

/// Query signature for matching against episode signatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySignature {
    /// Tools expected/needed in the query.
    pub expected_tools: HashSet<String>,
    /// Error types to avoid or handle.
    pub relevant_errors: HashSet<String>,
    /// Expected step pattern (if known).
    pub expected_pattern: Option<StepPattern>,
}

impl QuerySignature {
    /// Create a new query signature.
    #[must_use]
    pub fn new() -> Self {
        Self {
            expected_tools: HashSet::new(),
            relevant_errors: HashSet::new(),
            expected_pattern: None,
        }
    }

    /// Create a query signature from a query string.
    ///
    /// Extracts tool names and error patterns from text.
    #[must_use]
    pub fn from_query_text(query: &str) -> Self {
        let mut sig = Self::new();

        // Extract common tool names from query
        let common_tools = [
            "read",
            "write",
            "bash",
            "grep",
            "edit",
            "glob",
            "lsp",
            "webfetch",
            "websearch",
            "skill",
            "agent",
            "cargo",
            "rustc",
            "git",
            "npm",
            "pytest",
        ];

        for tool in common_tools {
            if query.to_lowercase().contains(tool) {
                sig.expected_tools.insert(tool.to_string());
            }
        }

        // Extract error patterns
        let error_patterns = [
            "timeout",
            "error",
            "failure",
            "panic",
            "exception",
            " deadlock",
            "race",
            "null",
            "missing",
            "invalid",
        ];

        for pattern in error_patterns {
            if query.to_lowercase().contains(pattern) {
                sig.relevant_errors.insert(normalize_error_type(pattern));
            }
        }

        sig
    }

    /// Add an expected tool.
    pub fn add_tool(&mut self, tool: &str) {
        self.expected_tools.insert(tool.to_lowercase());
    }

    /// Add a relevant error.
    pub fn add_error(&mut self, error: &str) {
        self.relevant_errors.insert(normalize_error_type(error));
    }

    /// Set expected pattern.
    pub fn set_pattern(&mut self, pattern: StepPattern) {
        self.expected_pattern = Some(pattern);
    }

    /// Check if query has any tool expectations.
    #[must_use]
    pub fn has_tool_expectations(&self) -> bool {
        !self.expected_tools.is_empty()
    }

    /// Check if query has any error expectations.
    #[must_use]
    pub fn has_error_expectations(&self) -> bool {
        !self.relevant_errors.is_empty()
    }
}

impl Default for QuerySignature {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of signature matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMatch {
    /// Episode ID that matched.
    pub episode_id: Uuid,
    /// Overall match score (0.0 to 1.0).
    pub score: f32,
    /// Tool overlap score component.
    pub tool_score: f32,
    /// Error overlap score component.
    pub error_score: f32,
    /// Structure similarity score component.
    pub structure_score: f32,
    /// Which components contributed to the match.
    pub contributing_components: Vec<String>,
}

impl SignatureMatch {
    /// Check if this is a strong match (score >= 0.5).
    #[must_use]
    pub fn is_strong_match(&self) -> bool {
        self.score >= 0.5
    }

    /// Check if this is a weak match (score >= 0.2).
    #[must_use]
    pub fn is_weak_match(&self) -> bool {
        self.score >= 0.2
    }
}

/// Normalize an error type for matching.
///
/// Maps similar error descriptions to canonical forms.
#[cfg(test)]
pub(crate) fn normalize_error_type(error: &str) -> String {
    let lower = error.to_lowercase();

    // Map common patterns to canonical forms
    let canonical = match lower.as_str() {
        "timeout" | "timed out" | "time out" => "timeout",
        "panic" | "panicked" => "panic",
        "deadlock" | "dead lock" => "deadlock",
        "race" | "race condition" => "race_condition",
        "null" | "null pointer" | "nullptr" => "null_pointer",
        "missing" | "not found" | "does not exist" => "missing",
        "invalid" | "invalid input" | "invalid argument" => "invalid",
        e => e.trim(),
    };

    canonical.to_string()
}

#[cfg(not(test))]
fn normalize_error_type(error: &str) -> String {
    let lower = error.to_lowercase();

    let canonical = match lower.as_str() {
        "timeout" | "timed out" | "time out" => "timeout",
        "panic" | "panicked" => "panic",
        "deadlock" | "dead lock" => "deadlock",
        "race" | "race condition" => "race_condition",
        "null" | "null pointer" | "nullptr" => "null_pointer",
        "missing" | "not found" | "does not exist" => "missing",
        "invalid" | "invalid input" | "invalid argument" => "invalid",
        e => e.trim(),
    };

    canonical.to_string()
}

#[cfg(test)]
mod tests;
