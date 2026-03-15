//! Recommendation Feedback MCP Tools
//!
//! This module provides MCP tools for recording and querying recommendation
//! feedback, enabling the system to learn which recommendations help agents succeed.

mod tool;
mod types;

pub use tool::RecommendationFeedbackTools;
pub use types::{
    RecommendationStatsOutput, RecordRecommendationFeedbackInput,
    RecordRecommendationFeedbackOutput, RecordRecommendationSessionInput,
    RecordRecommendationSessionOutput, TaskOutcomeJson,
};
