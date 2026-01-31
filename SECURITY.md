# Security Policy - Zero-Trust Architecture

This document describes the zero-trust security architecture implemented in the rust-self-learning-memory project.

## Overview

This project implements a **Zero-Trust security model** with multiple layers of defense:

1. **Never Trust, Always Verify** - All operations are validated
2. **Least Privilege** - Minimal access granted by default
3. **Assume Breach** - Design for resilience

## Security Layers

### 0. Code Security (Baseline)

#### Zero Clippy Warnings
- Strict linting with `-D warnings` flag
- All code passes clippy checks
- Security-focused lint rules enabled
- CI enforces zero warnings policy

#### Postcard Serialization
- **[BREAKING]** Migrated from bincode to postcard in v0.1.7
- Postcard provides better security guarantees
- Smaller binary sizes
- Inherent safety without manual size limits
- Prevents deserialization attacks
- Better no-std support for embedded systems

#### Unmaintained Dependencies
- **bincode v1.3.3 (RUSTSEC-2025-0141)**: Unmaintained crate (advisory date: 2025-12-16)
  - **Status**: Transitive dependency through streaming_algorithms, libsql, argmin
  - **Impact**: Low - used only in dependencies, not directly in codebase
  - **Mitigation**: 
    - Monitor upstream dependencies for updates
    - Consider alternatives if upstream doesn't update
    - Prefer postcard for new serialization requirements
    - No direct security vulnerability, but unmaintained status indicates higher risk
  - **References**: https://rustsec.org/advisories/RUSTSEC-2025-0141

#### Input Validation
- **Task Description**: Max 10KB (10,000 characters)
- **Execution Step Observation**: Max 10KB (10,000 characters)
- **Execution Step Parameters**: Max 1MB (1,000,000 characters)
- **Episode Artifacts**: Max 1MB (1,000,000 characters)
- **Episode Steps**: Max 1,000 steps per episode

### 1. Claude Code Hooks (Development-Time Security)

Located in `.claude/settings.json`, hooks enforce security at development time:

#### PreToolUse Hooks
- **Protect Sensitive Files**: Blocks editing of `.env`, `*.secret`, `*.key`, `.turso/` files
- **Secret Detection**: Scans for hardcoded credentials before file modification
- **Rust Syntax Validation**: Verifies syntax before allowing edits
- **Pre-Commit Security**: Runs comprehensive security checks before commits

#### PostToolUse Hooks
- **Auto-format**: Runs `cargo fmt` on all Rust files
- **Clippy Lints**: Enforces Rust best practices with `-D warnings`
- **Test Execution**: Runs tests for modified files
- **Security Audit**: Checks for vulnerabilities on Cargo.toml/lock changes

#### Stop Hooks
- **Final Verification**: Ensures build and tests pass before session ends

### 2. Supply Chain Security

#### cargo-deny (`deny.toml`)
- **Advisories**: Denies known vulnerabilities, yanked crates
- **Licenses**: Allows only MIT, Apache-2.0, BSD-3-Clause, ISC
- **Sources**: Restricts to crates.io only
- **Bans**: Prevents multiple versions and wildcards

#### cargo-audit
- Checks for security vulnerabilities in dependencies
- Runs in CI on every push
- Weekly scheduled scans

### 3. Build-Time Security

#### Cargo Configuration (`.cargo/config.toml`)
- **Overflow Checks**: Enabled in release mode
- **RELRO/BIND_NOW**: Security hardening flags for Linux
- **Custom Aliases**: `security-check`, `quality`, `coverage`

#### Rust Toolchain (`rust-toolchain.toml`)
- Pinned to stable channel
- Includes `rustfmt`, `clippy`, `llvm-tools-preview`
- Ensures consistent toolchain across team

### 4. CI/CD Security (GitHub Actions)

