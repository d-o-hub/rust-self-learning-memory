//! Monitoring storage operations for Turso

use crate::TursoStorage;
use libsql::Row;
use memory_core::{
    monitoring::types::{AgentMetrics, AgentType, ExecutionRecord, TaskMetrics},
    Error, Result,
};
use tracing::{debug, info};

impl TursoStorage {
    /// Store an execution record
    pub async fn store_execution_record(&self, record: &ExecutionRecord) -> Result<()> {
        debug!("Storing execution record for: {}", record.agent_name);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO execution_records (
                agent_name, agent_type, task_description, success,
                duration_ms, started_at, error_message
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        conn.execute(
            sql,
            libsql::params![
                record.agent_name.clone(),
                record.agent_type.to_string(),
                record.task_description.as_deref().unwrap_or(""),
                record.success,
                record.duration.as_millis() as i64,
                record.started_at.timestamp(),
                record.error_message.as_deref().unwrap_or(""),
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store execution record: {}", e)))?;

        info!(
            "Successfully stored execution record for: {}",
            record.agent_name
        );
        Ok(())
    }

    /// Store agent metrics
    pub async fn store_agent_metrics(&self, metrics: &AgentMetrics) -> Result<()> {
        debug!("Storing agent metrics: {}", metrics.agent_name);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO agent_metrics (
                agent_name, agent_type, total_executions, successful_executions,
                total_duration_ms, avg_duration_ms, last_execution_time
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        conn.execute(
            sql,
            libsql::params![
                metrics.agent_name.clone(),
                metrics.agent_type.to_string(),
                metrics.total_executions as i64,
                metrics.successful_executions as i64,
                metrics.total_duration.as_millis() as i64,
                metrics.avg_duration.as_secs_f64(),
                metrics.last_execution.map(|t| t.timestamp()),
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store agent metrics: {}", e)))?;

        info!("Successfully stored agent metrics: {}", metrics.agent_name);
        Ok(())
    }

    /// Store task metrics
    pub async fn store_task_metrics(&self, metrics: &TaskMetrics) -> Result<()> {
        debug!("Storing task metrics: {}", metrics.task_type);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO task_metrics (
                task_type, total_tasks, completed_tasks, avg_completion_time
            ) VALUES (?, ?, ?, ?)
        "#;

        conn.execute(
            sql,
            libsql::params![
                metrics.task_type.clone(),
                metrics.total_tasks as i64,
                metrics.completed_tasks as i64,
                metrics.avg_completion_time.as_millis() as i64,
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store task metrics: {}", e)))?;

        info!("Successfully stored task metrics: {}", metrics.task_type);
        Ok(())
    }

    /// Load agent metrics
    pub async fn load_agent_metrics(&self, agent_name: &str) -> Result<Option<AgentMetrics>> {
        debug!("Loading agent metrics: {}", agent_name);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT agent_name, agent_type, total_executions, successful_executions,
                   total_duration_ms, avg_duration_ms, last_execution_time
            FROM agent_metrics WHERE agent_name = ?
        "#;

        let mut rows = conn
            .query(sql, libsql::params![agent_name])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query agent metrics: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch metrics row: {}", e)))?
        {
            let metrics = self.row_to_agent_metrics(&row)?;
            Ok(Some(metrics))
        } else {
            Ok(None)
        }
    }

    /// Load execution records
    pub async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>> {
        debug!(
            "Loading execution records: agent={:?}, limit={}",
            agent_name, limit
        );
        let conn = self.get_connection().await?;

        let mut sql = String::from(
            r#"
            SELECT agent_name, agent_type, task_description, success,
                   duration_ms, started_at, error_message
            FROM execution_records
        "#,
        );

        let mut params = Vec::new();

        if let Some(name) = agent_name {
            sql.push_str(" WHERE agent_name = ?");
            params.push(name.to_string());
        }

        sql.push_str(" ORDER BY started_at DESC");
        sql.push_str(&format!(" LIMIT {}", limit));

        let mut rows = conn
            .query(&sql, libsql::params_from_iter(params))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query execution records: {}", e)))?;

        let mut records = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch record row: {}", e)))?
        {
            records.push(self.row_to_execution_record(&row)?);
        }

        info!("Found {} execution records", records.len());
        Ok(records)
    }

    /// Load task metrics
    pub async fn load_task_metrics(&self, task_type: &str) -> Result<Option<TaskMetrics>> {
        debug!("Loading task metrics: {}", task_type);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT task_type, total_tasks, completed_tasks, avg_completion_time
            FROM task_metrics WHERE task_type = ?
        "#;

        let mut rows = conn
            .query(sql, libsql::params![task_type])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query task metrics: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch metrics row: {}", e)))?
        {
            let metrics = self.row_to_task_metrics(&row)?;
            Ok(Some(metrics))
        } else {
            Ok(None)
        }
    }
}

/// Row conversion helpers
impl TursoStorage {
    pub(crate) fn row_to_execution_record(&self, row: &Row) -> Result<ExecutionRecord> {
        let agent_name: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
        let agent_type: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
        let task_description: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
        let success: bool = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
        let duration_ms: i64 = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
        let started_at_timestamp: i64 = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
        let error_message: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;

        Ok(ExecutionRecord {
            agent_name,
            agent_type: AgentType::from(agent_type.as_str()),
            success,
            duration: std::time::Duration::from_millis(duration_ms as u64),
            started_at: chrono::DateTime::from_timestamp(started_at_timestamp, 0)
                .unwrap_or_default(),
            task_description: if task_description.is_empty() {
                None
            } else {
                Some(task_description)
            },
            error_message: if error_message.is_empty() {
                None
            } else {
                Some(error_message)
            },
        })
    }

    pub(crate) fn row_to_agent_metrics(&self, row: &Row) -> Result<AgentMetrics> {
        let agent_name: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
        let agent_type: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
        let total_executions: i64 = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
        let successful_executions: i64 = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
        let total_duration_ms: i64 = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
        let avg_duration_ms: f64 = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
        let last_execution_time: Option<i64> = row.get(6).ok();

        Ok(AgentMetrics {
            agent_name,
            agent_type: AgentType::from(agent_type.as_str()),
            total_executions: total_executions as u64,
            successful_executions: successful_executions as u64,
            total_duration: std::time::Duration::from_millis(total_duration_ms as u64),
            avg_duration: std::time::Duration::from_secs_f64(avg_duration_ms / 1000.0),
            min_duration: std::time::Duration::MAX,
            max_duration: std::time::Duration::ZERO,
            last_execution: last_execution_time
                .and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
            current_streak: 0,
            longest_streak: 0,
        })
    }

    pub(crate) fn row_to_task_metrics(&self, row: &Row) -> Result<TaskMetrics> {
        let task_type: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
        let total_tasks: i64 = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
        let completed_tasks: i64 = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
        let avg_completion_time_ms: i64 = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;

        Ok(TaskMetrics {
            task_type,
            total_tasks: total_tasks as u64,
            completed_tasks: completed_tasks as u64,
            avg_completion_time: std::time::Duration::from_millis(avg_completion_time_ms as u64),
            agent_success_rates: std::collections::HashMap::new(),
        })
    }
}
