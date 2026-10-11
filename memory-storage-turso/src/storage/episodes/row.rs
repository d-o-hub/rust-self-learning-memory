//! Database row to Episode conversion operations.

use crate::TursoStorage;
use do_memory_core::{Episode, Error, Result, TaskType, semantic::EpisodeSummary};
use uuid::Uuid;

/// Context describing where a row was decoded.
///
/// Used to build corruption errors that name the offending column, the row's
/// position within the result set, and the query surface so callers can find
/// the data that needs repair instead of silently receiving a partial record.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RowDecodeContext<'a> {
    surface: &'a str,
    row_index: usize,
}

impl<'a> RowDecodeContext<'a> {
    /// Build a decode context for a specific query surface and result row.
    pub(crate) const fn new(surface: &'a str, row_index: usize) -> Self {
        Self { surface, row_index }
    }

    /// Build a contextual [`Error::Storage`] naming the column and row.
    pub(crate) fn error(&self, column: &str, detail: impl std::fmt::Display) -> Error {
        Error::Storage(format!(
            "corrupt row from {}: column `{}` at result row {}: {}",
            self.surface, column, self.row_index, detail
        ))
    }
}

impl TursoStorage {
    /// Convert a database row to an Episode
    #[expect(clippy::unused_async, clippy::unused_async_trait_impl)]
    pub async fn row_to_episode(&self, row: &libsql::Row) -> Result<Episode> {
        row_to_episode(row)
    }
}

/// Parse a genuinely nullable JSON column.
///
/// SQL `NULL` and the JSON literal `null` both decode to `None`: the storage
/// layer writes `Option<T>` via `serde_json::to_string`, so an absent value is
/// stored as the four-byte string `null`, not SQL `NULL`. Any other malformed
/// payload is rejected with a contextual error rather than dropped.
fn parse_optional_json<T: serde::de::DeserializeOwned>(
    json: Option<String>,
    ctx: &RowDecodeContext<'_>,
    field: &str,
) -> Result<Option<T>> {
    match json {
        None => Ok(None),
        Some(raw) => serde_json::from_str::<Option<T>>(&raw).map_err(|e| ctx.error(field, e)),
    }
}

/// Convert a database row to an Episode
///
/// Required columns must hold valid values; a malformed timestamp or JSON
/// payload returns [`Error::Storage`] naming the field instead of substituting
/// an epoch timestamp or dropping the value. Genuinely nullable columns
/// (`end_time`, `outcome`, `reward`, `reflection`) stay `None` when absent, but
/// invalid non-null values are still rejected.
pub fn row_to_episode(row: &libsql::Row) -> Result<Episode> {
    row_to_episode_inner(row, RowDecodeContext::new("row_to_episode", 0))
}

/// Convert a database row to an Episode, attributing corruption to a specific
/// query surface and zero-based result row index.
pub(crate) fn row_to_episode_at(
    row: &libsql::Row,
    surface: &str,
    row_index: usize,
) -> Result<Episode> {
    row_to_episode_inner(row, RowDecodeContext::new(surface, row_index))
}

