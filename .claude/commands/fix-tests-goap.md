Fix failing tests using GOAP agent orchestration.

Systematic test fixing with multi-agent handoff coordination.

Usage: /fix-tests-goap [test pattern or module]

Orchestration: goap-agent with 1-8 agents based on task complexity

Workflow:
1. Load existing plan files from plans/ for context
2. GOAP agent analyzes test failures and decomposes fixes
3. Spawn specialized agents (1-8) with handoff coordination:
   - test-fix: Diagnose and fix failing tests
   - debugger: Debug async/await issues
   - code-quality: Format and lint
   - build-compile: Verify compilation
4. Each agent hands off to next on completion
5. Update progress in plans/TASK_PROGRESS_*.md
6. Delete completed task files from plans/archive/

Quality Gates (mandatory):
- cargo fmt --all
- cargo clippy --all -- -D warnings
- cargo build --all
- cargo test --all

Git Workflow (atomic):
- Stage changes: git add .
- Commit: git commit -m "fix(tests): description"
- Push: git push origin develop

GitHub Integration:
- Use gh cli as source of truth
- Verify all GitHub Actions pass: gh run list --status success
- Monitor test-related issues: gh issue list --state open
- All CI checks must be green before commit

Plan Files:
- Read: plans/*.md for context
- Create: plans/TASK_PROGRESS_[task].md
- Delete: plans/archive/*.md when complete
- Only modify files in plans/ folder
