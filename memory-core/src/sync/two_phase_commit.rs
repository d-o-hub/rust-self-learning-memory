//! Two-phase commit implementation for atomic storage operations

use crate::Result;
use tracing::{debug, error, info, warn};

/// Two-phase commit strategy for episode storage
#[derive(Debug)]
pub struct TwoPhaseCommit {
    /// Phase 1 (cache write) completed
    pub phase1_complete: bool,
    /// Phase 2 (durable write) completed
    pub phase2_complete: bool,
    /// Rollback is needed
    pub rollback_needed: bool,
}

impl TwoPhaseCommit {
    /// Create a new two-phase commit transaction
    #[must_use]
    pub fn new() -> Self {
        Self {
            phase1_complete: false,
            phase2_complete: false,
            rollback_needed: false,
        }
    }

    /// Execute phase 1 - write to cache
    pub async fn phase1<F, Fut>(&mut self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        debug!("Two-phase commit: Phase 1 (cache write)");
        match operation().await {
            Ok(()) => {
                self.phase1_complete = true;
                Ok(())
            }
            Err(e) => {
                error!("Phase 1 failed: {}", e);
                Err(e)
            }
        }
    }

    /// Execute phase 2 - write to durable storage
    pub async fn phase2<F, Fut>(&mut self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        debug!("Two-phase commit: Phase 2 (durable write)");
        if !self.phase1_complete {
            return Err(crate::Error::Storage(
                "Cannot execute phase 2 before phase 1".to_string(),
            ));
        }

        match operation().await {
            Ok(()) => {
                self.phase2_complete = true;
                Ok(())
            }
            Err(e) => {
                error!("Phase 2 failed: {}", e);
                self.rollback_needed = true;
                Err(e)
            }
        }
    }

    /// Rollback phase 1 if phase 2 failed
    pub async fn rollback<F, Fut>(&mut self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        if !self.rollback_needed {
            return Ok(());
        }

        warn!("Rolling back two-phase commit");
        match operation().await {
            Ok(()) => {
                info!("Rollback successful");
                Ok(())
            }
            Err(e) => {
                error!("Rollback failed: {}", e);
                Err(crate::Error::Storage(format!("Rollback failed: {e}")))
            }
        }
    }

    /// Check if commit is complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.phase1_complete && self.phase2_complete && !self.rollback_needed
    }
}

impl Default for TwoPhaseCommit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_phase_commit_new() {
        let commit = TwoPhaseCommit::new();
        assert!(!commit.phase1_complete);
        assert!(!commit.phase2_complete);
        assert!(!commit.rollback_needed);
        assert!(!commit.is_complete());
    }

    #[tokio::test]
    async fn test_two_phase_commit_success() {
        let mut commit = TwoPhaseCommit::new();

        // Phase 1
        let result = commit.phase1(|| async { Ok(()) }).await;
        assert!(result.is_ok());
        assert!(commit.phase1_complete);

        // Phase 2
        let result = commit.phase2(|| async { Ok(()) }).await;
        assert!(result.is_ok());
        assert!(commit.phase2_complete);
        assert!(commit.is_complete());
    }

    #[tokio::test]
    async fn test_two_phase_commit_phase1_failure() {
        let mut commit = TwoPhaseCommit::new();

        let result = commit.phase1(|| async { Err(crate::Error::Storage("phase1 failed".into())) }).await;
        assert!(result.is_err());
        assert!(!commit.phase1_complete);
    }

    #[tokio::test]
    async fn test_two_phase_commit_no_phase1() {
        let mut commit = TwoPhaseCommit::new();

        let result = commit.phase2(|| async { Ok(()) }).await;
        assert!(result.is_err());
    }
}
