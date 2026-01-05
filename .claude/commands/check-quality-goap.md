Run code quality checks using GOAP agent orchestration.

Comprehensive quality verification with multi-agent handoff coordination.

Usage: /check-quality-goap

Orchestration: goap-agent with 1-8 agents based on task complexity

Workflow:
1. Load existing plan files from plans/ for context
2. GOAP agent plans quality check strategy
3. Spawn specialized agents (1-8) with handoff coordination:
   - code-quality: Run fmt, clippy, audit
   - test-runner: Execute tests
   - build-compile: Verify compilation
4. Each agent hands off to next on completion
5. Update progress in plans/TASK_PROGRESS_*.md
6. Delete completed task files from plans/archive/

Quality Commands:
- Format: cargo fmt --all
- Lint: cargo clippy --all -- -D warnings
- Audit: cargo audit
- Deny: cargo deny check
- Test: cargo test --all
- Coverage: ./scripts/quality-gates.sh

Quality Standards:
- Test Coverage: >90%
- Test Pass Rate: >95%
- Clippy Warnings: 0
- Code Formatting: 100% rustfmt

Git Workflow (atomic):
- Stage changes: git add .
- Commit: git commit -m "chore(quality): description"
- Push: git push origin develop

GitHub Integration:
- Use gh cli as source of truth
- Verify all GitHub Actions pass: gh run list --status success
- Check for security advisories: gh api advisories
- All CI checks must be green before commit

Plan Files:
- Read: plans/*.md for context
- Create: plans/TASK_PROGRESS_[task].md
- Delete: plans/archive/*.md when complete
- Only modify files in plans/ folder
