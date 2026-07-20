# Analysis: GitHub CLI Skills + Manual Best Practices (2026-07-20)

**Sources**

- Official agent skills: <https://github.com/cli/cli/tree/trunk/skills> (`gh`, `gh-skill`)
- Manual: <https://cli.github.com/manual/>
- Local `gh` version at audit: **2.95.0** (skills subsystem is **preview**)
- Official skill install smoke-test: `gh skill install cli/cli gh --scope user --agent claude-code` → OK  
  (installs under `~/.claude/skills/gh/`, not under this repo’s `.agents/skills/`)

**Related local policy**

- Release authority: `.agents/skills/release-guard/SKILL.md` + `scripts/release-manager.sh` + `.github/workflows/release.yml`
- PR authority: `.agents/skills/pr-readiness/SKILL.md`
- CI wait: `.agents/skills/ci-poll/SKILL.md`

---

## 1. What `cli/cli` ships as skills

| Skill | Purpose |
|-------|---------|
| **`gh`** | Patterns for agents **calling** the GitHub CLI (JSON, pagination, search vs list, `gh api` fallback, repo targeting, auth) |
| **`gh-skill`** | Patterns for agents **managing** skills via `gh skill` (search / preview / install / update / publish) |

These are **meta-skills for using GitHub + the skill ecosystem**, not a replacement for this repo’s domain skills (`release-guard`, `pr-readiness`, `do-memory-cli-ops`, …).

### Install model (from `gh-skill` + manual)

```bash
# Official GitHub CLI agent skill (user scope — recommended for all repos)
gh skill install cli/cli gh --scope user --agent <host>

# Skill manager skill (optional)
gh skill install cli/cli gh-skill --scope user --agent <host>

# Discover / maintain
gh skill search <query>
gh skill preview cli/cli gh
gh skill list
gh skill update --all
```

| Scope | Location | Use |
|-------|----------|-----|
| **user** | Home-dir host paths (e.g. `~/.claude/skills`) | Cross-repo `gh` patterns |
| **project** | Inside git repo | Repo-specific skills |

**Discovery conventions for *publishing* skills** (from `gh skill publish`):

- `skills/<name>/SKILL.md`
- `skills/<scope>/<name>/SKILL.md`
- root-level `<name>/SKILL.md`
- `plugins/<scope>/skills/<name>/SKILL.md`

This repo uses **`.agents/skills/<name>/SKILL.md`**, which is **not** the default discovery path for `gh skill install` from a remote clone unless:

- `--allow-hidden-dirs` is used (skill docs warn this has risks), or  
- skills are also mirrored under a non-hidden path for publish, or  
- install uses exact path / local install with explicit layout.

**Implication:** Our project skills are optimized for **in-repo agents** (Grok/Claude/AGENTS.md). They are only partially aligned with **`gh skill publish` / public skill search** conventions.

---

## 2. Core patterns from the official `gh` skill (agents)

Adopt these as **global** agent habits (they improve every PR/release/CI skill):

| Pattern | Practice | Why |
|---------|----------|-----|
| **Structured output** | Prefer `gh … --json field1,field2` + `--jq` | Human columns break parsers |
| **Discover fields** | `gh pr view --json` (no field list) lists available fields | Avoid guessing |
| **Pagination** | `-L` / `--limit`; `gh api --paginate` | Defaults (~30) silently truncate |
| **Repo targeting** | `-R OWNER/REPO` when not in clone / ambiguous remotes | Multi-remote safety |
| **Search vs list** | `gh search …` for cross-repo/filter; bare tokens for qualifiers | Quoted `repo:… is:open` becomes one keyword |
| **Bots as apps** | `--app dependabot` not bare `--author dependabot` | Apps author as `app/…` |
| **API fallback** | Inline PR review comments: `gh api repos/{owner}/{repo}/pulls/{n}/comments` | `gh pr view --comments` is issue conversation only |
| **Auth** | `gh auth status` / `GITHUB_TOKEN` / `GH_HOST` for GHE | Explicit context |
| **Non-interactive** | No pager gymnastics; pass required flags (`--title`/`--body`) | Agents already non-TTY |

### New / preview `gh` surfaces agents should know

| Area | Commands | Notes |
|------|----------|--------|
| **PR checks** | `gh pr checks [--watch] [--required] [--json]` | Exit code **8** = pending — better than raw sleep loops |
| **Actions** | `gh run list|view|watch|rerun|download|cancel` | Prefer for CI poll over only `statusCheckRollup` |
| **Repo content** | `gh repo read-file` / `read-dir` (preview) | Inspect remote without full clone |
| **Issues** | types / sub-issues / blocked-by (newer) | Useful for planning issues later |
| **Discussions** | `gh discussion …` (preview) | Optional community path |
| **Releases** | `gh release view|list|download|edit|verify` | **Observe / fix notes**; create only when policy allows |

