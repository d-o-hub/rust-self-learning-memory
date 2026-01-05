Build the project using GOAP agent orchestration.

Comprehensive build with multi-agent handoff coordination.

Usage: /build-goap

Orchestration: goap-agent with 1-8 agents based on build requirements

Workflow:
1. Load existing plan files from plans/ for context
2. GOAP agent plans build strategy
3. Spawn specialized agents (1-8) with handoff coordination:
   - build-compile: Compile all crates
   - code-quality: Format and lint
   - test-runner: Run tests
4. Each agent hands off to next on completion
5. Update progress in plans/TASK_PROGRESS_*.md
6. Delete completed task files from plans/archive/

Build Commands:
- Debug build: cargo build --workspace
- Release build: cargo build --release --workspace
- Check only: cargo check --all

Quality Gates (mandatory):
- cargo fmt --all
- cargo clippy --all -- -D warnings
- cargo build --all
- cargo test --all

Git Workflow (atomic):
- Stage changes: git add .
- Commit: git commit -m "chore(build): description"
- Push: git push origin develop

GitHub Integration:
- Use gh cli as source of truth
- Verify all GitHub Actions pass: gh run list --status success
- All CI checks must be green before commit

Plan Files:
- Read: plans/*.md for context
- Create: plans/TASK_PROGRESS_[task].md
- Delete: plans/archive/*.md when complete
- Only modify files in plans/ folder
