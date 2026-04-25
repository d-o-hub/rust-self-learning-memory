//! Pattern CRUD operations for Turso storage

use crate::TursoStorage;
use do_memory_core::{Error, Heuristic, Pattern as CorePattern, Result, TaskContext};
use tracing::{debug, info};

/// Internal structure for pattern_data JSON field (matches storage schema)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct PatternDataJson {
    pub(crate) description: String,
    pub(crate) context: TaskContext,
    pub(crate) heuristic: Heuristic,
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
    pub async fn get_pattern(
        &self,
        pattern_id: do_memory_core::episode::PatternId,
    ) -> Result<Option<CorePattern>> {
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
            let pattern = super::row::row_to_pattern(&row)?;
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
            sql.push_str(" LIMIT ?");
            params_vec.push((limit as i64).to_string());
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
            patterns.push(super::row::row_to_pattern(&row)?);
        }

        info!("Found {} patterns matching query", patterns.len());
        Ok(patterns)
    }
}
