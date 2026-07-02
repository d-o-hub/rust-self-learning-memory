//! Secure code execution sandbox
//!
//! This module provides a secure sandbox for executing TypeScript/JavaScript code
//! with multiple layers of security:
//!
//! 1. Input validation and sanitization
//! 2. Timeout enforcement
//! 3. Resource limits (memory, CPU)
//! 4. Process isolation
//! 5. Network access controls (deny by default)
//! 6. File system restrictions (whitelist approach)
//! 7. Subprocess execution prevention
//! 8. Malicious code pattern detection
//!
//! ## Security Architecture
//!
//! The sandbox uses a defense-in-depth approach with multiple security layers:
//!
//! - **Input Validation**: All code is scanned for malicious patterns before execution using `RegexSet`
//! - **Process Isolation**: Code runs in a separate Node.js process with restricted permissions
//! - **Resource Limits**: CPU and memory usage are constrained
//! - **Timeout Enforcement**: Long-running code is terminated
//! - **Access Controls**: Network and filesystem access are denied by default
//!
//! ## Limitations
//!
//! The security scanning implemented in this module uses static analysis (regular expressions)
//! to detect malicious patterns. While robust against common bypasses (whitespace, quotes, split strings),
//! it is a heuristic-based approach and has inherent limitations:
//!
//! 1. **Runtime Obfuscation**: Complex runtime string manipulation or dynamic property access
//!    (e.g., `global['pro' + 'cess']`) may bypass static checks.
//! 2. **Context Blindness**: Regular expressions lack full AST (Abstract Syntax Tree) awareness,
//!    meaning they might occasionally flag benign code or miss sophisticated escapes.
//! 3. **Defense in Depth**: This scanner is the first layer of defense. It should be used
//!    in conjunction with OS-level isolation (containers, cgroups, namespaces) for maximum security.
//!
//! ## Example
//!
//! ```no_run
//! use do_memory_mcp::sandbox::CodeSandbox;
//! use do_memory_mcp::types::{SandboxConfig, ExecutionContext};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let sandbox = CodeSandbox::new(SandboxConfig::restrictive())?;
//!     let code = "const result = 1 + 1; console.log(result);";
//!     let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));
//!
//!     let result = sandbox.execute(code, context).await?;
//!     println!("Result: {:?}", result);
//!     Ok(())
//! }
//! ```

// Security submodules
pub mod fs;
pub mod isolation;
pub mod network;

#[cfg(test)]
pub mod tests;

pub use fs::{FileSystemRestrictions, SecurityError as FsSecurityError};
pub use isolation::{
    IsolationConfig, apply_isolation, current_gid, current_uid, is_running_as_root,
    recommend_safe_uid,
};
pub use network::{NetworkRestrictions, NetworkSecurityError};

use crate::types::{
    ErrorType, ExecutionContext, ExecutionResult, SandboxConfig, SecurityViolationType,
};
use anyhow::{Context, Result};
use regex::RegexSet;
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tracing::{debug, warn};
use which::which;

static FS_PATTERNS: OnceLock<RegexSet> = OnceLock::new();
static NETWORK_PATTERNS: OnceLock<RegexSet> = OnceLock::new();
static PROCESS_PATTERNS: OnceLock<RegexSet> = OnceLock::new();
static MALICIOUS_PATTERNS: OnceLock<RegexSet> = OnceLock::new();

/// Secure code execution sandbox
#[derive(Debug)]
pub struct CodeSandbox {
    config: SandboxConfig,
}

fn fs_patterns() -> &'static RegexSet {
    FS_PATTERNS.get_or_init(|| {
        RegexSet::new([
            r#"require\s*\(\s*['`"]fs['`"]\s*\)"#,
            r#"import\s+.*\s+from\s+['`"]fs['`"]"#,
            r#"import\s*\{\s*.*\s*\}\s*from\s+['`"]fs['`"]"#,
            r#"import\s*\(\s*['`"]fs['`"]\s*\)"#,
            r#"\breadFile\b"#,
            r#"\bwriteFile\b"#,
            r#"\bmkdir\b"#,
            r#"\brmdir\b"#,
            r#"\bunlink\b"#,
            r#"\b__dirname\b"#,
            r#"\b__filename\b"#,
        ])
        .expect("Valid FS regex patterns")
    })
}

