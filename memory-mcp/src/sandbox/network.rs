//! Network access control for sandboxed code
//!
//! Implements network restrictions with:
//! - Domain whitelist enforcement
//! - HTTPS-only mode
//! - Request rate limiting
//! - IP address validation

use anyhow::{bail, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tracing::{debug, warn};

/// Network access restrictions configuration
#[derive(Debug, Clone)]
pub struct NetworkRestrictions {
    /// Block all network access
    pub block_all: bool,
    /// Allowed domains (empty = deny all if block_all is false)
    pub allowed_domains: Vec<String>,
    /// Allowed IP addresses
    pub allowed_ips: Vec<IpAddr>,
    /// Require HTTPS only (no HTTP)
    pub https_only: bool,
    /// Block private IP ranges (RFC1918)
    pub block_private_ips: bool,
    /// Block localhost
    pub block_localhost: bool,
    /// Maximum number of requests
    pub max_requests: usize,
}

impl Default for NetworkRestrictions {
    fn default() -> Self {
        Self {
            block_all: true,
            allowed_domains: vec![],
            allowed_ips: vec![],
            https_only: true,
            block_private_ips: true,
            block_localhost: true,
            max_requests: 0,
        }
    }
}

impl NetworkRestrictions {
    /// Create a deny-all configuration
    pub fn deny_all() -> Self {
        Self {
            block_all: true,
            ..Default::default()
        }
    }

    /// Create an allow-specific configuration
    pub fn allow_domains(domains: Vec<String>) -> Self {
        Self {
            block_all: false,
            allowed_domains: domains,
            https_only: true,
            block_private_ips: true,
            block_localhost: true,
            max_requests: 10,
            ..Default::default()
        }
    }

    /// Validate a URL for network access
    ///
    /// # Security
    ///
    /// This method checks:
    /// 1. If network access is allowed at all
    /// 2. Protocol (HTTP vs HTTPS)
    /// 3. Domain/hostname against whitelist
    /// 4. IP address restrictions
    pub fn validate_url(&self, url: &str) -> Result<()> {
        // Check if all network access is blocked
        if self.block_all {
            bail!(NetworkSecurityError::NetworkAccessDenied {
                reason: "All network access is blocked".to_string()
            });
        }

        // Parse URL
        let parsed = url::Url::parse(url).map_err(|e| NetworkSecurityError::InvalidUrl {
            url: url.to_string(),
            reason: e.to_string(),
        })?;

        // Validate scheme
        self.validate_scheme(&parsed)?;

        // Validate host
        if let Some(host) = parsed.host_str() {
            self.validate_host(host)?;
        } else {
            bail!(NetworkSecurityError::InvalidUrl {
                url: url.to_string(),
                reason: "No host specified".to_string()
            });
        }

        debug!("URL validated: {}", url);
        Ok(())
    }

    /// Validate a domain name for network access
    pub fn validate_domain(&self, domain: &str) -> Result<()> {
        if self.block_all {
            bail!(NetworkSecurityError::NetworkAccessDenied {
                reason: "All network access is blocked".to_string()
            });
        }

        // Check if domain is in whitelist
        if !self.is_domain_allowed(domain) {
            warn!("Domain access denied: {} (not in whitelist)", domain);
            bail!(NetworkSecurityError::DomainNotInWhitelist {
                domain: domain.to_string(),
                allowed_domains: self.allowed_domains.clone()
            });
        }

        // Check for localhost
        if self.block_localhost && is_localhost(domain) {
            bail!(NetworkSecurityError::LocalhostAccessDenied {
                domain: domain.to_string()
            });
        }

        debug!("Domain validated: {}", domain);
        Ok(())
    }

    /// Validate URL scheme (HTTP/HTTPS)
    fn validate_scheme(&self, url: &url::Url) -> Result<()> {
        let scheme = url.scheme();

        match scheme {
            "https" => Ok(()),
            "http" => {
                if self.https_only {
                    bail!(NetworkSecurityError::HttpNotAllowed {
                        url: url.to_string()
                    });
                }
                Ok(())
            }
            _ => bail!(NetworkSecurityError::UnsupportedProtocol {
                protocol: scheme.to_string(),
                url: url.to_string()
            }),
        }
    }

    /// Validate host (domain or IP)
    fn validate_host(&self, host: &str) -> Result<()> {
        // Try parsing as IP address first
        if let Ok(ip) = host.parse::<IpAddr>() {
            return self.validate_ip(&ip);
        }

        // Otherwise treat as domain name
        self.validate_domain(host)
    }

    /// Validate IP address
    fn validate_ip(&self, ip: &IpAddr) -> Result<()> {
        // Check if IP is in allowed list
        if !self.allowed_ips.is_empty() && !self.allowed_ips.contains(ip) {
            bail!(NetworkSecurityError::IpNotInWhitelist {
                ip: ip.to_string(),
                allowed_ips: self.allowed_ips.iter().map(|i| i.to_string()).collect()
            });
        }

        // Check for localhost
        if self.block_localhost && is_localhost_ip(ip) {
            bail!(NetworkSecurityError::LocalhostAccessDenied {
                domain: ip.to_string()
            });
        }

        // Check for private IPs
        if self.block_private_ips && is_private_ip(ip) {
            bail!(NetworkSecurityError::PrivateIpAccessDenied { ip: ip.to_string() });
        }

        Ok(())
    }

    /// Check if a domain is in the allowed list
    fn is_domain_allowed(&self, domain: &str) -> bool {
        if self.allowed_domains.is_empty() {
            // No whitelist = deny all
            return false;
        }

        // Exact match
        if self.allowed_domains.contains(&domain.to_string()) {
            return true;
        }

        // Subdomain match (e.g., "api.example.com" matches "example.com")
        for allowed in &self.allowed_domains {
            if domain.ends_with(&format!(".{}", allowed)) {
                return true;
            }
        }

        false
    }
}

/// Check if a domain name is localhost
fn is_localhost(domain: &str) -> bool {
    matches!(
        domain.to_lowercase().as_str(),
        "localhost" | "localhost.localdomain" | "127.0.0.1" | "::1" | "0.0.0.0"
    )
}

/// Check if an IP address is localhost
fn is_localhost_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback(),
        IpAddr::V6(ipv6) => ipv6.is_loopback(),
    }
}