---

## 3. Releases: official `gh` vs this repo’s policy

### What the manual says (`gh release create`)

- Creates a **GitHub Release** (and can auto-create a tag from default branch if missing).
- Flags: `--generate-notes`, `--notes-file`, `--verify-tag`, `--target`, `--draft`, assets, etc.
- **Immutable releases** (when enabled): tags/assets locked after publish.
- Manual path can **bypass** custom build/publish pipelines.

### What this repo requires (release-guard)

```text
main green → release-manager ship → git tag push → release.yml (cargo-dist)
                 ⛔ never: gh release create (manual)
```

| Concern | Manual `gh release create` | Tag + `release.yml` |
|---------|----------------------------|---------------------|
| Multi-platform binaries | You must build/upload | cargo-dist builds in Actions |
| Version preflight | Easy to skip | Tag must match `Cargo.toml` |
| CHANGELOG → notes | Manual / generate-notes | `parse-changelog` from CHANGELOG |
| Main-only | Easy to tag wrong commit | Policy + manager; preflight can harden |
| Single authority | Competing paths | One path |

**Conclusion:** For **this** repository, official best practice is:

1. Use **`gh` heavily** for **status, PR, checks, runs, viewing releases**.
2. Do **not** use **`gh release create`** as the ship step.
3. Use **git tag push** as the only release trigger; **`release.yml` is the only creator** of the GitHub Release (it may call `gh release create` *inside* Actions with artifacts — that is fine and intended).

That is **compatible** with the CLI manual: the manual documents the general tool; **repo policy** specializes release creation.

### Tag-only workflow (answers prior question)

| Layer | Mechanism |
|-------|-----------|
| Trigger | `on: push: tags:` in `release.yml` |
| “Main only” | Not a branch filter — enforce via **tag points at main** (manager + optional CI preflight) |
| Agent UX | `release-manager.sh ship --execute` = safe tag push |
| Observe | `gh release view`, `gh run watch`, `gh run list --workflow=release.yml` |

---

## 4. Gap analysis: this repo vs `gh` best practices

### Already strong

| Area | Evidence |
|------|----------|
| Structured PR state | `pr-readiness` uses `gh pr view --json mergeable,mergeStateStatus,statusCheckRollup` |
| Inline reviews via API | `gh api …/pulls/{n}/comments` (matches official `gh` skill guidance) |
| No admin merge | Explicit ban on `gh pr merge --admin` |
| No manual release create | Explicit ban in release-guard / AGENTS |
| Release observation | `gh release view`, wait on `release.yml` |

### Gaps / improvements

| Gap | Official better pattern | Recommendation |
|-----|-------------------------|----------------|
| **G1** Official `gh` skill not in agent bootstrap | `gh skill install cli/cli gh --scope user` | Document in AGENTS / onboarding; install for agents |
| **G2** `ci-poll` uses sleep + `statusCheckRollup` | `gh pr checks --watch --required` (exit 8 pending) | Prefer `gh pr checks --watch` / `gh run watch` |
| **G3** Update branch via raw API | `gh pr update-branch` | Prefer typed command |
| **G4** Skill publish layout | `.agents/skills/` vs `skills/` | Optional: publish adapter path or document `--allow-hidden-dirs` only for trusted local install |
| **G5** Release preflight “on main” | Tag can point off-main | Add `release.yml` check: tagged commit is ancestor of `origin/main` (or equals main tip) |
| **G6** `github-release-best-practices` skill | Can drift toward manual create | Keep as background; always defer to release-guard |
| **G7** Field selection | Some scripts may scrape human tables | Audit high-risk scripts for `--json` |
| **G8** Skill version pins | `gh skill install …@v` / `--pin` | Pin official `gh` skill to a tag for reproducibility |

### Skill layout dual model (recommended)

```text
User scope (all repos):
  gh skill install cli/cli gh --scope user --agent <host>
  gh skill install cli/cli gh-skill --scope user --agent <host>

Project scope (this monorepo):
  .agents/skills/*     ← domain + release-guard / pr-readiness / etc.
  .agents/SKILLS.md    ← inventory
```

Do **not** replace `release-guard` with generic `gh release create` examples from the open web.

---

## 5. Recommended command matrix (agents)

### PRs