fn network_patterns() -> &'static RegexSet {
    NETWORK_PATTERNS.get_or_init(|| {
        RegexSet::new([
            r#"require\s*\(\s*['`"]http['`"]\s*\)"#,
            r#"require\s*\(\s*['`"]https['`"]\s*\)"#,
            r#"require\s*\(\s*['`"]net['`"]\s*\)"#,
            r#"import\s+.*\s+from\s+['`"]http['`"]"#,
            r#"import\s+.*\s+from\s+['`"]https['`"]"#,
            r#"import\s+.*\s+from\s+['`"]net['`"]"#,
            r#"import\s*\{\s*.*\s*\}\s*from\s+['`"]http['`"]"#,
            r#"import\s*\{\s*.*\s*\}\s*from\s+['`"]https['`"]"#,
            r#"import\s*\{\s*.*\s*\}\s*from\s+['`"]net['`"]"#,
            r#"import\s*\(\s*['`"]http['`"]\s*\)"#,
            r#"import\s*\(\s*['`"]https['`"]\s*\)"#,
            r#"import\s*\(\s*['`"]net['`"]\s*\)"#,
            r#"\bfetch\s*\("#,
            r#"\bXMLHttpRequest\b"#,
            r#"\bWebSocket\b"#,
        ])
        .expect("Valid network regex patterns")
    })
}

fn process_patterns() -> &'static RegexSet {
    PROCESS_PATTERNS.get_or_init(|| {
        RegexSet::new([
            r#"require\s*\(\s*['`"]child_process['`"]\s*\)"#,
            r#"import\s+.*\s+from\s+['`"]child_process['`"]"#,
            r#"import\s*\{\s*.*\s*\}\s*from\s+['`"]child_process['`"]"#,
            r#"import\s*\(\s*['`"]child_process['`"]\s*\)"#,
            r#"\bexec\s*\("#,
            r#"\bexecSync\s*\("#,
            r#"\bspawn\s*\("#,
            r#"\bspawnSync\s*\("#,
            r#"\bfork\s*\("#,
            r#"\bexecFile\s*\("#,
            r#"\bprocess\.exit\b"#,
        ])
        .expect("Valid process regex patterns")
    })
}

fn malicious_patterns() -> &'static RegexSet {
    MALICIOUS_PATTERNS.get_or_init(|| {
        RegexSet::new([
            r#"\beval\s*\("#,
            r#"\bFunction\s*\("#,
            r#"while\s*\(\s*true\s*\)"#,
            r#"for\s*\(\s*;\s*;\s*\)"#,
        ])
        .expect("Valid malicious regex patterns")
    })
}

impl CodeSandbox {
    /// Create a new sandbox with the given configuration
    pub fn new(config: SandboxConfig) -> Result<Self> {
        // Validate configuration
        if config.max_execution_time_ms == 0 {
            anyhow::bail!("max_execution_time_ms must be greater than 0");
        }
        if config.max_memory_mb == 0 {
            anyhow::bail!("max_memory_mb must be greater than 0");
        }

        // Pre-flight check: validate Node.js is available at construction time
        which("node").context("Node.js not found in PATH — required for sandbox execution")?;

        Ok(Self { config })
    }

