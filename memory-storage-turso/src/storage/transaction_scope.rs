//! Crate-local transaction scope helper for libSQL write paths.
//!
//! A multi-statement write must never leave the connection sitting in an open
//! transaction when one of its statements fails. A leaked `BEGIN` is worse than the
//! lost write itself: every later statement on that (pooled) connection silently joins
//! the abandoned transaction, so a tag replacement can lose the episode's existing tags
//! and then have that loss undone - or committed - by an unrelated caller (issue #1088).
//!
//! The scope lives here instead of on `StorageBackend` so no public trait surface -
//! and no implementation of it - has to change.

use crate::{Error, Result};
use async_trait::async_trait;
use std::future::Future;
use tracing::warn;

/// The only connection capability a [`transactional`] scope needs: running the
/// parameter-less control statements (`BEGIN TRANSACTION`, `COMMIT`, `ROLLBACK`).
///
/// Deliberately minimal, so the commit/rollback behaviour of a scope can be unit tested
/// against a deterministic fake connection without the native libSQL path.
///
/// `Send + Sync` are supertraits so that `&dyn TxScopeConnection` stays `Send`: tag writes
/// are awaited from `Send` futures (including the resilient circuit breaker).
#[async_trait]
pub(crate) trait TxScopeConnection: Send + Sync {
    /// Execute one statement that takes no parameters.
    async fn execute_control(&self, sql: &str) -> Result<u64>;
}

#[async_trait]
impl TxScopeConnection for libsql::Connection {
    async fn execute_control(&self, sql: &str) -> Result<u64> {
        self.execute(sql, ())
            .await
            .map_err(|e| Error::Storage(format!("Failed to run '{sql}': {e}")))
    }
}

/// Run `body` inside a single transaction on `conn`.
///
/// `body` is an (unpolled) async block, so it is lazy: `BEGIN TRANSACTION` is issued
/// *before* any of its statements run, and none of them can run outside the transaction.
///
/// * `COMMIT` on `Ok`; the body's value is returned.
/// * `ROLLBACK` on `Err` - which covers every `?` early return inside the body - and the
///   body's **original** error is returned unchanged. A failed rollback is logged, never
///   substituted for the real cause.
/// * A failing `COMMIT` also triggers `ROLLBACK`, so no path leaves a transaction open.
///
/// Caveat: the scope can only roll back what it can await. If a caller drops the returned
/// future in the middle of `body` (task cancellation) the applied statements stay in an
/// open transaction; the tag write paths are always awaited to completion.
pub(crate) async fn transactional<T, Fut>(conn: &dyn TxScopeConnection, body: Fut) -> Result<T>
where
    Fut: Future<Output = Result<T>>,
{
    conn.execute_control("BEGIN TRANSACTION")
        .await
        .map_err(|e| Error::Storage(format!("Failed to begin transaction: {e}")))?;

    match body.await {
        Ok(value) => match conn.execute_control("COMMIT").await {
            Ok(_) => Ok(value),
            Err(commit_err) => {
                rollback(conn).await;
                Err(Error::Storage(format!(
                    "Failed to commit transaction: {commit_err}"
                )))
            }
        },
        Err(body_err) => {
            rollback(conn).await;
            Err(body_err)
        }
    }
}

