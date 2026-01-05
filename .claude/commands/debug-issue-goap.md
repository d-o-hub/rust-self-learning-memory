Debug a runtime issue using GOAP agent orchestration.

Systematic debugging with multi-agent handoff coordination.

Usage: /debug-issue-goap [issue description]

Common scenarios:
- Runtime panics
- Async deadlocks
- Database connection issues
- Performance degradation
- Race conditions

Orchestration: goap-agent with 1-8 agents based on task complexity

Workflow:
1. Load existing plan files from plans/ for context
2. GOAP agent analyzes issue and plans debugging strategy
3. Spawn specialized agents (1-8) with handoff coordination:
   - debugger: Diagnose root cause
   - build-compile: Verify compilation
   - test-runner: Reproduce issue
   - code-quality: Ensure code quality
4. Each agent hands off to next on completion
5. Update progress in plans/TASK_PROGRESS_*.md
6. Delete completed task files from plans/archive/

Debug Commands:
- Backtrace: RUST_BACKTRACE=1 cargo test
- Async debug: RUST_LOG=debug cargo test
- Memory: cargo tree -p memory-mcp --features javy-backend
- Database: sqlite3 debug queries

Quality Gates (mandatory):
- cargo fmt --all
- cargo clippy --all -- -D warnings
- cargo build --all
- cargo test --all

Git Workflow (atomic):
- Stage changes: git add .
- Commit: git commit -m "fix(issue): description"
- Push: git push origin develop

GitHub Integration:
- Use gh cli as source of truth
- Verify all GitHub Actions pass: gh run list --status success
- Create/update issue: gh issue create/close
- All CI checks must be green before commit

Plan Files:
- Read: plans/*.md for context
- Create: plans/TASK_PROGRESS_[task].md
- Delete: plans/archive/*.md when complete
- Only modify files in plans/ folder
