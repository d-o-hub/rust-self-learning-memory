Branch protection best practices (2025) — personal project (single maintainer)

Purpose
-------
This file documents recommended branch-protection settings for 2025 for small/personal repositories where you're the sole maintainer. It includes the exact JSON and commands to apply the protection using `gh api` (recommended) or `curl` (PAT), plus UI guidance and verification commands.

High-level recommendations
-------------------------
- Protect your `main` branch from force pushes and deletion.
- Require status checks (CI) to pass before merging. Use the exact job names from your CI (example: `build`, `test`, `lint`).
- Enforce branch protection for administrators (prevent accidental bypass) but know how to temporarily disable if you must push directly.
- For single-maintainer projects: do not require approval reviews by default (0 approvals) but encourage PRs for traceability.
- Consider requiring signed commits (`required_signatures`) if you want extra assurance.

Recommended settings (2025)
---------------------------
- allow_force_pushes: false
- allow_deletions: false
- required_status_checks.strict: true
- required_status_checks.contexts: ["build", "test", "lint"]
- enforce_admins: true
- required_pull_request_reviews.required_approving_review_count: 0
- required_signatures: true

Notes
-----
- Replace the `contexts` array with the actual check names your CI exposes. For GitHub Actions these are the job names from your workflow(s).
- `required_signatures` enforces commit signature verification. If you don't use signed commits, omit or set to false.
- The API expects JSON types (null for `restrictions`, objects for nested fields). Use the JSON payload below to avoid quoting/typing errors.

Apply using `gh` (recommended)
-----------------------------
1. Authenticate `gh` interactively as an account with admin access to this repo and ensure it has the `repo` and `workflow` scopes.

To avoid problems with environment tokens, in your shell do:

```bash
# Clear any ephemeral GITHUB_TOKEN for this shell session
export GITHUB_TOKEN=

# Interactive login (choose GitHub.com, HTTPS, and authorize with repo/workflow scopes)
gh auth login
```

2. Create the JSON file and call the REST endpoint using `gh api`:

```bash
cat > /tmp/protect-main.json <<'JSON'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["build", "test", "lint"]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "required_approving_review_count": 0
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_signatures": true
}
JSON

# Apply the protection
gh api --method PUT /repos/<OWNER>/<REPO>/branches/main/protection --input /tmp/protect-main.json
```

Replace `<OWNER>` and `<REPO>` (for this repository use `d-o-hub/rust-self-learning-memory`).

Apply using `curl` + PAT (alternative)
--------------------------------------
If you prefer a PAT instead of interactive `gh` login, create a token with `repo` scope and run:

```bash
# set PAT in an env var for the command
export GITHUB_PAT="ghp_..."

curl -sS -X PUT \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_PAT" \
  https://api.github.com/repos/d-o-hub/rust-self-learning-memory/branches/main/protection \
  -d @/tmp/protect-main.json
```

UI steps (GitHub web)
---------------------
1. Go to the repo Settings -> Branches -> Branch protection rules.
2. Add rule for `main`.
3. Check "Require status checks to pass before merging" and add your check names.
4. Check "Include administrators" if you want enforcement for admins.
5. Disable "Allow force pushes" and "Allow deletions".
6. Optionally check "Require signed commits" if desired.
7. Save.

Verifying the applied settings
------------------------------
Use `gh` to fetch and inspect the protection object:

```bash
gh api /repos/d-o-hub/rust-self-learning-memory/branches/main/protection --jq .
```

Or use `curl` with a PAT and inspect the JSON response.

Troubleshooting common errors
-----------------------------
- HTTP 403 / "Resource not accessible by integration": your `gh` session is using an environment `GITHUB_TOKEN` or an integration token lacking scopes. Clear `GITHUB_TOKEN` and re-run `gh auth login`.
- HTTP 422 / "Invalid request": usually caused by wrong JSON types or quoting. Use the `--input` or `@file` approach to pass JSON, not nested shell quoting.

Emergency / admin bypass
------------------------
If you need to make an emergency push (not recommended), you can temporarily remove the rule or disable "Include administrators" in the UI and then re-enable it afterward. Keep a short audit note in the repo (issue) describing why you bypassed the protection.

Customizing the contexts
------------------------
To find the exact job names used by your GitHub Actions, inspect the workflow files:

```bash
ls .github/workflows
sed -n '1,240p' .github/workflows/**/*.yml
```

Look for `jobs:` and job names — those are the contexts to use in the `contexts` array.

Example: adapt for this repo
---------------------------
If your workflows use names like `rust-ci`, `clippy`, `unit-tests`, change the contexts to:

["rust-ci", "clippy", "unit-tests"]

Contact / notes
----------------
If you want me to attempt the `gh api` call here, I can run it once you finish `gh auth login` interactively in this environment (so the CLI has the right credentials). Alternatively, run the command above locally and paste the output here and I'll validate it.

Changelog
---------
- 2025-11-06: Initial file added with recommended settings and both CLI and UI instructions.