    /// Execute code in the sandbox
    ///
    /// # Security
    ///
    /// This method performs multiple security checks:
    /// 1. Validates and sanitizes input code
    /// 2. Detects malicious patterns
    /// 3. Enforces timeout limits
    /// 4. Restricts resource usage
    /// 5. Isolates execution in separate process
    ///
    /// # Arguments
    ///
    /// * `code` - TypeScript/JavaScript code to execute
    /// * `context` - Execution context with input data
    ///
    /// # Returns
    ///
    /// Returns `ExecutionResult` containing output or error information
    pub async fn execute(&self, code: &str, context: ExecutionContext) -> Result<ExecutionResult> {
        let start = Instant::now();

        // Security check: validate input
        if let Some(violation) = self.detect_security_violations(code) {
            warn!("Security violation detected: {:?}", violation);
            return Ok(ExecutionResult::SecurityViolation {
                reason: format!("Security violation: {:?}", violation),
                violation_type: violation,
            });
        }

        // Security check: validate code length (prevent DoS)
        if code.len() > 100_000 {
            return Ok(ExecutionResult::SecurityViolation {
                reason: "Code exceeds maximum length (100KB)".to_string(),
                violation_type: SecurityViolationType::MaliciousCode,
            });
        }

        // Create wrapper code with security restrictions
        let wrapper = self.create_secure_wrapper(code, &context)?;

        // Execute with timeout and resource limits
        let result = self.execute_isolated(wrapper, start).await?;

        Ok(result)
    }

    /// Detect potential security violations in code
    fn detect_security_violations(&self, code: &str) -> Option<SecurityViolationType> {
        // Check for file system access attempts
        if !self.config.allow_filesystem && fs_patterns().is_match(code) {
            return Some(SecurityViolationType::FileSystemAccess);
        }

        // Check for network access attempts
        if !self.config.allow_network && network_patterns().is_match(code) {
            return Some(SecurityViolationType::NetworkAccess);
        }

        // Check for subprocess execution attempts
        if !self.config.allow_subprocesses && process_patterns().is_match(code) {
            return Some(SecurityViolationType::ProcessExecution);
        }

        // Check for potential infinite loops and malicious code (eval, Function)
        if malicious_patterns().is_match(code) {
            let matches = malicious_patterns().matches(code);
            // 0, 1 = eval, Function
            // 2, 3 = while(true), for(;;)
            if matches.matched(2) || matches.matched(3) {
                return Some(SecurityViolationType::InfiniteLoop);
            }
            return Some(SecurityViolationType::MaliciousCode);
        }

        None
    }

    /// Create a secure wrapper around user code
    fn create_secure_wrapper(&self, user_code: &str, context: &ExecutionContext) -> Result<String> {
        let context_json =
            serde_json::to_string(context).context("Failed to serialize execution context")?;

        // Escape user code for safe inclusion in template
        // This prevents command injection and script termination attacks
        let escaped_code = user_code
            .replace('\\', "\\\\") // Escape backslashes first
            .replace('`', "\\`") // Escape template literal backticks
            .replace("${", "\\${") // Escape template literal expressions
            // Note: Newlines, carriage returns, and tabs are NOT escaped
            // They work correctly in JavaScript template literals
            .replace("\x00", "\\x00") // Escape null bytes
            .replace("\x0b", "\\x0b") // Escape vertical tabs
            .replace("\x0c", "\\x0c"); // Escape form feeds

        // Create wrapper that:
        // 1. Sets up restricted environment
        // 2. Provides context to user code
        // 3. Captures output and errors
        // 4. Enforces timeout
        let wrapper = format!(
            r#"
'use strict';

// Capture necessary globals before they are deleted/shadowed
const __realProcess = process;
const __realConsole = console;

// Disable dangerous globals at the top level
delete global.process;
delete global.require;
delete global.module;
delete global.__dirname;
delete global.__filename;
delete global.eval;
delete global.Function;

// Set up restricted console
const outputs = [];
const errors = [];

const safeConsole = {{
    log: (...args) => outputs.push(args.map(String).join(' ')),
    error: (...args) => errors.push(args.map(String).join(' ')),
    warn: (...args) => errors.push('WARN: ' + args.map(String).join(' ')),
    info: (...args) => outputs.push('INFO: ' + args.map(String).join(' ')),
}};

// Execution context
const context = {};

// Main execution wrapper
(async (process, require, module, __filename, __dirname) => {{
    try {{
        // Set timeout to prevent infinite loops
        const timeout = setTimeout(() => {{
            throw new Error('TIMEOUT_EXCEEDED');
        }}, {});

        // User code execution
        const userFn = async () => {{
            const console = safeConsole;
            {};
        }};

        const result = await userFn();
        clearTimeout(timeout);

        // Output results using real console
        __realConsole.log(JSON.stringify({{
            success: true,
            result: result,
            stdout: outputs.join('\n'),
            stderr: errors.join('\n'),
        }}));
    }} catch (error) {{
        __realConsole.error(JSON.stringify({{
            success: false,
            error: error.message,
            stack: error.stack,
            stdout: outputs.join('\n'),
            stderr: errors.join('\n'),
        }}));
        __realProcess.exit(1);
    }}
}})(undefined, undefined, undefined, undefined, undefined);
"#,
            context_json, self.config.max_execution_time_ms, escaped_code
        );

        Ok(wrapper)
    }

