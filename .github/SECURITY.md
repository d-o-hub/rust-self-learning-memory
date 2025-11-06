# Security Policy

## Reporting a Vulnerability

**We appreciate your help in responsibly disclosing security vulnerabilities.** To ensure they can be fixed before public disclosure, please **do not report them through public channels** like issues, discussions, or pull requests.

### Private Vulnerability Reporting

If you discover a security vulnerability, please report it directly to us via **GitHub's private security advisory feature**. This allows us to discuss, fix, and publish the vulnerability in a private space.

1.  **Navigate to the "Security" tab** in this repository.
2.  Click on **"Security advisories"**.
3.  Click the **"New draft security advisory"** button to start the process[citation:3].

You can collaborate with our maintainers in the private draft advisory to discuss the finding and develop a patch[citation:1][citation:2].

## Best Practices for Writing a Security Advisory

To help us process the advisory efficiently and get it reviewed by GitHub, please provide the following information in a clear and standard format[citation:1]:

*   **Ecosystem:** Specify the package ecosystem (e.g., npm, PyPI, Go).
*   **Package name:** Provide the name of the affected package.
*   **Affected versions:** Clearly define the range of vulnerable versions using supported operators.

### Guidelines for Affected Versions

Please use the following syntax to describe which versions are affected[citation:1]:

*   **Supported operators:** `=`, `<=`, `<`, `>=`
*   **Specifying a range:** `>= lower_bound, < upper_bound` (e.g., `>= 1.2.0, < 1.2.5`)
*   **Single version:** `= 1.2.3`
*   **All versions up to:** `< 1.2.3` or `<= 1.2.3`

**Example:**
- Vulnerable Version Range: `>= 1.0.0, < 1.5.2`
- Patched Version: `1.5.2`

## What to Expect

After you submit a draft advisory:
- Our maintainers will be notified and will review the report.
- We may collaborate with you in the draft advisory to understand and fix the issue.
- Once a fix is ready and the advisory is finalized, we will publish it to inform the community and may request a CVE from GitHub[citation:3].

## Scope

Please report any vulnerability that could potentially impact the security of this project's users or systems.

---

Thank you for helping to keep our project and its users safe.
