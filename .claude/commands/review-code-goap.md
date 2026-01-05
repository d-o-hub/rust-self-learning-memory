Review code using GOAP agent orchestration.

Comprehensive code review with multi-agent handoff coordination.

Usage: /review-code-goap [files or modules]

Orchestration: goap-agent with 1-8 agents based on task complexity

Workflow:
1. Load existing plan files from plans/ for context
2. GOAP agent plans review strategy
3. Spawn specialized agents (1-8) with handoff coordination:
   - code-reviewer: Code quality review
   - rust-quality-reviewer: Rust best practices review
   - analysis-swarm: Multi-perspective analysis
4. Each agent hands off to next on completion
5. Update progress in plans/TASK_PROGRESS_*.md
6. Delete completed task files from plans/archive/

Review Areas:
- Code correctness
- Performance
- Security
- Testing
- Documentation
- Style compliance

Quality Gates (mandatory):
- cargo fmt --all
- cargo clippy --all -- -D warnings
- cargo build --all

Git Workflow (atomic):
- Stage changes: git add .
- Commit: git commit -m "docs(review): description"
- Push: git push origin develop

GitHub Integration:
- Use gh cli as source of truth
- Verify all GitHub Actions pass: gh run list --status success
- Create PR review comments: gh pr review
- All CI checks must be green before commit

Plan Files:
- Read: plans/*.md for context
- Create: plans/TASK_PROGRESS_[task].md
- Delete: plans/archive/*.md when complete
- Only modify files in plans/ folder
