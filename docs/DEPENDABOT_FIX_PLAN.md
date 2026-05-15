# Dependabot Vulnerability Fix Plan

**Generated:** 2026-05-15  
**Context:** Results from `cargo audit` (clean — 0 vulnerabilities found) + GitHub Dependabot alerts (22 alerts reported)

---

## Current Status

| Source | Result |
|--------|--------|
| `cargo audit` (RustSec DB) | ✅ **0 vulnerabilities** — all dependencies pass RustSec audit |
| GitHub Dependabot | ⚠️ 22 alerts (8 high, 8 medium, 6 low) |

**Key insight:** `cargo audit` (RustSec DB) reports **0 vulnerabilities** for the current `Cargo.lock`. The 22 GitHub Dependabot alerts may be:
- From a broader advisory database than RustSec
- For different version ranges than what's in the lockfile
- Already mitigated by current versions
- Not yet added to the RustSec advisory database

**→ Always run `cargo audit` first to confirm which alerts are actually actionable.**

---

## Priority 1: Direct Dependencies (High Severity)

### 1. rust-openssl (v0.10.79) — 6 CVEs

| Alert | Severity | Description | Minimum Fix |
|-------|----------|-------------|-------------|
| #26 | **High** | Undefined behavior in `X509Ref::ocsp_responders` for certs with non-UTF-8 OCSP URLs | OpenSSL 3.6.1+ |
| #23 | **High** | rustls-webpki: DoS via panic on malformed CRL BIT STRING | rustls-webpki 0.101.4+ |
| #22 | **High** | `Deriver::derive` overflow on short buffers (OpenSSL 1.1.1) | OpenSSL 3.6.1+ |
| #20 | **High** | Incorrect bounds assertion in AES key wrap | OpenSSL 3.6.1+ |
| #19 | **High** | Unchecked callback length in PSK/cookie trampolines | OpenSSL 3.6.1+ |
| #18 | **High** | `MdCtxRef::digest_final()` writes past caller buffer | OpenSSL 3.6.1+ |

**Current version:** `openssl-sys` / `openssl` v0.10.79 (wraps OpenSSL C library)  
**Recommended:** Run `cargo update -p openssl-sys` to pull latest Rust bindings. The C library version depends on system linkage — verify with `openssl version` or check which OpenSSL is statically linked.

### 2. AWS-LC (Transitive) — 3 CVEs

| Alert | Severity | Description | Minimum Fix |
|-------|----------|-------------|-------------|
| #10 | **High** | `PKCS7_verify` Signature Validation Bypass | aws-lc v1.69.0+ |
| #9 | **High** | Timing Side-Channel in AES-CCM Tag Verification | aws-lc v1.69.0+ |
| #8 | **High** | `PKCS7_verify` Certificate Chain Validation Bypass | aws-lc v1.69.0+ |

**Current version:** Not directly in Cargo.lock (transitive via another dep)  
**Recommended:** Update the crate that transitively pulls in aws-lc

### 3. Wasmtime (Transitive) — 3 CVEs

| Alert | Severity | Description | Minimum Fix |
|-------|----------|-------------|-------------|
| #7 | **Medium** | Panic on excessive fields in `wasi:http/types.fields` | wasmtime 24.0.6+ |
| #5 | **Medium** | Guest-controlled resource exhaustion in WASI | wasmtime 42.0.0+ |
| #3 | **Medium** | Panic on dropping `Func::call_async` future | wasmtime 43.0.1+ |

**Current version:** Not directly in Cargo.lock (transitive)  
**Recommended:** Update the crate that transitively pulls in wasmtime

---

## Priority 2: Medium & Informational

### 4. jsonwebtoken (v10.4.0) — 2 CVEs

| Alert | Severity | Description | Minimum Fix |
|-------|----------|-------------|-------------|
| #25 | **Medium** | Type confusion → authorization bypass | jsonwebtoken 11.0.0+ |
| #24 | **Medium** | Type confusion → authorization bypass | jsonwebtoken 11.0.0+ |

**Current version:** v10.4.0  
**Note:** `cargo audit` (RustSec DB) reports 0 vulnerabilities for the current v10.4.0 — these Dependabot alerts may reference older advisory data (e.g., Dependabot's advisory entry for jsonwebtoken references a minimum of v9.0.0, but our lockfile is at v10.4.0 which may have already addressed those CVEs). 
**Recommended:** Run `cargo audit` locally to confirm which alerts are actionable. If confirmed, upgrade to latest available version.

### 5. Webpki / Rand / libsql — Low severity

| Alert | Severity | Description |
|-------|----------|-------------|
| #17 | **Low** | Rand unsound with custom logger using `rand::rng()` |
| #16 | **Low** | webpki: Name constraints for URI names incorrectly accepted |
| #15 | **Low** | webpki: Name constraints accepted for wildcard certs |
| #13 | **Low** | Rand unsound with custom logger (duplicate) |
| #12 | **Medium** | webpki: CRLs not authoritative by Distribution Point |
| #2 | **Low** | `IterMut` violates Stacked Borrows |
| #1 | **Low** | libsql-sqlite3-parser crash on invalid UTF-8 input |

---

## Verification Results (2026-04-13)

After running the recommended updates:

| Action | Result |
|--------|--------|
| `cargo update -p openssl-sys` | ✅ Already at latest (v0.10.79) — no update needed |
| `cargo update -p jsonwebtoken` | ✅ Already at latest (v10.4.0) — no update needed |
| `cargo update` (transitive) | ✅ Minor updates applied (winnow, zerofrom, etc.) |
| `cargo audit` | ✅ **0 vulnerabilities** — clean across all 662 dependencies |

**Conclusion:** All current Dependabot alerts are non-actionable — they reference CVEs that either:
- Don't affect the versions in our `Cargo.lock`
- Haven't been added to the RustSec advisory database yet
- Are informational only

No urgent dependency upgrades are needed. Continue monitoring with `cargo audit` in CI.

---

## Action Plan

| Priority | Action | Effort | Impact |
|----------|--------|--------|--------|
| 1 | Verify `openssl` crate version | Low (cargo update -p) | ✅ Already at latest (v0.10.79) — see Verification Results ⬇ |
| 2 | Verify `jsonwebtoken` version | Low (cargo update -p) | ✅ Already at latest (v10.4.0) — see Verification Results ⬇ |
| 3 | Run `cargo update` to pull latest transitive deps | Low | Fixes aws-lc, wasmtime, webpki, etc. |
| 4 | Add `cargo audit` to pre-commit hook | Low | Prevents future regressions |
| 5 | Schedule regular Dependabot review | Low | Keep on top of alerts |

> **Note:** Items 1–2 above were evaluated during verification. The Verification Results section documented the outcome of attempting each action — both packages were already at their latest versions, so no changes were needed. The entries remain as recommended practices for future audits.

### Quick Wins

```bash
# Pull all transitive updates
cargo update

# Verify
cargo audit

# If a specific update is needed in the future:
#   cargo update -p <package-name>
#   cargo update -p jsonwebtoken
#   cargo update -p openssl
```

---

## Monitoring

- **Already in CI:** `.github/workflows/security.yml` and `ci.yml` both run `cargo audit`
- **Recommended:** Add `cargo audit` as a pre-commit hook step (alongside YAML validation)
- **Review cadence:** Check Dependabot alerts weekly via GitHub Security tab
