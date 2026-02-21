//! Pattern CRUD operations for Turso storage

use crate::TursoStorage;
use libsql::Row;
use memory_core::{
    Error, Heuristic, Pattern as CorePattern, Result, TaskContext, episode::PatternId,
};
use tracing::{debug, info};
use uuid::Uuid;

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
#[allow(dead_code)]
pub struct PatternMetadata {
    pub pattern_id: PatternId,
    pub pattern_type: String,
    pub success_rate: f32,
    pub occurrence_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Internal structure for pattern_data JSON field (matches storage schema)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct PatternDataJson {
    pub(crate) description: String,
    pub(crate) context: TaskContext,
    pub(crate) heuristic: Heuristic,
}

/// Storage-specific Pattern struct for database operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StoragePattern {
    pub pattern_id: PatternId,
    pub pattern_type: String,
    pub description: String,
    pub context: TaskContext,
    pub heuristic: Heuristic,
    pub success_rate: f32,
    pub occurrence_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl TursoStorage {
    /// Store a pattern
    pub async fn store_pattern(&self, pattern: &CorePattern) -> Result<()> {
        debug!("Storing pattern: {}", pattern.id());
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        // Extract data from Pattern enum
        let (description, context, heuristic, success_rate, occurrence_count) = match pattern {
            CorePattern::ToolSequence {
                id: _,
                tools,
                context,
                success_rate,
                avg_latency: _,
                occurrence_count,
                effectiveness: _,
            } => {
                // Use tools directly without cloning - join() accepts IntoIterator
                let desc = format!("Tool sequence: {}", tools.join(" -> "));
                let heur = Heuristic::new(
                    format!("When need tools: {}", tools.join(", ")),
                    format!("Use sequence: {}", tools.join(" -> ")),
                    *success_rate,
                );
                (
                    desc,
                    context.clone(),
                    heur,
                    *success_rate,
                    *occurrence_count,
                )
            }
            CorePattern::DecisionPoint {
                id: _,
                condition,
                action,
                outcome_stats,
                context,
                effectiveness: _,
            } => {
                let desc = format!("Decision: {} -> {}", condition, action);
                let heur = Heuristic::new(
                    condition.clone(),
                    action.clone(),
                    outcome_stats.success_rate(),
                );
                (
                    desc,
                    context.clone(),
                    heur,
                    outcome_stats.success_rate(),
                    outcome_stats.total_count,
                )
            }
            CorePattern::ErrorRecovery {
                id: _,
                error_type,
                recovery_steps,
                success_rate,
                context,
                effectiveness: _,
            } => {
                let desc = format!("Error recovery for: {}", error_type);
                let heur = Heuristic::new(
                    format!("Error: {}", error_type),
                    format!("Recovery: {}", recovery_steps.join(" -> ")),
                    *success_rate,
                );
                (
                    desc,
                    context.clone(),
                    heur,
                    *success_rate,
                    recovery_steps.len(),
                )
            }
            CorePattern::ContextPattern {
                id: _,
                context_features,
                recommended_approach,
                evidence: _,
                success_rate,
                effectiveness: _,
            } => {
                let desc = format!("Context pattern: {}", recommended_approach);
                let heur = Heuristic::new(
                    format!("Features: {}", context_features.join(", ")),
                    recommended_approach.clone(),
                    *success_rate,
                );
                (
                    desc,
                    TaskContext::default(),
                    heur,
                    *success_rate,
                    context_features.len(),
                )
            }
        };

        // Create pattern_data JSON blob - clone context for JSON serialization
        let pattern_data = PatternDataJson {
            description,
            context: context.clone(),
            heuristic,
        };
        let pattern_data_json =
            serde_json::to_string(&pattern_data).map_err(Error::Serialization)?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO patterns (
                pattern_id, pattern_type, pattern_data, success_rate,
                context_domain, context_language, context_tags, occurrence_count,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let context_tags_json =
            serde_json::to_string(&context.tags).map_err(Error::Serialization)?;

        let now = chrono::Utc::now();

        // Use prepared statement cache
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        stmt.execute(libsql::params![
            pattern.id().to_string(),
            format!("{:?}", pattern),
            pattern_data_json,
            success_rate,
            context.domain.clone(),
            context.language.clone(),
            context_tags_json,
            occurrence_count as i64,
            now.timestamp(),
            now.timestamp(),
        ])
        .await
        .map_err(|e| Error::Storage(format!("Failed to store pattern: {}", e)))?;

        info!("Successfully stored pattern: {}", pattern.id());
        Ok(())
    }

    /// Retrieve a pattern by ID
    pub async fn get_pattern(&self, pattern_id: PatternId) -> Result<Option<CorePattern>> {
        debug!("Retrieving pattern: {}", pattern_id);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            SELECT pattern_id, pattern_type, pattern_data, success_rate,
                   context_domain, context_language, context_tags, occurrence_count,
                   created_at, updated_at
            FROM patterns WHERE pattern_id = ?
        "#;

        // Use prepared statement cache
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![pattern_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query pattern: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch pattern row: {}", e)))?
        {
            let pattern = row_to_pattern(&row)?;
            Ok(Some(pattern))
        } else {
            Ok(None)
        }
    }

    /// Query patterns with filters
    pub async fn query_patterns(&self, query: &super::PatternQuery) -> Result<Vec<CorePattern>> {
        debug!("Querying patterns with filters: {:?}", query);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        let mut sql = String::from(
            r#"
            SELECT pattern_id, pattern_type, pattern_data, success_rate,
                   context_domain, context_language, context_tags, occurrence_count,
                   created_at, updated_at
            FROM patterns WHERE 1=1
        "#,
        );

        let mut params_vec = Vec::new();

        if let Some(ref domain) = query.domain {
            sql.push_str(" AND context_domain = ?");
            params_vec.push(domain.clone());
        }

        if let Some(ref language) = query.language {
            sql.push_str(" AND context_language = ?");
            params_vec.push(language.clone());
        }

        if let Some(min_rate) = query.min_success_rate {
            sql.push_str(" AND success_rate >= ?");
            params_vec.push(min_rate.to_string());
        }

        sql.push_str(" ORDER BY success_rate DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut rows = conn
            .query(&sql, libsql::params_from_iter(params_vec))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query patterns: {}", e)))?;

        let mut patterns = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch pattern row: {}", e)))?
        {
            patterns.push(row_to_pattern(&row)?);
        }

        info!("Found {} patterns matching query", patterns.len());
        Ok(patterns)
    }
}

/// Convert a database row to a Pattern enum (pub(crate) for use by search module)
pub(crate) fn row_to_pattern(row: &Row) -> Result<CorePattern> {
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
    let outcome_stats = memory_core::types::OutcomeStats {
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
        effectiveness: memory_core::pattern::PatternEffectiveness::default(),
    };

    Ok(pattern)
}
