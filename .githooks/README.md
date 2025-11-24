# Git Hooks for rust-self-learning-memory

This directory contains git hooks to ensure code quality and prevent releasing broken code.

## Installation

To install these hooks for your local repository:

```bash
# Configure git to use this hooks directory
git config core.hooksPath .githooks

# Make hooks executable (Unix-like systems)
chmod +x .githooks/pre-commit
chmod +x .githooks/pre-push
```

## Available Hooks

### pre-commit

Runs before every commit to ensure code quality:

- **Code Formatting**: Runs `cargo fmt --check` and auto-formats if needed
- **Clippy Linting**: Runs `cargo clippy -- -D warnings`
- **Unit Tests**: Runs `cargo test --lib --bins` to verify tests pass

**Bypass** (not recommended): `git commit --no-verify`

### pre-push

Runs before pushing tags to prevent releasing broken code:

- **Tag Verification**: Detects when pushing tags (e.g., `v0.1.4`)
- **Comprehensive Checks**:
  - Code formatting check
  - Clippy linting
  - Release build compilation
  - Full test suite execution
- **Automatic Checkout**: Temporarily checks out the tag to verify it

**Bypass** (emergency only): `git push --no-verify`

## Why These Hooks?

These hooks were added after v0.1.4 was released with broken tests due to:
- Incomplete refactoring that added parameters to functions
- Tests not updated to match new signatures
- Tests not run before creating the release tag

The hooks prevent this from happening again by:
1. Enforcing test runs before commits
2. Verifying full quality gates before pushing tags
3. Ensuring releases are actually tested

## Troubleshooting

### Hook doesn't run

Check if hooks are executable:
```bash
ls -l .githooks/
# Should show -rwxr-xr-x (executable bit set)
```

Make executable if needed:
```bash
chmod +x .githooks/pre-commit
chmod +x .githooks/pre-push
```

### Hook fails but code seems fine

Run the checks manually:
```bash
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
cargo test --lib --bins
```

### Need to bypass for emergency

Only use `--no-verify` if absolutely necessary:
```bash
git commit --no-verify -m "emergency fix"
git push --no-verify origin v0.1.x
```

**Important**: Always fix the underlying issue immediately after bypassing.

## CI/CD Integration

These hooks complement CI/CD pipelines by catching issues early:
- **Hooks**: Run locally before push (fast feedback)
- **CI/CD**: Run on server after push (verification)

Both layers ensure quality at different stages of the development process.
