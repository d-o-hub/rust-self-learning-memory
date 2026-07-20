//! Local embedding provider configuration and model artifact verification (S1.5b / F4.3).

use serde::{Deserialize, Serialize};

use super::OptimizationConfig;

/// Configuration for local embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// Model name/path
    pub model_name: String,
    /// Embedding dimension
    pub embedding_dimension: usize,
    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
    /// When true, allow mock embeddings for tests/dev (S1.5).
    ///
    /// Production defaults to `false`: if the real model cannot load, the
    /// provider reports degraded-mock / unavailable rather than silently
    /// advertising mock vectors as healthy (see `EmbeddingHealth`).
    #[serde(default)]
    pub allow_mock_fallback: bool,
    /// Optional model revision / pin label (S1.5b / F4.3).
    #[serde(default)]
    pub model_revision: Option<String>,
    /// Optional expected SHA-256 hex digest of the primary model artifact (S1.5b).
    #[serde(default)]
    pub expected_sha256: Option<String>,
    /// Optional maximum allowed artifact size in bytes (S1.5b).
    #[serde(default)]
    pub max_artifact_bytes: Option<u64>,
}

impl LocalConfig {
    /// Create a new local config.
    ///
    /// Mock fallback defaults to **enabled** so offline tests/dev keep working.
    /// Call [`Self::with_allow_mock_fallback`]`(false)` for production fail-closed
    /// loading (S1.5). Mock mode never reports production-ready via `is_available`.
    #[must_use]
    pub fn new(model_name: impl Into<String>, dimension: usize) -> Self {
        Self {
            model_name: model_name.into(),
            embedding_dimension: dimension,
            optimization: OptimizationConfig::local(),
            allow_mock_fallback: true,
            model_revision: None,
            expected_sha256: None,
            max_artifact_bytes: None,
        }
    }

    /// Enable mock embeddings for tests and local development (S1.5).
    #[must_use]
    pub fn with_allow_mock_fallback(mut self, allow: bool) -> Self {
        self.allow_mock_fallback = allow;
        self
    }

    /// Pin model revision string (S1.5b).
    #[must_use]
    pub fn with_model_revision(mut self, revision: impl Into<String>) -> Self {
        self.model_revision = Some(revision.into());
        self
    }

    /// Require SHA-256 hex digest of the model file (S1.5b).
    #[must_use]
    pub fn with_expected_sha256(mut self, digest_hex: impl Into<String>) -> Self {
        self.expected_sha256 = Some(digest_hex.into());
        self
    }

    /// Reject artifacts larger than `max_bytes` (S1.5b).
    #[must_use]
    pub fn with_max_artifact_bytes(mut self, max_bytes: u64) -> Self {
        self.max_artifact_bytes = Some(max_bytes);
        self
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.embedding_dimension
    }
}

/// Verify a model artifact against optional digest and size limits (S1.5b / F4.3).
///
/// # Errors
/// Returns an error when the file is missing, oversized, or digest mismatches.
pub fn verify_model_artifact(
    path: &std::path::Path,
    expected_sha256: Option<&str>,
    max_bytes: Option<u64>,
) -> anyhow::Result<()> {
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::io::Read;

    let meta = fs::metadata(path).map_err(|e| {
        anyhow::anyhow!("model artifact metadata failed for {}: {e}", path.display())
    })?;
    let len = meta.len();
    if let Some(max) = max_bytes {
        if len > max {
            anyhow::bail!(
                "model artifact {} is {len} bytes, exceeds max_artifact_bytes={max}",
                path.display()
            );
        }
    }
    if let Some(expected) = expected_sha256 {
        let expected = expected.trim().to_ascii_lowercase();
        let mut file = fs::File::open(path)
            .map_err(|e| anyhow::anyhow!("open model artifact {}: {e}", path.display()))?;
        let mut hasher = Sha256::new();
        let mut buf = [0_u8; 8192];
        loop {
            let n = file
                .read(&mut buf)
                .map_err(|e| anyhow::anyhow!("read model artifact {}: {e}", path.display()))?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let actual = format!("{:x}", hasher.finalize());
        if actual != expected {
            anyhow::bail!(
                "model artifact {} digest mismatch: expected {expected}, got {actual}",
                path.display()
            );
        }
    }
    Ok(())
}

impl Default for LocalConfig {
    fn default() -> Self {
        // Offline/dev default: mock allowed. Production should set
        // `allow_mock_fallback: false` and treat only `EmbeddingHealth::Real`
        // as production-ready.
        Self::new("sentence-transformers/all-MiniLM-L6-v2", 384)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn verify_rejects_oversize() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model.bin");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(&[0u8; 32]).unwrap();
        let err = verify_model_artifact(&path, None, Some(8)).unwrap_err();
        assert!(err.to_string().contains("exceeds max_artifact_bytes"));
    }
}