```bash
# Status (structured)
gh pr view N --json number,title,url,mergeable,mergeStateStatus,statusCheckRollup,headRefOid,baseRefName,headRefName

# Checks (preferred poll)
gh pr checks N --required --json name,state,bucket,workflow
gh pr checks N --required --watch --interval 30   # exit 8 while pending

# Branch up to date
gh pr update-branch N

# Comments (full surface)
gh api repos/{owner}/{repo}/pulls/N/comments --paginate
gh api repos/{owner}/{repo}/pulls/N/reviews --paginate
gh api repos/{owner}/{repo}/issues/N/comments --paginate

# Merge only when CLEAN + MERGEABLE + required checks pass (never --admin)
gh pr merge N --merge   # or squash/rebase per repo policy
```

### CI / Actions

```bash
gh run list --branch main --commit "$(git rev-parse origin/main)" --json name,status,conclusion,databaseId,url
gh run watch <run-id>
gh run view <run-id> --log-failed
gh run rerun <run-id> --failed
```

### Releases (this repo)

```bash
# Ship (only path)
./scripts/release-manager.sh status
./scripts/release-manager.sh ship --execute

# Observe (gh)
gh run list --workflow=release.yml --limit 5
gh run watch <id>
gh release view "v$(rg -n '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
gh release download "vX.Y.Z" -D /tmp/assets   # optional

# Forbidden for ship
# gh release create …   ← do not use for primary release
```

### Skills management

```bash
gh skill search github-cli
gh skill install cli/cli gh --scope user --agent claude-code --pin v2.96.0
gh skill list
gh skill update --all
# Project skills: edit .agents/skills/* and run local evals; optional future publish path
```

---

## 6. Mapping to prior “tag + main workflow only” discussion

| User intent | Official + repo resolution |
|-------------|----------------------------|
| “Only workflow for release” | `release.yml` on **tag push** (already) |
| “Only when new release tag” | Tag is the signal (already) |
| “Only for main” | Enforce **tag commit ∈ main** in manager + optional workflow preflight (improve G5) |
| “Use gh for everything” | Use **gh for GitHub API/UX**; use **tag + Actions** for ship; use **`gh skill`** for installing **how to use gh** |

---

## 7. Action recommendations (prioritized)

### P0 — Document & bootstrap

1. Add to `AGENTS.md` (or onboarding): install official skills once per machine:
   ```bash
   gh skill install cli/cli gh --scope user --agent <host>
   gh skill install cli/cli gh-skill --scope user --agent <host>
   ```
2. Keep **release-guard** as sole ship path; cite that `gh release create` is for **generic** repos / Actions internals only.

### P1 — Align high-traffic skills with official `gh` skill

3. **ci-poll**: prefer `gh pr checks --watch --required` and/or `gh run watch` over sleep loops.  
4. **pr-readiness**: prefer `gh pr update-branch` over raw update-branch API where possible.  
5. **release-guard**: add “observe with `gh run watch` / `gh release view`” examples; reiterate no create.

### P1 — Harden release workflow

6. In `release.yml` preflight: assert tagged SHA is reachable from `origin/main` (or equals tip).  
7. Optionally fail if tagger is not an allowed actor / environment.

### P2 — Skill ecosystem

8. Decide whether project skills should ever be `gh skill publish`-able (would need non-hidden layout or documented allow-hidden).  
9. Pin official `gh` skill version in a small `docs/agent_bootstrap.md`.  
10. Expand `skill-rules` so “GitHub CLI / gh skill” routes to a thin local skill that **defers to** installed `cli/cli` `gh` skill + points at release-guard for ship.

---

## 8. Explicit non-goals

- Replacing cargo-dist / `release.yml` with agent-run `gh release create`.  
- Moving domain skills only to user-scope `gh skill` installs (lose repo-reviewed truth).  
- Treating preview `gh skill` / `gh discussion` / `repo read-*` as stable CI contracts without pinning.

---

## 9. Summary

| Question | Answer |
|----------|--------|
| Should we use `cli/cli` skills? | **Yes** — install **`gh`** (and optionally **`gh-skill`**) at **user** scope as the canonical “how to use gh” skill. |
| Should we use the gh manual for PRs/CI? | **Yes** — structured JSON, `pr checks --watch`, `run watch`, `api --paginate`. |
| Should we use `gh release create` to ship? | **No** in this repo — **tag → release.yml** remains sole creator; `gh` is for **verify/list/download/edit notes**. |
| Does that conflict with “tag-only workflow”? | **No** — tag-only **is** the workflow model; `gh` is the **client**; `release-manager` is the **safe tagger**. |

**Next implementation slice (if approved):** G1 docs + G2 ci-poll upgrade + G5 main-ancestor preflight in `release.yml`.
