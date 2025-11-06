# CI/CD Configuration Guide

This guide explains how to configure GitHub Actions secrets for continuous integration and deployment.

## Overview

The project uses GitHub Actions for automated testing, security scanning, and deployment. The workflows require access to Turso database credentials stored as repository secrets.

## Required Secrets

### 1. Test Database Credentials

Used by integration tests in CI pipeline.

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `TEST_TURSO_URL` | Test database URL | `libsql://test-yourproject.turso.io` |
| `TEST_TURSO_TOKEN` | Test database auth token | `eyJhbGci...` (JWT token) |

**Purpose:** Run integration tests against a dedicated test database in CI/CD

### 2. Production Database Credentials (Optional)

Used for production deployments.

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `TURSO_URL` | Production database URL | `libsql://prod-yourproject.turso.io` |
| `TURSO_TOKEN` | Production database auth token | `eyJhbGci...` (JWT token) |

**Purpose:** Deploy and run production services

## How to Add Secrets to GitHub

### Step-by-Step Instructions

1. **Navigate to Repository Settings**
   - Go to your GitHub repository
   - Click **Settings** (top menu)
   - In left sidebar, navigate to **Secrets and variables** → **Actions**

2. **Add New Secret**
   - Click **New repository secret** button
   - Enter the secret name (exactly as shown above)
   - Paste the secret value
   - Click **Add secret**

3. **Repeat for All Required Secrets**
   - Add all secrets from the table above
   - Verify each secret name is spelled correctly

### Visual Guide

```
GitHub Repository
└── Settings
    └── Secrets and variables
        └── Actions
            ├── Repository secrets
            │   ├── TEST_TURSO_URL ✓
            │   ├── TEST_TURSO_TOKEN ✓
            │   ├── TURSO_URL (optional)
            │   └── TURSO_TOKEN (optional)
            └── [New repository secret] button
```

## Workflow Configuration

The secrets are automatically injected into workflow environments:

### Example: CI Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    env:
      TEST_TURSO_URL: ${{ secrets.TEST_TURSO_URL }}
      TEST_TURSO_TOKEN: ${{ secrets.TEST_TURSO_TOKEN }}

    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run integration tests
        run: cargo test --all -- --include-ignored
```

### Example: Deployment Workflow

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    env:
      TURSO_URL: ${{ secrets.TURSO_URL }}
      TURSO_TOKEN: ${{ secrets.TURSO_TOKEN }}

    steps:
      - uses: actions/checkout@v4
      - name: Deploy to production
        run: ./deploy.sh
```

## Obtaining Turso Credentials

### Create Test Database

```bash
# Install Turso CLI
curl -sSfL https://get.tur.so/install.sh | bash

# Login to Turso
turso auth login

# Create test database
turso db create test-yourproject --group default

# Get database URL
turso db show test-yourproject --url

# Generate auth token (90-day expiration)
turso db tokens create test-yourproject --expiration 90d
```

### Create Production Database

```bash
# Create production database
turso db create prod-yourproject --group default

# Get database URL
turso db show prod-yourproject --url

# Generate auth token
turso db tokens create prod-yourproject --expiration 90d
```

## Security Best Practices

### ✅ DO:
- Use separate databases for test and production
- Rotate tokens every 90 days
- Use tokens with expiration dates
- Set minimum required permissions
- Audit secret access regularly
- Document secret rotation in SECURITY_TOKENS.md

### ❌ DON'T:
- Share production tokens with test environments
- Use tokens without expiration
- Commit tokens to repository
- Share secrets in pull request comments
- Grant unnecessary permissions

## Testing CI Configuration

### Verify Secrets Are Set

1. Go to **Actions** tab in GitHub repository
2. Manually trigger workflow or push a commit
3. Check workflow run logs
4. Integration tests should run successfully

### Debugging Failed Workflows

#### Error: "TEST_TURSO_URL not set"

**Cause:** Secret not configured or name mismatch

**Solution:**
- Verify secret name is exactly `TEST_TURSO_URL` (case-sensitive)
- Check secret is set in repository settings
- Ensure workflow has correct `env:` configuration

#### Error: "Auth failed"

**Cause:** Invalid or expired token

**Solution:**
```bash
# Check token validity
turso db tokens list test-yourproject

# Generate new token if needed
turso db tokens create test-yourproject --expiration 90d

# Update GitHub secret with new token
```

#### Error: "Connection refused"

**Cause:** Database doesn't exist or network issue

**Solution:**
```bash
# Verify database exists
turso db list

# Check database status
turso db show test-yourproject

# Test connection locally
turso db shell test-yourproject
```

## Workflow Examples

### Full CI Workflow with Secrets

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  TEST_TURSO_URL: ${{ secrets.TEST_TURSO_URL }}
  TEST_TURSO_TOKEN: ${{ secrets.TEST_TURSO_TOKEN }}

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable

      - name: Install security tools
        run: |
          cargo install cargo-audit cargo-deny

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: cargo clippy --all -- -D warnings

      - name: Run unit tests
        run: cargo test --all --lib

      - name: Run integration tests
        run: cargo test --all -- --include-ignored
        continue-on-error: true  # Don't fail if network unavailable

      - name: Security audit
        run: cargo audit

      - name: License check
        run: cargo deny check
```

## Token Rotation Schedule

| Secret | Rotation Frequency | Responsible Team | Notification |
|--------|-------------------|------------------|--------------|
| `TEST_TURSO_TOKEN` | Every 90 days | DevOps | Email 7 days before expiry |
| `TURSO_TOKEN` | Every 60 days | Security Team | Slack + Email 14 days before |

## Monitoring & Alerts

### Set Up Alerts

1. **Token Expiration Alerts**
   - Create calendar reminders for token rotation
   - Set up Turso webhook notifications (if available)

2. **Failed Workflow Alerts**
   - Configure GitHub Actions notifications
   - Monitor workflow status in GitHub dashboard

3. **Database Health Checks**
   - Add periodic health check job in workflows
   - Alert on connection failures

## Emergency Procedures

### Token Compromise

If a token is exposed or compromised:

1. **Immediate (0-5 minutes):**
   ```bash
   # Revoke compromised token
   turso db tokens revoke <database> <token-id>
   ```

2. **Generate New Token (5-10 minutes):**
   ```bash
   # Create new token
   turso db tokens create <database> --expiration 90d
   ```

3. **Update GitHub Secret (10-15 minutes):**
   - Go to repository Settings → Secrets
   - Update the compromised secret with new token
   - Re-run any failed workflows

4. **Document (Within 1 hour):**
   - Log incident in `SECURITY_TOKENS.md`
   - Notify team
   - Review access logs

## Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [GitHub Secrets Guide](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [Turso CLI Reference](https://docs.turso.tech/reference/turso-cli)
- [Turso Token Management](https://docs.turso.tech/reference/turso-cli#tokens)
- Project Security Guide: `../SECURITY_TOKENS.md`
- Testing Guide: `../TESTING.md`

## Support

For issues with CI/CD configuration:

1. Check workflow logs in **Actions** tab
2. Review this documentation
3. Consult `SECURITY_TOKENS.md` for token issues
4. Open issue with label `ci-cd`

---

**Last Updated:** 2025-01-05
**Maintained By:** DevOps Team
**Next Review:** 2025-04-05
