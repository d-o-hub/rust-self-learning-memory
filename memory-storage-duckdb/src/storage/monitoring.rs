use crate::DuckDbStorage;
use chrono::{DateTime, Utc};
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;

impl DuckDbStorage {
    pub(crate) async fn store_execution_record_internal(
        &self,
        record: &do_memory_core::monitoring::types::ExecutionRecord,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let record = record.clone();
        let duration_ms = i64::try_from(record.duration.as_millis()).map_err(|e| {
            Error::Storage(format!("Duration overflow for execution record: {e}"))
        })?;

        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "INSERT INTO execution_records (
                    agent_name, agent_type, success, duration_ms, started_at, task_description, error_message
                ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    record.agent_name,
                    record.agent_type.to_string(),
                    record.success,
                    duration_ms,
                    record.started_at.to_rfc3339(),
                    record.task_description,
                    record.error_message,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store execution record: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn store_agent_metrics_internal(
        &self,
        metrics: &do_memory_core::monitoring::types::AgentMetrics,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let metrics = metrics.clone();

        let total_executions = i64::try_from(metrics.total_executions).map_err(|e| {
            Error::Storage(format!("total_executions overflow: {e}"))
        })?;
        let successful_executions = i64::try_from(metrics.successful_executions).map_err(|e| {
            Error::Storage(format!("successful_executions overflow: {e}"))
        })?;
        let total_duration_ms = i64::try_from(metrics.total_duration.as_millis()).map_err(|e| {
            Error::Storage(format!("total_duration overflow: {e}"))
        })?;
        let avg_duration_ms = i64::try_from(metrics.avg_duration.as_millis()).map_err(|e| {
            Error::Storage(format!("avg_duration overflow: {e}"))
        })?;
        let min_duration_ms = i64::try_from(metrics.min_duration.as_millis()).map_err(|e| {
            Error::Storage(format!("min_duration overflow: {e}"))
        })?;
        let max_duration_ms = i64::try_from(metrics.max_duration.as_millis()).map_err(|e| {
            Error::Storage(format!("max_duration overflow: {e}"))
        })?;
        let current_streak = i32::try_from(metrics.current_streak).map_err(|e| {
            Error::Storage(format!("current_streak overflow: {e}"))
        })?;
        let longest_streak = i32::try_from(metrics.longest_streak).map_err(|e| {
            Error::Storage(format!("longest_streak overflow: {e}"))
        })?;

        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "INSERT OR REPLACE INTO agent_metrics (
                    agent_name, agent_type, total_executions, successful_executions,
                    total_duration_ms, avg_duration_ms, min_duration_ms, max_duration_ms,
                    last_execution, current_streak, longest_streak, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
                params![
                    metrics.agent_name,
                    metrics.agent_type.to_string(),
                    total_executions,
                    successful_executions,
                    total_duration_ms,
                    avg_duration_ms,
                    min_duration_ms,
                    max_duration_ms,
                    metrics.last_execution.map(|t| t.to_rfc3339()),
                    current_streak,
                    longest_streak,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store agent metrics: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn store_task_metrics_internal(
        &self,
        metrics: &do_memory_core::monitoring::types::TaskMetrics,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let metrics = metrics.clone();

        let total_tasks = i64::try_from(metrics.total_tasks).map_err(|e| {
            Error::Storage(format!("total_tasks overflow: {e}"))
        })?;
        let completed_tasks = i64::try_from(metrics.completed_tasks).map_err(|e| {
            Error::Storage(format!("completed_tasks overflow: {e}"))
        })?;
        let avg_completion_time_ms = i64::try_from(metrics.avg_completion_time.as_millis()).map_err(|e| {
            Error::Storage(format!("avg_completion_time overflow: {e}"))
        })?;

        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let success_rates_json = serde_json::to_string(&metrics.agent_success_rates)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO task_metrics (
                    task_type, total_tasks, completed_tasks, avg_completion_time_ms,
                    agent_success_rates, updated_at
                ) VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
                params![
                    metrics.task_type,
                    total_tasks,
                    completed_tasks,
                    avg_completion_time_ms,
                    success_rates_json,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store task metrics: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn load_agent_metrics_internal(
        &self,
        agent_name: &str,
    ) -> Result<Option<do_memory_core::monitoring::types::AgentMetrics>> {
        let conn_arc = Arc::clone(&self.conn);
        let agent_name = agent_name.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT agent_name, agent_type, total_executions, successful_executions,
                     total_duration_ms, min_duration_ms, max_duration_ms,
                     strftime(CAST(last_execution AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                     current_streak, longest_streak
                     FROM agent_metrics WHERE agent_name = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![agent_name])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let agent_name: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let agent_type_str: String =
                    row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let total_executions: i64 =
                    row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let successful_executions: i64 =
                    row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let total_duration_ms: i64 =
                    row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let min_duration_ms: i64 = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
                let max_duration_ms: i64 = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
                let last_execution_str: Option<String> =
                    row.get(7).map_err(|e| Error::Storage(e.to_string()))?;
                let current_streak: i32 = row.get(8).map_err(|e| Error::Storage(e.to_string()))?;
                let longest_streak: i32 = row.get(9).map_err(|e| Error::Storage(e.to_string()))?;

                let agent_type =
                    do_memory_core::monitoring::types::AgentType::from(agent_type_str.as_str());

                let metrics = do_memory_core::monitoring::types::AgentMetrics {
                    agent_name,
                    agent_type,
                    total_executions: u64::try_from(total_executions).map_err(|e| Error::Storage(format!("total_executions conversion: {e}")))?,
                    successful_executions: u64::try_from(successful_executions).map_err(|e| Error::Storage(format!("successful_executions conversion: {e}")))?,
                    total_duration: std::time::Duration::from_millis(
                        u64::try_from(total_duration_ms).map_err(|e| Error::Storage(format!("total_duration conversion: {e}")))?
                    ),
                    avg_duration: if total_executions > 0 {
                        std::time::Duration::from_millis(
                            u64::try_from(total_duration_ms / total_executions).map_err(|e| Error::Storage(format!("avg_duration conversion: {e}")))?
                        )
                    } else {
                        std::time::Duration::ZERO
                    },
                    min_duration: std::time::Duration::from_millis(
                        u64::try_from(min_duration_ms).map_err(|e| Error::Storage(format!("min_duration conversion: {e}")))?
                    ),
                    max_duration: std::time::Duration::from_millis(
                        u64::try_from(max_duration_ms).map_err(|e| Error::Storage(format!("max_duration conversion: {e}")))?
                    ),
                    last_execution: last_execution_str
                        .map(|s| {
                            DateTime::parse_from_rfc3339(&s)
                                .or_else(|_| DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%fZ"))
                                .map(|dt| dt.with_timezone(&Utc))
                                .map_err(|e| Error::Storage(format!("last_execution parse: {e}")))
                        })
                        .transpose()?,
                    current_streak: u32::try_from(current_streak).map_err(|e| Error::Storage(format!("current_streak conversion: {e}")))?,
                    longest_streak: u32::try_from(longest_streak).map_err(|e| Error::Storage(format!("longest_streak conversion: {e}")))?,
                };
                Ok::<Option<do_memory_core::monitoring::types::AgentMetrics>, Error>(Some(metrics))
            } else {
                Ok::<Option<do_memory_core::monitoring::types::AgentMetrics>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn load_execution_records_internal(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<do_memory_core::monitoring::types::ExecutionRecord>> {
        let conn_arc = Arc::clone(&self.conn);
        let agent_name = agent_name.map(std::string::ToString::to_string);
        let limit_i64 = i64::try_from(limit).unwrap_or(1000);

        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut query = "SELECT agent_name, agent_type, success, duration_ms,
                             strftime(CAST(started_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                             task_description, error_message
                             FROM execution_records"
                .to_string();
            if agent_name.is_some() {
                query.push_str(" WHERE agent_name = ?");
            }
            query.push_str(" ORDER BY started_at DESC LIMIT ?");

            let mut stmt = conn
                .prepare(&query)
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = if let Some(ref name) = agent_name {
                stmt.query(params![name, limit_i64])
            } else {
                stmt.query(params![limit_i64])
            }
            .map_err(|e| Error::Storage(e.to_string()))?;

            let mut records = Vec::new();
            while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let agent_name: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let agent_type_str: String =
                    row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let success: bool = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let duration_ms: i64 = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let started_at_str: String =
                    row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let task_description: Option<String> =
                    row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
                let error_message: Option<String> =
                    row.get(6).map_err(|e| Error::Storage(e.to_string()))?;

                let agent_type =
                    do_memory_core::monitoring::types::AgentType::from(agent_type_str.as_str());
                let started_at = DateTime::parse_from_rfc3339(&started_at_str)
                    .or_else(|_| DateTime::parse_from_str(&started_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                    .map_err(|e| Error::Storage(e.to_string()))?
                    .with_timezone(&Utc);

                records.push(do_memory_core::monitoring::types::ExecutionRecord {
                    agent_name,
                    agent_type,
                    success,
                    duration: std::time::Duration::from_millis(
                        u64::try_from(duration_ms).map_err(|e| Error::Storage(format!("duration conversion: {e}")))?
                    ),
                    started_at,
                    task_description,
                    error_message,
                });
            }
            Ok::<Vec<do_memory_core::monitoring::types::ExecutionRecord>, Error>(records)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn load_task_metrics_internal(
        &self,
        task_type: &str,
    ) -> Result<Option<do_memory_core::monitoring::types::TaskMetrics>> {
        let conn_arc = Arc::clone(&self.conn);
        let task_type = task_type.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT task_type, total_tasks, completed_tasks, avg_completion_time_ms,
                     CAST(agent_success_rates AS VARCHAR)
                     FROM task_metrics WHERE task_type = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![task_type])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let task_type: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let total_tasks: i64 = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let completed_tasks: i64 = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let avg_completion_time_ms: i64 =
                    row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let agent_success_rates_json: String =
                    row.get(4).map_err(|e| Error::Storage(e.to_string()))?;

                let metrics = do_memory_core::monitoring::types::TaskMetrics {
                    task_type,
                    total_tasks: u64::try_from(total_tasks).map_err(|e| Error::Storage(format!("total_tasks conversion: {e}")))?,
                    completed_tasks: u64::try_from(completed_tasks).map_err(|e| Error::Storage(format!("completed_tasks conversion: {e}")))?,
                    avg_completion_time: std::time::Duration::from_millis(
                        u64::try_from(avg_completion_time_ms).map_err(|e| Error::Storage(format!("avg_completion_time conversion: {e}")))?
                    ),
                    agent_success_rates: serde_json::from_str(&agent_success_rates_json)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                };
                Ok::<Option<do_memory_core::monitoring::types::TaskMetrics>, Error>(Some(metrics))
            } else {
                Ok::<Option<do_memory_core::monitoring::types::TaskMetrics>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }
}
