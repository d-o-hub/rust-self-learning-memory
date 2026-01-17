Manage git workflow: commit, push, create PR, monitor CI, and fix GitHub action issues.

Usage: /git-workflow-goap

Orchestration: goap-agent with 2-9 agents based on workflow requirements

Workflow:
1. Analyze current git state: git status, git diff, git log
2. Review staged and unstaged changes
3. Draft commit message following project conventions
4. Create commit with proper message format
5. Push to remote branch
6. Create pull request if needed
7. Monitor GitHub Actions with gh cli
8. Diagnose and fix any failing CI checks

Sub-agents (2-9) with handoff coordination:
- git-agent: Git operations (status, add, commit, push)
- pr-agent: Create and manage pull requests
- github-monitor: Monitor CI runs with gh cli
- github-workflows: Diagnose and fix workflow issues
- test-runner: Run tests to verify fixes
- build-compile: Verify build after fixes
- code-reviewer: Review changes before commit
- release-guard: Ensure CI passes before merge

GitHub Integration Commands:
- List runs: gh run list --status [queued|in_progress|completed]
- View run: gh run view <run-id>
- Rerun job: gh run rerun <run-id>
- List workflow: gh workflow list
- View workflow: gh workflow view <workflow>
- List PRs: gh pr list
- Create PR: gh pr create --title "..." --body "..."
- Check PR checks: gh pr checks <pr-number>

Git Workflow (atomic):
1. git status - Show working tree status
2. git diff - Review unstaged changes
3. git diff --cached - Review staged changes
4. git add . - Stage all changes
5. git commit -m "$(cat <<'EOF'\n[module] description\n\n🤖 Generated with [Claude Code](https://claude.com/claude-code)\n\nCo-Authored-By: Claude <noreply@anthropic.com>\nEOF\n)"
6. git push origin $(git branch --show-current)

PR Creation:
- Title: Clear, descriptive (imperative mood)
- Body: Include summary, test plan, breaking changes
- Labels: Auto-assign based on changed files
- Reviewers: Suggest based on CODEOWNERS

CI/CD Fix Workflow:
1. gh run list --status failure
2. gh run view <run-id> --log <job>
3. Analyze error: extract failure type and message
4. Categorize fix:
   - Test failure → test-runner agent
   - Build failure → build-compile agent
   - Lint failure → code-quality agent
   - Workflow config → github-workflows agent
5. Apply fix with specialized agent
6. Commit and push fix
7. Verify CI passes: gh run list --status success

Quality Gates (mandatory):
- cargo fmt --all
- cargo clippy --all -- -D warnings
- cargo build --all (if code changes)
- cargo test --all (if tests changed)
- All CI checks green before merge

Plan Files:
- Read: @plans/*.md for context
- Create: @plans/TASK_PROGRESS_git-workflow.md
- Update: Track workflow steps and CI fixes
- Only modify files in @plans/ folder
