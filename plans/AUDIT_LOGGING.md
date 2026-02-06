# Audit Logging for Security Incident Investigation

## Overview

The audit logging module provides comprehensive security event logging for the MCP server. All security-relevant operations are logged in a structured JSON format, enabling security incident investigation and compliance auditing.

## Features

- **Structured JSON Logging**: All events logged in consistent JSON format
- **Configurable Destinations**: Log to stdout, file, or both
- **Log Rotation**: Automatic rotation based on file size with configurable retention
- **Sensitive Data Redaction**: Automatic redaction of sensitive fields (passwords, tokens, secrets)
- **Comprehensive Coverage**: All security-relevant operations are logged

## Configuration

Audit logging is configured via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `AUDIT_LOG_ENABLED` | Enable/disable audit logging | `true` |
| `AUDIT_LOG_DESTINATION` | Where to log (`stdout`, `file`, `both`) | `stdout` |
| `AUDIT_LOG_FILE_PATH` | Path to log file (when using file destination) | `audit.log` |
| `AUDIT_LOG_ENABLE_ROTATION` | Enable log rotation | `true` |
| `AUDIT_LOG_MAX_FILE_SIZE` | Max file size in bytes before rotation | `104857600` (100MB) |
| `AUDIT_LOG_MAX_ROTATED_FILES` | Number of rotated files to keep | `10` |
| `AUDIT_LOG_LEVEL` | Minimum log level (`debug`, `info`, `warn`, `error`) | `info` |
| `AUDIT_LOG_REDACT_FIELDS` | Comma-separated list of fields to redact | `password,token,secret,api_key,private_key` |

## Log Format

Each audit log entry is a JSON object with the following structure:

```json
{
  "timestamp": "2026-01-31T12:00:00Z",
  "level": "info",
  "client_id": "client-123",
  "operation": "create_episode",
  "result": "success",
  "metadata": {
    "episode_id": "uuid-here",
    "task_description": "example task"
  }
}
```

### Fields

- **timestamp**: ISO 8601 timestamp in UTC
- **level**: Log level (debug, info, warn, error)
- **client_id**: Identifier for the client/user performing the operation
- **operation**: The operation being performed
- **result**: Operation result (success, failure, blocked, detected)
- **metadata**: Additional operation-specific details

## Logged Operations

### Episode Operations

| Operation | Description | Log Level |
|-----------|-------------|-----------|
| `create_episode` | New episode creation | info |
| `modify_episode` | Episode modification | info |
| `delete_episode` | Episode deletion | warn |
| `add_episode_step` | Adding step to episode | debug |
| `complete_episode` | Episode completion | info |

### Tag Operations

| Operation | Description | Log Level |
|-----------|-------------|-----------|
| `add_episode_tags` | Add tags to episode | debug |
| `remove_episode_tags` | Remove tags from episode | debug |
| `set_episode_tags` | Replace episode tags | debug |
| `search_episodes_by_tags` | Search by tags | debug |

### Pattern Operations

| Operation | Description | Log Level |
|-----------|-------------|-----------|
| `analyze_patterns` | Pattern analysis | debug |
| `advanced_pattern_analysis` | Advanced statistical analysis | debug |
| `search_patterns` | Pattern search | debug |
| `recommend_patterns` | Pattern recommendations | debug |

### Configuration Operations

| Operation | Description | Log Level |
|-----------|-------------|-----------|
| `config_change` | Configuration modification | warn |
| `embedding_config_change` | Embedding provider change | warn |

### Security Operations

| Operation | Description | Log Level |
|-----------|-------------|-----------|
| `authentication` | Authentication events | info |
| `rate_limit_violation` | Rate limit exceeded | warn |
| `security_violation` | Security violation detected | error |
| `code_execution` | Code execution in sandbox | info |

### Query Operations

| Operation | Description | Log Level |
|-----------|-------------|-----------|
| `query_memory` | Memory query | debug |
| `query_semantic_memory` | Semantic memory query | debug |
| `bulk_episodes` | Bulk episode retrieval | debug |
| `batch_execute` | Batch operation execution | info |

