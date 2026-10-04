# Verify a Release

Every [GitHub Release](<https://github.com/d-o-hub/rust-self-learning-memory/releases>)
of this project is built by the repository's own release workflow
(`.github/workflows/release.yml`) from a `vX.Y.Z` tag. This chapter shows how to
confirm that a release you downloaded is authentic and has not been tampered
with: checksums, build provenance, SBOMs, and immutable-release attestations.

## What a release publishes

Each release attaches one archive per supported target for every dist-able crate:

| App | Archive names |
|-----|---------------|
| `do-memory-cli` | `do-memory-cli-<target>.tar.xz`, `do-memory-cli-<target>.zip` |
| `do-memory-mcp` | `do-memory-mcp-<target>.tar.xz`, `do-memory-mcp-<target>.zip` |
| `do-memory-examples` | `do-memory-examples-<target>.tar.xz`, `do-memory-examples-<target>.zip` |

The supported targets are `x86_64-unknown-linux-gnu`,
`aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, and
`x86_64-pc-windows-msvc` (`.zip` rather than `.tar.xz`).

Alongside the archives, cargo-dist attaches:

- a sibling `<archive>.sha256` checksum for every archive, plus a combined
  `sha256.sum`;
- `source.tar.gz` and its `source.tar.gz.sha256`;
- `dist-manifest.json`, the cargo-dist build manifest.

Releases cut by the release workflow after the SBOM/attestation pipeline landed
(issue #1121) additionally publish and attest:

- one CycloneDX SBOM per crate — `do-memory-cli.cdx.json`,
  `do-memory-mcp.cdx.json`, and `do-memory-examples.cdx.json`;
- **build-provenance** attestations for every archive
  (`https://slsa.dev/provenance/v1` predicate);
- **SBOM** attestations for each crate's archives
  (`https://cyclonedx.org/bom` predicate).

> **Note:** Checksums ship with *every* release. SBOM assets and attestations are
> produced only by the pipeline from issue #1121 onward. The older `v0.1.44`
> release predates that pipeline, so it carries only the archives, their
> checksums, `sha256.sum`, `source.tar.gz`, and `dist-manifest.json`.

## Verify checksums

Download an archive together with its `.sha256` sidecar, then verify it. The
sidecar is a standard `sha256sum` line that names the archive it covers, so
`sha256sum -c` works directly.

```bash
# Download one archive and its checksum from the release
gh release download v0.1.44 -R d-o-hub/rust-self-learning-memory \
  -p 'do-memory-cli-x86_64-unknown-linux-gnu.tar.xz*'

# Verify (prints "...tar.xz: OK" on success)
sha256sum -c do-memory-cli-x86_64-unknown-linux-gnu.tar.xz.sha256
```

The same files can be fetched with plain `curl`:

```bash
BASE=https://github.com/d-o-hub/rust-self-learning-memory/releases/download/v0.1.44
curl -LO "$BASE/do-memory-cli-x86_64-unknown-linux-gnu.tar.xz"
curl -LO "$BASE/do-memory-cli-x86_64-unknown-linux-gnu.tar.xz.sha256"
sha256sum -c do-memory-cli-x86_64-unknown-linux-gnu.tar.xz.sha256
```

Optionally download every archive and verify them all at once against the
combined manifest:

```bash
sha256sum -c sha256.sum
```

On macOS, `sha256sum` is not installed by default; use `shasum -a 256 -c` with
the same file.

## Verify build provenance

Archives built by the release workflow carry a SLSA build-provenance attestation
signed through Sigstore. Verify it with the GitHub CLI:

```bash
gh attestation verify do-memory-cli-x86_64-unknown-linux-gnu.tar.xz \
  -R d-o-hub/rust-self-learning-memory
```

This enforces the default `https://slsa.dev/provenance/v1` predicate and checks
that the archive was built by this repository's workflow. Run it on the
**downloaded** file whose digest you hold — the whole point is to bind the bytes
you actually have to a signed claim about their origin.

**Requirements.** The `gh attestation` command needs GitHub CLI **2.49.0 or
newer**:

```bash
gh --version    # e.g. gh version 2.49.0
```

See the GitHub CLI manual for
[`gh attestation verify`](<https://cli.github.com/manual/gh_attestation_verify>)
for the full flag set (`--signer-workflow`, `--format json`, offline bundles, …).

## Verify an SBOM attestation

CycloneDX SBOMs are attested with the CycloneDX predicate type. Pass it
explicitly with `--predicate-type`:

```bash
gh attestation verify do-memory-cli-x86_64-unknown-linux-gnu.tar.xz \
  -R d-o-hub/rust-self-learning-memory \
  --predicate-type https://cyclonedx.org/bom
```

To inspect the attested SBOM itself, add JSON output:

```bash
gh attestation verify do-memory-cli-x86_64-unknown-linux-gnu.tar.xz \
  -R d-o-hub/rust-self-learning-memory \
  --predicate-type https://cyclonedx.org/bom \
  --format json \
  --jq '.[].verificationResult.statement.predicate'
```

`https://cyclonedx.org/bom` is the predicate type CycloneDX defines for its
bill-of-material flavours; see the
[CycloneDX specification overview](<https://cyclonedx.org/specification/overview>).

## Verify an immutable release

When repository immutability is enabled, GitHub generates a signed release
attestation binding the tag, commit, and assets. Verify it with:

```bash
gh release verify v0.1.44
```

Use `gh release verify-asset v0.1.44 <path-to-downloaded-archive>` to check that
a local file exactly matches a published release asset.

> **Note:** `gh release verify` only proves something once repository
> **immutability is enabled** for the repository (a repository setting). Until
> then, releases are not locked and no release attestation is generated. The
> command needs GitHub CLI **2.81.0 or newer**.

See [Immutable releases](<https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases>)
and [Verifying the integrity of a release](<https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/verify-release-integrity>).

## If verification fails

- **Do not use** a release whose checksum or attestation does not verify.
  Delete the downloaded files rather than running them.
- Re-download from
  <https://github.com/d-o-hub/rust-self-learning-memory/releases> — a corrupted
  download is the most common cause of a checksum mismatch.
- If the checksum is consistently wrong, or `gh attestation verify` /
  `gh release verify` fails against a fresh download, treat it as a supply-chain
  incident and report it through the process in
  [SECURITY.md](<https://github.com/d-o-hub/rust-self-learning-memory/blob/main/SECURITY.md>)
  (a private security advisory — do **not** open a public issue).

## References

- [Artifact attestations](<https://docs.github.com/en/actions/concepts/security/artifact-attestations>)
- [Using artifact attestations to establish provenance](<https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations>)
- [Verifying the integrity of a release](<https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/verify-release-integrity>)
- [Immutable releases](<https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases>)
- [`gh attestation verify`](<https://cli.github.com/manual/gh_attestation_verify>)
- [`gh release verify`](<https://cli.github.com/manual/gh_release_verify>)