fn row_to_episode_inner(row: &libsql::Row, ctx: RowDecodeContext<'_>) -> Result<Episode> {
    let id_str: String = row.get(0).map_err(|e| ctx.error("episode_id", e))?;
    let episode_id = Uuid::parse_str(&id_str).map_err(|e| ctx.error("episode_id", e))?;

    let task_type_str: String = row.get(1).map_err(|e| ctx.error("task_type", e))?;
    let task_type = task_type_str
        .parse::<TaskType>()
        .map_err(|e| ctx.error("task_type", e))?;

    let task_description: String = row.get(2).map_err(|e| ctx.error("task_description", e))?;

    let context_json: String = row.get(3).map_err(|e| ctx.error("context", e))?;
    let context = serde_json::from_str(&context_json).map_err(|e| ctx.error("context", e))?;

    let start_time_ts: i64 = row.get(4).map_err(|e| ctx.error("start_time", e))?;
    let start_time = chrono::DateTime::from_timestamp(start_time_ts, 0).ok_or_else(|| {
        ctx.error(
            "start_time",
            format!("timestamp {start_time_ts} is outside the representable range"),
        )
    })?;

    let end_time_ts: Option<i64> = row.get(5).map_err(|e| ctx.error("end_time", e))?;
    let end_time = match end_time_ts {
        Some(ts) => Some(chrono::DateTime::from_timestamp(ts, 0).ok_or_else(|| {
            ctx.error(
                "end_time",
                format!("timestamp {ts} is outside the representable range"),
            )
        })?),
        None => None,
    };

    let steps_json: String = row.get(6).map_err(|e| ctx.error("steps", e))?;
    let steps = serde_json::from_str(&steps_json).map_err(|e| ctx.error("steps", e))?;

    let outcome_json: Option<String> = row.get(7).map_err(|e| ctx.error("outcome", e))?;
    let outcome = parse_optional_json(outcome_json, &ctx, "outcome")?;

    let reward_json: Option<String> = row.get(8).map_err(|e| ctx.error("reward", e))?;
    let reward = parse_optional_json(reward_json, &ctx, "reward")?;

    let reflection_json: Option<String> = row.get(9).map_err(|e| ctx.error("reflection", e))?;
    let reflection = parse_optional_json(reflection_json, &ctx, "reflection")?;

    let patterns_json: String = row.get(10).map_err(|e| ctx.error("patterns", e))?;
    let patterns = serde_json::from_str(&patterns_json).map_err(|e| ctx.error("patterns", e))?;

    let heuristics_json: String = row.get(11).map_err(|e| ctx.error("heuristics", e))?;
    let heuristics =
        serde_json::from_str(&heuristics_json).map_err(|e| ctx.error("heuristics", e))?;

    let checkpoints_json: String = row.get(12).map_err(|e| ctx.error("checkpoints", e))?;
    let checkpoints =
        serde_json::from_str(&checkpoints_json).map_err(|e| ctx.error("checkpoints", e))?;

    let metadata_json: String = row.get(13).map_err(|e| ctx.error("metadata", e))?;
    let metadata = serde_json::from_str(&metadata_json).map_err(|e| ctx.error("metadata", e))?;

    let episode = Episode {
        episode_id,
        task_type,
        task_description,
        context,
        start_time,
        end_time,
        steps,
        outcome,
        reward,
        reflection,
        patterns,
        heuristics,
        applied_patterns: Vec::new(),
        salient_features: None,
        metadata,
        tags: Vec::new(),
        checkpoints,
    };

    Ok(episode)
}

/// Convert a database row to an EpisodeSummary
pub fn row_to_summary(row: &libsql::Row) -> Result<EpisodeSummary> {
    let ctx = RowDecodeContext::new("row_to_summary", 0);

    let episode_id_str: String = row.get(0).map_err(|e| ctx.error("episode_id", e))?;
    let episode_id = Uuid::parse_str(&episode_id_str).map_err(|e| ctx.error("episode_id", e))?;

    let summary_text: String = row.get(1).map_err(|e| ctx.error("summary_text", e))?;
    let key_concepts_json: String = row.get(2).map_err(|e| ctx.error("key_concepts", e))?;
    let key_steps_json: String = row.get(3).map_err(|e| ctx.error("key_steps", e))?;

    let key_concepts =
        serde_json::from_str(&key_concepts_json).map_err(|e| ctx.error("key_concepts", e))?;
    let key_steps = serde_json::from_str(&key_steps_json).map_err(|e| ctx.error("key_steps", e))?;

    let summary_embedding: Option<Vec<u8>> =
        row.get(4).map_err(|e| ctx.error("summary_embedding", e))?;
    let summary_embedding = summary_embedding.map(|bytes| {
        let mut floats = Vec::with_capacity(bytes.len() / 4);
        for chunk in bytes.chunks_exact(4) {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(chunk);
            floats.push(f32::from_le_bytes(arr));
        }
        floats
    });

    Ok(EpisodeSummary {
        episode_id,
        summary_text,
        key_concepts,
        key_steps,
        summary_embedding,
        created_at: chrono::Utc::now(),
    })
}