## Usage Examples

### Basic Usage

```rust
use memory_mcp::server::audit::{AuditConfig, AuditLogger};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create logger with default configuration
    let config = AuditConfig::from_env();
    let logger = AuditLogger::new(config).await?;

    // Log an episode creation
    logger.log_episode_creation(
        "client-123",
        "episode-uuid",
        "task description",
        true,
        None
    ).await;

    Ok(())
}
```

### Custom Configuration

```rust
use memory_mcp::server::audit::{AuditConfig, AuditDestination, AuditLogLevel};
use std::collections::HashSet;

let config = AuditConfig {
    enabled: true,
    destination: AuditDestination::Both,
    file_path: Some("/var/log/mcp-audit.log".into()),
    enable_rotation: true,
    max_file_size: 50 * 1024 * 1024, // 50MB
    max_rotated_files: 5,
    log_level: AuditLogLevel::Info,
    redact_fields: {
        let mut fields = HashSet::new();
        fields.insert("password".to_string());
        fields.insert("token".to_string());
        fields
    },
};
```

### Integration with MemoryMCPServer

The audit logger is automatically integrated with `MemoryMCPServer`:

```rust
use memory_mcp::server::MemoryMCPServer;
use memory_mcp::types::SandboxConfig;
use memory_core::SelfLearningMemory;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let memory = Arc::new(SelfLearningMemory::new());
    let server = MemoryMCPServer::new(SandboxConfig::default(), memory).await?;

    // Access the audit logger
    let audit_logger = server.audit_logger();

    // Log custom events
    audit_logger.log_security_violation(
        "client-456",
        "unauthorized_access",
        "attempted to access restricted resource"
    ).await;

    Ok(())
}
```

## Log Rotation

When log rotation is enabled:

1. When the current log file reaches `max_file_size`, it is renamed with a `.1` suffix
2. Existing rotated files are incremented (`.1` becomes `.2`, etc.)
3. Files beyond `max_rotated_files` are deleted
4. A new empty log file is created

Example rotation sequence:
```
audit.log      →  audit.log.1
audit.log.1    →  audit.log.2
audit.log.2    →  audit.log.3
... (up to max_rotated_files)
```

## Sensitive Data Redaction

By default, the following fields are redacted:
- `password`
- `token`
- `secret`
- `api_key`
- `private_key`

Any field name containing these substrings will have its value replaced with `[REDACTED]`.

Example:
```json
// Before redaction
{
  "username": "admin",
  "password": "supersecret123",
  "api_key": "abc123xyz"
}

// After redaction
{
  "username": "admin",
  "password": "[REDACTED]",
  "api_key": "[REDACTED]"
}
```

## Security Considerations

1. **Immutable Logs**: Audit logs should be treated as append-only. Consider:
   - Using append-only file systems
   - Forwarding logs to a SIEM system
   - Regular log backups

2. **Access Control**: Restrict access to audit log files:
   ```bash
   chmod 600 /var/log/mcp-audit.log
   chown mcp:mcp /var/log/mcp-audit.log
   ```

3. **Log Integrity**: Consider signing or hashing log files to detect tampering

4. **Retention Policy**: Define and enforce log retention policies based on compliance requirements

## Testing

Run the audit logging tests:

```bash
cargo test --package memory-mcp --test audit_tests
```

## Troubleshooting

### Logs not appearing

1. Check `AUDIT_LOG_ENABLED` is set to `true`
2. Verify `AUDIT_LOG_LEVEL` is set appropriately
3. Check file permissions if logging to file
4. Review application logs for initialization errors

### Log file not created

1. Ensure parent directory exists and is writable
2. Check disk space availability
3. Verify file path is valid

### Performance issues

1. Consider using `AuditDestination::File` instead of `Both` in high-throughput scenarios
2. Increase `max_file_size` to reduce rotation frequency
3. Use SSD storage for log files
4. Consider async log shipping to external systems
