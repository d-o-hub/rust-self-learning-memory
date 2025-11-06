# Security Token Management

## ⚠️ IMPORTANT: Token Rotation Required

**If you shared your Turso authentication token in plain text (chat, email, commit, etc.), you MUST rotate it immediately.**

### How to Rotate Your Turso Token

1. **Log in to Turso Dashboard**
   - Visit: https://app.turso.tech
   - Navigate to your database settings

2. **Generate New Token**
   ```bash
   turso db tokens create <database-name>
   ```

3. **Update Local Environment**
   - Update `.env` file with new token
   - Never commit `.env` to git (it's already in .gitignore)

4. **Update CI/CD Secrets**
   - GitHub: Settings → Secrets and variables → Actions
   - Update `TEST_TURSO_TOKEN` and `TURSO_TOKEN` secrets

5. **Revoke Old Token**
   ```bash
   turso db tokens revoke <database-name> <old-token-id>
   ```

## Token Security Best Practices

### ✅ DO:
- Store tokens in `.env` files (git-ignored)
- Use environment variables in code (`std::env::var()`)
- Rotate tokens regularly (every 90 days minimum)
- Use separate tokens for development, testing, and production
- Revoke tokens immediately after exposure
- Use GitHub Actions secrets for CI/CD
- Set token expiration dates when creating

### ❌ DON'T:
- Hard-code tokens in source code
- Commit tokens to git repositories
- Share tokens in plain text (chat, email, screenshots)
- Use production tokens in development/testing
- Reuse tokens across multiple services
- Store tokens in documentation files committed to git

## Environment Variable Naming Convention

Use clear naming to distinguish between environments:

```bash
# Development (local database)
DEV_TURSO_URL=libsql://dev-yourproject.turso.io
DEV_TURSO_TOKEN=your-dev-token

# Testing (test database)
TEST_TURSO_URL=libsql://test-yourproject.turso.io
TEST_TURSO_TOKEN=your-test-token

# Production (production database)
TURSO_URL=libsql://prod-yourproject.turso.io
TURSO_TOKEN=your-production-token
```

## Token Exposure Checklist

If you suspect token exposure, complete this checklist immediately:

- [ ] **Immediate**: Revoke the exposed token
- [ ] **Within 5 minutes**: Generate and deploy new token
- [ ] **Within 15 minutes**: Review database access logs
- [ ] **Within 1 hour**: Check for unauthorized database modifications
- [ ] **Within 24 hours**: Complete security audit of affected systems
- [ ] **Document**: Log the incident and actions taken

## CI/CD Configuration

### GitHub Actions Secrets

Required secrets for GitHub Actions workflows:

```yaml
# .github/workflows/ci.yml
env:
  TEST_TURSO_URL: ${{ secrets.TEST_TURSO_URL }}
  TEST_TURSO_TOKEN: ${{ secrets.TEST_TURSO_TOKEN }}
  TURSO_URL: ${{ secrets.TURSO_URL }}
  TURSO_TOKEN: ${{ secrets.TURSO_TOKEN }}
```

To add secrets:
1. Go to repository Settings
2. Navigate to Secrets and variables → Actions
3. Click "New repository secret"
4. Add each secret with the exact name

### Required Secrets:

| Secret Name | Description | When to Use |
|-------------|-------------|-------------|
| `TEST_TURSO_URL` | Test database URL | Integration tests in CI |
| `TEST_TURSO_TOKEN` | Test database token | Integration tests in CI |
| `TURSO_URL` | Production database URL | Production deployments |
| `TURSO_TOKEN` | Production database token | Production deployments |

## Token Lifecycle Management

### Creation
```bash
# Create a new token with 90-day expiration
turso db tokens create mydb --expiration 90d
```

### Listing Active Tokens
```bash
# List all tokens for a database
turso db tokens list mydb
```

### Revocation
```bash
# Revoke a specific token
turso db tokens revoke mydb <token-id>
```

### Rotation Schedule

| Environment | Rotation Frequency | Who Manages |
|-------------|-------------------|-------------|
| Development | 90 days | Developers |
| Testing | 90 days | CI/CD Admin |
| Production | 60 days | Security Team |

## Emergency Contact

If you discover a security incident involving exposed tokens:

1. **Immediate Action**: Revoke the token (don't wait for approval)
2. **Notify**: Post in #security-alerts Slack channel
3. **Document**: Create incident report in security/incidents/
4. **Follow-up**: Schedule post-mortem within 48 hours

## Audit Trail

Keep a log of all token rotations:

```
Date       | Environment | Action  | Performed By | Reason
-----------|-------------|---------|--------------|------------------
2025-01-05 | TEST        | Rotated | Claude       | Scheduled rotation
2025-01-05 | PROD        | Revoked | Admin        | Suspected exposure
```

## Additional Resources

- [Turso Token Management Docs](https://docs.turso.tech/reference/turso-cli#tokens)
- [GitHub Actions Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [OWASP Secrets Management](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
- Project Security Policy: `SECURITY.md`

---

**Last Updated:** 2025-01-05
**Review Frequency:** Quarterly
**Next Review:** 2025-04-05
