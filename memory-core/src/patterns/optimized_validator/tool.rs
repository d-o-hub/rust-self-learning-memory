//! Tool representation for compatibility assessment

use crate::types::TaskContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool representation for compatibility assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub capabilities: Vec<String>,
    pub typical_contexts: Vec<TaskContext>,
    pub success_history: HashMap<String, f32>, // context_domain -> success_rate
}

/// Tool compatibility assessment result
#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    pub overall_score: f32,
    pub historical_success_rate: f32,
    pub context_compatibility: f32,
    pub capability_match: f32,
}

impl Tool {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            capabilities: Vec::new(),
            typical_contexts: Vec::new(),
            success_history: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    #[must_use]
    pub fn with_typical_context(mut self, contexts: Vec<TaskContext>) -> Self {
        self.typical_contexts = contexts;
        self
    }

    #[must_use]
    pub fn with_success_history(mut self, history: HashMap<String, f32>) -> Self {
        self.success_history = history;
        self
    }
}
