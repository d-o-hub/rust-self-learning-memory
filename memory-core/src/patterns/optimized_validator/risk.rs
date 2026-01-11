//! Risk assessment structures

/// Risk assessment result for pattern application
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub overall_risk_score: f32,
    pub step_level_risks: Vec<f32>,
    pub context_complexity_risk: f32,
    pub tool_compatibility_risk: f32,
    pub should_inject_validation: bool,
}