#### Main CI Pipeline (`.github/workflows/ci.yml`)
- **Format Check**: Enforces code style
- **Clippy**: Linting with `-D warnings`
- **Test Matrix**: Ubuntu, macOS, Windows × stable, beta
- **Coverage**: Tracks code coverage with cargo-llvm-cov
- **Security Audit**: rustsec/audit-check
- **Supply Chain**: cargo-deny checks

#### Security Workflow (`.github/workflows/security.yml`)
- **Secret Scanning**: Gitleaks on every push
- **Dependency Review**: GitHub's dependency review action
- **Supply Chain Audit**: cargo-audit with JSON reports
- **Scheduled Scans**: Weekly security checks

#### Dependabot (`.github/dependabot.yml`)
- Weekly automated dependency updates
- Separate for Cargo and GitHub Actions
- Auto-labels with "dependencies" and "security"

## Hook Scripts

### `protect-secrets.sh`
Prevents LLM access to sensitive files and detects hardcoded secrets.

**Blocked Patterns:**
- Files: `*.env`, `*.secret`, `*.key`, `.turso/*`
- Content: `api_key`, `password`, `secret`, `token`, `credential`

### `pre-commit-security.sh`
Comprehensive pre-commit validation:

1. Code formatting (`cargo fmt --check`)
2. Linting (`cargo clippy`)
3. Security audit (`cargo audit`)
4. Supply chain check (`cargo deny check`)
5. Test execution (`cargo test`)
6. Unsafe code detection (`cargo geiger`)
7. Secret scanning (grep for patterns)

### `final-check.sh`
Session-end verification:

1. Full build check
2. Complete test suite
3. Cargo.lock change detection

## Security Requirements

### For Developers

**MUST:**
- Use environment variables for all secrets
- Run security checks before committing
- Document all unsafe code blocks
- Use parameterized queries (no SQL injection)
- Handle errors with `?` operator (avoid `.unwrap()`)

**MUST NOT:**
- Hardcode credentials
- Edit `.env` or credential files
- Bypass security hooks without justification
- Use wildcards in dependencies
- Commit secrets to version control

### For Dependencies

**Requirements:**
- Must be from crates.io only
- Must have compatible licenses (MIT, Apache-2.0, BSD-3-Clause, ISC)
- Must not have known security advisories
- Must not be yanked or unmaintained

### Input Validation & Bounds

**Implemented P0 Security Improvements:**

#### Input Size Limits
- **Task Description**: Max 10KB (10,000 characters)
- **Execution Step Observation**: Max 10KB (10,000 characters)
- **Execution Step Parameters**: Max 1MB (1,000,000 characters)
- **Episode Artifacts**: Max 1MB (1,000,000 characters)
- **Episode Steps**: Max 1,000 steps per episode

#### Postcard Serialization (v0.1.7+)
- **Inherent Safety**: Postcard has built-in protection against deserialization attacks
- **No Manual Size Limits Required**: Postcard's design prevents OOM attacks
- **Smaller Binary Sizes**: More efficient serialization
- **Better No-std Support**: Suitable for embedded systems
- **Migration Note**: Existing redb databases must be recreated after upgrade

#### Path Traversal Protection (v0.1.7+)
- **Fixed Vulnerability**: Path traversal in sandbox filesystem access
- **Proper Path Validation**: All paths sanitized before access
- **Whitelist/Blacklist**: Filesystem access restricted to designated directories
- **Prevents Escape**: Cannot access files outside allowed directories
- **Updated Tests**: Security tests now cover path traversal scenarios

#### Error Types
- **QuotaExceeded**: For resource limit enforcement
- **RateLimitExceeded**: For rate limiting with backoff

#### Validation Logic
- All input validation occurs at API boundaries
- Validation errors are logged but don't prevent episode creation (backward compatibility)
- Size limits prevent DoS attacks via oversized inputs
- Postcard serialization provides inherent protection against deserialization attacks
- Path traversal protection prevents unauthorized filesystem access
- All file access in sandbox is validated against whitelist/blacklist

## Reporting Security Vulnerabilities

