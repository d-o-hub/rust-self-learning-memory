# Security Policy - Zero-Trust Architecture

This document describes the zero-trust security architecture implemented in the rust-self-learning-memory project.

## Overview

This project implements a **Zero-Trust security model** with multiple layers of defense:

1. **Never Trust, Always Verify** - All operations are validated
2. **Least Privilege** - Minimal access granted by default
3. **Assume Breach** - Design for resilience

## Security Layers

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

#### cargo-geiger
- Detects unsafe code usage
- Generates reports on unsafe blocks
- Helps minimize attack surface

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
- **Unsafe Code**: cargo-geiger detection

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
cargo install cargo-geiger --locked
cargo install cargo-llvm-cov --locked

# Verify installation
cargo audit --version
cargo deny --version
cargo geiger --version
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

## Audit Log

All security-relevant events should be logged:

- Dependency updates (Dependabot PRs)
- Security scan results (CI artifacts)
- Vulnerability fixes (CHANGELOG entries)
- Policy changes (git history)

---

**Last Updated**: 2025-11-06
**Version**: 1.0
**Status**: Active
