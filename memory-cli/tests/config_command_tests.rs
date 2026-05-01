//! Tests for config command output formatting and serialization
//!
//! These tests verify the human-readable output and JSON serialization
//! of configuration command types.

#![allow(missing_docs)]

use do_memory_cli::commands::{
    CliConfigDisplay, ConfigCheck, ConfigDisplay, ConfigIssue, ConfigValidation,
    ConnectivityStatus, DatabaseConfigDisplay, IssueLevel, StorageConfigDisplay,
};
use do_memory_cli::output::Output;

#[test]
fn test_config_validation_write_human_invalid() {
    let validation = ConfigValidation {
        is_valid: false,
        issues: vec![ConfigIssue {
            level: IssueLevel::Error,
            category: "database".to_string(),
            message: "Test error".to_string(),
            suggestion: Some("Fix it".to_string()),
        }],
        connectivity: ConnectivityStatus {
            turso_connected: false,
            redb_accessible: false,
            latency_ms: None,
            errors: vec!["Connection failed".to_string()],
        },
    };

    let mut buffer = Vec::new();
    validation.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("Configuration has issues"));
    assert!(output.contains("ERROR"));
    assert!(output.contains("Test error"));
    assert!(output.contains("Fix it"));
    assert!(output.contains("Turso: Not connected"));
    assert!(output.contains("redb: Not accessible"));
    assert!(output.contains("Connection failed"));
}

#[test]
fn test_config_validation_write_human_with_warnings() {
    let validation = ConfigValidation {
        is_valid: true,
        issues: vec![ConfigIssue {
            level: IssueLevel::Warning,
            category: "performance".to_string(),
            message: "Cache size is low".to_string(),
            suggestion: Some("Increase cache".to_string()),
        }],
        connectivity: ConnectivityStatus {
            turso_connected: true,
            redb_accessible: true,
            latency_ms: Some(50),
            errors: vec![],
        },
    };

    let mut buffer = Vec::new();
    validation.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("Configuration is valid"));
    assert!(output.contains("WARN"));
    assert!(output.contains("Cache size is low"));
    assert!(output.contains("Turso: Connected"));
    assert!(output.contains("redb: Accessible"));
    assert!(output.contains("Latency: 50ms"));
}

#[test]
fn test_config_check_write_human_with_security_issues() {
    let check = ConfigCheck {
        validation: ConfigValidation {
            is_valid: true,
            issues: vec![],
            connectivity: ConnectivityStatus {
                turso_connected: true,
                redb_accessible: true,
                latency_ms: None,
                errors: vec![],
            },
        },
        recommendations: vec!["Enable progress bars".to_string()],
        security_issues: vec!["Token missing".to_string()],
    };

    let mut buffer = Vec::new();
    check.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("Recommendations"));
    assert!(output.contains("Enable progress bars"));
    assert!(output.contains("Security Issues"));
    assert!(output.contains("Token missing"));
}

#[test]
fn test_config_display_write_human_with_none_urls() {
    let display = ConfigDisplay {
        database: DatabaseConfigDisplay {
            turso_url: None,
            turso_token_configured: false,
            redb_path: None,
        },
        storage: StorageConfigDisplay {
            max_episodes_cache: 1000,
            cache_ttl_seconds: 3600,
            pool_size: 10,
        },
        cli: CliConfigDisplay {
            default_format: "human".to_string(),
            progress_bars: true,
            batch_size: 100,
        },
        features: vec!["turso".to_string(), "redb".to_string()],
    };

    let mut buffer = Vec::new();
    display.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("Turso URL: Not configured"));
    assert!(output.contains("Turso Token: Not configured"));
    assert!(output.contains("redb Path: Not configured"));
    assert!(output.contains("Enabled Features"));
    assert!(output.contains("turso"));
    assert!(output.contains("redb"));
}

#[test]
fn test_config_display_write_human_with_empty_features() {
    let display = ConfigDisplay {
        database: DatabaseConfigDisplay {
            turso_url: Some("file:test.db".to_string()),
            turso_token_configured: true,
            redb_path: Some("memory.redb".to_string()),
        },
        storage: StorageConfigDisplay {
            max_episodes_cache: 500,
            cache_ttl_seconds: 1800,
            pool_size: 5,
        },
        cli: CliConfigDisplay {
            default_format: "json".to_string(),
            progress_bars: false,
            batch_size: 50,
        },
        features: vec![],
    };

    let mut buffer = Vec::new();
    display.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("Turso URL: file:test.db"));
    assert!(output.contains("Turso Token: Configured"));
    assert!(output.contains("redb Path: memory.redb"));
    assert!(!output.contains("Enabled Features"));
}

#[test]
fn test_config_issue_level_info() {
    let validation = ConfigValidation {
        is_valid: true,
        issues: vec![ConfigIssue {
            level: IssueLevel::Info,
            category: "tips".to_string(),
            message: "Consider enabling feature X".to_string(),
            suggestion: None,
        }],
        connectivity: ConnectivityStatus {
            turso_connected: true,
            redb_accessible: true,
            latency_ms: None,
            errors: vec![],
        },
    };

    let mut buffer = Vec::new();
    validation.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("INFO"));
    assert!(output.contains("Consider enabling feature X"));
}

#[test]
fn test_config_validation_json_serialization() {
    let validation = ConfigValidation {
        is_valid: true,
        issues: vec![],
        connectivity: ConnectivityStatus {
            turso_connected: true,
            redb_accessible: true,
            latency_ms: Some(100),
            errors: vec![],
        },
    };

    // Test that we can serialize to JSON
    let json = serde_json::to_string(&validation).unwrap();
    assert!(json.contains("is_valid"));
    assert!(json.contains("turso_connected"));
    assert!(json.contains("latency_ms"));
}

#[test]
fn test_connectivity_status_with_errors() {
    let connectivity = ConnectivityStatus {
        turso_connected: false,
        redb_accessible: false,
        latency_ms: None,
        errors: vec![
            "Turso connection timeout".to_string(),
            "redb permission denied".to_string(),
        ],
    };

    let validation = ConfigValidation {
        is_valid: true,
        issues: vec![],
        connectivity,
    };

    let mut buffer = Vec::new();
    validation.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("Connection errors"));
    assert!(output.contains("Turso connection timeout"));
    assert!(output.contains("redb permission denied"));
}
