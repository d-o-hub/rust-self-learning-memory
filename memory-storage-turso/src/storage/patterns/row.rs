//! Pattern row conversion

use crate::storage::patterns::PatternDataJson;
use do_memory_core::{Error, Pattern as CorePattern, Result};
use uuid::Uuid;

/// Convert a database row to a Pattern enum (pub(crate) for use by search module)
pub(crate) fn row_to_pattern(row: &libsql::Row) -> Result<CorePattern> {
    let pattern_id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
    let _pattern_type: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
    let pattern_data_json: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
    let success_rate: f64 = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
    let _context_domain: String = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
    let _context_language: Option<String> = row.get(5).ok();
    let _context_tags_json: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
    let occurrence_count: i64 = row.get(7).map_err(|e| Error::Storage(e.to_string()))?;
    let _created_at_timestamp: i64 = row.get(8).map_err(|e| Error::Storage(e.to_string()))?;
    let _updated_at_timestamp: i64 = row.get(9).map_err(|e| Error::Storage(e.to_string()))?;

    let pattern_data: PatternDataJson = serde_json::from_str(&pattern_data_json)
        .map_err(|e| Error::Storage(format!("Failed to parse pattern data: {}", e)))?;

    let pattern_id = Uuid::parse_str(&pattern_id)
        .map_err(|e| Error::Storage(format!("Invalid pattern ID: {}", e)))?;

    // Convert to CorePattern enum
    // For simplicity, store as DecisionPoint variant with condition=description, action=heuristic
    let success_rate_f32 = success_rate as f32;
    let outcome_stats = do_memory_core::types::OutcomeStats {
        success_count: (success_rate_f32 * occurrence_count as f32) as usize,
        failure_count: ((1.0 - success_rate_f32) * occurrence_count as f32) as usize,
        total_count: occurrence_count as usize,
        avg_duration_secs: 0.0,
    };

    let pattern = CorePattern::DecisionPoint {
        id: pattern_id,
        condition: pattern_data.description,
        action: format!("Heuristic: {}", pattern_data.heuristic.condition),
        outcome_stats,
        context: pattern_data.context,
        effectiveness: do_memory_core::pattern::PatternEffectiveness::default(),
    };

    Ok(pattern)
}
