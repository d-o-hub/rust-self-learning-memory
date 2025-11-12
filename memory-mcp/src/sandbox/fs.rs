//! File system access restrictions for sandboxed code
//!
//! Implements whitelist-based file system access control with:
//! - Path validation and sanitization
//! - Read-only mode enforcement
//! - Path traversal attack prevention
//! - Symlink resolution and validation

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// File system restrictions configuration
#[derive(Debug, Clone)]
pub struct FileSystemRestrictions {
    /// Allowed paths (whitelist) - only these paths and subdirectories are accessible
    pub allowed_paths: Vec<PathBuf>,
    /// Read-only mode - no write operations allowed
    pub read_only: bool,
    /// Maximum path depth to prevent deep directory attacks
    pub max_path_depth: usize,
    /// Follow symlinks (risky if enabled)
    pub follow_symlinks: bool,
}

impl Default for FileSystemRestrictions {
    fn default() -> Self {
        Self {
            allowed_paths: vec![],
            read_only: true,
            max_path_depth: 10,
            follow_symlinks: false,
        }
    }
}

impl FileSystemRestrictions {
    /// Create a restrictive configuration (deny all)
    pub fn deny_all() -> Self {
        Self {
            allowed_paths: vec![],
            read_only: true,
            max_path_depth: 10,
            follow_symlinks: false,
        }
    }

    /// Create a read-only configuration for specific paths
    pub fn read_only(allowed_paths: Vec<PathBuf>) -> Self {
        Self {
            allowed_paths,
            read_only: true,
            max_path_depth: 10,
            follow_symlinks: false,
        }
    }

    /// Create a read-write configuration for specific paths (use with caution)
    pub fn read_write(allowed_paths: Vec<PathBuf>) -> Self {
        Self {
            allowed_paths,
            read_only: false,
            max_path_depth: 10,
            follow_symlinks: false,
        }
    }

    /// Validate a path for read access
    ///
    /// # Security
    ///
    /// This method performs:
    /// 1. Path canonicalization to resolve .. and symlinks
    /// 2. Path depth validation
    /// 3. Whitelist checking
    /// 4. Path traversal attack detection
    pub fn validate_read_path(&self, path: &Path) -> Result<PathBuf> {
        self.validate_path(path, false)
    }

    /// Validate a path for write access
    pub fn validate_write_path(&self, path: &Path) -> Result<PathBuf> {
        if self.read_only {
            bail!(SecurityError::WriteAccessDenied {
                path: path.to_string_lossy().to_string()
            });
        }
        self.validate_path(path, true)
    }

    /// Internal path validation
    fn validate_path(&self, path: &Path, _is_write: bool) -> Result<PathBuf> {
        // Check if any paths are allowed
        if self.allowed_paths.is_empty() {
            bail!(SecurityError::FileSystemAccessDenied {
                reason: "No file system access allowed (empty whitelist)".to_string()
            });
        }

        // Sanitize path - remove . and .. components
        let sanitized = sanitize_path(path)?;

        // Check path depth
        let depth = sanitized.components().count();
        if depth > self.max_path_depth {
            bail!(SecurityError::PathTooDeep {
                path: sanitized.to_string_lossy().to_string(),
                depth,
                max_depth: self.max_path_depth
            });
        }

        // Resolve symlinks if needed (and allowed)
        let resolved = if self.follow_symlinks {
            canonicalize_path(&sanitized)?
        } else {
            sanitized.clone()
        };

        // Check if path is within allowed paths
        let allowed = self.is_path_allowed(&resolved)?;
        if !allowed {
            warn!(
                "Path access denied: {} (not in whitelist)",
                resolved.display()
            );
            bail!(SecurityError::PathNotInWhitelist {
                path: resolved.to_string_lossy().to_string(),
                allowed_paths: self
                    .allowed_paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect()
            });
        }

        debug!("Path validated: {}", resolved.display());
        Ok(resolved)
    }

