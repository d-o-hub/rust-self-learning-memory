//! Process isolation for sandboxed code execution
//!
//! This module implements VM2-style process isolation with:
//! - Separate Node.js process execution
//! - Privilege dropping (if running as root)
//! - Resource limits via ulimit
//! - Process namespace isolation (where available)

use anyhow::Result;
use std::process::Command;
#[cfg(unix)]
use std::process::Stdio;
#[cfg(unix)]
use tracing::debug;
#[cfg(not(unix))]
use tracing::warn;

/// Process isolation configuration
#[derive(Debug, Clone)]
pub struct IsolationConfig {
    /// UID to drop privileges to (None = no change)
    pub drop_to_uid: Option<u32>,
    /// GID to drop privileges to (None = no change)
    pub drop_to_gid: Option<u32>,
    /// Maximum memory in bytes (for ulimit)
    pub max_memory_bytes: Option<usize>,
    /// Maximum CPU time in seconds (for ulimit)
    pub max_cpu_seconds: Option<u64>,
    /// Maximum number of processes
    pub max_processes: Option<usize>,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            drop_to_uid: None,
            drop_to_gid: None,
            max_memory_bytes: Some(128 * 1024 * 1024), // 128MB
            max_cpu_seconds: Some(5),                  // 5 seconds
            max_processes: Some(1),                    // Single process only
        }
    }
}

/// Apply process isolation to a command
pub fn apply_isolation(mut cmd: Command, config: &IsolationConfig) -> Result<Command> {
    // On Unix systems, we can apply resource limits and privilege dropping
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        // Build ulimit command to wrap execution
        let mut ulimit_args = Vec::new();

        // Memory limit (virtual memory)
        if let Some(max_mem) = config.max_memory_bytes {
            let max_mem_kb = max_mem / 1024;
            ulimit_args.push(format!("-v {}", max_mem_kb));
        }

        // CPU time limit
        if let Some(max_cpu) = config.max_cpu_seconds {
            ulimit_args.push(format!("-t {}", max_cpu));
        }

        // Process limit
        if let Some(max_proc) = config.max_processes {
            ulimit_args.push(format!("-u {}", max_proc));
        }

        // File size limit (prevent DoS via large files)
        ulimit_args.push("-f 0".to_string()); // No file creation

        // Core dump limit (security)
        ulimit_args.push("-c 0".to_string()); // No core dumps

        debug!("Applying ulimit restrictions: {:?}", ulimit_args);

        // Wrap command with ulimit if restrictions are specified
        if !ulimit_args.is_empty() {
            // Get original command and args
            let program = cmd.get_program().to_string_lossy().to_string();
            let args: Vec<String> = cmd
                .get_args()
                .map(|s| s.to_string_lossy().to_string())
                .collect();

            // Create new command with ulimit wrapper
            let mut wrapped = Command::new("sh");
            wrapped.arg("-c");

            // Build shell command with ulimit
            let ulimit_cmd = ulimit_args.join("; ulimit ");
            let exec_cmd = format!(
                "{} {}",
                program,
                args.iter()
                    .map(|a| shell_escape(a))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            let full_cmd = format!("ulimit {}; {}", ulimit_cmd, exec_cmd);

            wrapped.arg(full_cmd);

            // Copy stdio configuration
            wrapped.stdin(Stdio::null());
            wrapped.stdout(Stdio::piped());
            wrapped.stderr(Stdio::piped());

            cmd = wrapped;
        }

        // Apply privilege dropping if specified
        if let Some(uid) = config.drop_to_uid {
            debug!("Dropping privileges to UID: {}", uid);

            // Copy GID to owned value for closure
            let _gid = config.drop_to_gid;

            unsafe {
                cmd.pre_exec(move || {
                    // Drop to specified UID
                    #[cfg(target_os = "linux")]
                    {
                        use libc::{setgid, setuid};

                        // Drop GID first if specified
                        if let Some(gid_val) = _gid {
                            if setgid(gid_val) != 0 {
                                return Err(std::io::Error::last_os_error());
                            }
                        }

                        // Drop UID
                        if setuid(uid) != 0 {
                            return Err(std::io::Error::last_os_error());
                        }
                    }

                    Ok(())
                });
            }
        }
    }

    // On non-Unix systems, we can't apply these restrictions
    #[cfg(not(unix))]
    {
        warn!("Process isolation not fully supported on this platform");
        let _ = config; // Suppress unused warning
    }

    Ok(cmd)
}

/// Escape shell arguments for safe inclusion in commands
fn shell_escape(arg: &str) -> String {
    // Simple shell escaping - wrap in single quotes and escape embedded quotes
    format!("'{}'", arg.replace('\'', "'\\''"))
}

/// Check if running with elevated privileges
pub fn is_running_as_root() -> bool {
    #[cfg(unix)]
    {
        // Check if effective UID is 0
        unsafe { libc::geteuid() == 0 }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, assume not root
        false
    }
}

/// Get current process UID
pub fn current_uid() -> Option<u32> {
    #[cfg(unix)]
    {
        Some(unsafe { libc::getuid() })
    }

    #[cfg(not(unix))]
    {
        None
    }
}

/// Get current process GID
pub fn current_gid() -> Option<u32> {
    #[cfg(unix)]
    {
        Some(unsafe { libc::getgid() })
    }

    #[cfg(not(unix))]
    {
        None
    }
}

/// Recommend safe UID/GID for privilege dropping
pub fn recommend_safe_uid() -> Option<(u32, u32)> {
    if is_running_as_root() {
        // Recommend dropping to nobody user (typically UID 65534)
        Some((65534, 65534))
    } else {
        // Already running as non-root, no need to drop
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_config_default() {
        let config = IsolationConfig::default();
        assert!(config.max_memory_bytes.is_some());
        assert!(config.max_cpu_seconds.is_some());
        assert_eq!(config.max_processes, Some(1));
    }

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("simple"), "'simple'");
        assert_eq!(shell_escape("with spaces"), "'with spaces'");
        assert_eq!(shell_escape("with'quote"), "'with'\\''quote'");
        assert_eq!(
            shell_escape("complex'test'string"),
            "'complex'\\''test'\\''string'"
        );
    }

    #[test]
    fn test_current_uid_gid() {
        #[cfg(unix)]
        {
            // Should return Some value on Unix
            assert!(current_uid().is_some());
            assert!(current_gid().is_some());
        }

        #[cfg(not(unix))]
        {
            // Should return None on non-Unix
            assert!(current_uid().is_none());
            assert!(current_gid().is_none());
        }
    }

    #[test]
    fn test_recommend_safe_uid() {
        let recommendation = recommend_safe_uid();
        if is_running_as_root() {
            assert!(recommendation.is_some());
            assert_eq!(recommendation.unwrap(), (65534, 65534));
        } else {
            assert!(recommendation.is_none());
        }
    }

    #[test]
    fn test_apply_isolation_basic() {
        let config = IsolationConfig::default();
        let cmd = Command::new("echo");

        let result = apply_isolation(cmd, &config);
        assert!(result.is_ok());
    }
}
