//! Pattern storage module
//!
//! Pattern CRUD operations and query functionality.

mod crud;
mod raw_query;
mod row;

pub(crate) use crud::PatternDataJson;
pub use raw_query::{PATTERN_SELECT_COLUMNS, RawPatternQuery};
pub(crate) use row::row_to_pattern;

use do_memory_core::episode::PatternId;

/// Query builder for patterns
#[derive(Debug, Clone, Default)]
pub struct PatternQuery {
    pub domain: Option<String>,
    pub language: Option<String>,
    pub min_success_rate: Option<f32>,
    pub limit: Option<usize>,
}

/// Pattern metadata including timestamps
#[derive(Debug, Clone)]
pub struct PatternMetadata {
    pub pattern_id: PatternId,
    pub pattern_type: String,
    pub success_rate: f32,
    pub occurrence_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
