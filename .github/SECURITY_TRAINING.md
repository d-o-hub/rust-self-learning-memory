# Security Training Guide - Zero-Trust Hooks

**Purpose**: This guide helps team members understand and work effectively with the zero-trust security hooks implemented in this project.

**Target Audience**: All developers contributing to rust-self-learning-memory

**Last Updated**: 2025-11-06

---

## Table of Contents

1. [Overview](#overview)
2. [Hook System Fundamentals](#hook-system-fundamentals)
3. [Pre-Tool-Use Hooks](#pre-tool-use-hooks)
4. [Post-Tool-Use Hooks](#post-tool-use-hooks)
5. [Stop Hooks](#stop-hooks)
6. [Common Scenarios](#common-scenarios)
7. [Troubleshooting](#troubleshooting)
8. [Best Practices](#best-practices)
9. [Quiz & Exercises](#quiz--exercises)

---

## Overview

### What Are Security Hooks?

Security hooks are **automated validation scripts** that run during development to enforce security and quality standards. They operate at three key moments:

- **PreToolUse**: Before you perform an action (edit, write, commit)
- **PostToolUse**: After you perform an action (auto-format, lint, test)
- **Stop**: When you end your development session

### Why Zero-Trust?

**Never Trust, Always Verify** - Even AI agents and automated tools must pass security checks before modifying code.

### Zero-Trust Principles

1. **Least Privilege**: Only access files necessary for your task
2. **Assume Breach**: Every operation is validated as if the system is compromised
3. **Defense in Depth**: Multiple layers of security checks

---

## Hook System Fundamentals

### Configuration Location

All hooks are configured in:
- **Config**: `.claude/settings.json`
- **Scripts**: `.claude/hooks/*.sh`

### Hook Execution Flow

```
┌─────────────────────────────────────────────────────┐
│ Developer Action (Edit/Write/Commit)               │
└──────────────┬──────────────────────────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │  PreToolUse Hooks    │ ← Blocks if validation fails
    │  - Protect Secrets   │
    │  - Validate Syntax   │
    │  - Pre-Commit Check  │
    └──────────┬───────────┘
               │ ✅ Pass
               ▼
    ┌──────────────────────┐
    │   Action Executes    │
    │   (File Modified)    │
    └──────────┬───────────┘
               │
               ▼
    ┌──────────────────────┐
    │  PostToolUse Hooks   │ ← Auto-fix or warn
    │  - Auto-format       │
    │  - Clippy Lints      │
    │  - Run Tests         │
    │  - Security Audit    │
    └──────────┬───────────┘
               │
               ▼
    ┌──────────────────────┐
    │  Session Continues   │
    └──────────────────────┘
```

---

## Pre-Tool-Use Hooks

### 1. Protect Sensitive Files

**Script**: `.claude/hooks/protect-secrets.sh`

**Purpose**: Prevents editing of credential files

**Blocked File Patterns**:
- `*.env`, `.env.*` - Environment variables
- `*.secret` - Secret files
- `*.key` - Key files
- `.turso/*` - Database credentials

**Example Scenario**:
```bash
# ❌ This will be BLOCKED:
vim .env

# Output:
# 🚫 BLOCKED: Cannot edit sensitive file: .env
# Reason: Zero-Trust policy prohibits LLM access to credential files
```

**How to Work Around**:
- Use environment variables instead of hardcoded values
- Manually edit `.env` files outside of the Claude Code session
- Never commit credential files to git

---

### 2. Validate Rust Syntax

**Purpose**: Ensures valid Rust syntax before allowing edits

**When It Runs**: Before editing any `*.rs` file

**Example Scenario**:
```rust
// ❌ If you try to write invalid Rust:
fn broken_function( {
    // Missing closing parenthesis
}

// Output:
// ❌ Rust syntax error detected
// Hook BLOCKS the edit
```

**How It Works**:
```bash
# Runs: cargo check --quiet
# If exit code != 0, edit is blocked
```

---

### 3. Pre-Commit Security Check

**Script**: `.claude/hooks/pre-commit-security.sh`

**Purpose**: Comprehensive validation before `git commit`

**Checks Performed** (in order):

1. **Code Formatting** (`cargo fmt --check`)
   - Auto-fixes if issues found

2. **Clippy Lints** (`cargo clippy -- -D warnings`)
   - Blocks commit if warnings found

3. **Security Audit** (`cargo audit`)
   - Blocks if vulnerabilities found in dependencies

4. **Supply Chain Check** (`cargo deny check`)
   - Validates licenses, sources, advisories

5. **Test Execution** (`cargo test --all`)
   - Blocks if any tests fail

6. **Unsafe Code Scan** (`cargo geiger`)
   - Reports unsafe code usage

7. **Secret Scanning** (grep pattern matching)
   - Blocks if hardcoded secrets detected

**Example Scenario**:
```bash
# You attempt to commit:
git commit -m "feat: add new feature"

# Hook output:
# 🔒 Running Zero-Trust pre-commit security checks...
# 📝 Checking code formatting...
# ✓ Code formatted
# 🔍 Running Clippy lints...
# ✓ No issues
# 🛡️  Auditing dependencies...
# ✓ No vulnerabilities
# 📋 Running cargo-deny checks...
# ✓ All policies pass
# 🧪 Running tests...
# ✓ All tests pass
# ☢️  Checking for unsafe code...
# ✓ No unsafe code
# 🔐 Scanning for secrets...
# ✓ No secrets detected
# ✅ All security checks passed!
```

**If Any Check Fails**:
```bash
# Example: Tests fail
# ❌ Tests failed. Fix them before committing.
# [Hook exits with code 1, commit is BLOCKED]
```

---

## Post-Tool-Use Hooks

### 1. Auto-Format Rust Code

**Purpose**: Automatically formats code after editing

**When It Runs**: After editing any `*.rs` file

**Example**:
```rust
// Before (you write this):
fn messy_function(  x:i32,y:i32  )->i32{x+y}

// After auto-format (hook applies this):
fn messy_function(x: i32, y: i32) -> i32 {
    x + y
}

// Output:
// ✓ Formatted src/lib.rs
```

**Tool**: `cargo fmt -- "$file_path"`

---

### 2. Run Clippy Lints

**Purpose**: Enforces Rust best practices with `-D warnings`

**When It Runs**: After editing any `*.rs` file

**Example**:
```rust
// You write:
let x = vec![1, 2, 3];
for i in 0..x.len() {
    println!("{}", x[i]);
}

// Clippy warns:
// ⚠️  Clippy warnings found
// help: for loop over `x.iter()` instead
```

**Tool**: `cargo clippy --quiet -- -D warnings`

**Note**: This hook **blocks** edits with warnings (exit code 1)

---

### 3. Run Tests for Modified Files

**Purpose**: Runs tests in the background after modifications

**When It Runs**: After editing any `src/**/*.rs` file

**Example**:
```bash
# You edit src/storage/turso.rs
# Hook automatically runs:
# cargo test --quiet

# Output (async in background):
# ⚠️  Some tests failed
# (or no output if tests pass)
```

**Note**: Runs in **background mode** (doesn't block your work)

---

### 4. Security Audit

**Purpose**: Checks for dependency vulnerabilities

**When It Runs**: After editing `Cargo.toml` or `Cargo.lock`

**Example**:
```bash
# You add a new dependency to Cargo.toml
# Hook runs: cargo audit --quiet

# If vulnerability found:
# ⚠️  Security vulnerabilities detected
# Run 'cargo audit' to see details
```

**Tool**: `cargo-audit`

---

## Stop Hooks

### Final Verification

**Script**: `.claude/hooks/final-check.sh`

**Purpose**: Ensures everything works before session ends

**When It Runs**: When you end your Claude Code session

**Checks**:
1. **Build Check**: `cargo build --all`
2. **Test Check**: `cargo test --all --quiet`
3. **Cargo.lock Check**: Reminds you to commit if modified

**Example Output**:
```bash
# 🏁 Running final session verification...
# 📊 Verifying Rust code quality...
# ✅ Build passed
# ✅ Tests passed
# 📦 Cargo.lock was modified. Remember to commit it.
# ✅ Session verification complete
```

---

## Common Scenarios

### Scenario 1: Adding a New Feature

**Steps**:
```bash
# 1. Create feature branch
git checkout -b feat/new-memory-retrieval

# 2. Write code (auto-formatted by PostToolUse hook)
vim src/retrieval.rs

# 3. Run tests manually (or wait for background hook)
cargo test

# 4. Commit (triggers PreToolUse pre-commit hook)
git commit -m "feat: add semantic retrieval"
# → All security checks run automatically

# 5. Push
git push origin feat/new-memory-retrieval
```

**Hook Timeline**:
- **After editing**: Auto-format, Clippy, Background tests
- **Before commit**: Full security check suite
- **Session end**: Final build + test verification

---

### Scenario 2: Fixing a Hardcoded Secret

**❌ Wrong Approach**:
```rust
// In src/config.rs
const DB_TOKEN: &str = "turso_abc123xyz"; // ❌ BLOCKED by secret scan
```

**✅ Correct Approach**:
```rust
// In src/config.rs
use std::env;

pub fn get_db_token() -> String {
    env::var("TURSO_TOKEN")
        .expect("TURSO_TOKEN must be set")
}
```

**Environment Setup**:
```bash
# .env (NOT committed to git)
TURSO_TOKEN=turso_abc123xyz

# .gitignore (ensure this is present)
.env
.env.*
```

---

### Scenario 3: Handling Hook Failures

**Problem**: Pre-commit hook fails due to Clippy warnings

**Solution**:
```bash
# 1. Read the Clippy error message carefully
git commit -m "feat: add feature"
# → ❌ Clippy found issues. Fix them before committing.

# 2. Run Clippy manually to see details
cargo clippy --all -- -D warnings

# 3. Fix the issues in your code
vim src/lib.rs

# 4. Try commit again
git commit -m "feat: add feature"
# → ✅ All security checks passed!
```

---

### Scenario 4: Dependency Updates

**Adding New Dependency**:
```bash
# 1. Add to Cargo.toml
vim Cargo.toml
# [dependencies]
# new-crate = "1.0"

# 2. PostToolUse hook runs cargo audit
# → Checks for vulnerabilities in new-crate

# 3. If audit fails:
# ⚠️  Security vulnerabilities detected
# → Remove the dependency or find alternative

# 4. Commit Cargo.lock with changes
git add Cargo.toml Cargo.lock
git commit -m "chore(deps): add new-crate"
# → PreToolUse runs full security suite
```

---

## Troubleshooting

### Issue: "Hook timed out"

**Cause**: Hook script took too long (exceeded timeout)

**Solution**:
```bash
# Check .claude/settings.json for timeout values
# Default timeouts:
# - protect-secrets: 10s
# - validate-syntax: 60s
# - pre-commit: 180s (3 minutes)
# - post-format: 30s
# - post-clippy: 120s
# - post-tests: 300s (5 minutes, background)
# - final-check: 120s

# If timeout is too short, contact maintainer
```

---

### Issue: "cargo-audit not installed"

**Output**:
```
⚠️  cargo-audit not installed. Run: cargo install cargo-audit --locked
```

**Solution**:
```bash
# Install security tools (one-time setup)
cargo install cargo-audit --locked
cargo install cargo-deny --locked
cargo install cargo-geiger --locked
cargo install cargo-llvm-cov --locked

# Verify installation
cargo audit --version
cargo deny --version
```

---

### Issue: "Tests failed in background hook"

**Output**:
```
⚠️  Some tests failed
```

**Solution**:
```bash
# 1. Run tests manually to see details
cargo test --workspace

# 2. Fix failing tests
vim tests/integration_test.rs

# 3. Re-run to verify
cargo test --workspace

# 4. Commit your fixes
git commit -m "fix(tests): resolve integration test failures"
```

---

### Issue: "Pre-commit hook blocks legitimate code"

**Example**: Using `unsafe` block for performance

**Solution**:
```rust
// Document why unsafe is necessary
/// SAFETY: This is safe because we ensure the pointer is valid
/// and the memory is properly aligned. Required for zero-copy
/// deserialization of large episode blobs.
unsafe {
    // ... unsafe code ...
}

// cargo-geiger will report this, but won't block
// Maintainers will review during PR
```

---

## Best Practices

### ✅ DO

1. **Run security tools locally before committing**
   ```bash
   cargo fmt --all
   cargo clippy --all -- -D warnings
   cargo test --all
   cargo audit
   cargo deny check
   ```

2. **Use environment variables for secrets**
   ```rust
   let token = env::var("API_KEY")?;
   ```

3. **Keep hook scripts executable**
   ```bash
   chmod +x .claude/hooks/*.sh
   ```

4. **Document unsafe code blocks**
   ```rust
   /// SAFETY: Reasoning here
   unsafe { ... }
   ```

5. **Review hook outputs carefully**
   - Read error messages
   - Understand what failed
   - Fix the root cause

6. **Commit Cargo.lock with changes**
   ```bash
   git add Cargo.toml Cargo.lock
   ```

---

### ❌ DON'T

1. **Don't hardcode secrets**
   ```rust
   // ❌ NEVER DO THIS
   const API_KEY: &str = "sk_live_abc123";
   ```

2. **Don't bypass hooks without justification**
   - Hooks exist for a reason
   - If you need to bypass, discuss with team first

3. **Don't commit credential files**
   ```bash
   # ❌ NEVER
   git add .env
   ```

4. **Don't ignore Clippy warnings**
   ```bash
   # ❌ DON'T
   #[allow(clippy::all)]

   # ✅ DO (specific suppression with justification)
   #[allow(clippy::module_name_repetitions)] // False positive: internal API
   ```

5. **Don't use wildcard dependencies**
   ```toml
   # ❌ BLOCKED by cargo-deny
   [dependencies]
   foo = "*"

   # ✅ ALLOWED
   [dependencies]
   foo = "1.2.3"
   ```

---

## Quiz & Exercises

### Quiz

**Q1**: What happens if you try to edit `.env` file?
<details>
<summary>Answer</summary>
The protect-secrets.sh hook blocks the edit with message: "🚫 BLOCKED: Cannot edit sensitive file: .env"
</details>

**Q2**: When does the pre-commit-security.sh hook run?
<details>
<summary>Answer</summary>
It runs before git commit executes, triggered by the PreToolUse hook matcher for Bash commands containing "git commit"
</details>

**Q3**: Which hook runs in background mode?
<details>
<summary>Answer</summary>
The "Run Tests for Modified Files" PostToolUse hook runs in background (run_in_background: true)
</details>

**Q4**: What are the three zero-trust principles?
<details>
<summary>Answer</summary>
1. Never Trust, Always Verify
2. Least Privilege
3. Assume Breach
</details>

**Q5**: Which tool enforces license compliance?
<details>
<summary>Answer</summary>
cargo-deny (configured in deny.toml)
</details>

---

### Practical Exercises

**Exercise 1: Trigger Secret Detection**

Try to add this code and observe the hook behavior:
```rust
// What happens?
let api_key = "sk_test_12345";
```

**Exercise 2: Fix Clippy Warnings**

Add this code and fix the Clippy warnings:
```rust
fn example() {
    let v = vec![1, 2, 3];
    for i in 0..v.len() {
        println!("{}", v[i]);
    }
}
```

**Exercise 3: Add Dependency Safely**

1. Add `serde_json = "1.0"` to Cargo.toml
2. Observe the PostToolUse security audit
3. Commit the changes
4. Watch the PreToolUse pre-commit checks

---

## Additional Resources

- [SECURITY.md](../SECURITY.md) - Comprehensive security policy
- [AGENTS.md](../AGENTS.md) - Project guidelines
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution workflow
- [.claude/CLAUDE.md](../.claude/CLAUDE.md) - Claude Code workflow
- [deny.toml](../deny.toml) - cargo-deny configuration

---

## Getting Help

**If hooks fail repeatedly**:
1. Check the error messages carefully
2. Review this training guide
3. Run tools manually: `cargo fmt`, `cargo clippy`, `cargo test`
4. Ask team members in #security channel
5. File an issue if hooks have bugs

**Emergency Contact**:
- Security issues: Use GitHub Security Advisories
- Hook bugs: File issue with label `hooks`
- Questions: Team chat or discussions

---

**🎓 Congratulations!** You've completed the security training. Remember:
- **Never Trust, Always Verify**
- Hooks protect everyone on the team
- When in doubt, ask before bypassing

**Version**: 1.0
**Last Updated**: 2025-11-06
**Maintainer**: Security Team
