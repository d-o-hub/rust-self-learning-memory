# PR Comments Reference

Companion to [SKILL.md](./SKILL.md). How to fetch, classify, and close the loop on PR feedback.

## Fetch commands (always run all three)

```bash
OWNER_REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
N=<pr_number>

gh api "repos/${OWNER_REPO}/pulls/${N}/comments" --paginate   # inline
gh api "repos/${OWNER_REPO}/pulls/${N}/reviews" --paginate    # review submissions
gh api "repos/${OWNER_REPO}/issues/${N}/comments" --paginate  # conversation
```

Optional: `gh pr view ${N} --comments` for a human-readable dump.

## Classification cheatsheet

| Author pattern | Action |
|----------------|--------|
| Real GitHub user | Read carefully; fix or evidence-waive |
| `codecov[bot]` | Parse missing-files table; add tests |
| `codacy*` | Fix new issues if count > 0 |
| `coderabbitai*` / Cursor bots | Concrete code suggestions → implement or waive |
| `github-actions[bot]` benchmarks | Informational unless regression noted |
| `CHANGES_REQUESTED` state | Hard blocker |

## Close the loop template

```markdown
### Addressing feedback

| Comment | Action | Commit |
|---------|--------|--------|
| Codecov: config_template.rs 0% | Added unit tests for init/template | abc1234 |
| Reviewer: path collision | Sibling paths in cli_overrides | def5678 |

CI re-running on this push.
```

## Related

- Skill procedure: [SKILL.md](./SKILL.md)
- Script: `./scripts/check-pr-readiness.sh`
- CI failures: `../ci-fix/SKILL.md`