/// Check if an IP address is private (RFC1918)
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => is_private_ipv4(ipv4),
        IpAddr::V6(ipv6) => is_private_ipv6(ipv6),
    }
}

/// Check if IPv4 address is private
fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    // RFC1918 private ranges:
    // 10.0.0.0/8
    // 172.16.0.0/12
    // 192.168.0.0/16
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
}

/// Check if IPv6 address is private
fn is_private_ipv6(ip: &Ipv6Addr) -> bool {
    // Check loopback (::1)
    if ip.is_loopback() {
        return true;
    }

    // Check unique local addresses (fc00::/7)
    // This is equivalent to is_unique_local() which requires MSRV 1.84.0
    let segments = ip.segments();
    if (segments[0] & 0xfe00) == 0xfc00 {
        return true;
    }

    // Check multicast
    if ip.is_multicast() {
        return true;
    }

    false
}

/// Network security errors
#[derive(Debug, thiserror::Error)]
pub enum NetworkSecurityError {
    #[error("Network access denied: {reason}")]
    NetworkAccessDenied { reason: String },

    #[error("Invalid URL: {url} - {reason}")]
    InvalidUrl { url: String, reason: String },

    #[error("HTTP not allowed (HTTPS only): {url}")]
    HttpNotAllowed { url: String },

    #[error("Unsupported protocol: {protocol} in URL: {url}")]
    UnsupportedProtocol { protocol: String, url: String },

    #[error("Domain not in whitelist: {domain} (allowed: {allowed_domains:?})")]
    DomainNotInWhitelist {
        domain: String,
        allowed_domains: Vec<String>,
    },

    #[error("IP not in whitelist: {ip} (allowed: {allowed_ips:?})")]
    IpNotInWhitelist {
        ip: String,
        allowed_ips: Vec<String>,
    },

    #[error("Localhost access denied: {domain}")]
    LocalhostAccessDenied { domain: String },

    #[error("Private IP access denied: {ip}")]
    PrivateIpAccessDenied { ip: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deny_all() {
        let restrictions = NetworkRestrictions::deny_all();
        let result = restrictions.validate_url("https://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_https_only() {
        let restrictions = NetworkRestrictions::allow_domains(vec!["example.com".to_string()]);
        assert!(restrictions.validate_url("https://example.com").is_ok());
        assert!(restrictions.validate_url("http://example.com").is_err());
    }

    #[test]
    fn test_domain_whitelist() {
        let restrictions = NetworkRestrictions::allow_domains(vec!["example.com".to_string()]);

        assert!(restrictions.validate_url("https://example.com").is_ok());
        assert!(restrictions.validate_url("https://api.example.com").is_ok());
        assert!(restrictions.validate_url("https://evil.com").is_err());
    }

    #[test]
    fn test_localhost_blocking() {
        let mut restrictions = NetworkRestrictions::allow_domains(vec!["localhost".to_string()]);
        restrictions.block_localhost = true;

        assert!(restrictions.validate_domain("localhost").is_err());
        assert!(restrictions.validate_domain("127.0.0.1").is_err());
    }

    #[test]
    fn test_private_ip_blocking() {
        let restrictions = NetworkRestrictions {
            block_all: false,
            block_private_ips: true,
            ..Default::default()
        };

        let private_ips = vec![
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "127.0.0.1",
            "169.254.1.1",
        ];

        for ip in private_ips {
            let addr: IpAddr = ip.parse().unwrap();
            assert!(restrictions.validate_ip(&addr).is_err());
        }
    }

    #[test]
    fn test_public_ip_allowed() {
        let restrictions = NetworkRestrictions {
            block_all: false,
            allowed_ips: vec!["8.8.8.8".parse().unwrap()],
            block_private_ips: true,
            ..Default::default()
        };

        let ip: IpAddr = "8.8.8.8".parse().unwrap();
        assert!(restrictions.validate_ip(&ip).is_ok());
    }

    #[test]
    fn test_is_localhost() {
        assert!(is_localhost("localhost"));
        assert!(is_localhost("LOCALHOST"));
        assert!(is_localhost("127.0.0.1"));
        assert!(!is_localhost("example.com"));
    }

    #[test]
    fn test_is_private_ipv4() {
        let private = Ipv4Addr::new(192, 168, 1, 1);
        assert!(is_private_ipv4(&private));

        let public = Ipv4Addr::new(8, 8, 8, 8);
        assert!(!is_private_ipv4(&public));
    }

    #[test]
    fn test_subdomain_matching() {
        let restrictions = NetworkRestrictions::allow_domains(vec!["example.com".to_string()]);

        assert!(restrictions.is_domain_allowed("example.com"));
        assert!(restrictions.is_domain_allowed("api.example.com"));
        assert!(restrictions.is_domain_allowed("foo.bar.example.com"));
        assert!(!restrictions.is_domain_allowed("examplecom"));
        assert!(!restrictions.is_domain_allowed("evil.com"));
    }
}