/// Roll the still-open transaction back, logging (but never propagating) a failure: the
/// caller's own error is the actionable one and must not be masked.
async fn rollback(conn: &dyn TxScopeConnection) {
    if let Err(e) = conn.execute_control("ROLLBACK").await {
        warn!("Failed to rollback transaction: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, PoisonError};

    /// Deterministic fake: records every control statement and fails on a marker, so a
    /// mid-transaction failure can be injected without any database.
    struct FakeConn {
        executed: Mutex<Vec<String>>,
        /// Statement substring that fails (no failure when `None`).
        fail_on: Option<String>,
    }

    impl FakeConn {
        fn new() -> Self {
            Self {
                executed: Mutex::new(Vec::new()),
                fail_on: None,
            }
        }

        fn failing_on(marker: &str) -> Self {
            Self {
                executed: Mutex::new(Vec::new()),
                fail_on: Some(marker.to_string()),
            }
        }

        /// Statements recorded so far. The lock guard never survives past this
        /// statement, so nothing is held across an await.
        fn recorded(&self) -> Vec<String> {
            self.executed
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
        }
    }

    #[async_trait]
    impl TxScopeConnection for FakeConn {
        async fn execute_control(&self, sql: &str) -> Result<u64> {
            self.executed
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(sql.to_string());
            if let Some(marker) = &self.fail_on {
                if sql.contains(marker) {
                    return Err(Error::Storage(format!("injected failure for '{sql}'")));
                }
            }
            Ok(1)
        }
    }

    /// One recorded statement of a scope body.
    const STEP: &str = "DELETE FROM episode_tags WHERE episode_id = 'x'";

    #[tokio::test]
    async fn transactional_begins_before_the_body_and_commits_once() {
        let conn = FakeConn::new();

        let value = transactional(&conn, async {
            conn.execute_control(STEP).await?;
            Ok(7)
        })
        .await
        .unwrap();

        assert_eq!(value, 7);
        assert_eq!(
            conn.recorded(),
            vec![
                "BEGIN TRANSACTION".to_string(),
                STEP.to_string(),
                "COMMIT".to_string()
            ],
            "exactly one BEGIN and one COMMIT, no ROLLBACK"
        );
    }

    #[tokio::test]
    async fn transactional_rolls_back_and_returns_the_original_error() {
        let conn = FakeConn::new();

        let err = transactional(&conn, async {
            conn.execute_control(STEP).await?;
            Err::<(), _>(Error::Storage("Failed to insert tag: injected".to_string()))
        })
        .await
        .expect_err("body error must propagate");

        assert!(
            err.to_string().contains("Failed to insert tag: injected"),
            "the body's original error must be returned unchanged, got: {err}"
        );
        assert_eq!(
            conn.recorded(),
            vec![
                "BEGIN TRANSACTION".to_string(),
                STEP.to_string(),
                "ROLLBACK".to_string()
            ],
            "failure after the first statement must roll back and never commit"
        );
    }

    #[tokio::test]
    async fn transactional_rolls_back_on_early_return_from_the_body() {
        // The `?` propagation case: the second statement fails, so the body returns
        // before reaching its own end.
        let conn = FakeConn::failing_on("gamma");
        let insert = "INSERT INTO episode_tags VALUES 'gamma'";

        let err = transactional(&conn, async {
            conn.execute_control(STEP).await?;
            conn.execute_control(insert).await?;
            Ok(())
        })
        .await
        .expect_err("injected statement error must propagate");

        assert!(err.to_string().contains("injected failure"));
        let recorded = conn.recorded();
        assert_eq!(
            recorded,
            vec![
                "BEGIN TRANSACTION".to_string(),
                STEP.to_string(),
                insert.to_string(),
                "ROLLBACK".to_string()
            ]
        );
        assert!(
            !recorded.iter().any(|s| s == "COMMIT"),
            "COMMIT must not run after a failed statement"
        );
    }

    #[tokio::test]
    async fn transactional_rolls_back_when_commit_fails() {
        let conn = FakeConn::failing_on("COMMIT");

        let err = transactional(&conn, async {
            conn.execute_control(STEP).await?;
            Ok(())
        })
        .await
        .expect_err("commit failure must propagate");

        assert!(err.to_string().contains("Failed to commit transaction"));
        assert_eq!(
            conn.recorded(),
            vec![
                "BEGIN TRANSACTION".to_string(),
                STEP.to_string(),
                "COMMIT".to_string(),
                "ROLLBACK".to_string()
            ],
            "a rejected COMMIT still has to close the transaction"
        );
    }

    #[tokio::test]
    async fn transactional_does_not_run_the_body_when_begin_fails() {
        let conn = FakeConn::failing_on("BEGIN");

        let err = transactional(&conn, async {
            conn.execute_control(STEP).await?;
            Ok(())
        })
        .await
        .expect_err("begin failure must propagate");

        assert!(err.to_string().contains("Failed to begin transaction"));
        assert_eq!(
            conn.recorded(),
            vec!["BEGIN TRANSACTION".to_string()],
            "no statement may run outside an open transaction"
        );
    }
}
