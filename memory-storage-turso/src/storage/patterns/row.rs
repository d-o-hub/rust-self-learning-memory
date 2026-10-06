//! Pattern row conversion

use crate::storage::episodes::row::RowDecodeContext;
use crate::storage::patterns::PatternDataJson;
use do_memory_core::{Pattern as CorePattern, Result};
use uuid::Uuid;

/// Convert a database row to a Pattern enum (pub(crate) for use by search module)
pub(crate) fn row_to_pattern(row: &libsql::Row) -> Result<CorePattern> {
    row_to_pattern_inner(row, RowDecodeContext::new("row_to_pattern", 0))
}

/// Convert a database row to a Pattern, attributing corruption to a specific
/// query surface and zero-based result row index.
pub(crate) fn row_to_pattern_at(
    row: &libsql::Row,
    surface: &str,
    row_index: usize,
) -> Result<CorePattern> {
    row_to_pattern_inner(row, RowDecodeContext::new(surface, row_index))
}

fn row_to_pattern_inner(row: &libsql::Row, ctx: RowDecodeContext<'_>) -> Result<CorePattern> {
    let pattern_id: String = row.get(0).map_err(|e| ctx.error("pattern_id", e))?;
    let _pattern_type: String = row.get(1).map_err(|e| ctx.error("pattern_type", e))?;
    let pattern_data_json: String = row.get(2).map_err(|e| ctx.error("pattern_data", e))?;
    let success_rate: f64 = row.get(3).map_err(|e| ctx.error("success_rate", e))?;
    let _context_domain: String = row.get(4).map_err(|e| ctx.error("context_domain", e))?;
    // Genuinely nullable: absent stays `None`, an invalid non-null value errors.
    let _context_language: Option<String> =
        row.get(5).map_err(|e| ctx.error("context_language", e))?;
    let _context_tags_json: String = row.get(6).map_err(|e| ctx.error("context_tags", e))?;
    let occurrence_count: i64 = row.get(7).map_err(|e| ctx.error("occurrence_count", e))?;
    let _created_at_timestamp: i64 = row.get(8).map_err(|e| ctx.error("created_at", e))?;
    let _updated_at_timestamp: i64 = row.get(9).map_err(|e| ctx.error("updated_at", e))?;

    let pattern_data: PatternDataJson =
        serde_json::from_str(&pattern_data_json).map_err(|e| ctx.error("pattern_data", e))?;

    let pattern_id = Uuid::parse_str(&pattern_id).map_err(|e| ctx.error("pattern_id", e))?;

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
        effectiveness: do_memory_core::patterns::PatternEffectiveness::default(),
    };

    Ok(pattern)
}
