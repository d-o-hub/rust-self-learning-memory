//! Planned step structure for validation injection

/// Planned step structure for validation injection
#[derive(Debug, Clone)]
pub struct PlannedStep {
    pub tool: String,
    pub action: String,
    pub expected_duration_ms: u64,
    pub parameters: serde_json::Value,
}