If you discover a security vulnerability:

1. **DO NOT** open a public issue
2. Email security contact with details
3. Allow 90 days for fix before public disclosure
4. Receive credit in CHANGELOG if desired

## Security Checklist

Before every commit, verify:

- [ ] No hardcoded secrets
- [ ] All tests pass
- [ ] Code formatted with `cargo fmt`
- [ ] No Clippy warnings
- [ ] No new unsafe code (or documented if required)
- [ ] Dependencies audited
- [ ] Licenses compatible

## Tools Installation

```bash
# Install security tools
cargo install cargo-audit --locked
cargo install cargo-deny --locked
cargo install cargo-llvm-cov --locked

# Verify installation
cargo audit --version
cargo deny --version
```

## References

- [SECURITY.md](./.github/SECURITY.md) - GitHub Security Policy 
- [AGENTS.md](./AGENTS.md) - Project overview and guidelines
- [CONTRIBUTING.md](./CONTRIBUTING.md) - Contribution guidelines
- [.claude/CLAUDE.md](./.claude/CLAUDE.md) - Claude Code workflow
- [deny.toml](./deny.toml) - cargo-deny configuration
- [.cargo/config.toml](./.cargo/config.toml) - Cargo configuration

## Compliance

This security architecture helps ensure:

- **OWASP Top 10** mitigations
- **Supply chain security** best practices
- **Secure development lifecycle** (SDL)
- **Principle of least privilege**
- **Defense in depth**

## Recent Security Improvements (2026-01-31)

### Commit: fix(security): remove sensitive files from git tracking (222ff71)

**Files Removed from Git History**:
- `.env` (42 lines) - Contained `MISTRAL_API_KEY`, `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`
- `mcp.json` (20 lines) - Revealed internal system architecture and deployment paths
- `mcp-config-memory.json` (20 lines) - Template for API key injection

**Security Issues Addressed**:
1. **Hardcoded API Keys**: Removed `MISTRAL_API_KEY` from version control
2. **Database Credentials**: Removed `TURSO_DATABASE_URL` and `TURSO_AUTH_TOKEN` from repository
3. **Configuration Templates**: Removed MCP configuration files that exposed deployment patterns

**Git Configuration Improvements**:
- **`.gitignore`** (lines 42-43): Added `.env` to prevent future accidental commits
- **`.gitleaksignore`** (lines 1-6): Configured to allow test keys in local development while blocking real keys

**Best Practices Implemented**:
- All secrets now stored in environment variables
- No hardcoded credentials in source code
- `.env` files excluded from version control
- Configuration documented but not included in repository
- Example configurations provided without sensitive values

**Related CI Integration**: GitHub Actions Security workflow
- [Security Scan Results](https://github.com/d-o-hub/rust-self-learning-memory/actions/runs/21523399928)

### Relationship Module Security Features

**Parameterized Queries**: All database operations use parameterized statements to prevent SQL injection
- UUID validation before database operations
- Type-safe query construction
- String escaping for single quotes in metadata

**Input Validation**:
- JSON serialization with validation prevents code injection
- UUID validation prevents injection attacks
- CASCADE deletes prevent orphaned data

**Schema Security**:
```sql
FOREIGN KEY (from_episode_id) REFERENCES episodes(episode_id) ON DELETE CASCADE
FOREIGN KEY (to_episode_id) REFERENCES episodes(episode_id) ON DELETE CASCADE
```

## Audit Log

All security-relevant events should be logged:

- **2026-01-31**: Removed sensitive files (.env, mcp.json) from git tracking (commit 222ff71)
- **2026-01-31**: Added relationship module with parameterized queries and input validation (commit 5884aae)
- Dependency updates (Dependabot PRs)
- Security scan results (CI artifacts)
- Vulnerability fixes (CHANGELOG entries)
- Policy changes (git history)

---

**Last Updated**: 2026-01-31
**Version**: 1.1
**Status**: Active