    /// Check if a path is within allowed paths
    fn is_path_allowed(&self, path: &Path) -> Result<bool> {
        // Always canonicalize the path being checked for consistent comparison
        let canonical_path = canonicalize_path(path)?;

        for allowed_path in &self.allowed_paths {
            // Canonicalize allowed path for consistent comparison
            let canonical_allowed = canonicalize_path(allowed_path)?;

            // Check if path starts with allowed path
            if canonical_path.starts_with(&canonical_allowed) {
                return Ok(true);
            }

            // Also check if they're the same path
            if canonical_path == canonical_allowed {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// Sanitize a path by removing . and .. components
fn sanitize_path(path: &Path) -> Result<PathBuf> {
    let mut sanitized = PathBuf::new();
    let mut depth = 0i32;

    for component in path.components() {
        match component {
            std::path::Component::Prefix(_) => {
                // Windows prefix
                sanitized.push(component);
            }
            std::path::Component::RootDir => {
                sanitized.push(component);
                depth = 0;
            }
            std::path::Component::CurDir => {
                // Skip . components
                continue;
            }
            std::path::Component::ParentDir => {
                // Handle .. components
                if depth > 0 {
                    sanitized.pop();
                    depth -= 1;
                } else {
                    // Attempted to traverse above root - security violation
                    bail!(SecurityError::PathTraversalAttempt {
                        path: path.to_string_lossy().to_string()
                    });
                }
            }
            std::path::Component::Normal(name) => {
                // Check for suspicious names
                let name_str = name.to_string_lossy();
                if is_suspicious_filename(&name_str) {
                    bail!(SecurityError::SuspiciousFilename {
                        filename: name_str.to_string()
                    });
                }

                sanitized.push(component);
                depth += 1;
            }
        }
    }

    Ok(sanitized)
}

/// Canonicalize a path, handling non-existent paths by canonicalizing the first existing ancestor
fn canonicalize_path(path: &Path) -> Result<PathBuf> {
    // If the path exists, canonicalize it directly
    if path.exists() {
        return path
            .canonicalize()
            .context("Failed to canonicalize existing path");
    }

    // Otherwise, find the first existing ancestor and canonicalize that
    let mut current = path.to_path_buf();
    let mut missing_components = Vec::new();

    loop {
        if current.exists() {
            // Found an existing ancestor, canonicalize it
            let canonical_base = current
                .canonicalize()
                .context("Failed to canonicalize ancestor path")?;

            // Rebuild the path with the missing components
            let mut result = canonical_base;
            for component in missing_components.iter().rev() {
                result.push(component);
            }

            return Ok(result);
        }

        // Try the parent
        if let Some(file_name) = current.file_name() {
            missing_components.push(file_name.to_os_string());
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                // No parent - this shouldn't happen with absolute paths
                // Just return the original path
                return Ok(path.to_path_buf());
            }
        } else {
            // No file name - we're at the root
            return Ok(path.to_path_buf());
        }
    }
}

/// Check if a filename is suspicious
fn is_suspicious_filename(name: &str) -> bool {
    // Check for null bytes
    if name.contains('\0') {
        return true;
    }

    // Check for control characters
    if name.chars().any(|c| c.is_control()) {
        return true;
    }

    // Check for hidden Unicode characters
    if name.chars().any(|c| {
        matches!(
            c,
            '\u{200B}' // Zero-width space
            | '\u{200C}' // Zero-width non-joiner
            | '\u{200D}' // Zero-width joiner
            | '\u{FEFF}' // Zero-width no-break space
        )
    }) {
        return true;
    }

    false
}

/// Security errors for file system operations
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("File system access denied: {reason}")]
    FileSystemAccessDenied { reason: String },

    #[error("Write access denied for path: {path}")]
    WriteAccessDenied { path: String },

    #[error("Path not in whitelist: {path} (allowed: {allowed_paths:?})")]
    PathNotInWhitelist {
        path: String,
        allowed_paths: Vec<String>,
    },

    #[error("Path too deep: {path} (depth: {depth}, max: {max_depth})")]
    PathTooDeep {
        path: String,
        depth: usize,
        max_depth: usize,
    },

    #[error("Path traversal attempt detected: {path}")]
    PathTraversalAttempt { path: String },

    #[error("Suspicious filename detected: {filename}")]
    SuspiciousFilename { filename: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_path_basic() {
        let path = Path::new("/tmp/test");
        let sanitized = sanitize_path(path).unwrap();
        assert_eq!(sanitized, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_sanitize_path_removes_current_dir() {
        let path = Path::new("/tmp/./test");
        let sanitized = sanitize_path(path).unwrap();
        assert_eq!(sanitized, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_sanitize_path_handles_parent_dir() {
        let path = Path::new("/tmp/foo/../test");
        let sanitized = sanitize_path(path).unwrap();
        assert_eq!(sanitized, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_sanitize_path_prevents_traversal_above_root() {
        let path = Path::new("/../etc/passwd");
        let result = sanitize_path(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_suspicious_filename_null_byte() {
        assert!(is_suspicious_filename("file\0name"));
    }

    #[test]
    fn test_is_suspicious_filename_control_chars() {
        assert!(is_suspicious_filename("file\nname"));
        assert!(is_suspicious_filename("file\rname"));
    }

    #[test]
    fn test_is_suspicious_filename_zero_width() {
        assert!(is_suspicious_filename("file\u{200B}name"));
    }

    #[test]
    fn test_is_suspicious_filename_normal() {
        assert!(!is_suspicious_filename("normal_file.txt"));
        assert!(!is_suspicious_filename("file-name.json"));
    }

    #[test]
    fn test_deny_all() {
        let restrictions = FileSystemRestrictions::deny_all();
        let result = restrictions.validate_read_path(Path::new("/tmp/test"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_only_mode_denies_writes() {
        let temp_dir = TempDir::new().unwrap();
        let restrictions = FileSystemRestrictions::read_only(vec![temp_dir.path().to_path_buf()]);

        let test_path = temp_dir.path().join("test.txt");
        let result = restrictions.validate_write_path(&test_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_whitelist_allows_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();

        let restrictions = FileSystemRestrictions::read_write(vec![temp_dir.path().to_path_buf()]);

        let test_path = sub_dir.join("test.txt");
        let result = restrictions.validate_write_path(&test_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_whitelist_denies_outside_paths() {
        let temp_dir = TempDir::new().unwrap();
        let restrictions = FileSystemRestrictions::read_only(vec![temp_dir.path().to_path_buf()]);

        let outside_path = Path::new("/etc/passwd");
        let result = restrictions.validate_read_path(outside_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_depth_limit() {
        let temp_dir = TempDir::new().unwrap();
        let mut restrictions =
            FileSystemRestrictions::read_only(vec![temp_dir.path().to_path_buf()]);
        restrictions.max_path_depth = 2;

        // Create deep path
        let deep_path = temp_dir.path().join("a/b/c/d/e/f/g");

        let result = restrictions.validate_read_path(&deep_path);
        assert!(result.is_err());
    }
}