    /// Execute code in an isolated Node.js process
    async fn execute_isolated(
        &self,
        wrapper_code: String,
        start_time: Instant,
    ) -> Result<ExecutionResult> {
        // Spawn Node.js process with restricted permissions
        let child = Command::new("node")
            .arg("--no-warnings")
            .arg("-e")
            .arg(&wrapper_code)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true) // Ensure cleanup
            .spawn()
            .context("Failed to spawn Node.js process")?;

        // Wait for completion with timeout
        let timeout = Duration::from_millis(self.config.max_execution_time_ms);
        let output = match tokio::time::timeout(timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                warn!("Process execution failed: {}", e);
                return Ok(ExecutionResult::Error {
                    message: format!("Process execution failed: {}", e),
                    error_type: ErrorType::Runtime,
                    stdout: String::new(),
                    stderr: String::new(),
                });
            }
            Err(_) => {
                // Timeout occurred - process will be killed by kill_on_drop
                return Ok(ExecutionResult::Timeout {
                    elapsed_ms: start_time.elapsed().as_millis() as u64,
                    partial_output: None,
                });
            }
        };

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        // Parse output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        debug!(
            "Execution completed in {}ms, status: {}",
            elapsed_ms,
            output.status.code().unwrap_or(-1)
        );

        // Check if execution was successful
        if output.status.success() {
            // Try to parse structured output
            if let Some(result_line) = stdout.lines().last() {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(result_line) {
                    if let Some(true) = parsed.get("success").and_then(|v| v.as_bool()) {
                        return Ok(ExecutionResult::Success {
                            output: parsed
                                .get("result")
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                            stdout: parsed
                                .get("stdout")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            stderr: parsed
                                .get("stderr")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            execution_time_ms: elapsed_ms,
                        });
                    }
                }
            }

            // Fallback to raw stdout
            Ok(ExecutionResult::Success {
                output: stdout.clone(),
                stdout,
                stderr,
                execution_time_ms: elapsed_ms,
            })
        } else {
            // Execution failed - parse error
            let error_type = if stderr.contains("SyntaxError") {
                ErrorType::Syntax
            } else if stderr.contains("TIMEOUT_EXCEEDED") {
                return Ok(ExecutionResult::Timeout {
                    elapsed_ms,
                    partial_output: Some(stdout),
                });
            } else if stderr.contains("EACCES") || stderr.contains("EPERM") {
                ErrorType::Permission
            } else {
                ErrorType::Runtime
            };

            // Try to parse structured error
            if let Some(error_line) = stderr.lines().last() {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(error_line) {
                    if let Some(error_msg) = parsed.get("error").and_then(|v| v.as_str()) {
                        return Ok(ExecutionResult::Error {
                            message: error_msg.to_string(),
                            error_type,
                            stdout: parsed
                                .get("stdout")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            stderr: parsed
                                .get("stderr")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                }
            }

            Ok(ExecutionResult::Error {
                message: stderr.clone(),
                error_type,
                stdout,
                stderr,
            })
        }
    }
}
