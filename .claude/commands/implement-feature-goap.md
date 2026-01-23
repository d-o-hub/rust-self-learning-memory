Implement missing features from plan files or implement a new feature using GOAP agent orchestration.

Guide through the complete feature implementation with multi-agent handoff coordination.

Usage:
- `/implement-feature-goap` - Read plans/ folder and implement missing/incomplete tasks
- `/implement-feature-goap [feature description]` - Implement a new feature from description

Orchestration: goap-agent with 1-8 agents based on task complexity

Workflow (no argument - read plans/ folder):
1. Scan plans/ folder for *.md files
2. Parse each plan file to identify tasks
3. Filter for tasks marked as "incomplete" or "pending"
4. GOAP agent decomposes each incomplete task into subtasks
5. Spawn specialized agents (1-8) with handoff coordination
6. Update plan files with completion status
7. Archive completed plan files to plans/archive/

Workflow (with feature description):
1. Create new plan file: plans/TASK_PROGRESS_[task_name].md
2. GOAP agent decomposes task into subtasks
3. Spawn specialized agents (1-8) with handoff coordination:
   - feature-implementer: Core implementation
   - code-quality: Format and lint
   - test-runner: Write and run tests
   - build-compile: Verify compilation
   - specialist agents based on the task
4. Each agent hands off to next on completion
5. Update progress in plans/TASK_PROGRESS_*.md

Plan File Format:
```markdown
# Task: [Task Name]
Status: incomplete | in_progress | complete
Created: YYYY-MM-DD
Completed: YYYY-MM-DD (when complete)

## Implementation Steps
- [ ] Step 1
- [ ] Step 2

## Notes
Any additional context
```

Quality Gates (mandatory):
- cargo fmt --all
- cargo clippy --all -- -D warnings
- cargo build --all
- cargo test --all

Git Workflow (atomic):
- Stage changes: git add .
- Commit: git commit -m "[module] description"
- Push: git push origin develop

GitHub Integration:
- Use gh cli as source of truth
- Verify all GitHub Actions pass: gh run list --status success
- Monitor issues: gh issue list --state open
- All CI checks must be green before commit

Plan Files:
- Read: plans/*.md for context
- Create: plans/TASK_PROGRESS_[task].md
- Update: Modify status and steps in existing plan files
- Archive: Move to plans/archive/ when complete
- Only modify files in plans/ folder
