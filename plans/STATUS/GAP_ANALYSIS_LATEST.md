# Gap Analysis — Live Register

**Last Updated**: 2026-11-17
**Workspace Version**: `0.1.51` · **Released Tag**: `v0.1.50` (`0fe1b89d`)
**Main baseline**: `f02d81f9` · **Open PRs**: `gh pr list` is the live source · **Open issues**: 0
**Historical register**: [`archive/2026/GAP_ANALYSIS_2026-11-01.md`](archive/2026/GAP_ANALYSIS_2026-11-01.md)
(the 2026-10-04 audit snapshot and its 22 verdicts)

## Live state

- **No open code gap.** The audited #1063–#1092 set, the follow-ups (#1068/#1089/#1076),
  the release series (#1109/#1165), the drift issue (#1223/#1236) and the CLI contract
  follow-up (#1232/#1233) are closed.
- **Tracker empty 2026-11-17:** #1281 (external-consumer adaptive-discovery example) merged
  as #1285 and dogfooded from `main`; there are no open issues and no open pull requests.
  The next release is unscheduled — cadence reports `within_cadence`.
- The 2026-10-04 register claimed 22 open findings because the issues had never been
  entered into the trackers. Every one now has a merged PR or a recorded closure in the
  historical register; do not re-open them without regression evidence.
- **Process gaps closed:** released-CLI dogfood is a mandatory release-guard step; the
  generated `.opencode/package-lock.json` is untracked/ignored (alert #35 `fixed`); the
  toolchain is pinned to `1.99.1`; ADR-079 stage 4 is proven live (#1229); the harness pin
  is measured (`v0.4.6`) with a status-contract probe.
- **Plans gap closed 2026-11-15:** completed registers archived and the active set
  rewritten forward-only; the unmeasured `forced_go` R-F records are superseded by a
  schema-valid `NO-GO` backlog artifact.
- **R-F2 promoted 2026-11-17:** opt-in OpenTelemetry OTLP tracing landed behind the
  `otel` feature (ADR-084) after `R-F2-otel-2026-11-17.json` passed every
  pre-approved threshold; the backlog row was not flipped on a forced-GO note.
- **ADR lifecycle closed 2026-11-16:** ADR-080/081/082 are Accepted with test evidence;
  ADR-028/042/043 were rebaselined to the implemented code and ADR-060 was retired with
  the last stale `#[allow(deprecated)]` removed.

## Upstream-blocked dependency advisories (ACT-404)

Dependabot reports the same advisory IDs against both `Cargo.lock` and `fuzz/Cargo.lock`.
They are **not** a bump away: each row names the chain that owns the fix. `cargo audit`
and `cargo deny` are separate registers from GHSA; a green local audit does not clear a
Dependabot alert.

| Package family | Advisory IDs | Current pinned chain | Fixed in / status |
|----------------|--------------|----------------------|-------------------|
| `rustls` | GHSA-w44r-93j7-qv28, GHSA-c8m5-c952-6cm4, GHSA-h2v8-394m-p7vv | `rustls 0.22.4` → `hyper-rustls 0.25` → `libsql 0.9.30` | `0.23.46`; blocked by the libsql/hyper chain |
| `rustls-webpki` | GHSA-82j2-j2ch-gfr8, GHSA-xgp8-3hg3-c2mh, GHSA-965h-392x-2mh5, GHSA-pwjx-qhcg-rvj4 | `rustls-webpki 0.102.8` under `rustls 0.22.4` | `0.103.10`/`0.103.12`/`0.103.13`; same chain |
| `lru` | GHSA-rhfx-m35p-ff5j | `lru 0.9.0` via `augurs-changepoint`/`rv`; `0.12.5` via `agentfs-sdk` | `0.16.3`; both paths upstream |
| `libsql-sqlite3-parser` | GHSA-8m95-fffc-h4c5 | `0.13.0` via `libsql 0.9.30` | no patched release |

**Revalidated 2026-11-16** (no chain moved): `libsql 0.10.0` still resolves
`hyper-rustls 0.25 → rustls 0.22.4`; `augurs-changepoint 0.10.3` still resolves
`rv 0.16.5 → lru 0.9.0`; `agentfs-sdk 0.6.4` still resolves `lru 0.12.5`; no patched
`libsql-sqlite3-parser` release is available. No bump PR is justified.

**Revalidation commands** (run before writing any bump PR):

```bash
gh api repos/:owner/:repo/dependabot/alerts?state=open --paginate
cargo tree --locked -i rustls
cargo tree --locked -i lru
cd fuzz && cargo tree --locked -i rustls
```

A bump PR opens only when `cargo tree` on a scratch crate at the candidate version proves
the chain moved. Record an unfixable advisory as upstream-blocked with the chain named.

## How this register stays truthful

- Count from the live source (`gh issue list --state open`), never from a tracker's
  restated count.
- Every closed finding keeps its PR/commit evidence in the historical register.
- New gaps get an action ID in `ACTIONS.md`, an acceptance command, and a row here.
- Plans/validator drift is a gap: `./scripts/validate-plans.sh --all` must be green before
  a plan refresh is called complete.
