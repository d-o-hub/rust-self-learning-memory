# Development Workflow for rust-self-learning-memory

You are working on a **Zero-Trust AI agent memory system** in Rust. Security and code quality are paramount.

## Core Principles

1. **Never Trust, Always Verify**: Every file modification triggers validation hooks
2. **Least Privilege**: Only access files necessary for the current task
3. **Security First**: All commits must pass security checks before push

## File Access Rules

**NEVER edit these files:**
- `.env`, `.env.*` - Environment variables (contain secrets)
- `*.key`, `*.secret` - Credential files
- `.turso/` - Database credentials

**Always use environment variables for:**
- Turso database URLs and tokens
- API keys
- Any authentication credentials

## Development Commands

**Before starting work:**
```bash
# Verify environment
cargo check --all
cargo test --all
```

**After making changes:**
```bash
# Format code (happens automatically via hooks)
cargo fmt --all

# Run lints (happens automatically via hooks)
cargo clippy --all -- -D warnings

# Run tests
cargo test --all
```

**Before committing:**
```bash
# The pre-commit hook will automatically run:
# 1. cargo fmt check
# 2. cargo clippy
# 3. cargo audit (security vulnerabilities)
# 4. cargo deny check (licenses, advisories)
# 5. cargo test
# 6. Secret scanning
```

## Code Style Guidelines

- Keep files ≤ 500 LOC (split into submodules if needed)
- Use `anyhow::Result` for top-level functions
- Use `thiserror` for typed errors
- Always use `tokio::spawn_blocking` for redb operations
- Document all public APIs
- Add tests for new functionality

## Security Requirements

- **No hardcoded secrets**: Use `std::env::var()` to read from environment
- **Sanitize inputs**: Validate all episode artifacts before storing
- **Parameterized queries**: Never construct SQL with string concatenation
- **Error handling**: Use `?` operator, avoid `.unwrap()` in production code

## Git Workflow

When you complete a task:
1. Verify all tests pass
2. Review changed files
3. Create commits with clear, descriptive messages
4. The hooks will automatically validate security before allowing commits

## Zero-Trust Hooks

This project uses Claude Code hooks to enforce security:

- **PreToolUse**: Validates file access and blocks editing of sensitive files
- **PostToolUse**: Auto-formats code, runs Clippy, and executes tests
- **Stop**: Performs final verification before session ends

## Troubleshooting

If hooks fail:
1. Read the error message carefully
2. Fix the issue (formatting, linting, tests, security)
3. Try again

If you need to bypass a hook temporarily (only in emergencies):
- Consult with the team lead first
- Document the reason in the commit message

## Resources

- [AGENTS.md](../AGENTS.md) - Project overview and agent responsibilities
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [TESTING.md](../TESTING.md) - Testing infrastructure and best practices
