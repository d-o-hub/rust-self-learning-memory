# Deployment Security Guide

**Version**: v0.1.13  
**Last Updated**: 2026-02-01  
**Audience**: DevOps Engineers, System Administrators, Security Teams  

---

## Table of Contents

- [Overview](#overview)
- [Production Hardening Checklist](#production-hardening-checklist)
- [Secret Management](#secret-management)
- [Network Security](#network-security)
- [Infrastructure Security](#infrastructure-security)
- [Container Security](#container-security)
- [Database Security](#database-security)
- [Monitoring and Alerting](#monitoring-and-alerting)

---

## Overview

This guide provides comprehensive security hardening procedures for deploying the Rust Self-Learning Memory System in production environments. Following these guidelines ensures defense-in-depth protection against common attack vectors.

### Security Objectives

1. **Confidentiality** - Protect sensitive data and credentials
2. **Integrity** - Ensure data and code haven't been tampered with
3. **Availability** - Maintain service uptime and resilience
4. **Auditability** - Track all security-relevant events

---

## Production Hardening Checklist

### Pre-Deployment

#### Code Security

- [ ] **Zero Clippy Warnings**
  ```bash
  cargo clippy --all -- -D warnings
  ```

- [ ] **Security Audit Pass**
  ```bash
  cargo audit
  ```
  Expected: 0 critical/high/medium vulnerabilities

- [ ] **Supply Chain Check**
  ```bash
  cargo deny check
  ```

- [ ] **License Compliance**
  ```bash
  cargo deny list
  ```
  Verify all licenses are in allowlist (MIT, Apache-2.0, BSD-3-Clause, ISC)

#### Build Security

- [ ] **Release Build with Optimizations**
  ```bash
  cargo build --release --workspace
  ```

- [ ] **Binary Stripping** (optional)
  ```bash
  strip target/release/memory-mcp-server
  ```

- [ ] **Checksum Generation**
  ```bash
  sha256sum target/release/memory-mcp-server > memory-mcp-server.sha256
  ```

### Deployment Configuration

#### Environment Variables

- [ ] **No Hardcoded Secrets**
  ```bash
  # Verify no secrets in environment files
  grep -r "password\|secret\|token\|key" .env* 2>/dev/null || echo "No secrets found"
  ```

- [ ] **Secure File Permissions**
  ```bash
  chmod 600 /etc/memory-mcp/.env
  chown memory-mcp:memory-mcp /etc/memory-mcp/.env
  ```

- [ ] **Required Environment Variables Set**
  ```bash
  # Required
  export TURSO_DATABASE_URL="libsql://your-db.turso.io"
  export TURSO_AUTH_TOKEN="your-token"
  
  # Security
  export MCP_RATE_LIMIT_ENABLED=true
  export AUDIT_LOG_ENABLED=true
  export AUDIT_LOG_DESTINATION=file
  export AUDIT_LOG_FILE_PATH=/var/log/memory-mcp/audit.log
  ```

#### System Hardening

- [ ] **Dedicated Service User**
  ```bash
  sudo useradd --system --no-create-home --shell /bin/false memory-mcp
  ```

- [ ] **Limited File System Access**
  ```bash
  # Create restricted directories
  sudo mkdir -p /var/lib/memory-mcp
  sudo mkdir -p /var/log/memory-mcp
  sudo chown -R memory-mcp:memory-mcp /var/lib/memory-mcp /var/log/memory-mcp
  sudo chmod 750 /var/lib/memory-mcp
  sudo chmod 755 /var/log/memory-mcp
  ```

- [ ] **Systemd Security Directives**
  ```ini
  [Service]
  User=memory-mcp
  Group=memory-mcp
  
  # Security hardening
  NoNewPrivileges=true
  PrivateTmp=true
  ProtectSystem=strict
  ProtectHome=true
  ReadWritePaths=/var/lib/memory-mcp /var/log/memory-mcp
  ProtectKernelTunables=true
  ProtectKernelModules=true
  ProtectControlGroups=true
  RestrictRealtime=true
  RestrictNamespaces=true
  LockPersonality=true
  MemoryDenyWriteExecute=true
  ```

---

## Secret Management

### Secret Storage Options

#### Option 1: Environment Variables (Basic)

**Best for**: Development, single-server deployments

```bash
# /etc/memory-mcp/.env
TURSO_DATABASE_URL=libsql://prod-db.turso.io
TURSO_AUTH_TOKEN=your-token-here
MCP_RATE_LIMIT_ENABLED=true
```

**Security**:
```bash
# Restrict access
sudo chmod 600 /etc/memory-mcp/.env
sudo chown root:root /etc/memory-mcp/.env

# Load in systemd
# /etc/systemd/system/memory-mcp.service
[Service]
EnvironmentFile=/etc/memory-mcp/.env
```

#### Option 2: HashiCorp Vault (Recommended)

**Best for**: Production, multi-server deployments

```bash
# Install Vault agent
sudo apt-get install vault

# Configure Vault agent
# /etc/vault/agent.hcl
vault {
  address = "https://vault.example.com:8200"
}

auto_auth {
  method "approle" {
    config = {
      role_id_file_path = "/etc/vault/role-id"
      secret_id_file_path = "/etc/vault/secret-id"
    }
  }
}

template {
  destination = "/etc/memory-mcp/.env"
  contents = <<EOT
TURSO_DATABASE_URL={{ with secret "secret/data/memory-mcp" }}{{ .Data.data.database_url }}{{ end }}
TURSO_AUTH_TOKEN={{ with secret "secret/data/memory-mcp" }}{{ .Data.data.auth_token }}{{ end }}
EOT
}
```

#### Option 3: Kubernetes Secrets

**Best for**: Container orchestration

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: memory-mcp-secrets
type: Opaque
stringData:
  TURSO_DATABASE_URL: "libsql://prod-db.turso.io"
  TURSO_AUTH_TOKEN: "your-token"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: memory-mcp
spec:
  template:
    spec:
      containers:
      - name: memory-mcp
        image: memory-mcp:v0.1.13
        envFrom:
        - secretRef:
            name: memory-mcp-secrets
```

#### Option 4: Cloud Provider Secret Managers

**AWS Secrets Manager**:
```bash
# Retrieve secret
aws secretsmanager get-secret-value \
  --secret-id memory-mcp/production \
  --query SecretString \
  --output text | jq -r 'to_entries | .[] | "\(.key)=\(.value)"' > /etc/memory-mcp/.env
```

**Google Secret Manager**:
```bash
gcloud secrets versions access latest --secret=memory-mcp-prod > /etc/memory-mcp/.env
```

**Azure Key Vault**:
```bash
az keyvault secret show --name memory-mcp-prod --vault-name my-vault --query value -o tsv > /etc/memory-mcp/.env
```

### Secret Rotation

#### Automated Rotation Script

```bash
#!/bin/bash
# rotate-secrets.sh

set -euo pipefail

SERVICE_NAME="memory-mcp"
LOG_FILE="/var/log/memory-mcp/secret-rotation.log"

log() {
    echo "$(date -Iseconds) - $1" | tee -a "$LOG_FILE"
}

# 1. Generate new Turso token
log "Generating new Turso token..."
NEW_TOKEN=$(turso db tokens create prod-memory-db --expiration 90d)

# 2. Update secret store (example with Vault)
log "Updating Vault secret..."
vault kv put secret/memory-mcp auth_token="$NEW_TOKEN"

# 3. Signal service to reload (graceful rotation)
log "Reloading service..."
sudo systemctl kill -s HUP "$SERVICE_NAME"

# 4. Verify service health
sleep 5
if ! systemctl is-active --quiet "$SERVICE_NAME"; then
    log "ERROR: Service failed after rotation!"
    exit 1
fi

# 5. Revoke old token (after grace period)
log "Scheduling old token revocation..."
(
    sleep 3600  # 1 hour grace period
    # turso db tokens revoke prod-memory-db "$OLD_TOKEN"
    log "Old token revoked"
) &

log "Secret rotation completed successfully"
```

#### Rotation Schedule

| Secret Type | Rotation Frequency | Method |
|-------------|-------------------|--------|
| Database tokens | 90 days | Automated |
| API keys | 30 days | Manual |
| OAuth credentials | 180 days | Manual |
| TLS certificates | 365 days | Automated (Let's Encrypt) |

---

## Network Security

### TLS Configuration

#### Turso Connection Security

Turso connections use TLS by default with the `libsql://` protocol.

```bash
# Verify TLS is enforced
export TURSO_DATABASE_URL="libsql://your-db.turso.io"
# HTTP/HTTPS protocols are explicitly rejected
```

#### Custom TLS Certificates

```rust
// For self-hosted scenarios
use memory_storage_turso::TursoConfig;

let config = TursoConfig {
    tls_cert_path: Some("/etc/ssl/certs/memory-mcp.crt".to_string()),
    tls_key_path: Some("/etc/ssl/private/memory-mcp.key".to_string()),
    ..Default::default()
};
```

### Firewall Configuration

#### UFW (Ubuntu)

```bash
# Default deny
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (adjust port as needed)
sudo ufw allow 22/tcp

# Allow MCP server port
sudo ufw allow 3000/tcp

# Allow health checks from monitoring
sudo ufw allow from 10.0.0.0/8 to any port 3000

# Deny all other incoming
sudo ufw enable
```

#### iptables

```bash
#!/bin/bash
# iptables-rules.sh

# Flush existing rules
iptables -F
iptables -X

# Default policy
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# Allow loopback
iptables -A INPUT -i lo -j ACCEPT

# Allow established connections
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# Allow SSH
iptables -A INPUT -p tcp --dport 22 -j ACCEPT

# Allow MCP server from specific networks only
iptables -A INPUT -p tcp -s 10.0.0.0/8 --dport 3000 -j ACCEPT
iptables -A INPUT -p tcp -s 172.16.0.0/12 --dport 3000 -j ACCEPT

# Rate limit new connections
iptables -A INPUT -p tcp --dport 3000 -m limit --limit 25/minute --limit-burst 100 -j ACCEPT

# Log and drop everything else
iptables -A INPUT -j LOG --log-prefix "IPTABLES-DROP: "
iptables -A INPUT -j DROP
```

### Network Segmentation

#### VPC/Subnet Design

```
┌─────────────────────────────────────────────────────────┐
│                      VPC (10.0.0.0/16)                  │
│  ┌─────────────────┐  ┌─────────────────┐              │
│  │  Public Subnet  │  │  Private Subnet │              │
│  │  (10.0.1.0/24)  │  │  (10.0.2.0/24)  │              │
│  │                 │  │                 │              │
│  │  Load Balancer  │  │  Memory MCP     │              │
│  │  NAT Gateway    │  │  Servers        │              │
│  └─────────────────┘  └─────────────────┘              │
│           │                    │                        │
│           └────────────────────┘                        │
│                    │                                    │
│  ┌─────────────────┐  ┌─────────────────┐              │
│  │  Database Subnet│  │  Admin Subnet   │              │
│  │  (10.0.3.0/24)  │  │  (10.0.4.0/24)  │              │
│  │                 │  │                 │              │
│  │  Turso          │  │  Bastion Host   │              │
│  │  (via TLS)      │  │  Monitoring     │              │
│  └─────────────────┘  └─────────────────┘              │
└─────────────────────────────────────────────────────────┘
```

#### Security Groups (AWS Example)

```json
{
  "GroupName": "memory-mcp-sg",
  "Description": "Security group for Memory MCP servers",
  "VpcId": "vpc-xxxxxxxx",
  "IngressRules": [
    {
      "IpProtocol": "tcp",
      "FromPort": 3000,
      "ToPort": 3000,
      "UserIdGroupPairs": [
        {"GroupId": "sg-load-balancer"}
      ]
    },
    {
      "IpProtocol": "tcp",
      "FromPort": 22,
      "ToPort": 22,
      "IpRanges": [
        {"CidrIp": "10.0.4.0/24"}
      ]
    }
  ],
  "EgressRules": [
    {
      "IpProtocol": "-1",
      "IpRanges": [{"CidrIp": "0.0.0.0/0"}]
    }
  ]
}
```

---

## Infrastructure Security

### Server Hardening

#### OS-Level Security

```bash
#!/bin/bash
# server-hardening.sh

# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install security packages
sudo apt-get install -y fail2ban ufw unattended-upgrades

# Configure automatic updates
sudo dpkg-reconfigure -plow unattended-upgrades

# Configure fail2ban
sudo tee /etc/fail2ban/jail.local > /dev/null <<EOF
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 5

[sshd]
enabled = true
port = 22

[memory-mcp]
enabled = true
port = 3000
filter = memory-mcp
logpath = /var/log/memory-mcp/audit.log
maxretry = 10
bantime = 3600
EOF

# Create fail2ban filter
sudo tee /etc/fail2ban/filter.d/memory-mcp.conf > /dev/null <<EOF
[Definition]
failregex = ^.*"client_id":"<HOST>".*"operation":"authentication".*"result":"failure".*$
ignoreregex = ^.*"result":"success".*$
EOF

sudo systemctl restart fail2ban
```

#### File Integrity Monitoring

```bash
# Install AIDE
sudo apt-get install -y aide

# Initialize database
sudo aideinit

# Daily check
sudo tee /etc/cron.daily/aide-check > /dev/null <<'EOF'
#!/bin/bash
/usr/bin/aide --check | mail -s "AIDE Check $(hostname)" security@example.com
EOF
sudo chmod +x /etc/cron.daily/aide-check
```

### Log Security

#### Secure Log Storage

```bash
# Create dedicated log partition
sudo mkdir -p /var/log/memory-mcp

# Set restrictive permissions
sudo chmod 755 /var/log/memory-mcp
sudo chown root:root /var/log/memory-mcp

# Enable audit log rotation
sudo tee /etc/logrotate.d/memory-mcp > /dev/null <<EOF
/var/log/memory-mcp/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0600 memory-mcp memory-mcp
    postrotate
        /bin/kill -HUP \$(cat /var/run/syslog-ng.pid 2> /dev/null) 2> /dev/null || true
    endscript
}
EOF

# Ship logs to remote SIEM
sudo tee /etc/rsyslog.d/99-memory-mcp.conf > /dev/null <<EOF
# Forward audit logs to SIEM
:programname, isequal, "memory-mcp" @@siem.example.com:514
EOF

sudo systemctl restart rsyslog
```

---

## Container Security

### Dockerfile Security

```dockerfile
# Dockerfile.secure
FROM rust:1.83-slim AS builder

# Create non-root user
RUN useradd -m -u 1000 -s /bin/false appuser

WORKDIR /usr/src/app
COPY . .

# Build release binary
RUN cargo build --release --workspace

# Runtime stage
FROM gcr.io/distroless/cc-debian12

# Copy binary
COPY --from=builder /usr/src/app/target/release/memory-mcp-server /usr/local/bin/

# Use non-root user
USER 1000:1000

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/memory-mcp-server", "health-check"] || exit 1

ENTRYPOINT ["/usr/local/bin/memory-mcp-server"]
```

### Container Runtime Security

#### Docker Security Options

```bash
# Run with security options
docker run -d \
  --name memory-mcp \
  --read-only \
  --security-opt no-new-privileges:true \
  --security-opt seccomp=default.json \
  --cap-drop ALL \
  --cap-add NET_BIND_SERVICE \
  --memory=512m \
  --memory-swap=512m \
  --cpus=1.0 \
  --pids-limit=1000 \
  --tmpfs /tmp:noexec,nosuid,size=100m \
  -p 3000:3000 \
  -e TURSO_DATABASE_URL \
  -e TURSO_AUTH_TOKEN \
  memory-mcp:v0.1.13
```

#### Kubernetes Security

```yaml
# deployment-secure.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: memory-mcp
spec:
  template:
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000
        seccompProfile:
          type: RuntimeDefault
      containers:
      - name: memory-mcp
        image: memory-mcp:v0.1.13
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
        resources:
          limits:
            memory: "512Mi"
            cpu: "1000m"
            ephemeral-storage: "1Gi"
          requests:
            memory: "256Mi"
            cpu: "500m"
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: cache
          mountPath: /var/lib/memory-mcp
      volumes:
      - name: tmp
        emptyDir:
          sizeLimit: 100Mi
      - name: cache
        emptyDir:
          sizeLimit: 500Mi
---
apiVersion: policy/v1
kind: PodSecurityPolicy
metadata:
  name: memory-mcp-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'emptyDir'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
```

---

## Database Security

### Turso Security

#### Connection Security

```bash
# Always use libsql:// protocol (enforces TLS)
export TURSO_DATABASE_URL="libsql://prod-db.turso.io"

# Never use http:// or https://
# export TURSO_DATABASE_URL="http://prod-db.turso.io"  # WRONG!
```

#### Token Management

```bash
# Create token with expiration
turso db tokens create prod-db --expiration 90d

# List active tokens
turso db tokens list prod-db

# Revoke compromised token
turso db tokens revoke prod-db <token-id>
```

#### Database Access Control

```sql
-- Create read-only user (if supported by Turso)
-- Note: Turso uses token-based auth, this is for reference

-- Limit query results
PRAGMA max_page_count = 1000000;

-- Enable WAL mode for better concurrency
PRAGMA journal_mode = WAL;
```

### redb Cache Security

```bash
# Secure cache directory
sudo mkdir -p /var/lib/memory-mcp
sudo chmod 700 /var/lib/memory-mcp
sudo chown memory-mcp:memory-mcp /var/lib/memory-mcp

# Encrypt cache at rest (if needed)
# Using LUKS for volume encryption
sudo cryptsetup luksFormat /dev/sdb1
sudo cryptsetup open /dev/sdb1 memory-cache
sudo mkfs.ext4 /dev/mapper/memory-cache
sudo mount /dev/mapper/memory-cache /var/lib/memory-mcp
```

---

## Monitoring and Alerting

### Security Monitoring Stack

```yaml
# docker-compose.monitoring.yml
version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"
  
  grafana:
    image: grafana/grafana:latest
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=secure-password
    volumes:
      - grafana-storage:/var/lib/grafana
      - ./grafana-dashboards:/etc/grafana/provisioning/dashboards
    ports:
      - "3001:3000"
  
  alertmanager:
    image: prom/alertmanager:latest
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml
    ports:
      - "9093:9093"

volumes:
  grafana-storage:
```

### Security Alerts

```yaml
# prometheus-alerts.yml
groups:
  - name: security
    rules:
      - alert: UnauthorizedAccessAttempt
        expr: increase(audit_authentication_failures_total[5m]) > 10
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Multiple failed authentication attempts"
          
      - alert: PrivilegeEscalationAttempt
        expr: audit_security_violations_total > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Security violation detected"
          
      - alert: UnusualDataAccess
        expr: rate(audit_bulk_operations_total[10m]) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Unusual bulk data access detected"
```

---

## Security Verification

### Automated Security Scan

```bash
#!/bin/bash
# security-scan.sh

echo "=== Memory MCP Security Scan ==="
echo "Date: $(date)"
echo ""

FAILED=0

# 1. Check for hardcoded secrets
echo "[1/10] Checking for hardcoded secrets..."
if grep -r "password\|secret\|token" --include="*.rs" src/ 2>/dev/null | grep -v "// " | grep -v "test"; then
    echo "  FAIL: Potential hardcoded secrets found"
    FAILED=$((FAILED + 1))
else
    echo "  PASS: No hardcoded secrets detected"
fi

# 2. Verify security audit
echo "[2/10] Running cargo audit..."
if cargo audit 2>&1 | grep -q "error\|critical\|high"; then
    echo "  FAIL: Security vulnerabilities found"
    FAILED=$((FAILED + 1))
else
    echo "  PASS: No critical vulnerabilities"
fi

# 3. Check file permissions
echo "[3/10] Checking file permissions..."
if [ "$(stat -c %a /etc/memory-mcp/.env 2>/dev/null)" != "600" ]; then
    echo "  FAIL: .env file permissions too permissive"
    FAILED=$((FAILED + 1))
else
    echo "  PASS: .env file permissions correct"
fi

# 4. Verify rate limiting
echo "[4/10] Checking rate limiting configuration..."
if [ "$MCP_RATE_LIMIT_ENABLED" != "true" ]; then
    echo "  FAIL: Rate limiting not enabled"
    FAILED=$((FAILED + 1))
else
    echo "  PASS: Rate limiting enabled"
fi

# 5. Check audit logging
echo "[5/10] Checking audit logging..."
if [ "$AUDIT_LOG_ENABLED" != "true" ]; then
    echo "  FAIL: Audit logging not enabled"
    FAILED=$((FAILED + 1))
else
    echo "  PASS: Audit logging enabled"
fi

# 6. Verify TLS configuration
echo "[6/10] Checking TLS configuration..."
if [[ "$TURSO_DATABASE_URL" != libsql://* ]]; then
    echo "  FAIL: Turso URL not using libsql:// (TLS)"
    FAILED=$((FAILED + 1))
else
    echo "  PASS: TLS enforced for database"
fi

# 7. Check service user
echo "[7/10] Checking service user..."
if ! id memory-mcp >/devdev/null 2>&1; then
    echo "  WARN: Dedicated service user not found"
else
    echo "  PASS: Service user exists"
fi

# 8. Verify firewall
echo "[8/10] Checking firewall status..."
if ! sudo ufw status | grep -q "Status: active"; then
    echo "  WARN: Firewall not active"
else
    echo "  PASS: Firewall active"
fi

# 9. Check log rotation
echo "[9/10] Checking log rotation..."
if [ ! -f /etc/logrotate.d/memory-mcp ]; then
    echo "  WARN: Log rotation not configured"
else
    echo "  PASS: Log rotation configured"
fi

# 10. Verify monitoring
echo "[10/10] Checking monitoring..."
if ! systemctl is-active --quiet prometheus 2>/dev/null; then
    echo "  WARN: Prometheus not running"
else
    echo "  PASS: Monitoring active"
fi

echo ""
echo "=== Scan Complete ==="
echo "Failed checks: $FAILED"

if [ $FAILED -gt 0 ]; then
    echo "RESULT: FAILED - Address security issues before deployment"
    exit 1
else
    echo "RESULT: PASSED - Security configuration verified"
    exit 0
fi
```

---

## See Also

- [API Reference](./API_REFERENCE.md)
- [Security Operations Guide](./SECURITY_OPERATIONS.md)
- [Security Features Guide](./SECURITY_FEATURES_GUIDE.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-01  
**Maintained By**: Security Team
